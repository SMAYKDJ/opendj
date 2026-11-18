//! The scene director: which cameras the vertical picture shows, chosen
//! from what is heard, not from where the crossfader sits.
//!
//! B2B (two DJs, a camera each): the deck heard with the least time left is
//! the one on its way out, so its DJ is the one playing now and the other
//! DJ is next. A deck counts as heard when its level in the master is above
//! [`Config::audible_db`], which works for DJs who mix on the channel
//! faders as well as on the crossfader.
//!
//! - Only the playing DJ's decks heard: that DJ alone.
//! - The playing track near its end, or both DJs heard at once: both.
//! - The old track stops: the next DJ is now the one playing, alone.
//! - Nothing changes for a while ([`Config::rotate`]): the shot moves on,
//!   alone to both and back, so the picture never sits still too long.
//! - A shot is held at least [`Config::min_hold`], so it never flickers.
//! - A shot picked by hand ([`Director::set_manual`]) pauses all this.
//!
//! Solo (one DJ, one or two cameras): the cameras take turns as the main
//! picture every [`Config::rotate`]; with one camera it stays put.

use serde::{Deserialize, Serialize};

use opendj_control::ControlRegistry;

/// One DJ or two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// One DJ: camera 1 the face, camera 2 (optional) the hands.
    Solo,
    /// Back to back: camera 1 DJ A, camera 2 DJ B.
    #[default]
    B2b,
}

/// DJ A is 0, DJ B is 1.
pub type Dj = usize;

/// What the camera row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    /// B2B: one DJ fills the row.
    Only(Dj),
    /// B2B: both DJs side by side, VS between them.
    Both,
    /// Solo: camera `main` fills the row, the other one (if any) sits small
    /// in its corner.
    Solo { main: usize },
}

/// A deck as the director sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeckState {
    pub playing: bool,
    /// Its level in the master, in dBFS: the channel meter with the channel
    /// fader and the crossfader applied. Minus infinity when silent.
    pub level_db: f32,
    /// Seconds of its track left; None without a track.
    pub remaining: Option<f32>,
}

impl Default for DeckState {
    fn default() -> Self {
        Self {
            playing: false,
            level_db: f32::NEG_INFINITY,
            remaining: None,
        }
    }
}

/// A length in bars (at the master tempo) or in seconds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Interval {
    Bars(u32),
    Seconds(f32),
}

impl Interval {
    /// In seconds, at `bpm` (120 when unknown); bars are four beats.
    pub fn seconds(self, bpm: f32) -> f64 {
        match self {
            Interval::Bars(n) => {
                let bpm = if bpm.is_finite() && bpm > 20.0 {
                    bpm
                } else {
                    120.0
                };
                n as f64 * 4.0 * 60.0 / bpm as f64
            }
            Interval::Seconds(s) => s.max(0.0) as f64,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// A deck is heard from this level up (dBFS in the master).
    pub audible_db: f32,
    /// The playing track counts as near its end with this many seconds left.
    pub near_end: f32,
    /// How long a shot stays when nothing happens.
    pub rotate: Interval,
    /// The shortest a shot is shown.
    pub min_hold: Interval,
    /// The DJ on each deck: decks 1 and 3 DJ A, 2 and 4 DJ B.
    pub dj_of_deck: [Dj; 4],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audible_db: -30.0,
            near_end: 60.0,
            rotate: Interval::Bars(16),
            min_hold: Interval::Bars(4),
            dj_of_deck: [0, 1, 0, 1],
        }
    }
}

/// The other DJ is up next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Next {
    pub dj: Dj,
    /// Seconds left of the track playing now.
    pub remaining: f32,
}

/// What to show now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub scene: Scene,
    /// B2B, with the playing track near its end: who comes in.
    pub next: Option<Next>,
    /// The DJ playing now (B2B), for the picture's labels.
    pub playing: Option<Dj>,
    /// Picked by hand: the automatic director is paused.
    pub manual: bool,
}

pub struct Director {
    mode: Mode,
    config: Config,
    /// Cameras sending pictures (Solo uses it to know whether to rotate).
    cameras: usize,
    scene: Option<Scene>,
    /// When `scene` was put up.
    shown_at: f64,
    /// What the rules ask for, and since when; rotation steps away from it
    /// and back.
    base: Option<Scene>,
    rotate_step: usize,
    playing: Option<Dj>,
    manual: Option<Scene>,
}

