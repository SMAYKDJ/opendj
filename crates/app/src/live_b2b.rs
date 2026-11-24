//! B2B Online Remote — Fase 1: Room Code generation and clipboard helpers.
//! Fase 2 (real WebRTC P2P networking) will be built on top of this module.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use opendj_control::ControlRegistry;
use serde::{Deserialize, Serialize};

pub const DEFAULT_B2B_PORT: u16 = 7498;

/// Estado instantâneo de um Deck transmitido pela rede (baixo consumo de banda, ~40 bytes).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DeckStateMessage {
    pub deck_index: usize, // 0 = Deck A, 1 = Deck B
    pub playing: bool,
    pub playposition: f32, // 0.0 .. 1.0
    pub duration: f32,     // segundos
    pub bpm: f32,
    pub volume: f32,       // 0.0 .. 1.0
    pub rate: f32,         // pitch slider
    pub eq_low: f32,       // grave
    pub eq_mid: f32,       // médio
    pub eq_high: f32,      // agudo
    pub vu_meter: f32,     // level meter
}

/// Mensagem de sincronização B2B transmitida em tempo real.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum B2bMessage {
    Handshake { guest_name: String, code: String },
    DeckUpdate(DeckStateMessage),
    CueTrigger { deck_index: usize, position: f32 },
    Heartbeat,
}

/// Sessão ativa de rede B2B (Host ou Convidado).
pub struct B2bNetworkSession {
    pub connected: Arc<AtomicBool>,
    sender: Sender<B2bMessage>,
    receiver: Receiver<B2bMessage>,
    stop: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
}

