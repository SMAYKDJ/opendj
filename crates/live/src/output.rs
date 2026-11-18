//! Encoding and sending: ffmpeg takes the picture on its standard input
//! (raw RGBA), the master over a socket on this computer (32-bit float
//! stereo), and writes H.264 and AAC to an RTMP server (FLV), to a file
//! (fragmented MP4, playable even if the app stops suddenly), or both.
//!
//! ffmpeg is not bundled: openDJ looks for it where Homebrew and the usual
//! package managers put it (`brew install ffmpeg` on a Mac), or at the path
//! in `OPENDJ_FFMPEG`.

use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Where the stream goes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Destinations {
    /// The RTMP server and stream key joined: `rtmp://host/app/key`.
    pub rtmp: Option<String>,
    /// A file to record to (`.mp4`).
    pub record: Option<PathBuf>,
}

impl Destinations {
    pub fn is_empty(&self) -> bool {
        self.rtmp.is_none() && self.record.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// The master's sample rate.
    pub sample_rate: u32,
    pub video_kbps: u32,
    pub audio_kbps: u32,
    pub destinations: Destinations,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 720,
            height: 1280,
            fps: 30,
            sample_rate: 48_000,
            video_kbps: 3000,
            audio_kbps: 160,
            destinations: Destinations::default(),
        }
    }
}

/// The server and the key as one RTMP address. The key is the last part
/// of the path, as every platform's ingest expects.
pub fn rtmp_url(server: &str, key: &str) -> Option<String> {
    let server = server.trim().trim_end_matches('/');
    let key = key.trim().trim_start_matches('/');
    if !(server.starts_with("rtmp://") || server.starts_with("rtmps://")) || key.is_empty() {
        return None;
    }
    Some(format!("{server}/{key}"))
}

