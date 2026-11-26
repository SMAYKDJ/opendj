//! The vertical picture, laid out as the design (`docs/smaykdj/live-tiktok.md`):
//! from the top, the band TikTok covers, the camera row, the "next" notice,
//! the vote bar, openDJ's screen, and the bottom third left to TikTok's
//! comments. Lengths below are in the design's units, a 540 × 960 frame,
//! scaled to the output.

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use tiny_skia::{
    Color, FillRule, FilterQuality, IntSize, Paint, Path, PathBuilder, Pattern, Pixmap, Rect,
    SpreadMode, Transform,
};

use crate::director::{Scene, Shot};
use crate::Frame;

const FONT: &[u8] = include_bytes!("../../app/ui/fonts/InterTabular-Bold.otf");

const DESIGN_W: f32 = 540.0;
const BACKGROUND: u32 = 0x0b0b0e;
const PANEL: u32 = 0x17151d;
const TEXT: u32 = 0xf2f0f7;
const MUTED: u32 = 0xa59fb5;

/// How the picture looks: the DJs' names and colours.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// DJ A and DJ B (Solo uses the first).
    pub names: [String; 2],
    /// Each DJ's colour, 0xRRGGBB.
    pub colors: [u32; 2],
    /// Under "openDJ" in the corner, as "B2B BATTLE".
    pub tagline: String,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            names: ["DJ A".into(), "DJ B".into()],
            colors: [0xb49cff, 0xff9f43],
            tagline: "B2B BATTLE".into(),
        }
    }
}

/// The audience's votes, #A and #B in the chat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Votes {
    pub a: u32,
    pub b: u32,
}

/// Draws the vertical picture. Keeps the latest picture of each camera and
/// of the screen, so they can arrive at their own pace.
pub struct Composer {
    canvas: Pixmap,
    scale: f32,
    font: FontRef<'static>,
    cameras: [Option<Pixmap>; 2],
    screen: Option<Pixmap>,
}

