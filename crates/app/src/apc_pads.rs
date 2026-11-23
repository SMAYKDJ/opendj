//! The APC Key 25 mk2's top two pad rows as phrase bars: the first row for
//! deck 1, the second for deck 2. Each fills one pad at a time from the
//! right across the phrase playing (a pad per eighth of the phrase), so
//! the edge between lit and dark pads is the needle, and it empties when
//! the next phrase starts. A deck stopped or empty leaves its row dark.
//!
//! This module works out how many pads are lit and publishes it as
//! `[ChannelN],phrase_pads` (0 to 8) with the look picked on Settings ›
//! APC Key 25 under `[APC]`; the controller (crates/controllers) lights
//! the pads from those, and the settings page draws the same pads.

use std::cell::RefCell;

use opendj_control::{ControlHandle, ControlRegistry};

use crate::beat_counter::{Count, DEFAULT_PHRASE_BARS};
use crate::settings;

/// Pads in a row.
pub const PADS: i64 = 8;

/// The settings, and the `[APC]` controls carrying them to the controller.
pub const ON_KEY: &str = "apc_phrase_pads";
pub const TRAIL_KEY: &str = "apc_phrase_trail";
pub const COLOR_KEYS: [&str; 2] = ["apc_phrase_color_1", "apc_phrase_color_2"];

/// The colours offered, as the APC's palette numbers, with the colour the
/// settings page draws them in.
pub const COLORS: [(&str, u8, u32); 8] = [
    ("Verde", 21, 0x2fd45a),
    ("Azul", 45, 0x2f6bff),
    ("Laranja", 9, 0xff6a1a),
    ("Vermelho", 5, 0xff2a2a),
    ("Amarelo", 13, 0xffe01a),
    ("Roxo", 49, 0xa64dff),
    ("Ciano", 37, 0x1ad8ff),
    ("Branco", 3, 0xf2f2f2),
];
/// The colour each deck starts with: green for deck 1, blue for deck 2.
const DEFAULT_COLOR: [usize; 2] = [0, 1];

struct Controls {
    on: ControlHandle,
    trail: ControlHandle,
    color: [ControlHandle; 2],
    lit: [ControlHandle; 2],
}

thread_local! {
    static CONTROLS: RefCell<Option<Controls>> = const { RefCell::new(None) };
    /// [`on`], kept here: the settings file is read once, not every frame.
    static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the rows show the phrases, without reading the settings file.
pub fn active() -> bool {
    ACTIVE.with(|a| a.get())
}

/// Makes the controls the controller reads, set from the settings.
pub fn install(reg: &ControlRegistry) {
    let c = Controls {
        on: reg.free("[APC]", "phrase_pads"),
        trail: reg.free("[APC]", "phrase_trail"),
        color: [reg.free("[APC]", "phrase_color_1"), reg.free("[APC]", "phrase_color_2")],
        lit: [
            reg.free("[Channel1]", "phrase_pads"),
            reg.free("[Channel2]", "phrase_pads"),
        ],
    };
    CONTROLS.with(|s| *s.borrow_mut() = Some(c));
    apply();
}

/// Whether the rows show the phrases.
pub fn on() -> bool {
    settings::flag(ON_KEY, true)
}

/// Whether the pads already played show dimmer than the one playing.
pub fn trail() -> bool {
    settings::flag(TRAIL_KEY, true)
}

/// Deck `d`'s colour, as its place in [`COLORS`].
pub fn color(d: usize) -> usize {
    let i = settings::number(COLOR_KEYS[d], DEFAULT_COLOR[d] as f64);
    (i.max(0.0) as usize).min(COLORS.len() - 1)
}

/// Sends the settings to the controller.
pub fn apply() {
    ACTIVE.with(|a| a.set(on()));
    CONTROLS.with(|s| {
        if let Some(c) = s.borrow().as_ref() {
            c.on.set(if on() { 1.0 } else { 0.0 });
            c.trail.set(if trail() { 1.0 } else { 0.0 });
            for d in 0..2 {
                c.color[d].set(f64::from(COLORS[color(d)].1));
            }
        }
    });
}

/// Shows the settings on Settings › APC Key 25's picture.
pub fn show(app: &crate::AppWindow) {
    use slint::ComponentHandle;
    let g = app.global::<crate::ApcPads>();
    g.set_on(on());
    g.set_trail(trail());
    let colors: Vec<slint::Color> = (0..2)
        .map(|d| slint::Color::from_argb_encoded(0xff00_0000 | COLORS[color(d)].2))
        .collect();
    g.set_colors(slint::ModelRc::new(slint::VecModel::from(colors)));
    // A model the frames can change row by row (an array literal can't be).
    use slint::Model;
    if g.get_lit().row_count() != 2 || !is_vec_model(&g.get_lit()) {
        g.set_lit(slint::ModelRc::new(slint::VecModel::from(vec![0, 0])));
    }
}

fn is_vec_model(m: &slint::ModelRc<i32>) -> bool {
    use slint::Model;
    m.as_any().downcast_ref::<slint::VecModel<i32>>().is_some()
}

/// How many of a row's pads are lit at `count`: one from the phrase's
/// start, one more each eighth of it, all eight in its last eighth.
pub fn lit(count: &Count) -> i64 {
    if count.phrase == 0 || count.bar_in_phrase == 0 {
        return 0;
    }
    let (bar, bars) = if count.bars_left > 0 {
        (count.bar_in_phrase - 1, count.bar_in_phrase - 1 + count.bars_left)
    } else {
        // The last phrase runs to the end: eight bars at a time.
        (
            (count.bar_in_phrase - 1) % DEFAULT_PHRASE_BARS,
            DEFAULT_PHRASE_BARS,
        )
    };
    let beats = (bar * 4 + count.beat) as f64;
    let total = (bars.max(1) * 4) as f64;
    ((beats / total * PADS as f64).floor() as i64 + 1).clamp(1, PADS)
}

/// Deck `d`'s row now: lit as `count` says while it plays, dark otherwise.
/// Returns how many pads are lit, for the settings page's picture.
pub fn update(d: usize, count: Option<&Count>, playing: bool) -> i64 {
    if d >= 2 {
        return 0;
    }
    let n = match count {
        Some(c) if playing => lit(c),
        _ => 0,
    };
    CONTROLS.with(|s| {
        if let Some(c) = s.borrow().as_ref() {
            if c.lit[d].get() != n as f64 {
                c.lit[d].set(n as f64);
            }
        }
    });
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(bar_in_phrase: i64, bars_left: i64, beat: i64) -> Count {
        Count {
            beat,
            bar_in_group: 0,
            phrase: 1,
            bar_in_phrase,
            bars_left,
            next: None,
        }
    }

    #[test]
    fn a_phrase_of_eight_bars_lights_a_pad_a_bar() {
        assert_eq!(lit(&at(1, 8, 0)), 1);
        assert_eq!(lit(&at(1, 8, 3)), 1);
        assert_eq!(lit(&at(2, 7, 0)), 2);
        assert_eq!(lit(&at(8, 1, 3)), 8);
    }

    #[test]
    fn a_phrase_of_sixteen_bars_lights_a_pad_every_two() {
        assert_eq!(lit(&at(2, 15, 3)), 1);
        assert_eq!(lit(&at(3, 14, 0)), 2);
        assert_eq!(lit(&at(16, 1, 0)), 8);
    }

    #[test]
    fn before_the_first_bar_the_row_is_dark() {
        let mut c = at(1, 8, 0);
        c.phrase = 0;
        c.bar_in_phrase = 0;
        assert_eq!(lit(&c), 0);
    }
}
