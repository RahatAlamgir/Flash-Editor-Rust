//! A letter's outline as a drawing path.

use rustybuzz::Face;
use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};
use tiny_skia::{Path, PathBuilder, Transform};

struct Collect {
    b: PathBuilder,
    scale: f32,
    ox: f32,
    oy: f32,
}

impl Collect {
    fn p(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + x * self.scale, self.oy - y * self.scale) // font units go up, the picture goes down
    }
}

impl OutlineBuilder for Collect {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.p(x, y);
        self.b.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.p(x, y);
        self.b.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let ((x1, y1), (x, y)) = (self.p(x1, y1), self.p(x, y));
        self.b.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let ((x1, y1), (x2, y2), (x, y)) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        self.b.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.b.close();
    }
}

/// The letter `glyph` of `size` pixels with its origin (left end of the baseline) at (`ox`, `oy`). `None` for letters
/// without an outline (spaces).
pub fn glyph_path(face: &Face, glyph: GlyphId, size: f64, ox: f64, oy: f64) -> Option<Path> {
    let scale = (size / f64::from(face.units_per_em())) as f32;
    let mut c = Collect { b: PathBuilder::new(), scale, ox: ox as f32, oy: oy as f32 };
    face.outline_glyph(glyph, &mut c)?;
    c.b.finish()
}

/// The same path moved / turned.
pub fn transformed(path: &Path, t: Transform) -> Option<Path> {
    path.clone().transform(t)
}
