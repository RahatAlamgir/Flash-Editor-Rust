//! Liquify (C# `Liquify`): bends, swells and pinches a picture under a soft round brush, with smooth edges.

use crate::imaging::{PixelRect, TileMap};
use crate::transform::sample_bilinear;

/// What the Liquify brush does to the pixels under it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LiquifyMode {
    /// Pushes the pixels along in the direction the brush moves (like smearing wet paint).
    #[default]
    Push,
    /// Makes what is under the brush bigger (swells it from the middle).
    Bloat,
    /// Makes what is under the brush smaller (pinches it towards the middle).
    Pucker,
}

impl LiquifyMode {
    pub const ALL: [LiquifyMode; 3] = [LiquifyMode::Push, LiquifyMode::Bloat, LiquifyMode::Pucker];

    pub fn name(self) -> &'static str {
        match self {
            LiquifyMode::Push => "Push",
            LiquifyMode::Bloat => "Bloat",
            LiquifyMode::Pucker => "Pucker",
        }
    }
}

/// Applies the brush once, centered on `(cx, cy)`. For `Push`, `(dx, dy)` is how far the brush has just moved.
/// Strength is 0..1. Returns the changed rectangle.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    target: &mut TileMap,
    canvas: PixelRect,
    cx: f64,
    cy: f64,
    radius: f64,
    strength: f64,
    mode: LiquifyMode,
    dx: f64,
    dy: f64,
) -> PixelRect {
    if radius < 1.0 || strength <= 0.0 {
        return PixelRect::EMPTY;
    }
    let side = (radius * 2.0).ceil() as i32 + 3;
    let area = PixelRect::new((cx - radius).floor() as i32 - 1, (cy - radius).floor() as i32 - 1, side, side)
        .intersect(&canvas);
    if area.is_empty() || (mode == LiquifyMode::Push && dx.abs() < 1e-6 && dy.abs() < 1e-6) {
        return PixelRect::EMPTY;
    }
    let source = target.to_buffer(area); // what the pixels looked like before this step
    let r2 = radius * radius;
    for y in area.y..area.y + area.height {
        let py = f64::from(y) + 0.5 - cy;
        for x in area.x..area.x + area.width {
            let px = f64::from(x) + 0.5 - cx;
            let d2 = px * px + py * py;
            if d2 >= r2 {
                continue;
            }
            let t = 1.0 - d2 / r2;
            let falloff = t * t * strength; // 1 in the middle, smoothly down to 0 at the edge
            let (sx, sy) = match mode {
                LiquifyMode::Push => (f64::from(x) + 0.5 - dx * falloff, f64::from(y) + 0.5 - dy * falloff),
                LiquifyMode::Bloat => {
                    (f64::from(x) + 0.5 - px * falloff * 0.35, f64::from(y) + 0.5 - py * falloff * 0.35)
                }
                LiquifyMode::Pucker => {
                    (f64::from(x) + 0.5 + px * falloff * 0.35, f64::from(y) + 0.5 + py * falloff * 0.35)
                }
            };
            let from_x = (sx - f64::from(area.x) - 0.5).clamp(0.0, f64::from(source.width()) - 1.0);
            let from_y = (sy - f64::from(area.y) - 0.5).clamp(0.0, f64::from(source.height()) - 1.0);
            let p = sample_bilinear(&source, from_x, from_y);
            target.set_pixel(x, y, crate::imaging::ColorRgba::from_slice(&p));
        }
    }
    area
}
