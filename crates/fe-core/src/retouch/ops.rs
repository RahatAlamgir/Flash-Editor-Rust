//! What a retouching tool does to one pixel (C# `IRetouchOp`: clone, heal, dodge / burn, blur, sharpen, highlight).
//! The tool decides where and how strongly (the brush coverage); the operation decides what the pixel becomes.

use crate::imaging::color_math::clamp_byte;
use crate::imaging::{ColorRgba, PixelRect, TileMap};

/// Which part of the brightness Dodge / Burn works on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToneRange {
    Shadows,
    #[default]
    Midtones,
    Highlights,
}

impl ToneRange {
    pub const ALL: [ToneRange; 3] = [ToneRange::Shadows, ToneRange::Midtones, ToneRange::Highlights];

    pub fn name(self) -> &'static str {
        match self {
            ToneRange::Shadows => "Shadows",
            ToneRange::Midtones => "Midtones",
            ToneRange::Highlights => "Highlights",
        }
    }
}

/// One thing a retouching stroke can do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetouchOp {
    /// Clone Stamp: the pixel becomes the pixel `(dx, dy)` away.
    Clone { dx: i32, dy: i32 },
    /// Healing: texture from `(dx, dy)` away, light and color of the surroundings of the painted place.
    Heal { dx: i32, dy: i32, radius: i32 },
    /// Dodge (lighten) or Burn (darken), mostly inside the chosen range.
    Tone { burn: bool, range: ToneRange },
    /// Softens the pixels under the brush (radius 1..8).
    Blur { radius: i32 },
    /// Unsharp mask under the brush.
    Sharpen { radius: i32, amount: f64 },
    /// Tints towards a color (Spot Healing shows the painted area with it until the mouse is released).
    Highlight { color: ColorRgba, amount: f64 },
}

/// What an operation can look at: the picture before the stroke, and which pixels are being painted over.
pub struct Scene<'a> {
    pub original: &'a TileMap,
    /// Pixels outside are not read (the canvas).
    pub bounds: PixelRect,
    /// True for the pixels being painted (the blemish): healing leaves them out when it measures the surroundings.
    pub painted: &'a (dyn Fn(i32, i32) -> bool + Sync),
}

impl Scene<'_> {
    fn get(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        self.bounds.contains(x, y).then(|| self.original.pixel(x, y).to_array())
    }
}

/// The average color around a pixel (transparent pixels count less): (r, g, b, mean alpha). `None` when nothing
/// visible is near.
fn local_mean(
    scene: &Scene,
    x: i32,
    y: i32,
    radius: i32,
    step: i32,
    skip: Option<&(dyn Fn(i32, i32) -> bool + Sync)>,
) -> Option<(f64, f64, f64, f64)> {
    let (mut sr, mut sg, mut sb, mut sa, mut count) = (0.0, 0.0, 0.0, 0.0, 0u32);
    let mut yy = y - radius;
    while yy <= y + radius {
        let mut xx = x - radius;
        while xx <= x + radius {
            if let Some(p) = scene.get(xx, yy)
                && !skip.is_some_and(|f| f(xx, yy))
            {
                let a = f64::from(p[3]);
                sr += f64::from(p[0]) * a;
                sg += f64::from(p[1]) * a;
                sb += f64::from(p[2]) * a;
                sa += a;
                count += 1;
            }
            xx += step;
        }
        yy += step;
    }
    if count == 0 || sa < 1.0 {
        return None;
    }
    Some((sr / sa, sg / sa, sb / sa, sa / f64::from(count)))
}

/// `dst` moves towards `to` by `t` (all four channels).
fn lerp(dst: &mut [u8; 4], from: [u8; 4], to: [u8; 4], t: f64) {
    for c in 0..4 {
        dst[c] = clamp_byte(f64::from(from[c]) + (f64::from(to[c]) - f64::from(from[c])) * t);
    }
}

impl RetouchOp {
    /// How far around a pixel the heal looks for "the surrounding light and color" for a brush of `size`.
    pub fn heal_radius_for_brush(size: f64) -> i32 {
        ((size / 2.0) as i32 + 6).clamp(6, 120)
    }

    /// Changes the pixel (x, y) in `dst` (which holds the original pixel when called). `strength` is 0..1: how far
    /// the pixel goes towards the result.
    pub fn apply(&self, scene: &Scene, x: i32, y: i32, dst: &mut [u8; 4], strength: f64) {
        let own = *dst;
        match *self {
            RetouchOp::Clone { dx, dy } => {
                if let Some(p) = scene.get(x + dx, y + dy) {
                    lerp(dst, own, p, strength);
                }
            }
            RetouchOp::Heal { dx, dy, radius } => {
                let Some(p) = scene.get(x + dx, y + dy) else { return };
                if p[3] < 8 {
                    return; // nothing there to copy
                }
                let radius = radius.max(2);
                let step = (radius / 6).max(1);
                let (mut r, mut g, mut b) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
                let target = local_mean(scene, x, y, radius, step, Some(scene.painted));
                let source = local_mean(scene, x + dx, y + dy, radius, step, None);
                if let (Some(t), Some(l)) = (target, source) {
                    // Texture from the source, tone from the surroundings of the target.
                    r += t.0 - l.0;
                    g += t.1 - l.1;
                    b += t.2 - l.2;
                }
                lerp(dst, own, [clamp_byte(r), clamp_byte(g), clamp_byte(b), p[3]], strength);
            }
            RetouchOp::Tone { burn, range } => {
                let (mut r, mut g, mut b) = (f64::from(own[0]), f64::from(own[1]), f64::from(own[2]));
                let light = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
                let w = match range {
                    ToneRange::Shadows => (1.0 - light) * (1.0 - light),
                    ToneRange::Highlights => light * light,
                    ToneRange::Midtones => 4.0 * light * (1.0 - light),
                };
                let k = strength * (0.15 + 0.85 * w); // a little effect everywhere, most inside the range
                if burn {
                    r -= r * k;
                    g -= g * k;
                    b -= b * k;
                } else {
                    r += (255.0 - r) * k;
                    g += (255.0 - g) * k;
                    b += (255.0 - b) * k;
                }
                dst[..3].copy_from_slice(&[clamp_byte(r), clamp_byte(g), clamp_byte(b)]);
            }
            RetouchOp::Blur { radius } => {
                if let Some((r, g, b, a)) = local_mean(scene, x, y, radius.clamp(1, 8), 1, None) {
                    lerp(dst, own, [clamp_byte(r), clamp_byte(g), clamp_byte(b), clamp_byte(a)], strength);
                }
            }
            RetouchOp::Sharpen { radius, amount } => {
                if own[3] == 0 {
                    return;
                }
                if let Some((mr, mg, mb, _)) = local_mean(scene, x, y, radius.clamp(1, 8), 1, None) {
                    let k = amount * strength;
                    dst[0] = clamp_byte(f64::from(own[0]) + (f64::from(own[0]) - mr) * k);
                    dst[1] = clamp_byte(f64::from(own[1]) + (f64::from(own[1]) - mg) * k);
                    dst[2] = clamp_byte(f64::from(own[2]) + (f64::from(own[2]) - mb) * k);
                }
            }
            RetouchOp::Highlight { color, amount } => {
                let t = strength * amount;
                for (d, c) in dst.iter_mut().zip([color.r, color.g, color.b]) {
                    *d = clamp_byte(f64::from(*d) + (f64::from(c) - f64::from(*d)) * t);
                }
            }
        }
    }
}
