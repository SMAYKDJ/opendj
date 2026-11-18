//! A Battle Live session: one thread draws the picture at the output's
//! frame rate (the director picks the shot, the cameras and openDJ's screen
//! are the newest pictures to hand) and hands it to ffmpeg; another sends
//! the master. The app gives it the screen and reads its state; it never
//! waits on the network or on ffmpeg.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::compose::{Composer, Look, Votes};
use crate::director::{Config, DeckState, Director, Mode, Scene, Shot};
use crate::output::{self, Output};
use crate::phones::{Phones, SlotStatus, SLOTS};
use crate::Frame;

/// The master, from the engine: as the broadcast's source.
pub trait AudioSource: Send + 'static {
    /// Moves the interleaved stereo waiting since the last call onto `out`.
    fn drain_into(&mut self, out: &mut Vec<f32>);
    /// Starts or stops the copying.
    fn set_active(&self, on: bool);
}

/// The decks and the master tempo, read when the director needs them.
pub type Decks = Box<dyn FnMut() -> (Vec<DeckState>, f32) + Send>;

/// What the DJ sets in the Battle Live window.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub look: Look,
    pub director: Config,
    pub votes: Option<Votes>,
    /// A shot picked by hand; None leaves it to the director.
    pub manual: Option<Scene>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::B2b,
            look: Look::default(),
            director: Config::default(),
            votes: None,
            manual: None,
        }
    }
}

/// What the app shows about the session.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub cameras: [SlotStatus; SLOTS],
    pub shot: Option<Shot>,
    /// Sending or recording.
    pub on_air: bool,
    pub seconds: f64,
    pub output: output::Status,
    /// Why going on air failed, or why it stopped.
    pub error: Option<String>,
    /// Pictures drawn a second, lately.
    pub fps: f32,
}

struct Shared {
    settings: Mutex<Settings>,
    screen: Mutex<Option<Frame>>,
    preview: Mutex<Option<Arc<Frame>>>,
    state: Mutex<State>,
    stop: AtomicBool,
    /// Asked for by the app; picked up by the drawing thread.
    go: Mutex<Option<output::Settings>>,
    halt: AtomicBool,
}

pub struct Session {
    shared: Arc<Shared>,
    phones: Arc<Phones>,
    thread: Option<JoinHandle<()>>,
    width: u32,
    height: u32,
    fps: u32,
    sample_rate: u32,
    ffmpeg: Option<PathBuf>,
}

impl Session {
    /// Opens the camera server and starts drawing (a preview until
    /// [`Session::go_live`]).
    pub fn start(
        cert_dir: &std::path::Path,
        size: (u32, u32),
        fps: u32,
        sample_rate: u32,
        decks: Decks,
        audio: Box<dyn AudioSource>,
        settings: Settings,
    ) -> std::io::Result<Self> {
        let phones = Arc::new(Phones::start(cert_dir)?);
        let shared = Arc::new(Shared {
            settings: Mutex::new(settings),
            screen: Mutex::new(None),
            preview: Mutex::new(None),
            state: Mutex::default(),
            stop: AtomicBool::new(false),
            go: Mutex::new(None),
            halt: AtomicBool::new(false),
        });
        let (s, p) = (shared.clone(), phones.clone());
        let fps = fps.clamp(10, 60);
        let thread = std::thread::Builder::new()
            .name("live-picture".into())
            .spawn(move || run(s, p, size, fps, decks, audio))?;
        Ok(Self {
            shared,
            phones,
            thread: Some(thread),
            width: size.0,
            height: size.1,
            fps,
            sample_rate,
            ffmpeg: output::find_ffmpeg(),
        })
    }

    pub fn phones(&self) -> &Phones {
        &self.phones
    }

    pub fn ffmpeg(&self) -> Option<&PathBuf> {
        self.ffmpeg.as_ref()
    }