/// Where ffmpeg is: `OPENDJ_FFMPEG`, the search path, then where Homebrew
/// and MacPorts install it (apps opened from the Finder don't get the
/// shell's search path).
pub fn find_ffmpeg() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("OPENDJ_FFMPEG") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|d| d.join(name)));
    }
    for d in [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/local/bin",
        "/usr/bin",
    ] {
        candidates.push(Path::new(d).join(name));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// The H.264 encoder: the Mac's hardware one, else x264.
fn video_encoder(kbps: u32, fps: u32) -> Vec<String> {
    let gop = (fps * 2).to_string();
    let rate = [
        "-b:v".into(),
        format!("{kbps}k"),
        "-maxrate".into(),
        format!("{kbps}k"),
        "-bufsize".into(),
        format!("{}k", kbps * 2),
        "-g".into(),
        gop,
        "-pix_fmt".into(),
        "yuv420p".into(),
    ];
    let mut v: Vec<String> = if cfg!(target_os = "macos") {
        [
            "-c:v",
            "h264_videotoolbox",
            "-realtime",
            "1",
            "-allow_sw",
            "1",
            "-profile:v",
            "high",
        ]
        .map(String::from)
        .to_vec()
    } else {
        [
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-tune",
            "zerolatency",
            "-profile:v",
            "high",
        ]
        .map(String::from)
        .to_vec()
    };
    v.extend(rate);
    v
}

/// ffmpeg's arguments, the master arriving on `audio_port`.
pub fn arguments(s: &Settings, audio_port: u16) -> Vec<String> {
    let mut a: Vec<String> = [
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "warning",
        "-stats_period",
        "2",
        "-stats",
    ]
    .map(String::from)
    .to_vec();
    // The picture.
    a.extend([
        "-thread_queue_size".into(),
        "64".into(),
        "-f".into(),
        "rawvideo".into(),
        "-pix_fmt".into(),
        "rgba".into(),
        "-s".into(),
        format!("{}x{}", s.width, s.height),
        "-framerate".into(),
        s.fps.to_string(),
        "-i".into(),
        "pipe:0".into(),
    ]);
    // The master.
    a.extend([
        "-thread_queue_size".into(),
        "1024".into(),
        "-f".into(),
        "f32le".into(),
        "-ar".into(),
        s.sample_rate.to_string(),
        "-ac".into(),
        "2".into(),
        "-i".into(),
        format!("tcp://127.0.0.1:{audio_port}"),
    ]);
    a.extend(["-map", "0:v", "-map", "1:a"].map(String::from));
    a.extend(video_encoder(s.video_kbps, s.fps));
    a.extend([
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        format!("{}k", s.audio_kbps),
        "-ac".into(),
        "2".into(),
    ]);
    let d = &s.destinations;
    match (&d.rtmp, &d.record) {
        (Some(url), None) => {
            a.extend(["-f", "flv", "-flvflags", "no_duration_filesize"].map(String::from));
            a.push(url.clone());
        }
        (None, Some(file)) => {
            a.extend(
                [
                    "-f",
                    "mp4",
                    "-movflags",
                    "+frag_keyframe+empty_moov+default_base_moof",
                    "-y",
                ]
                .map(String::from),
            );
            a.push(file.to_string_lossy().into_owned());
        }
        (Some(url), Some(file)) => {
            a.extend(["-flags", "+global_header", "-f", "tee"].map(String::from));
            a.push(format!(
                "[f=flv:flvflags=no_duration_filesize:onfail=abort]{}|[f=mp4:movflags=+frag_keyframe+empty_moov+default_base_moof:onfail=ignore]{}",
                tee_escape(url),
                tee_escape(&file.to_string_lossy())
            ));
        }
        (None, None) => {
            a.extend(["-f", "null", "-"].map(String::from));
        }
    }
    a
}

/// Escapes the characters the tee muxer's list treats specially.
fn tee_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '|' | '[' | ']' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Pictures waiting for ffmpeg.
const VIDEO_QUEUE: usize = 4;
/// Chunks of the master waiting for ffmpeg (about two seconds).
const AUDIO_QUEUE: usize = 64;

/// What ffmpeg says about itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    pub running: bool,
    /// From ffmpeg's progress line: pictures a second and bits a second.
    pub fps: f32,
    pub kbps: f32,
    /// Encoding speed against real time (1.0 keeps up).
    pub speed: f32,
    /// Its last warnings and errors.
    pub log: Vec<String>,
    /// Why it stopped, when it stopped on its own.
    pub ended: Option<String>,
}

struct Shared {
    status: Mutex<Status>,
}

/// A running ffmpeg. Feed it with [`Output::write_frame`] and
/// [`Output::write_audio`]; stops when dropped.
pub struct Output {
    child: Child,
    video: Option<SyncSender<Arc<Vec<u8>>>>,
    audio: Option<SyncSender<Vec<f32>>>,
    connected: Arc<AtomicBool>,
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
    frame_bytes: usize,
    started: Instant,
}

