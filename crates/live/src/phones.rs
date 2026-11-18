//! Phone cameras: a phone scans the QR code, its browser opens a page
//! served here, and the page sends the camera as JPEG pictures over a
//! WebSocket.
//!
//! Browsers only lend a page the camera over HTTPS, so this server speaks
//! TLS with a certificate made on this computer for its own address. The
//! phone warns about it the first time ("not private"); the DJ taps on
//! through. The certificate is kept, so the warning comes once per network
//! address rather than every time.
//!
//! Each QR code carries a random key made when Battle Live opens, and the
//! camera slot (1 or 2): only phones that scanned it get in, and a phone
//! that joins a slot takes it over from the one before.
//!
//! One thread per phone, as in the remote: only ever two or three.

use std::io::{self, ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Message;

use opendj_remote::http;

use crate::Frame;

/// The port tried first; the next few are tried if it's taken.
pub const DEFAULT_PORT: u16 = 7497;
const PORT_TRIES: u16 = 10;
/// Camera slots: camera 1 and camera 2.
pub const SLOTS: usize = 2;
/// Largest picture a phone may send.
const MAX_MESSAGE: usize = 4 * 1024 * 1024;
/// How long a connection waits on its socket before checking whether to
/// stop.
const POLL: Duration = Duration::from_millis(250);
/// A camera that sent nothing for this long counts as gone.
const STALE: Duration = Duration::from_secs(4);

/// The page the phone opens.
pub const PAGE: &str = include_str!("camera.html");

/// A camera slot as the app shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotStatus {
    /// A phone is sending pictures.
    pub live: bool,
    /// Pictures a second, lately.
    pub fps: f32,
    /// What the phone calls itself, from its browser.
    pub device: String,
}

#[derive(Default)]
struct Slot {
    frame: Option<Arc<Frame>>,
    /// Counts pictures, so a reader can tell a new one.
    seq: u64,
    last: Option<Instant>,
    fps: f32,
    device: String,
    /// The connection that owns the slot; an older one stops when it sees
    /// another number here.
    owner: u64,
}

struct Shared {
    key: String,
    slots: [Mutex<Slot>; SLOTS],
    stop: AtomicBool,
    next_conn: AtomicU64,
    tls: Arc<ServerConfig>,
}

/// The camera server. Stops when dropped.
pub struct Phones {
    shared: Arc<Shared>,
    port: u16,
    ip: Option<IpAddr>,
    thread: Option<JoinHandle<()>>,
}