impl Composer {
    /// A picture `width` × `height`, for example 720 × 1280 or 1080 × 1920.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            canvas: Pixmap::new(width.max(2), height.max(2)).expect("picture size"),
            scale: width as f32 / DESIGN_W,
            font: FontRef::try_from_slice(FONT).expect("bundled font"),
            cameras: [None, None],
            screen: None,
        }
    }

    pub fn width(&self) -> u32 {
        self.canvas.width()
    }

    pub fn height(&self) -> u32 {
        self.canvas.height()
    }

    /// The newest picture from camera `slot` (0 or 1), or None when it left.
    pub fn set_camera(&mut self, slot: usize, frame: Option<Frame>) {
        if let Some(c) = self.cameras.get_mut(slot) {
            *c = frame.and_then(pixmap);
        }
    }

    /// The newest picture of openDJ's screen. A big window (a Retina
    /// screen) is first shrunk to about the size it is drawn at.
    pub fn set_screen(&mut self, frame: Option<Frame>) {
        let want = self.canvas.width() * 3 / 2;
        self.screen = frame.map(|f| shrink(f, want)).and_then(pixmap);
    }

    pub fn has_camera(&self, slot: usize) -> bool {
        self.cameras.get(slot).is_some_and(Option::is_some)
    }

    /// Draws the picture for `shot` and returns it, RGBA.
    pub fn render(&mut self, look: &Look, shot: &Shot, votes: Option<Votes>) -> &[u8] {
        self.canvas.fill(color(BACKGROUND));
        let s = self.scale;

        // openDJ's screen, as on the laptop.
        let screen_box = rect(0.0, 354.0 * s, 540.0 * s, 340.0 * s);
        match self.screen.take() {
            Some(p) => {
                self.image(&p, screen_box, 0.0);
                self.screen = Some(p);
            }
            None => self.fill_round((0.0, 354.0 * s, 540.0 * s, 340.0 * s), 0.0, PANEL),
        }

        // The camera row.
        let row = (10.0 * s, 76.0 * s, 520.0 * s, 224.0 * s);
        let gap = 8.0 * s;
        let half = (row.2 - gap) / 2.0;
        match shot.scene {
            Scene::Both => {
                self.camera(0, (row.0, row.1, half, row.3), look, Some(0));
                self.camera(1, (row.0 + half + gap, row.1, half, row.3), look, Some(1));
                let d = 52.0 * s;
                let (cx, cy) = (row.0 + row.2 / 2.0, row.1 + row.3 / 2.0);
                self.circle(cx, cy, d / 2.0 + 2.0 * s, TEXT);
                self.circle(cx, cy, d / 2.0, BACKGROUND);
                self.text_centered("VS", cx, cy + 6.5 * s, 18.0 * s, TEXT);
            }
            Scene::Only(dj) => {
                let dj = dj.min(1);
                self.camera(dj, row, look, Some(dj));
            }
            Scene::Solo { main } => {
                let main = main.min(1);
                let other = 1 - main;
                self.camera(main, row, look, (main == 0).then_some(0));
                if self.has_camera(other) {
                    let (w, h) = (150.0 * s, 86.0 * s);
                    let pip = (
                        row.0 + row.2 - w - 10.0 * s,
                        row.1 + row.3 - h - 10.0 * s,
                        w,
                        h,
                    );
                    self.fill_round(grow(pip, 2.0 * s), 12.0 * s, TEXT);
                    let p = self.cameras[other].take().expect("checked");
                    self.image(&p, rect(pip.0, pip.1, pip.2, pip.3), 10.0 * s);
                    self.cameras[other] = Some(p);
                }
            }
        }

        // Who comes in next.
        if let Some(next) = shot.next {
            let dj = next.dj.min(1);
            let label = format!("PRÓXIMO: {}", look.names[dj].to_uppercase());
            let left = format!("faixa atual acaba em {}", clock(next.remaining));
            let (l1, l2) = (11.0 * s, 11.0 * s);
            let w1 = self.text_width(&label, l1);
            let w2 = self.text_width(&left, l2);
            let pad = 14.0 * s;
            let w = pad * 2.0 + w1 + 8.0 * s + w2;
            let h = 26.0 * s;
            let x = (540.0 * s - w) / 2.0;
            let y = 262.0 * s;
            self.fill_round((x, y, w, h), h / 2.0, look.colors[dj]);
            self.fill_round(grow((x, y, w, h), -s), h / 2.0, BACKGROUND);
            let base = y + h / 2.0 + 4.0 * s;
            self.text(&label, x + pad, base, l1, look.colors[dj]);
            self.text(&left, x + pad + w1 + 8.0 * s, base, l2, MUTED);
        }

        // The vote bar.
        if let Some(v) = votes {
            let total = v.a + v.b;
            let share = if total == 0 {
                0.5
            } else {
                v.a as f32 / total as f32
            };
            let size = 13.0 * s;
            let a = format!("♥ {} #A", v.a);
            let b = format!("#B {} ♥", v.b);
            let wa = self.text_width(&a, size);
            let wb = self.text_width(&b, size);
            let y = 306.0 * s;
            let base = y + 15.0 * s + 4.5 * s;
            self.text(&a, 10.0 * s, base, size, look.colors[0]);
            self.text(&b, 530.0 * s - wb, base, size, look.colors[1]);
            let bx = 10.0 * s + wa + 8.0 * s;
            let bw = (530.0 * s - wb - 8.0 * s) - bx;
            if bw > 0.0 {
                let bar = (bx, y + 10.0 * s, bw, 10.0 * s);
                self.fill_round(bar, 5.0 * s, look.colors[1]);
                if share > 0.0 {
                    self.fill_round(
                        (bar.0, bar.1, (bw * share).max(bar.3), bar.3),
                        5.0 * s,
                        look.colors[0],
                    );
                }
            }
            self.text_centered(
                "COMENTE #A OU #B PARA VOTAR NO SEU DJ",
                270.0 * s,
                345.0 * s,
                10.0 * s,
                MUTED,
            );
        }

        self.canvas.data()
    }

    /// Camera `slot` filling `area`, framed in its DJ's colour and named.
    fn camera(&mut self, slot: usize, area: (f32, f32, f32, f32), look: &Look, dj: Option<usize>) {
        let s = self.scale;
        let border = dj.map_or(look.colors[0], |d| look.colors[d.min(1)]);
        self.fill_round(area, 14.0 * s, border);
        let inner = grow(area, -2.0 * s);
        match self.cameras[slot].take() {
            Some(p) => {
                self.image(&p, rect(inner.0, inner.1, inner.2, inner.3), 12.0 * s);
                self.cameras[slot] = Some(p);
            }
            None => {
                self.fill_round(inner, 12.0 * s, PANEL);
                let (cx, cy) = (inner.0 + inner.2 / 2.0, inner.1 + inner.3 / 2.0);
                self.text_centered(&format!("Celular {}", slot + 1), cx, cy, 13.0 * s, MUTED);
                self.text_centered("aguardando a câmera", cx, cy + 18.0 * s, 11.0 * s, MUTED);
            }
        }
        // The name, in the DJ's colour, in the corner away from the middle.
        let name = match dj {
            Some(d) => look.names[d.min(1)].clone(),
            None => return,
        };
        let size = 13.0 * s;
        let pad = 10.0 * s;
        let w = self.text_width(&name, size) + pad * 2.0;
        let h = 24.0 * s;
        let y = area.1 + area.3 - h - 10.0 * s;
        let x = if slot == 1 && area.2 < 300.0 * s {
            area.0 + area.2 - w - 10.0 * s
        } else {
            area.0 + 10.0 * s
        };
        self.fill_round((x, y, w, h), h / 2.0, border);
        self.text(&name, x + pad, y + h / 2.0 + 4.5 * s, size, 0x14101f);
    }

    /// `p` covering `area` (cropped to fill it, centred), corners rounded.
    fn image(&mut self, p: &Pixmap, area: Rect, radius: f32) {
        let (pw, ph) = (p.width() as f32, p.height() as f32);
        let k = (area.width() / pw).max(area.height() / ph);
        let (dw, dh) = (pw * k, ph * k);
        let tx = area.x() + (area.width() - dw) / 2.0;
        let ty = area.y() + (area.height() - dh) / 2.0;
        let quality = if k < 0.5 {
            FilterQuality::Bicubic
        } else {
            FilterQuality::Bilinear
        };
        let paint = Paint {
            shader: Pattern::new(
                p.as_ref(),
                SpreadMode::Pad,
                quality,
                1.0,
                Transform::from_row(k, 0.0, 0.0, k, tx, ty),
            ),
            anti_alias: true,
            ..Paint::default()
        };
        if let Some(path) = round_rect((area.x(), area.y(), area.width(), area.height()), radius) {
            self.canvas.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn fill_round(&mut self, area: (f32, f32, f32, f32), radius: f32, rgb: u32) {
        let mut paint = Paint::default();
        paint.set_color(color(rgb));
        paint.anti_alias = true;
        if let Some(path) = round_rect(area, radius) {
            self.canvas.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn circle(&mut self, cx: f32, cy: f32, r: f32, rgb: u32) {
        let mut paint = Paint::default();
        paint.set_color(color(rgb));
        paint.anti_alias = true;
        if let Some(path) = PathBuilder::from_circle(cx, cy, r) {
            self.canvas.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn text_width(&self, text: &str, size: f32) -> f32 {
        let font = self.font.as_scaled(PxScale::from(size));
        let mut w = 0.0;
        let mut last = None;
        for ch in text.chars() {
            let id = font.glyph_id(ch);
            if let Some(prev) = last {
                w += font.kern(prev, id);
            }
            w += font.h_advance(id);
            last = Some(id);
        }
        w
    }

    fn text_centered(&mut self, text: &str, cx: f32, baseline: f32, size: f32, rgb: u32) {
        let w = self.text_width(text, size);
        self.text(text, cx - w / 2.0, baseline, size, rgb);
    }

    /// `text` from `x`, sitting on `baseline`.
    fn text(&mut self, text: &str, x: f32, baseline: f32, size: f32, rgb: u32) {
        let scale = PxScale::from(size);
        let font = self.font.as_scaled(scale);
        let (cw, ch) = (self.canvas.width() as i32, self.canvas.height() as i32);
        let [r, g, b] = [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8];
        let data = self.canvas.data_mut();
        let mut pen = x;
        let mut last = None;
        for c in text.chars() {
            let id = font.glyph_id(c);
            if let Some(prev) = last {
                pen += font.kern(prev, id);
            }
            let glyph = id.with_scale_and_position(scale, ab_glyph::point(pen, baseline));
            pen += font.h_advance(id);
            last = Some(id);
            let Some(outline) = self.font.outline_glyph(glyph) else {
                continue;
            };
            let bounds = outline.px_bounds();
            outline.draw(|gx, gy, cov| {
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                if px < 0 || py < 0 || px >= cw || py >= ch || cov <= 0.0 {
                    return;
                }
                let i = (py as usize * cw as usize + px as usize) * 4;
                let a = cov.min(1.0);
                for (k, v) in [r, g, b].into_iter().enumerate() {
                    let d = data[i + k] as f32;
                    data[i + k] = (v as f32 * a + d * (1.0 - a)).round() as u8;
                }
                data[i + 3] = 255;
            });
        }
    }
}

/// A frame as a pixmap; frames are opaque, so no premultiplying is needed
/// (alpha is forced to opaque).
fn pixmap(frame: Frame) -> Option<Pixmap> {
    let Frame {
        width,
        height,
        mut rgba,
    } = frame;
    for px in rgba.chunks_exact_mut(4) {
        px[3] = 255;
    }
    Pixmap::from_vec(rgba, IntSize::from_wh(width, height)?)
}

/// `f` shrunk by a whole factor (averaging each block) until it is at most
/// about `max_width` wide.
pub fn shrink(f: Frame, max_width: u32) -> Frame {
    let k = f.width.div_ceil(max_width.max(1)).max(1);
    if k == 1 {
        return f;
    }
    let (w, h) = (f.width / k, f.height / k);
    if w == 0 || h == 0 {
        return f;
    }
    let mut out = vec![0u8; (w * h * 4) as usize];
    let stride = f.width as usize * 4;
    let n = k * k;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let mut acc = [0u32; 4];
            for dy in 0..k as usize {
                let row = (y * k as usize + dy) * stride;
                for dx in 0..k as usize {
                    let i = row + (x * k as usize + dx) * 4;
                    for (a, v) in acc.iter_mut().zip(&f.rgba[i..i + 4]) {
                        *a += *v as u32;
                    }
                }
            }
            let o = (y * w as usize + x) * 4;
            for c in 0..4 {
                out[o + c] = (acc[c] / n) as u8;
            }
        }
    }
    Frame {
        width: w,
        height: h,
        rgba: out,
    }
}

fn color(rgb: u32) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255)
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_xywh(x, y, w.max(1.0), h.max(1.0)).expect("finite rect")
}

fn grow((x, y, w, h): (f32, f32, f32, f32), by: f32) -> (f32, f32, f32, f32) {
    (x - by, y - by, w + 2.0 * by, h + 2.0 * by)
}

fn round_rect((x, y, w, h): (f32, f32, f32, f32), r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

/// Seconds as m:ss.
pub fn clock(seconds: f32) -> String {
    let s = seconds.max(0.0).round() as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::director::Next;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Frame {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        Frame::new(w, h, v).unwrap()
    }

    fn px(c: &Composer, x: u32, y: u32) -> [u8; 3] {
        let d = c.canvas.data();
        let i = ((y * c.width() + x) * 4) as usize;
        [d[i], d[i + 1], d[i + 2]]
    }

    fn shot(scene: Scene) -> Shot {
        Shot {
            scene,
            next: None,
            playing: Some(0),
            manual: false,
        }
    }

    #[test]
    fn both_cameras_side_by_side_over_the_screen() {
        let mut c = Composer::new(720, 1280);
        c.set_camera(0, Some(solid(640, 360, [255, 0, 0])));
        c.set_camera(1, Some(solid(640, 360, [0, 0, 255])));
        c.set_screen(Some(solid(1440, 900, [0, 255, 0])));
        let out = c.render(&Look::default(), &shot(Scene::Both), None);
        assert_eq!(out.len(), 720 * 1280 * 4);
        // Left camera, right camera, the screen, the empty bottom.
        assert_eq!(px(&c, 80, 200), [255, 0, 0]);
        assert_eq!(px(&c, 640, 200), [0, 0, 255]);
        assert_eq!(px(&c, 360, 600), [0, 255, 0]);
        assert_eq!(px(&c, 100, 1200), [0x0b, 0x0b, 0x0e]);
    }

    #[test]
    fn one_dj_fills_the_row() {
        let mut c = Composer::new(540, 960);
        c.set_camera(1, Some(solid(320, 240, [0, 0, 255])));
        c.render(&Look::default(), &shot(Scene::Only(1)), None);
        assert_eq!(px(&c, 60, 150), [0, 0, 255]);
        assert_eq!(px(&c, 480, 150), [0, 0, 255]);
    }

    #[test]
    fn writes_preview_pngs() {
        let Some(dir) = std::env::var_os("LIVE_PREVIEW_DIR") else {
            return;
        };
        let mut c = Composer::new(720, 1280);
        c.set_camera(0, Some(solid(1280, 720, [90, 60, 140])));
        c.set_camera(1, Some(solid(720, 1280, [140, 90, 40])));
        c.set_screen(Some(solid(1440, 900, [40, 50, 60])));
        let mut both = shot(Scene::Both);
        both.next = Some(Next {
            dj: 1,
            remaining: 42.0,
        });
        for (name, s, votes) in [
            ("both", both, Some(Votes { a: 1284, b: 976 })),
            ("only", shot(Scene::Only(0)), None),
            ("solo", shot(Scene::Solo { main: 0 }), None),
        ] {
            c.render(&Look::default(), &s, votes);
            let path = std::path::Path::new(&dir).join(format!("{name}.png"));
            c.canvas.save_png(path).unwrap();
        }
    }

    #[test]
    fn a_big_screen_is_shrunk() {
        let f = solid(2880, 1800, [10, 20, 30]);
        let s = shrink(f, 1080);
        assert_eq!((s.width, s.height), (960, 600));
        assert_eq!(&s.rgba[..4], &[10, 20, 30, 255]);
        let small = solid(800, 500, [1, 2, 3]);
        assert_eq!(shrink(small.clone(), 1080), small);
    }

    #[test]
    fn clock_reads_minutes_and_seconds() {
        assert_eq!(clock(42.0), "0:42");
        assert_eq!(clock(125.4), "2:05");
        assert_eq!(clock(-3.0), "0:00");
    }
}
