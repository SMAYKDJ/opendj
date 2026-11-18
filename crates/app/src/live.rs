//! Battle Live in the app: LIVE in the top bar opens the panel, which
//! starts the camera server (the phones' QR codes) and the picture. The
//! window's own picture is handed to the session a few times a second; the
//! session draws the vertical picture, picks the shot and runs ffmpeg.
//!
//! Settings: the names, the mode and where it goes are in settings.json;
//! the stream key is in the system keychain.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opendj_control::ControlRegistry;
use opendj_engine::{AudioEngine, MasterFeed, NUM_DECKS};
use opendj_live::compose::Look;
use opendj_live::director::{self, Interval, Mode, Scene};
use opendj_live::output::{self, Destinations};
use opendj_live::session::{AudioSource, Decks, Session, Settings};
use opendj_live::Frame;
use serde_json::json;
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode};

use crate::{settings, AppWindow, LiveUi};

const MODE: &str = "live_b2b";
const DJ_A: &str = "live_dj_a";
const DJ_B: &str = "live_dj_b";
const ROTATE: &str = "live_rotate_bars";
const STREAM: &str = "live_stream";
const RECORD: &str = "live_record";
const SERVER: &str = "live_rtmp_server";
const KEYCHAIN_SERVICE: &str = "openDJ live";
const KEYCHAIN_ID: &str = "rtmp_key";

/// The picture sent: 720 × 1280 at 30 pictures a second, what TikTok asks
/// of a vertical live.
const SIZE: (u32, u32) = (720, 1280);
const FPS: u32 = 30;
/// How often the window is copied for the stream, and for the preview only.
const SCREEN_ON_AIR: Duration = Duration::from_millis(100);
const SCREEN_PREVIEW: Duration = Duration::from_millis(400);

/// The engine's second master copy, shared so a new session can take it
/// up after the cameras were switched off.
#[derive(Clone)]
struct Master(Arc<Mutex<MasterFeed>>);

impl AudioSource for Master {
    fn drain_into(&mut self, out: &mut Vec<f32>) {
        if let Ok(mut f) = self.0.lock() {
            f.drain_into(out);
        }
    }
    fn set_active(&self, on: bool) {
        if let Ok(f) = self.0.lock() {
            f.set_active(on);
        }
    }
}

/// Copying nothing, when the engine has no master copy to give.
struct Silence;

impl AudioSource for Silence {
    fn drain_into(&mut self, _: &mut Vec<f32>) {}
    fn set_active(&self, _: bool) {}
}

pub struct Live {
    registry: Arc<ControlRegistry>,
    master: Option<Master>,
    rate: u32,
    session: Option<Session>,
    timer: Timer,
    last_screen: Instant,
    last_preview: Instant,
    key: String,
}

