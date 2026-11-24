//! B2B Online Remote — Fase 1: Room Code generation and clipboard helpers.
//! Fase 2 (real WebRTC P2P networking) will be built on top of this module.

use std::sync::Arc;
use opendj_control::ControlRegistry;

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

