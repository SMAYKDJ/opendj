# Tasks

## 1. Output queue in crates/controllers

- [ ] 1.1 Add `FEEDBACK_OUT: Mutex<HashMap<String, Vec<Vec<u8>>>>` and `FEEDBACK_ACTIVE: Mutex<HashMap<String, Vec<Vec<u8>>>>` statics in `manager.rs`
- [ ] 1.2 In `run_controller`'s `flush` closure, after draining `take_output`, drain `FEEDBACK_OUT[port]` and send each message through `send`; move each sent message into `FEEDBACK_ACTIVE[port]`
- [ ] 1.3 Add `Controllers::send_feedback(&self, port: &str, msg: &[u8])` — pushes to `FEEDBACK_OUT[port]`
- [ ] 1.4 Add `Controllers::clear_feedback(&self, port: &str)` — for each port in scope, builds Note Off for every entry in `FEEDBACK_ACTIVE` (same channel and note, velocity 0), pushes them to `FEEDBACK_OUT`, then clears `FEEDBACK_ACTIVE`; empty `port` means all ports

## 2. Wire-up in crates/app

- [ ] 2.1 In `Mapper::pick(key)` (`map_mode.rs`): after the existing `g.set_armed(...)` call, look up the most recent `(port, raw_bytes)` for the given key in the last `take_monitored()` snapshot; if found, call `controllers.send_feedback(port, &raw_bytes[..3])` with the first three bytes (status, note, velocity unchanged — the hardware's own velocity lights many controllers proportionally; use velocity 0x7F to guarantee full brightness)
- [ ] 2.2 In `Mapper::done()` (`map_mode.rs`): call `controllers.clear_feedback("")` after clearing the armed state and before closing Map mode

## 3. Try it

- [ ] 3.1 Test with APC Key 25: arm a pad in Map mode, verify it lights; confirm mapping, verify LED stays; click Done, verify all pads go dark
- [ ] 3.2 Test with a controller that has no MIDI output port: verify Map mode still works with no error