impl Output {
    pub fn start(ffmpeg: &Path, settings: &Settings) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        let mut child = Command::new(ffmpeg)
            .args(arguments(settings, port))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;
        let shared = Arc::new(Shared {
            status: Mutex::new(Status {
                running: true,
                ..Status::default()
            }),
        });
        let mut threads = Vec::new();
        if let Some(err) = child.stderr.take() {
            let s = shared.clone();
            threads.push(
                std::thread::Builder::new()
                    .name("live-ffmpeg-log".into())
                    .spawn(move || read_log(err, s))?,
            );
        }
        // The master goes through its own thread: ffmpeg reads its inputs in
        // turn, so writing both from one thread could leave each waiting
        // on the other.
        let (audio, rx) = mpsc::sync_channel::<Vec<f32>>(AUDIO_QUEUE);
        let connected = Arc::new(AtomicBool::new(false));
        {
            let connected = connected.clone();
            listener.set_nonblocking(true)?;
            threads.push(
                std::thread::Builder::new()
                    .name("live-ffmpeg-audio".into())
                    .spawn(move || send_audio(listener, rx, connected))?,
            );
        }
        // The picture too, so the caller never waits on ffmpeg.
        let (video, frames) = mpsc::sync_channel::<Arc<Vec<u8>>>(VIDEO_QUEUE);
        if let Some(mut stdin) = child.stdin.take() {
            threads.push(
                std::thread::Builder::new()
                    .name("live-ffmpeg-video".into())
                    .spawn(move || {
                        while let Ok(f) = frames.recv() {
                            if stdin.write_all(&f).is_err() {
                                break;
                            }
                        }
                    })?,
            );
        }
        Ok(Self {
            child,
            video: Some(video),
            audio: Some(audio),
            connected,
            shared,
            threads,
            frame_bytes: settings.width as usize * settings.height as usize * 4,
            started: Instant::now(),
        })
    }

    /// One picture, RGBA, the size given at the start. False when ffmpeg
    /// is behind and the picture was not taken.
    pub fn write_frame(&mut self, rgba: &[u8]) -> io::Result<bool> {
        if rgba.len() != self.frame_bytes {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "wrong picture size",
            ));
        }
        let Some(v) = &self.video else {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "ffmpeg stopped"));
        };
        match v.try_send(Arc::new(rgba.to_vec())) {
            Ok(()) => Ok(true),
            Err(TrySendError::Full(_)) => Ok(false),
            Err(TrySendError::Disconnected(_)) => {
                Err(io::Error::new(ErrorKind::BrokenPipe, "ffmpeg stopped"))
            }
        }
    }

    /// Interleaved stereo from the master. Dropped until ffmpeg has
    /// connected for it, or when it falls behind.
    pub fn write_audio(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Some(a) = &self.audio {
            let _ = a.try_send(samples.to_vec());
        }
    }

    pub fn audio_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    pub fn seconds(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    pub fn status(&mut self) -> Status {
        if let Ok(Some(code)) = self.child.try_wait() {
            let mut s = self.shared.status.lock().unwrap();
            if s.running {
                s.running = false;
                let last = s.log.last().cloned().unwrap_or_default();
                s.ended = Some(if last.is_empty() {
                    format!("o ffmpeg parou ({code})")
                } else {
                    last
                });
            }
        }
        self.shared.status.lock().unwrap().clone()
    }

    /// Ends the stream: closes the inputs so ffmpeg finishes the file, and
    /// waits a little for it.
    pub fn stop(mut self) {
        self.finish();
    }

    fn finish(&mut self) {
        self.video.take();
        self.audio.take();
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if let Ok(Some(_)) = self.child.try_wait() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if self.video.is_some() || !self.threads.is_empty() {
            self.finish();
        }
    }
}