impl Live {
    pub fn install(
        app: &AppWindow,
        registry: &Arc<ControlRegistry>,
        engine: &AudioEngine,
    ) -> Rc<RefCell<Self>> {
        let master = engine
            .take_live_feed()
            .map(|f| Master(Arc::new(Mutex::new(f))));
        let key = keychain()
            .and_then(|e| e.get_password().ok())
            .unwrap_or_default();
        let live = Rc::new(RefCell::new(Self {
            registry: registry.clone(),
            master,
            rate: engine.sample_rate().round() as u32,
            session: None,
            timer: Timer::default(),
            last_screen: Instant::now(),
            last_preview: Instant::now(),
            key: key.clone(),
        }));

        let ui = app.global::<LiveUi>();
        ui.set_b2b(settings::flag(MODE, true));
        ui.set_dj_a(name_or(DJ_A, "DJ A").into());
        ui.set_dj_b(name_or(DJ_B, "DJ B").into());
        ui.set_rotate_bars(settings::number(ROTATE, 16.0) as i32);
        ui.set_stream(settings::flag(STREAM, true));
        ui.set_record(settings::flag(RECORD, true));
        ui.set_server(settings::string(SERVER).into());
        ui.set_key(key.into());
        ui.set_record_folder(recordings_dir().display().to_string().into());

        let weak = app.as_weak();
        {
            let live = live.clone();
            let weak = weak.clone();
            ui.on_opened(move || {
                if let Some(app) = weak.upgrade() {
                    Self::open(&live, &app);
                }
            });
        }
        ui.on_set_b2b({
            let live = live.clone();
            let weak = weak.clone();
            move |b2b| {
                settings::set(MODE, json!(b2b));
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_b2b(b2b);
                    app.global::<LiveUi>().set_manual(-1);
                }
                live.borrow().apply(&weak);
            }
        });
        ui.on_set_names({
            let live = live.clone();
            let weak = weak.clone();
            move |a, b| {
                settings::set(DJ_A, json!(a.as_str()));
                settings::set(DJ_B, json!(b.as_str()));
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_dj_a(a);
                    app.global::<LiveUi>().set_dj_b(b);
                }
                live.borrow().apply(&weak);
            }
        });
        ui.on_set_rotate({
            let live = live.clone();
            let weak = weak.clone();
            move |bars| {
                settings::set(ROTATE, json!(bars));
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_rotate_bars(bars);
                }
                live.borrow().apply(&weak);
            }
        });
        ui.on_set_stream({
            let weak = weak.clone();
            move |on| {
                settings::set(STREAM, json!(on));
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_stream(on);
                }
            }
        });
        ui.on_set_record({
            let weak = weak.clone();
            move |on| {
                settings::set(RECORD, json!(on));
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_record(on);
                }
            }
        });
        ui.on_set_server(|s| settings::set(SERVER, json!(s.trim())));
        ui.on_set_key({
            let live = live.clone();
            move |k| {
                let k = k.trim().to_string();
                if let Some(e) = keychain() {
                    let _ = if k.is_empty() {
                        e.delete_credential()
                    } else {
                        e.set_password(&k)
                    };
                }
                live.borrow_mut().key = k;
            }
        });
        ui.on_pick({
            let live = live.clone();
            let weak = weak.clone();
            move |i| {
                if let Some(app) = weak.upgrade() {
                    app.global::<LiveUi>().set_manual(i);
                }
                live.borrow().apply(&weak);
            }
        });
        ui.on_go({
            let live = live.clone();
            let weak = weak.clone();
            move || {
                if let Some(app) = weak.upgrade() {
                    live.borrow().go(&app);
                }
            }
        });
        ui.on_stop({
            let live = live.clone();
            move || {
                if let Some(s) = &live.borrow().session {
                    s.stop_live();
                }
            }
        });
        ui.on_finish({
            let live = live.clone();
            let weak = weak.clone();
            move || {
                let mut l = live.borrow_mut();
                l.timer.stop();
                l.session = None;
                if let Some(app) = weak.upgrade() {
                    let ui = app.global::<LiveUi>();
                    ui.set_started(false);
                    ui.set_on_air(false);
                    ui.set_qr1(Image::default());
                    ui.set_qr2(Image::default());
                    ui.set_preview(Image::default());
                }
            }
        });
        live
    }

    /// The panel opened: start the cameras and the picture, once.
    fn open(live: &Rc<RefCell<Self>>, app: &AppWindow) {
        let ui = app.global::<LiveUi>();
        if live.borrow().session.is_none() {
            let started = {
                let l = live.borrow();
                let reg = l.registry.clone();
                let decks: Decks = Box::new(move || director::read_decks(&reg, NUM_DECKS));
                let audio: Box<dyn AudioSource> = match &l.master {
                    Some(m) => Box::new(m.clone()),
                    None => Box::new(Silence),
                };
                let dir = settings::config_dir()
                    .unwrap_or_else(std::env::temp_dir)
                    .join("live");
                Session::start(&dir, SIZE, FPS, l.rate, decks, audio, settings_from(&ui))
            };
            match started {
                Ok(session) => {
                    for slot in 0..2 {
                        let qr = session.phones().qr(slot, 4).map(|(side, px)| {
                            Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                &px,
                                side as u32,
                                side as u32,
                            ))
                        });
                        let url = session.phones().url(slot).unwrap_or_default();
                        if slot == 0 {
                            ui.set_qr1(qr.unwrap_or_default());
                            ui.set_url1(url.into());
                        } else {
                            ui.set_qr2(qr.unwrap_or_default());
                            ui.set_url2(url.into());
                        }
                    }
                    ui.set_ffmpeg_missing(session.ffmpeg().is_none());
                    ui.set_error(if live.borrow().master.is_none() {
                        "Sem cópia do master: a transmissão sai sem som.".into()
                    } else {
                        "".into()
                    });
                    ui.set_started(true);
                    live.borrow_mut().session = Some(session);
                }
                Err(e) => {
                    ui.set_error(format!("As câmeras não ligaram: {e}").into());
                    return;
                }
            }
            let weak = app.as_weak();
            let l2 = live.clone();
            live.borrow()
                .timer
                .start(TimerMode::Repeated, Duration::from_millis(50), move || {
                    if let Some(app) = weak.upgrade() {
                        l2.borrow_mut().poll(&app);
                    }
                });
        }
    }

    /// Hands the session what the panel says.
    fn apply(&self, weak: &slint::Weak<AppWindow>) {
        let (Some(s), Some(app)) = (&self.session, weak.upgrade()) else {
            return;
        };
        let new = settings_from(&app.global::<LiveUi>());
        s.update(|cur| {
            *cur = Settings {
                votes: cur.votes,
                ..new
            }
        });
    }

    fn go(&self, app: &AppWindow) {
        let ui = app.global::<LiveUi>();
        let Some(s) = &self.session else {
            return;
        };
        let rtmp = if ui.get_stream() {
            match output::rtmp_url(&ui.get_server(), &self.key) {
                Some(u) => Some(u),
                None => {
                    ui.set_error(
                        "Cole a URL do servidor (rtmp://…) e a chave de transmissão do TikTok."
                            .into(),
                    );
                    return;
                }
            }
        } else {
            None
        };
        let record = ui.get_record().then(|| {
            let dir = recordings_dir();
            let _ = std::fs::create_dir_all(&dir);
            dir.join(format!("Battle Live {}.mp4", stamp()))
        });
        match s.go_live(Destinations { rtmp, record }) {
            Ok(()) => ui.set_error("".into()),
            Err(e) => ui.set_error(e.into()),
        }
    }

    fn poll(&mut self, app: &AppWindow) {
        let Some(s) = &self.session else {
            return;
        };
        let ui = app.global::<LiveUi>();
        let st = s.state();
        // The window's picture, for the screen part of the stream.
        let every = if st.on_air {
            SCREEN_ON_AIR
        } else {
            SCREEN_PREVIEW
        };
        if self.last_screen.elapsed() >= every {
            self.last_screen = Instant::now();
            if let Ok(shot) = app.window().take_snapshot() {
                let (w, h) = (shot.width(), shot.height());
                if let Some(f) = Frame::new(w, h, shot.as_bytes().to_vec()) {
                    s.set_screen(f);
                }
            }
        }
        if self.last_preview.elapsed() >= Duration::from_millis(200) {
            self.last_preview = Instant::now();
            if let Some(p) = s.preview() {
                ui.set_preview(Image::from_rgba8(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&p.rgba, p.width, p.height),
                ));
            }
            let cams = &st.cameras;
            ui.set_cam1_live(cams[0].live);
            ui.set_cam2_live(cams[1].live);
            ui.set_cam1_status(camera_text(&cams[0]).into());
            ui.set_cam2_status(camera_text(&cams[1]).into());
            ui.set_on_air(st.on_air);
            ui.set_time(clock(st.seconds).into());
            ui.set_health(
                if st.output.fps > 0.0 {
                    format!(
                        "{:.0} quadros/s · {:.0} kbit/s · velocidade {:.2}×",
                        st.output.fps, st.output.kbps, st.output.speed
                    )
                } else {
                    "Conectando…".into()
                }
                .into(),
            );
            if let Some(e) = &st.error {
                ui.set_error(e.as_str().into());
            }
            if let Some(shot) = st.shot {
                let names = [ui.get_dj_a().to_string(), ui.get_dj_b().to_string()];
                ui.set_shot(shot_text(&shot, &names).into());
            }
        }
    }
}

