//! The APC Key 25 mk2's top two pad rows as phrase bars (Settings › APC Key
//! 25 in the app): row 1 for deck 1, row 2 for deck 2. The app publishes
//! how many pads each row has lit as `[ChannelN],phrase_pads` (0 to 8) and
//! the look under `[APC]`; this lights the pads from them.
//!
//! The pads take their brightness from the Note On's channel (channel 1 is
//! 10 %, channel 7 full) and their colour from the velocity.

use opendj_control::{ControlHandle, ControlRegistry};

/// The first note of each row, left to right: row 1 is notes 20–27, row 2
/// 18–1F. The rightmost pad is the first to light.
const ROW_START: [u8; 2] = [0x20, 0x18];
/// Full brightness, and the trail's (channel 2, 25 %).
const FULL: u8 = 0x96;
const DIM: u8 = 0x91;

/// Whether `port` is an APC Key 25 mk2's pads and knobs (not its keys).
pub fn is_apc(port: &str) -> bool {
    let p = port.to_ascii_lowercase();
    p.contains("apc key 25 mk2") && !p.contains("keys")
}

/// Whether a mapping's LED on `status`/`note` sits in the two rows, which
/// these bars take over while they are on.
pub fn takes(status: u8, note: u8) -> bool {
    status & 0xF0 == 0x90 && (0x18..=0x27).contains(&note)
}

struct Controls {
    on: ControlHandle,
    trail: ControlHandle,
    color: [ControlHandle; 2],
    lit: [ControlHandle; 2],
}

#[derive(Default)]
pub struct PhraseRows {
    controls: Option<Controls>,
    /// What each of the 16 pads was last sent: (status, velocity).
    last: [Option<(u8, u8)>; 16],
    was_on: bool,
}

impl PhraseRows {
    /// Whether the bars are showing now.
    pub fn on(&self) -> bool {
        self.was_on
    }

    /// Sends what changed. Returns true when the bars just went off, so the
    /// mapping's own LEDs in those rows should be sent again.
    pub fn poll(&mut self, registry: &ControlRegistry, sent: &mut Vec<Vec<u8>>) -> bool {
        if self.controls.is_none() {
            // Made by the app; absent in tests and tools.
            let get = |g: &str, k: &str| registry.get(g, k);
            self.controls = (|| {
                Some(Controls {
                    on: get("[APC]", "phrase_pads")?,
                    trail: get("[APC]", "phrase_trail")?,
                    color: [get("[APC]", "phrase_color_1")?, get("[APC]", "phrase_color_2")?],
                    lit: [get("[Channel1]", "phrase_pads")?, get("[Channel2]", "phrase_pads")?],
                })
            })();
        }
        let Some(c) = &self.controls else {
            return false;
        };
        let on = c.on.get() > 0.5;
        if !on {
            if self.was_on {
                self.was_on = false;
                for (i, last) in self.last.iter_mut().enumerate() {
                    if last.is_some() {
                        sent.push(vec![FULL, note(i), 0]);
                        *last = None;
                    }
                }
                return true;
            }
            return false;
        }
        self.was_on = true;
        let trail = c.trail.get() > 0.5;
        for row in 0..2 {
            let lit = c.lit[row].get().round().clamp(0.0, 8.0) as usize;
            let color = c.color[row].get().round().clamp(0.0, 127.0) as u8;
            for k in 1..=8usize {
                // Pad k from the right.
                let i = row * 8 + (8 - k);
                let want = if k > lit {
                    (FULL, 0)
                } else if k == lit || !trail {
                    (FULL, color)
                } else {
                    (DIM, color)
                };
                if self.last[i] != Some(want) {
                    sent.push(vec![want.0, note(i), want.1]);
                    self.last[i] = Some(want);
                }
            }
        }
        false
    }

    /// Everything is sent again on the next poll (the device restarted).
    pub fn resend(&mut self) {
        self.last = [None; 16];
    }

    /// The pads lit go dark (the controller is closing).
    pub fn dark(&mut self, sent: &mut Vec<Vec<u8>>) {
        if !self.was_on {
            return;
        }
        for (i, last) in self.last.iter_mut().enumerate() {
            if last.is_some_and(|(_, v)| v != 0) {
                sent.push(vec![FULL, note(i), 0]);
            }
            *last = None;
        }
    }
}

/// The note of pad `i`: 0–7 row 1 left to right, 8–15 row 2.
fn note(i: usize) -> u8 {
    ROW_START[i / 8] + (i % 8) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(lit1: f64, trail: bool) -> std::sync::Arc<ControlRegistry> {
        let r = ControlRegistry::new();
        r.free("[APC]", "phrase_pads").set(1.0);
        r.free("[APC]", "phrase_trail").set(if trail { 1.0 } else { 0.0 });
        r.free("[APC]", "phrase_color_1").set(21.0);
        r.free("[APC]", "phrase_color_2").set(45.0);
        r.free("[Channel1]", "phrase_pads").set(lit1);
        r.free("[Channel2]", "phrase_pads").set(0.0);
        r
    }

    #[test]
    fn the_row_fills_from_the_right_with_a_trail() {
        let r = registry(3.0, true);
        let mut p = PhraseRows::default();
        let mut sent = Vec::new();
        p.poll(&r, &mut sent);
        assert_eq!(sent.len(), 16);
        // The rightmost two dim, the third full, the rest dark.
        assert!(sent.contains(&vec![DIM, 0x27, 21]));
        assert!(sent.contains(&vec![DIM, 0x26, 21]));
        assert!(sent.contains(&vec![FULL, 0x25, 21]));
        assert!(sent.contains(&vec![FULL, 0x24, 0]));
        assert!(sent.contains(&vec![FULL, 0x1F, 0]));
        // Nothing changed: nothing sent.
        sent.clear();
        p.poll(&r, &mut sent);
        assert!(sent.is_empty());
    }

    #[test]
    fn turned_off_the_pads_go_dark_once() {
        let r = registry(8.0, false);
        let mut p = PhraseRows::default();
        let mut sent = Vec::new();
        p.poll(&r, &mut sent);
        r.get("[APC]", "phrase_pads").unwrap().set(0.0);
        sent.clear();
        assert!(p.poll(&r, &mut sent));
        assert_eq!(sent.len(), 16);
        sent.clear();
        assert!(!p.poll(&r, &mut sent));
        assert!(sent.is_empty());
    }

    #[test]
    fn only_the_control_port_is_the_pads() {
        assert!(is_apc("APC Key 25 mk2 Control"));
        assert!(!is_apc("APC Key 25 mk2 Keys"));
        assert!(!is_apc("DDJ-FLX4"));
    }
}