impl Phones {
    /// Starts listening on the local network. The certificate is kept in
    /// `cert_dir`.
    pub fn start(cert_dir: &Path) -> io::Result<Self> {
        let ip = opendj_remote::local_addresses().first().map(|(_, ip)| *ip);
        let tls = tls_config(cert_dir, ip)?;
        let (listener, port) = bind()?;
        listener.set_nonblocking(true)?;
        let shared = Arc::new(Shared {
            key: random_key(),
            slots: Default::default(),
            stop: AtomicBool::new(false),
            next_conn: AtomicU64::new(1),
            tls,
        });
        let s = shared.clone();
        let thread = std::thread::Builder::new()
            .name("live-phones".into())
            .spawn(move || accept_loop(listener, s))?;
        Ok(Self {
            shared,
            port,
            ip,
            thread: Some(thread),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// The address phone `slot` (0 or 1) opens, which its QR code holds.
    /// None when this computer has no network address a phone could reach.
    pub fn url(&self, slot: usize) -> Option<String> {
        let ip = self.ip?;
        Some(camera_url(ip, self.port, slot, &self.shared.key))
    }

    /// The QR code for camera `slot`: its size in pixels and RGBA pixels.
    pub fn qr(&self, slot: usize, scale: usize) -> Option<(usize, Vec<u8>)> {
        opendj_remote::qr::rgba(&self.url(slot)?, scale)
    }

    /// The newest picture from camera `slot`, if newer than `seen` (a
    /// number this returned before; 0 at first).
    pub fn latest(&self, slot: usize, seen: u64) -> Option<(u64, Arc<Frame>)> {
        let s = self
            .shared
            .slots
            .get(slot)?
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if s.seq == seen || !fresh(&s) {
            return None;
        }
        s.frame.clone().map(|f| (s.seq, f))
    }

    pub fn status(&self) -> [SlotStatus; SLOTS] {
        std::array::from_fn(|i| {
            let s = self.shared.slots[i]
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            SlotStatus {
                live: fresh(&s),
                fps: if fresh(&s) { s.fps } else { 0.0 },
                device: s.device.clone(),
            }
        })
    }
}

impl Drop for Phones {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn fresh(s: &Slot) -> bool {
    s.frame.is_some() && s.last.is_some_and(|t| t.elapsed() < STALE)
}

pub fn camera_url(ip: IpAddr, port: u16, slot: usize, key: &str) -> String {
    let host = match ip {
        IpAddr::V6(v6) => format!("[{v6}]"),
        IpAddr::V4(v4) => v4.to_string(),
    };
    format!("https://{host}:{port}/cam?s={}&k={key}", slot + 1)
}

fn bind() -> io::Result<(TcpListener, u16)> {
    let mut last = None;
    for port in DEFAULT_PORT..DEFAULT_PORT + PORT_TRIES {
        match TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)) {
            Ok(l) => return Ok((l, port)),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| io::Error::new(ErrorKind::AddrInUse, "no free port")))
}

fn random_key() -> String {
    let mut b = [0u8; 12];
    let _ = getrandom::fill(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The TLS setup, with the kept certificate for `ip` or a new one.
fn tls_config(dir: &Path, ip: Option<IpAddr>) -> io::Result<Arc<ServerConfig>> {
    let name = ip.map_or_else(
        || "local".to_string(),
        |ip| ip.to_string().replace(':', "_"),
    );
    let cert_path: PathBuf = dir.join(format!("live-camera-{name}.cert.der"));
    let key_path: PathBuf = dir.join(format!("live-camera-{name}.key.der"));
    let (cert, key) = match (std::fs::read(&cert_path), std::fs::read(&key_path)) {
        (Ok(c), Ok(k)) => (c, k),
        _ => {
            let mut names = vec!["localhost".to_string(), "opendj.local".to_string()];
            if let Some(ip) = ip {
                names.push(ip.to_string());
            }
            let made = rcgen::generate_simple_self_signed(names).map_err(io::Error::other)?;
            let c = made.cert.der().to_vec();
            let k = made.signing_key.serialize_der();
            let _ = std::fs::create_dir_all(dir);
            let _ = std::fs::write(&cert_path, &c);
            let _ = std::fs::write(&key_path, &k);
            (c, k)
        }
    };
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(io::Error::other)?
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert)],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)),
        )
        .map_err(io::Error::other)?;
    Ok(Arc::new(config))
}

fn accept_loop(listener: TcpListener, shared: Arc<Shared>) {
    while !shared.stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let s = shared.clone();
                let _ = std::thread::Builder::new()
                    .name("live-phone".into())
                    .spawn(move || {
                        let _ = serve(stream, s);
                    });
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}

type Tls = StreamOwned<ServerConnection, TcpStream>;

/// A stream that gives back bytes already read before reading on.
struct Replay {
    buf: Vec<u8>,
    at: usize,
    inner: Tls,
}

impl Read for Replay {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.at < self.buf.len() {
            let n = out.len().min(self.buf.len() - self.at);
            out[..n].copy_from_slice(&self.buf[self.at..self.at + n]);
            self.at += n;
            return Ok(n);
        }
        self.inner.read(out)
    }
}