fn settings_from(ui: &LiveUi) -> Settings {
    let b2b = ui.get_b2b();
    let manual = match (b2b, ui.get_manual()) {
        (_, -1) => None,
        (true, 0) => Some(Scene::Only(0)),
        (true, 1) => Some(Scene::Only(1)),
        (true, _) => Some(Scene::Both),
        (false, i) => Some(Scene::Solo {
            main: (i.clamp(0, 1)) as usize,
        }),
    };
    let rotate = ui.get_rotate_bars().clamp(4, 64) as u32;
    Settings {
        mode: if b2b { Mode::B2b } else { Mode::Solo },
        look: Look {
            names: [ui.get_dj_a().to_string(), ui.get_dj_b().to_string()],
            tagline: if b2b {
                "B2B BATTLE".into()
            } else {
                "LIVE SET".into()
            },
            ..Look::default()
        },
        director: director::Config {
            rotate: Interval::Bars(rotate),
            ..director::Config::default()
        },
        votes: None,
        manual,
    }
}

fn camera_text(c: &opendj_live::phones::SlotStatus) -> String {
    if c.live {
        format!("{} · {:.0} quadros/s", c.device, c.fps)
    } else {
        "Aguardando o celular".into()
    }
}

fn shot_text(shot: &director::Shot, names: &[String; 2]) -> String {
    let what = match shot.scene {
        Scene::Only(d) => format!("Câmera: só {}", names[d.min(1)]),
        Scene::Both => "Câmera: os dois".into(),
        Scene::Solo { main: 0 } => "Câmera: rosto".into(),
        Scene::Solo { .. } => "Câmera: mãos".into(),
    };
    let how = if shot.manual {
        " (na mão)"
    } else {
        " (automático)"
    };
    match shot.next {
        Some(n) => format!(
            "{what}{how} · próximo: {} em {}",
            names[n.dj.min(1)],
            opendj_live::compose::clock(n.remaining)
        ),
        None => format!("{what}{how}"),
    }
}

fn name_or(key: &str, default: &str) -> String {
    let s = settings::string(key);
    if s.trim().is_empty() {
        default.into()
    } else {
        s
    }
}

/// Where recordings go: beside the mix recordings.
fn recordings_dir() -> PathBuf {
    settings::recording().dir
}

fn keychain() -> Option<keyring::Entry> {
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ID).ok()
}

fn clock(seconds: f64) -> String {
    let s = seconds.max(0.0) as u64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Now as a name for a file: 2026-11-18 21.04.05 (UTC).
fn stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
    let (y, m, d) = civil(days);
    let t = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{d:02} {:02}.{:02}.{:02}",
        t / 3600,
        t / 60 % 60,
        t % 60
    )
}

/// Days since 1970-01-01 as a date (Howard Hinnant's algorithm).
fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