    pub fn settings(&self) -> Settings {
        self.shared.settings.lock().unwrap().clone()
    }

    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.shared.settings.lock().unwrap());
    }

    /// The newest picture of openDJ's screen.
    pub fn set_screen(&self, frame: Frame) {
        *self.shared.screen.lock().unwrap() = Some(frame);
    }

    /// The picture as the audience sees it, a few times a second.
    pub fn preview(&self) -> Option<Arc<Frame>> {
        self.shared.preview.lock().unwrap().clone()
    }

    pub fn state(&self) -> State {
        self.shared.state.lock().unwrap().clone()
    }

    /// Starts sending to `destinations`. Errors at once when there is no
    /// ffmpeg or nowhere to send; later errors show in [`State::error`].
    pub fn go_live(&self, destinations: output::Destinations) -> Result<(), String> {
        if self.ffmpeg.is_none() {
            return Err(
                "O ffmpeg não foi encontrado. No Mac, instale com: brew install ffmpeg".into(),
            );
        }
        if destinations.is_empty() {
            return Err("Escolha a transmissão, a gravação ou as duas.".into());
        }
        self.shared.halt.store(false, Ordering::Release);
        *self.shared.go.lock().unwrap() = Some(output::Settings {
            width: self.width,
            height: self.height,
            fps: self.fps,
            sample_rate: self.sample_rate,
            destinations,
            ..output::Settings::default()
        });
        Ok(())
    }

    /// Stops sending and recording; the preview carries on.
    pub fn stop_live(&self) {
        self.shared.go.lock().unwrap().take();
        self.shared.halt.store(true, Ordering::Release);
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// How often the preview is refreshed, in pictures.
const PREVIEW_EVERY: u32 = 5;

fn run(
    shared: Arc<Shared>,
    phones: Arc<Phones>,
    (w, h): (u32, u32),
    fps: u32,
    mut decks: Decks,
    audio: Box<dyn AudioSource>,
) {
    let mut composer = Composer::new(w, h);
    let first = shared.settings.lock().unwrap().clone();
    let mut director = Director::new(first.mode, first.director.clone());
    let started = Instant::now();
    let period = Duration::from_secs_f64(1.0 / fps as f64);
    let mut next = Instant::now();
    let mut seen = [0u64; SLOTS];
    let mut out: Option<Output> = None;
    let audio = Arc::new(Mutex::new(audio));
    let mut samples: Vec<f32> = Vec::new();
    let mut n = 0u32;
    let mut window = (Instant::now(), 0u32);
    let mut drawn_fps = 0.0f32;
    let mut written = 0u64;

    while !shared.stop.load(Ordering::Acquire) {
        // Going on air, or off.
        if let Some(settings) = shared.go.lock().unwrap().take() {
            out = None;
            let ffmpeg = output::find_ffmpeg();
            let started = ffmpeg
                .ok_or_else(|| "O ffmpeg não foi encontrado.".to_string())
                .and_then(|f| Output::start(&f, &settings).map_err(|e| e.to_string()));
            match started {
                Ok(o) => {
                    let a = audio.lock().unwrap();
                    a.set_active(true);
                    out = Some(o);
                    written = 0;
                    shared.state.lock().unwrap().error = None;
                }
                Err(e) => shared.state.lock().unwrap().error = Some(e),
            }
            samples.clear();
        }
        if shared.halt.swap(false, Ordering::AcqRel) {
            if let Some(o) = out.take() {
                o.stop();
            }
            audio.lock().unwrap().set_active(false);
        }

        // The newest pictures.
        for (slot, seen) in seen.iter_mut().enumerate() {
            if let Some((seq, f)) = phones.latest(slot, *seen) {
                *seen = seq;
                composer.set_camera(slot, Some((*f).clone()));
            } else if !phones.status()[slot].live && composer.has_camera(slot) {
                composer.set_camera(slot, None);
                *seen = 0;
            }
        }
        if let Some(screen) = shared.screen.lock().unwrap().take() {
            composer.set_screen(Some(screen));
        }

        // The shot.
        let settings = shared.settings.lock().unwrap().clone();
        director.set_mode(settings.mode);
        if director.config() != &settings.director {
            director.set_config(settings.director.clone());
        }
        director.set_manual(settings.manual);
        let cameras = (0..SLOTS).filter(|&s| composer.has_camera(s)).count();
        director.set_cameras(cameras.max(1));
        let (states, bpm) = decks();
        let shot = director.tick(started.elapsed().as_secs_f64(), &states, bpm);
        let picture = composer.render(&settings.look, &shot, settings.votes);

        // Out.
        let mut failed = None;
        if let Some(o) = out.as_mut() {
            // As many pictures as the time on air calls for, so the picture
            // keeps pace with the sound when drawing falls behind.
            let due = (o.seconds() * fps as f64).floor() as u64 + 1;
            let mut repeats = 0;
            while written < due && repeats < 5 {
                match o.write_frame(picture) {
                    Ok(true) => written += 1,
                    Ok(false) => break,
                    Err(e) => {
                        failed = Some(e.to_string());
                        break;
                    }
                }
                repeats += 1;
            }
            samples.clear();
            audio.lock().unwrap().drain_into(&mut samples);
            o.write_audio(&samples);
        }
        n += 1;
        if n.is_multiple_of(PREVIEW_EVERY) {
            if let Some(f) = Frame::new(w, h, picture.to_vec()) {
                *shared.preview.lock().unwrap() = Some(Arc::new(f));
            }
        }
        window.1 += 1;
        if window.0.elapsed() >= Duration::from_secs(1) {
            drawn_fps = window.1 as f32 / window.0.elapsed().as_secs_f32();
            window = (Instant::now(), 0);
        }

        // What the app shows.
        {
            let mut st = shared.state.lock().unwrap();
            st.cameras = phones.status();
            st.shot = Some(shot);
            st.fps = drawn_fps;
            match out.as_mut() {
                Some(o) => {
                    st.on_air = true;
                    st.seconds = o.seconds();
                    st.output = o.status();
                    if let Some(why) = st.output.ended.clone() {
                        failed.get_or_insert(why);
                    }
                }
                None => {
                    st.on_air = false;
                    st.seconds = 0.0;
                }
            }
            if let Some(e) = failed.take() {
                let log = st.output.log.last().cloned();
                st.error = Some(log.filter(|l| !l.is_empty()).unwrap_or(e));
                st.on_air = false;
                drop(st);
                if let Some(o) = out.take() {
                    o.stop();
                }
                audio.lock().unwrap().set_active(false);
            }
        }

        // Keep time; when behind, catch up without a burst.
        next += period;
        let now = Instant::now();
        match next.checked_duration_since(now) {
            Some(wait) => std::thread::sleep(wait),
            None if now - next > period * 4 => next = now,
            None => {}
        }
    }
    if let Some(o) = out.take() {
        o.stop();
    }
    audio.lock().unwrap().set_active(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::director::DeckState;

    /// A 440 Hz tone at real time.
    struct Tone {
        started: Instant,
        sent: usize,
    }

    impl AudioSource for Tone {
        fn drain_into(&mut self, out: &mut Vec<f32>) {
            let due = (self.started.elapsed().as_secs_f64() * 48_000.0) as usize;
            for i in self.sent..due {
                let x = (i as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 0.3;
                out.extend_from_slice(&[x, x]);
            }
            self.sent = due;
        }
        fn set_active(&self, _: bool) {}
    }

    #[test]
    fn records_picture_and_sound_in_step() {
        let Some(_) = output::find_ffmpeg() else {
            return;
        };
        let dir = std::env::temp_dir().join(format!("opendj-live-session-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("set.mp4");
        let decks: Decks = Box::new(|| {
            let d = DeckState {
                playing: true,
                level_db: -10.0,
                remaining: Some(30.0),
            };
            (vec![d, DeckState::default()], 128.0)
        });
        let audio = Box::new(Tone {
            started: Instant::now(),
            sent: 0,
        });
        let s = Session::start(
            &dir,
            (180, 320),
            30,
            48_000,
            decks,
            audio,
            Settings::default(),
        )
        .unwrap();
        s.set_screen(Frame::new(16, 10, vec![200; 16 * 10 * 4]).unwrap());
        s.go_live(output::Destinations {
            rtmp: None,
            record: Some(file.clone()),
        })
        .unwrap();
        std::thread::sleep(Duration::from_secs(3));
        let st = s.state();
        assert!(st.on_air, "{st:?}");
        assert_eq!(st.shot.map(|s| s.scene), Some(Scene::Both), "near the end");
        assert!(s.preview().is_some());
        s.stop_live();
        std::thread::sleep(Duration::from_millis(1500));
        drop(s);
        let len = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
        assert!(len > 1000, "recorded {len} bytes");
        if let Ok(o) = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "stream=codec_type,duration",
                "-of",
                "csv=p=0",
            ])
            .arg(&file)
            .output()
        {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            eprintln!("{text}");
            assert!(text.contains("video") && text.contains("audio"), "{text}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