impl Director {
    pub fn new(mode: Mode, config: Config) -> Self {
        Self {
            mode,
            config,
            cameras: 2,
            scene: None,
            shown_at: 0.0,
            base: None,
            rotate_step: 0,
            playing: None,
            manual: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if mode != self.mode {
            self.mode = mode;
            self.scene = None;
            self.base = None;
            self.manual = None;
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// How many cameras are sending pictures.
    pub fn set_cameras(&mut self, n: usize) {
        self.cameras = n;
    }

    /// Shows `scene` until called again with None, which hands the picture
    /// back to the rules.
    pub fn set_manual(&mut self, scene: Option<Scene>) {
        self.manual = scene;
    }

    /// The shot for `now` (seconds, from any fixed start), from the decks
    /// and the master tempo.
    pub fn tick(&mut self, now: f64, decks: &[DeckState], bpm: f32) -> Shot {
        let (base, next) = match self.mode {
            Mode::Solo => (Some(Scene::Solo { main: 0 }), None),
            Mode::B2b => self.b2b_rules(decks),
        };
        let rotate = self.config.rotate.seconds(bpm);
        let min_hold = self.config.min_hold.seconds(bpm);

        if let Some(base) = base {
            if self.base != Some(base) {
                self.base = Some(base);
                self.rotate_step = 0;
            }
        }
        let cycle = self.base.map(|b| self.cycle(b));
        // What should be up, and whether getting there is a rotation step.
        let (wanted, rotating) = match (&cycle, self.scene) {
            (None, shown) => (shown, false),
            (Some(c), None) => (Some(c[self.rotate_step % c.len()]), false),
            (Some(c), Some(shown)) => {
                let at = c[self.rotate_step % c.len()];
                if at != shown {
                    // The rules moved on.
                    (Some(at), false)
                } else if c.len() > 1 && now - self.shown_at >= rotate {
                    (Some(c[(self.rotate_step + 1) % c.len()]), true)
                } else {
                    (Some(shown), false)
                }
            }
        };
        if let Some(wanted) = wanted {
            let held = self.scene.is_none_or(|_| now - self.shown_at >= min_hold);
            if self.scene != Some(wanted) && held {
                self.scene = Some(wanted);
                self.shown_at = now;
                if rotating {
                    self.rotate_step += 1;
                }
            }
        }
        let auto = self.scene.unwrap_or(match self.mode {
            Mode::Solo => Scene::Solo { main: 0 },
            Mode::B2b => Scene::Both,
        });
        Shot {
            scene: self.manual.unwrap_or(auto),
            next,
            playing: self.playing,
            manual: self.manual.is_some(),
        }
    }

    /// The shots rotation goes through, starting from what the rules ask.
    fn cycle(&self, base: Scene) -> Vec<Scene> {
        match base {
            Scene::Only(dj) => vec![Scene::Only(dj), Scene::Both],
            Scene::Both => match self.playing {
                Some(dj) => vec![
                    Scene::Both,
                    Scene::Only(1 - dj),
                    Scene::Both,
                    Scene::Only(dj),
                ],
                None => vec![Scene::Both],
            },
            Scene::Solo { .. } if self.cameras >= 2 => {
                vec![Scene::Solo { main: 0 }, Scene::Solo { main: 1 }]
            }
            Scene::Solo { .. } => vec![Scene::Solo { main: 0 }],
        }
    }

    /// What the B2B rules ask for, and who is next; None when nothing is
    /// heard (the picture stays as it is).
    fn b2b_rules(&mut self, decks: &[DeckState]) -> (Option<Scene>, Option<Next>) {
        let heard: Vec<usize> = decks
            .iter()
            .enumerate()
            .filter(|(_, d)| d.playing && d.level_db >= self.config.audible_db)
            .map(|(i, _)| i)
            .collect();
        let left = |i: usize| decks[i].remaining.unwrap_or(f32::INFINITY);
        // The deck on its way out: the least time left.
        let Some(&out) = heard.iter().min_by(|&&a, &&b| left(a).total_cmp(&left(b))) else {
            return (None, None);
        };
        let dj_of = |i: usize| {
            self.config
                .dj_of_deck
                .get(i)
                .copied()
                .unwrap_or(i % 2)
                .min(1)
        };
        let playing = dj_of(out);
        self.playing = Some(playing);
        let both_heard = heard.iter().any(|&i| dj_of(i) != playing);
        let near_end = left(out) < self.config.near_end;
        let next = near_end.then(|| Next {
            dj: 1 - playing,
            remaining: left(out),
        });
        let scene = if both_heard || near_end {
            Scene::Both
        } else {
            Scene::Only(playing)
        };
        (Some(scene), next)
    }
}

/// A channel meter (0 to 1 for −40 to 0 dBFS, as the engine's `VuMeter`)
/// as dBFS.
pub fn meter_db(vu: f64) -> f32 {
    if vu <= 0.0 {
        f32::NEG_INFINITY
    } else {
        (vu * 40.0 - 40.0) as f32
    }
}

/// How much of a deck the crossfader lets through: left decks
/// (orientation 0) fade as it moves right, right decks (2) as it moves
/// left, others never.
pub fn crossfader_side(orientation: i32, xfader: f64) -> f64 {
    match orientation {
        0 => (1.0 - xfader).clamp(0.0, 1.0),
        2 => (1.0 + xfader).clamp(0.0, 1.0),
        _ => 1.0,
    }
}

/// The decks and the master tempo, read from the controls.
pub fn read_decks(registry: &ControlRegistry, decks: usize) -> (Vec<DeckState>, f32) {
    let get = |g: &str, k: &str, d: f64| registry.get(g, k).map_or(d, |c| c.get());
    let xfader = if get("[Mixer]", "crossfader_enabled", 1.0) > 0.5 {
        get("[Master]", "crossfader", 0.0)
    } else {
        0.0
    };
    let mut bpm = 0.0f32;
    let mut loudest = f32::NEG_INFINITY;
    let states = (0..decks)
        .map(|d| {
            let g = format!("[Channel{}]", d + 1);
            let playing = get(&g, "play", 0.0) > 0.5;
            let gain = get(&g, "volume", 0.0)
                * crossfader_side(get(&g, "orientation", 1.0).round() as i32, xfader);
            let level_db = if gain <= 1e-4 {
                f32::NEG_INFINITY
            } else {
                meter_db(get(&g, "VuMeter", 0.0)) + 20.0 * (gain as f32).log10()
            };
            let duration = get(&g, "duration", 0.0);
            let remaining = (duration > 0.0)
                .then(|| (duration * (1.0 - get(&g, "playposition", 0.0).clamp(0.0, 1.0))) as f32);
            if playing && level_db > loudest {
                loudest = level_db;
                bpm = get(&g, "bpm", 0.0) as f32;
            }
            DeckState {
                playing,
                level_db,
                remaining,
            }
        })
        .collect();
    (states, bpm)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOUD: f32 = -8.0;

    fn deck(playing: bool, level_db: f32, remaining: f32) -> DeckState {
        DeckState {
            playing,
            level_db,
            remaining: Some(remaining),
        }
    }

    fn off() -> DeckState {
        DeckState::default()
    }

    fn secs() -> Config {
        Config {
            rotate: Interval::Seconds(30.0),
            min_hold: Interval::Seconds(8.0),
            ..Config::default()
        }
    }

    #[test]
    fn one_dj_heard_shows_that_dj() {
        let mut d = Director::new(Mode::B2b, secs());
        let shot = d.tick(0.0, &[deck(true, LOUD, 200.0), off()], 128.0);
        assert_eq!(shot.scene, Scene::Only(0));
        assert_eq!(shot.next, None);
        assert_eq!(shot.playing, Some(0));
    }

    #[test]
    fn channel_fader_down_is_not_heard() {
        // Deck 2 plays in the headphones only: its level in the master is
        // below the threshold, whatever the crossfader does.
        let mut d = Director::new(Mode::B2b, secs());
        let shot = d.tick(
            0.0,
            &[deck(true, LOUD, 200.0), deck(true, -45.0, 300.0)],
            128.0,
        );
        assert_eq!(shot.scene, Scene::Only(0));
    }

    #[test]
    fn both_heard_shows_both_after_the_hold() {
        let mut d = Director::new(Mode::B2b, secs());
        d.tick(0.0, &[deck(true, LOUD, 200.0), off()], 128.0);
        // DJ B brings a track in at 3 s: too soon to cut.
        let both = [deck(true, LOUD, 197.0), deck(true, -20.0, 300.0)];
        assert_eq!(d.tick(3.0, &both, 128.0).scene, Scene::Only(0));
        assert_eq!(d.tick(8.0, &both, 128.0).scene, Scene::Both);
    }

    #[test]
    fn near_the_end_shows_both_and_who_is_next() {
        let mut d = Director::new(Mode::B2b, secs());
        d.tick(0.0, &[deck(true, LOUD, 100.0), off()], 128.0);
        let shot = d.tick(50.0, &[deck(true, LOUD, 50.0), off()], 128.0);
        assert_eq!(shot.scene, Scene::Both);
        assert_eq!(
            shot.next,
            Some(Next {
                dj: 1,
                remaining: 50.0
            })
        );
    }

    #[test]
    fn the_least_time_left_is_the_one_going_out() {
        // DJ B's track (deck 2) ends first: B is playing, A is next.
        let mut d = Director::new(Mode::B2b, secs());
        let shot = d.tick(
            0.0,
            &[deck(true, LOUD, 280.0), deck(true, LOUD, 30.0)],
            128.0,
        );
        assert_eq!(shot.playing, Some(1));
        assert_eq!(shot.next.map(|n| n.dj), Some(0));
    }

    #[test]
    fn when_the_old_track_stops_the_next_dj_is_shown_alone() {
        let mut d = Director::new(Mode::B2b, secs());
        d.tick(
            0.0,
            &[deck(true, LOUD, 40.0), deck(true, LOUD, 300.0)],
            128.0,
        );
        let shot = d.tick(
            20.0,
            &[
                deck(false, f32::NEG_INFINITY, 20.0),
                deck(true, LOUD, 280.0),
            ],
            128.0,
        );
        assert_eq!(shot.scene, Scene::Only(1));
        assert_eq!(shot.next, None);
    }

    #[test]
    fn silence_keeps_the_picture() {
        let mut d = Director::new(Mode::B2b, secs());
        d.tick(0.0, &[deck(true, LOUD, 200.0), off()], 128.0);
        assert_eq!(d.tick(20.0, &[off(), off()], 128.0).scene, Scene::Only(0));
    }

    #[test]
    fn a_long_stretch_rotates() {
        let mut d = Director::new(Mode::B2b, secs());
        let decks = [deck(true, LOUD, 300.0), off()];
        assert_eq!(d.tick(0.0, &decks, 128.0).scene, Scene::Only(0));
        assert_eq!(d.tick(29.0, &decks, 128.0).scene, Scene::Only(0));
        assert_eq!(d.tick(30.0, &decks, 128.0).scene, Scene::Both);
        assert_eq!(d.tick(45.0, &decks, 128.0).scene, Scene::Both);
        assert_eq!(d.tick(60.0, &decks, 128.0).scene, Scene::Only(0));
    }

    #[test]
    fn bars_follow_the_tempo() {
        assert!((Interval::Bars(16).seconds(120.0) - 32.0).abs() < 1e-9);
        assert!((Interval::Bars(4).seconds(128.0) - 7.5).abs() < 1e-9);
        assert!((Interval::Bars(4).seconds(0.0) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn manual_pauses_the_rules() {
        let mut d = Director::new(Mode::B2b, secs());
        d.set_manual(Some(Scene::Only(1)));
        let shot = d.tick(0.0, &[deck(true, LOUD, 200.0), off()], 128.0);
        assert_eq!(shot.scene, Scene::Only(1));
        assert!(shot.manual);
        d.set_manual(None);
        assert_eq!(
            d.tick(1.0, &[deck(true, LOUD, 199.0), off()], 128.0).scene,
            Scene::Only(0)
        );
    }

    #[test]
    fn solo_rotates_only_with_two_cameras() {
        let mut d = Director::new(Mode::Solo, secs());
        d.set_cameras(1);
        assert_eq!(d.tick(0.0, &[], 128.0).scene, Scene::Solo { main: 0 });
        assert_eq!(d.tick(100.0, &[], 128.0).scene, Scene::Solo { main: 0 });
        d.set_cameras(2);
        assert_eq!(d.tick(131.0, &[], 128.0).scene, Scene::Solo { main: 1 });
        assert_eq!(d.tick(140.0, &[], 128.0).scene, Scene::Solo { main: 1 });
        assert_eq!(d.tick(161.0, &[], 128.0).scene, Scene::Solo { main: 0 });
        assert_eq!(d.tick(161.0, &[], 128.0).next, None);
    }

    #[test]
    fn levels_from_the_meter() {
        assert_eq!(meter_db(0.0), f32::NEG_INFINITY);
        assert!((meter_db(1.0) - 0.0).abs() < 1e-6);
        assert!((meter_db(0.25) + 30.0).abs() < 1e-6);
        assert_eq!(crossfader_side(0, 1.0), 0.0);
        assert_eq!(crossfader_side(2, 1.0), 1.0);
        assert_eq!(crossfader_side(1, -1.0), 1.0);
    }
}