impl B2bNetworkSession {
    /// Inicia modo anfitrião (Host): aguarda conexão na porta TCP do B2B.
    pub fn start_host(port: u16) -> Self {
        let connected = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let (out_tx, out_rx) = channel::<B2bMessage>();
        let (in_tx, in_rx) = channel::<B2bMessage>();

        let c = connected.clone();
        let s = stop.clone();

        let thread = std::thread::spawn(move || {
            let addr = format!("0.0.0.0:{port}");
            let Ok(listener) = TcpListener::bind(&addr) else {
                return;
            };
            let _ = listener.set_nonblocking(true);

            while !s.load(Ordering::Relaxed) {
                if let Ok((mut stream, _)) = listener.accept() {
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
                    c.store(true, Ordering::Release);

                    let mut buf = [0u8; 1024];
                    while !s.load(Ordering::Relaxed) {
                        // Envio de mensagens pendentes
                        while let Ok(msg) = out_rx.try_recv() {
                            if let Ok(json) = serde_json::to_vec(&msg) {
                                let len = (json.len() as u32).to_le_bytes();
                                if stream.write_all(&len).is_err() || stream.write_all(&json).is_err() {
                                    c.store(false, Ordering::Release);
                                    break;
                                }
                            }
                        }

                        // Recepção de mensagens
                        let mut len_buf = [0u8; 4];
                        if stream.read_exact(&mut len_buf).is_ok() {
                            let len = u32::from_le_bytes(len_buf) as usize;
                            if len > 0 && len < buf.len() {
                                if stream.read_exact(&mut buf[..len]).is_ok() {
                                    if let Ok(msg) = serde_json::from_slice::<B2bMessage>(&buf[..len]) {
                                        let _ = in_tx.send(msg);
                                    }
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    c.store(false, Ordering::Release);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        Self {
            connected,
            sender: out_tx,
            receiver: in_rx,
            stop,
            _thread: Some(thread),
        }
    }

    /// Inicia modo convidado (Guest): conecta ao IP/Host do DJ anfitrião.
    pub fn start_guest(host: &str, port: u16) -> Self {
        let connected = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let (out_tx, out_rx) = channel::<B2bMessage>();
        let (in_tx, in_rx) = channel::<B2bMessage>();

        let c = connected.clone();
        let s = stop.clone();
        let target = format!("{host}:{port}");

        let thread = std::thread::spawn(move || {
            let Ok(mut stream) = TcpStream::connect(&target) else {
                return;
            };
            let _ = stream.set_nodelay(true);
            let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
            c.store(true, Ordering::Release);

            let mut buf = [0u8; 1024];
            while !s.load(Ordering::Relaxed) {
                // Envio
                while let Ok(msg) = out_rx.try_recv() {
                    if let Ok(json) = serde_json::to_vec(&msg) {
                        let len = (json.len() as u32).to_le_bytes();
                        if stream.write_all(&len).is_err() || stream.write_all(&json).is_err() {
                            c.store(false, Ordering::Release);
                            return;
                        }
                    }
                }

                // Recepção
                let mut len_buf = [0u8; 4];
                if stream.read_exact(&mut len_buf).is_ok() {
                    let len = u32::from_le_bytes(len_buf) as usize;
                    if len > 0 && len < buf.len() {
                        if stream.read_exact(&mut buf[..len]).is_ok() {
                            if let Ok(msg) = serde_json::from_slice::<B2bMessage>(&buf[..len]) {
                                let _ = in_tx.send(msg);
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            c.store(false, Ordering::Release);
        });

        Self {
            connected,
            sender: out_tx,
            receiver: in_rx,
            stop,
            _thread: Some(thread),
        }
    }

    pub fn send(&self, msg: B2bMessage) {
        let _ = self.sender.send(msg);
    }

    pub fn try_recv(&self) -> Option<B2bMessage> {
        self.receiver.try_recv().ok()
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    pub fn close(&self) {
        self.stop.store(true, Ordering::Release);
        self.connected.store(false, Ordering::Release);
    }
}

/// Extrai o estado atual de um deck local para envio pela rede.
pub fn capture_local_deck(reg: &ControlRegistry, deck_index: usize) -> DeckStateMessage {
    let g = format!("[Channel{}]", deck_index + 1);
    let eq = format!("[EqualizerRack1_{}_Effect1]", g);
    let get = |group: &str, key: &str, def: f32| -> f32 {
        reg.get(group, key).map_or(def, |c| c.get() as f32)
    };

    DeckStateMessage {
        deck_index,
        playing: get(&g, "play", 0.0) > 0.5,
        playposition: get(&g, "playposition", 0.0),
        duration: get(&g, "duration", 0.0),
        bpm: get(&g, "bpm", 0.0),
        volume: get(&g, "volume", 1.0),
        rate: get(&g, "rate", 0.0),
        eq_low: get(&eq, "parameter1", 1.0),
        eq_mid: get(&eq, "parameter2", 1.0),
        eq_high: get(&eq, "parameter3", 1.0),
        vu_meter: get(&g, "VuMeter", 0.0),
    }
}

/// Aplica o estado recebido do DJ parceiro no deck espelhado local.
pub fn apply_remote_deck(reg: &ControlRegistry, msg: &DeckStateMessage) {
    let g = format!("[Channel{}]", msg.deck_index + 1);
    let eq = format!("[EqualizerRack1_{}_Effect1]", g);
    let force = |group: &str, key: &str, val: f32| {
        if let Some(c) = reg.get(group, key) {
            c.force(val as f64);
        }
    };

    force(&g, "play", if msg.playing { 1.0 } else { 0.0 });
    force(&g, "playposition", msg.playposition);
    force(&g, "duration", msg.duration);
    force(&g, "bpm", msg.bpm);
    force(&g, "volume", msg.volume);
    force(&g, "rate", msg.rate);
    force(&eq, "parameter1", msg.eq_low);
    force(&eq, "parameter2", msg.eq_mid);
    force(&eq, "parameter3", msg.eq_high);
    force(&g, "VuMeter", msg.vu_meter);
}

/// Generates a random 4-digit room code as a string (e.g. "7842").
/// Range: 1000–9999 so the code always has exactly 4 digits.
pub fn generate_room_code() -> String {
    let mut bytes = [0u8; 2];
    // getrandom is already a dependency of crates/live
    getrandom::fill(&mut bytes).unwrap_or(());
    let n = u16::from_le_bytes(bytes) as u32 % 9000 + 1000;
    format!("{n}")
}

/// Copies `text` to the system clipboard.
/// macOS: uses the built-in `pbcopy` command (no extra crate needed).
pub fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Applies Deck Ownership in Collaborative Mode to the `ControlRegistry`.
///
/// In B2B Collaborative Mode:
/// - Both decks (controls, EQs, faders, pitch bend, jog, cues) remain 100% UNLOCKED.
/// - This allows either DJ to adjust EQs (bass swapping), trim levels, jog/pitch bend
///   and perform natural, seamless transitions during battle / B2B sets.
/// - The chosen deck ownership is used purely for visual badge indication and stream layout.
pub fn apply_deck_lock(registry: &Arc<ControlRegistry>, _connected: bool, _own_deck: i32) {
    // Both decks stay completely unlocked for full collaborative DJing
    registry.lock_group("[Channel1]", false);
    registry.lock_group("[Channel2]", false);
    registry.lock_group("[EqualizerRack1_[Channel1]", false);
    registry.lock_group("[EqualizerRack1_[Channel2]", false);
}