/// Waits for ffmpeg to connect, then writes the master to it as it comes.
fn send_audio(listener: TcpListener, rx: Receiver<Vec<f32>>, connected: Arc<AtomicBool>) {
    let until = Instant::now() + Duration::from_secs(20);
    let mut stream = loop {
        match listener.accept() {
            Ok((s, _)) => break s,
            Err(e) if e.kind() == ErrorKind::WouldBlock && Instant::now() < until => {
                // Not connected yet: what arrives now would only arrive late.
                while rx.try_recv().is_ok() {}
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return,
        }
    };
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_nodelay(true);
    while rx.try_recv().is_ok() {}
    connected.store(true, Ordering::Release);
    let mut bytes = Vec::new();
    while let Ok(chunk) = rx.recv() {
        bytes.clear();
        for x in &chunk {
            bytes.extend_from_slice(&x.to_le_bytes());
        }
        if stream.write_all(&bytes).is_err() {
            break;
        }
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn read_log(err: std::process::ChildStderr, shared: Arc<Shared>) {
    let mut r = BufReader::new(err);
    let mut buf = Vec::new();
    let mut log: VecDeque<String> = VecDeque::new();
    loop {
        buf.clear();
        // Progress lines end in \r, messages in \n.
        let mut byte = [0u8; 1];
        let mut ended = false;
        loop {
            match std::io::Read::read(&mut r, &mut byte) {
                Ok(0) => {
                    ended = true;
                    break;
                }
                Ok(_) if byte[0] == b'\r' || byte[0] == b'\n' => break,
                Ok(_) => buf.push(byte[0]),
                Err(_) => {
                    ended = true;
                    break;
                }
            }
        }
        let line = String::from_utf8_lossy(&buf).trim().to_string();
        if !line.is_empty() {
            let mut s = shared.status.lock().unwrap();
            if let Some(p) = parse_progress(&line) {
                s.fps = p.0;
                s.kbps = p.1;
                s.speed = p.2;
            } else {
                log.push_back(line);
                while log.len() > 12 {
                    log.pop_front();
                }
                s.log = log.iter().cloned().collect();
            }
        }
        if ended {
            let _ = r.fill_buf();
            return;
        }
    }
}

/// `frame=  120 fps= 30 ... bitrate=2950.1kbits/s speed=1.00x` as
/// (fps, kbps, speed).
pub fn parse_progress(line: &str) -> Option<(f32, f32, f32)> {
    if !line.starts_with("frame=") {
        return None;
    }
    let field = |name: &str| -> Option<&str> {
        let i = line.find(name)? + name.len();
        line[i..].split_whitespace().next()
    };
    let num = |s: Option<&str>, suffix: &str| -> f32 {
        s.map(|v| v.trim_end_matches(suffix))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0)
    };
    Some((
        num(field("fps="), ""),
        num(field("bitrate="), "kbits/s"),
        num(field("speed="), "x"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_server_and_key() {
        assert_eq!(
            rtmp_url("rtmp://push.tiktok.com/live/ ", " abc123"),
            Some("rtmp://push.tiktok.com/live/abc123".into())
        );
        assert_eq!(rtmp_url("http://x", "k"), None);
        assert_eq!(rtmp_url("rtmp://x/live", ""), None);
    }

    #[test]
    fn reads_progress() {
        let p = parse_progress("frame=  120 fps= 30 q=-0.0 size=    1024kB time=00:00:04.00 bitrate=2950.1kbits/s speed=1.01x");
        assert_eq!(p, Some((30.0, 2950.1, 1.01)));
        assert_eq!(parse_progress("Error opening output"), None);
    }

    #[test]
    fn records_and_streams_with_tee() {
        let s = Settings {
            destinations: Destinations {
                rtmp: Some("rtmp://a/live/k".into()),
                record: Some("/tmp/x.mp4".into()),
            },
            ..Settings::default()
        };
        let args = arguments(&s, 5000);
        assert!(args.contains(&"tee".to_string()));
        assert!(args.contains(&"tcp://127.0.0.1:5000".to_string()));
        assert!(args.last().unwrap().contains("rtmp://a/live/k"));
    }

    /// Records two seconds through the real ffmpeg, when there is one.
    #[test]
    fn records_a_file_with_ffmpeg() {
        let Some(ffmpeg) = find_ffmpeg() else {
            return;
        };
        let file = std::env::temp_dir().join(format!("opendj-live-{}.mp4", std::process::id()));
        let settings = Settings {
            width: 180,
            height: 320,
            fps: 30,
            destinations: Destinations {
                rtmp: None,
                record: Some(file.clone()),
            },
            ..Settings::default()
        };
        let mut out = Output::start(&ffmpeg, &settings).unwrap();
        let frame = vec![128u8; 180 * 320 * 4];
        let tone: Vec<f32> = (0..1600 * 2)
            .map(|i| ((i / 2) as f32 * 0.05).sin() * 0.2)
            .collect();
        let start = Instant::now();
        for n in 0..60u32 {
            let due = start + Duration::from_secs_f64(n as f64 / 30.0);
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
            out.write_frame(&frame).unwrap();
            out.write_audio(&tone);
        }
        assert!(out.audio_connected(), "{:?}", out.status());
        out.stop();
        let len = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_file(&file);
        assert!(len > 1000, "recorded {len} bytes");
    }
}
