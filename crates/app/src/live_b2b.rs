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

/// Applies Deck Lock to the `ControlRegistry` based on B2B Online state and chosen deck.
///
/// In B2B Online (state == 3, Connected):
/// - If `own_deck == 0` (Local DJ controls Deck A / Channel 1), then Channel 2 is locked.
/// - If `own_deck == 1` (Local DJ controls Deck B / Channel 2), then Channel 1 is locked.
/// In any other state (not connected), both decks are unlocked.
pub fn apply_deck_lock(registry: &Arc<ControlRegistry>, connected: bool, own_deck: i32) {
    if !connected {
        // Unlock both decks when not connected
        registry.lock_group("[Channel1]", false);
        registry.lock_group("[Channel2]", false);
        registry.lock_group("[EqualizerRack1_[Channel1]", false);
        registry.lock_group("[EqualizerRack1_[Channel2]", false);
        return;
    }

    if own_deck == 0 {
        // Local DJ has Deck A (Channel 1); Lock Deck B (Channel 2)
        registry.lock_group("[Channel1]", false);
        registry.lock_group("[EqualizerRack1_[Channel1]", false);
        registry.lock_group("[Channel2]", true);
        registry.lock_group("[EqualizerRack1_[Channel2]", true);
    } else {
        // Local DJ has Deck B (Channel 2); Lock Deck A (Channel 1)
        registry.lock_group("[Channel2]", false);
        registry.lock_group("[EqualizerRack1_[Channel2]", false);
        registry.lock_group("[Channel1]", true);
        registry.lock_group("[EqualizerRack1_[Channel1]", true);
    }
}