impl Write for Replay {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.inner.write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn serve(tcp: TcpStream, shared: Arc<Shared>) -> io::Result<()> {
    tcp.set_nonblocking(false)?;
    tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
    tcp.set_nodelay(true)?;
    let conn = ServerConnection::new(shared.tls.clone()).map_err(io::Error::other)?;
    let mut tls = StreamOwned::new(conn, tcp);
    // The request head.
    let mut buf = Vec::with_capacity(1024);
    let head = loop {
        let mut chunk = [0u8; 1024];
        let n = tls.read(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(head) = http::parse_head(&buf) {
            break head;
        }
        if buf.len() > http::MAX_HEAD {
            return Ok(());
        }
    };
    let path = head.path.clone();
    let route = path.split('?').next().unwrap_or("");
    let slot = http::query(&path, "s")
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|s| (1..=SLOTS).contains(s))
        .map(|s| s - 1);
    let key_ok = http::query(&path, "k") == Some(shared.key.as_str());

    match (head.method.as_str(), route, slot, key_ok) {
        ("GET", "/cam", Some(_), true) => respond(
            &mut tls,
            "200 OK",
            "text/html; charset=utf-8",
            PAGE.as_bytes(),
        ),
        ("GET", "/cam/ws", Some(slot), true) if head.is_upgrade() => {
            let device = head
                .header("user-agent")
                .map(device_name)
                .unwrap_or_default();
            let stream = Replay {
                buf,
                at: 0,
                inner: tls,
            };
            camera(stream, &shared, slot, device)
        }
        ("GET", "/cam" | "/cam/ws", _, false) => respond(
            &mut tls,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            "Leia de novo o QR code no openDJ.".as_bytes(),
        ),
        _ => respond(&mut tls, "404 Not Found", "text/plain", b"not found"),
    }
}

fn respond(tls: &mut Tls, status: &str, kind: &str, body: &[u8]) -> io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    tls.write_all(head.as_bytes())?;
    tls.write_all(body)?;
    tls.flush()?;
    tls.conn.send_close_notify();
    let _ = tls.flush();
    let _ = tls.sock.shutdown(Shutdown::Both);
    Ok(())
}

