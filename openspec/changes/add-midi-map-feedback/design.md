# Design

## Decisions (2026-11-21)

### Output channel — reuse the existing `send` closure

Each device thread already has a `send: impl FnMut(&[u8])` closure that wraps its `MidiOutput` connection (see `run_device` in `crates/controllers/src/manager.rs`). This closure is polled by `take_output()` from the `Controller`. The simplest addition is a second global queue — a `Mutex<HashMap<String, Vec<Vec<u8>>>>` — that maps a port name to a list of raw MIDI bytes pending delivery. At each `flush` call in `run_controller`, after draining `take_output`, the loop also drains any bytes queued for that port and sends them through the same `send` closure. This requires no new threads, no new channels, and the existing output path is reused unchanged.

### New public API: `send_feedback` and `clear_feedback`

```rust
/// Queues one raw MIDI message for delivery to the device at `port`.
/// The message is sent the next time the device thread polls (≤ POLL, 50 ms).
/// Does nothing if the port is unknown or has no output connection.
pub fn send_feedback(&self, port: &str, msg: &[u8]);

/// Queues Note Off (velocity 0) for every message previously sent via
/// `send_feedback` to `port`, then clears the record. If `port` is empty,
/// clears all ports. Called by `Mapper::done()` to clean up all controllers.
pub fn clear_feedback(&self, port: &str);
```

Internally, two new statics in `manager.rs`:

```rust
/// Bytes queued by send_feedback, by port.  Drained in run_controller.
static FEEDBACK_OUT: Mutex<HashMap<String, Vec<Vec<u8>>>> = Mutex::new(HashMap::new());
/// Last Note On sent per port, so clear_feedback can send Note Off.
static FEEDBACK_ACTIVE: Mutex<HashMap<String, Vec<Vec<u8>>>> = Mutex::new(HashMap::new());
```

### Wire-up in `map_mode.rs`

`Mapper::pick(key)` already sets `Mapper::armed`. After it updates the UI global, it also calls `controllers.send_feedback(port, &[status, note, 0x7F])` where `port` and `[status, note]` come from the most recent `take_monitored()` entry for the key being armed. Because `take_monitored()` returns `(port, raw_bytes)`, the port and the channel+note to light are known.

`Mapper::done()` calls `controllers.clear_feedback("")` (empty port = all ports) to send Note Off for every lit LED.

### Fallback for a control with no prior MIDI event

If a key has never been pressed on hardware (the MapMark was just drawn on screen but the hardware pressed it in a previous session), there is no entry in the monitored buffer. In that case `send_feedback` is a no-op — the LED simply does not light. This is the same behaviour as the "no output port" case and requires no special handling.

### No UI changes

The MapMark and Map mode machinery in `mapmark.slint` and `map_mode.rs` already manage the visual state. This change only adds MIDI output side effects.