/// A phone's camera: pictures in until it leaves, another phone takes the
/// slot, or the server stops.
fn camera(stream: Replay, shared: &Shared, slot: usize, device: String) -> io::Result<()> {
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE))
        .max_frame_size(Some(MAX_MESSAGE));
    let mut ws = tungstenite::accept_with_config(stream, Some(config))
        .map_err(|e| io::Error::other(e.to_string()))?;
    ws.get_mut().inner.sock.set_read_timeout(Some(POLL))?;
    let me = shared.next_conn.fetch_add(1, Ordering::Relaxed);
    {
        let mut s = shared.slots[slot].lock().unwrap_or_else(|e| e.into_inner());
        s.owner = me;
        s.device = device;
        s.fps = 0.0;
    }
    let mut window = (Instant::now(), 0u32);
    loop {
        if shared.stop.load(Ordering::Acquire) {
            break;
        }
        if shared.slots[slot]
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .owner
            != me
        {
            let _ = ws.send(Message::text("{\"replaced\":true}"));
            break;
        }
        match ws.read() {
            Ok(Message::Binary(jpeg)) => {
                let Some(frame) = decode(&jpeg) else {
                    continue;
                };
                window.1 += 1;
                let mut s = shared.slots[slot].lock().unwrap_or_else(|e| e.into_inner());
                if s.owner != me {
                    break;
                }
                s.frame = Some(Arc::new(frame));
                s.seq += 1;
                s.last = Some(Instant::now());
                let dt = window.0.elapsed().as_secs_f32();
                if dt >= 1.0 {
                    s.fps = window.1 as f32 / dt;
                    window = (Instant::now(), 0);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
    }
    let mut s = shared.slots[slot].lock().unwrap_or_else(|e| e.into_inner());
    if s.owner == me {
        s.frame = None;
        s.last = None;
        s.fps = 0.0;
    }
    Ok(())
}

/// A JPEG from the phone as a frame.
pub fn decode(jpeg: &[u8]) -> Option<Frame> {
    let img = image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Frame::new(w, h, rgba.into_raw())
}

/// A short name for the phone from its browser's user agent.
fn device_name(ua: &str) -> String {
    if ua.contains("iPhone") {
        "iPhone".into()
    } else if ua.contains("iPad") {
        "iPad".into()
    } else if ua.contains("Android") {
        "Android".into()
    } else {
        "Navegador".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{ServerName, UnixTime};
    use rustls::{ClientConfig, ClientConnection, DigitallySignedStruct, SignatureScheme};

    /// Trusts any certificate, as a phone does once the DJ taps on through.
    #[derive(Debug)]
    struct AnyCert;

    impl ServerCertVerifier for AnyCert {
        fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn verify_tls13_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            rustls::crypto::ring::default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }

    fn connect(port: u16) -> StreamOwned<ClientConnection, TcpStream> {
        let config =
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(AnyCert))
                .with_no_client_auth();
        let conn =
            ClientConnection::new(Arc::new(config), ServerName::try_from("localhost").unwrap())
                .unwrap();
        let tcp = TcpStream::connect(("127.0.0.1", port)).unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        StreamOwned::new(conn, tcp)
    }

    fn key_of(p: &Phones) -> String {
        p.shared.key.clone()
    }

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, _| image::Rgb([(x % 256) as u8, 80, 160]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
        out.into_inner()
    }

    fn temp_dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("opendj-live-test-{}", random_key()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn serves_the_page_over_https_only_with_the_key() {
        let p = Phones::start(&temp_dir()).unwrap();
        let mut s = connect(p.port());
        let req = format!(
            "GET /cam?s=1&k={} HTTP/1.1\r\nHost: localhost\r\n\r\n",
            key_of(&p)
        );
        s.write_all(req.as_bytes()).unwrap();
        let mut body = String::new();
        let _ = s.read_to_string(&mut body);
        assert!(body.starts_with("HTTP/1.1 200"), "{body}");
        assert!(body.contains("getUserMedia"));

        let mut s = connect(p.port());
        s.write_all(b"GET /cam?s=1&k=wrong HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut body = String::new();
        let _ = s.read_to_string(&mut body);
        assert!(body.starts_with("HTTP/1.1 403"), "{body}");
    }

    #[test]
    fn a_phone_sends_pictures_to_its_slot() {
        let p = Phones::start(&temp_dir()).unwrap();
        let url = format!("wss://localhost:{}/cam/ws?s=2&k={}", p.port(), key_of(&p));
        let (mut ws, _) = tungstenite::client(url, connect(p.port())).unwrap();
        ws.send(Message::binary(jpeg(64, 48))).unwrap();
        let mut got = None;
        for _ in 0..100 {
            if let Some(f) = p.latest(1, 0) {
                got = Some(f);
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let (seq, frame) = got.expect("a picture in slot 2");
        assert_eq!((frame.width, frame.height), (64, 48));
        assert!(p.latest(1, seq).is_none(), "nothing newer yet");
        assert!(p.latest(0, 0).is_none(), "slot 1 is empty");
        assert!(p.status()[1].live);
        assert!(!p.status()[0].live);

        // It leaves: the slot empties.
        ws.close(None).unwrap();
        let _ = ws.flush();
        for _ in 0..100 {
            if !p.status()[1].live {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!p.status()[1].live);
    }

    #[test]
    fn urls_carry_the_slot_and_key() {
        let url = camera_url("192.168.0.12".parse().unwrap(), 7497, 1, "abc");
        assert_eq!(url, "https://192.168.0.12:7497/cam?s=2&k=abc");
    }

    #[test]
    fn the_certificate_is_kept() {
        let dir = temp_dir();
        drop(Phones::start(&dir).unwrap());
        let files = std::fs::read_dir(&dir).unwrap().count();
        assert_eq!(files, 2);
        drop(Phones::start(&dir).unwrap());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
    }
}
