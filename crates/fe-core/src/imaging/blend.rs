//! Per-pixel mixing math (straight alpha). Same formulas as the C# `BlendMath`.
//!
//! Every function works on one RGBA pixel given as a 4-byte slice.

use super::BlendMode;
use super::color_math::clamp_byte;

/// Paints color `rgb` with `alpha` (0..1) over the pixel `dst`.
#[inline]
pub fn over(dst: &mut [u8], rgb: [u8; 3], alpha: f64) {
    if alpha <= 0.0 {
        return;
    }
    if alpha >= 1.0 {
        dst[..3].copy_from_slice(&rgb);
        dst[3] = 255;
        return;
    }
    let keep = dst[3] as f64 / 255.0 * (1.0 - alpha);
    let out_a = alpha + keep;
    if out_a <= 0.0 {
        return;
    }
    for c in 0..3 {
        dst[c] = clamp_byte((rgb[c] as f64 * alpha + dst[c] as f64 * keep) / out_a);
    }
    dst[3] = clamp_byte(out_a * 255.0);
}

/// Paints pixel `src` over `dst` with an extra opacity (0..1).
#[inline]
pub fn over_pixel(dst: &mut [u8], src: &[u8], opacity: f64) {
    over(dst, [src[0], src[1], src[2]], src[3] as f64 / 255.0 * opacity);
}

/// Mixes pixel `a` towards pixel `b` by `t` (0..1) into `dst`, weighting colors by their alpha.
pub fn lerp_pixel(dst: &mut [u8], a: &[u8], b: &[u8], t: f64) {
    if t <= 0.0 {
        dst[..4].copy_from_slice(&a[..4]);
        return;
    }
    if t >= 1.0 {
        dst[..4].copy_from_slice(&b[..4]);
        return;
    }
    let (aa, ba) = (a[3] as f64, b[3] as f64);
    let out_a = aa + (ba - aa) * t;
    if out_a < 0.5 {
        dst[..4].fill(0);
        return;
    }
    for c in 0..3 {
        let (ca, cb) = (a[c] as f64 * aa, b[c] as f64 * ba);
        dst[c] = clamp_byte((ca + (cb - ca) * t) / out_a);
    }
    dst[3] = clamp_byte(out_a);
}

/// Mixes pixel `src` into `dst` with the given blend mode and opacity (0..1).
#[inline]
pub fn blend_pixel(dst: &mut [u8], src: &[u8], opacity: f64, mode: BlendMode) {
    if mode == BlendMode::Normal {
        over_pixel(dst, src, opacity);
        return;
    }
    let sa = src[3] as f64 / 255.0 * opacity;
    if sa <= 0.0 {
        return;
    }
    let ba = dst[3] as f64 / 255.0;
    if ba <= 0.0 {
        // Nothing below to blend with.
        over_pixel(dst, src, opacity);
        return;
    }
    let out_a = sa + ba * (1.0 - sa);
    for c in 0..3 {
        let cb = dst[c] as f64 / 255.0;
        let cs = src[c] as f64 / 255.0;
        let mixed = blend_channel(mode, cb, cs);
        let value = ((1.0 - sa) * ba * cb + (1.0 - ba) * sa * cs + sa * ba * mixed) / out_a;
        dst[c] = clamp_byte(value * 255.0);
    }
    dst[3] = clamp_byte(out_a * 255.0);
}

/// The blend formula for one channel: `cb` = below, `cs` = the layer (both 0..1).
#[inline]
pub fn blend_channel(mode: BlendMode, cb: f64, cs: f64) -> f64 {
    match mode {
        BlendMode::Multiply => cb * cs,
        BlendMode::Screen => cb + cs - cb * cs,
        BlendMode::Overlay => hard_light(cs, cb),
        BlendMode::HardLight => hard_light(cb, cs),
        BlendMode::SoftLight => soft_light(cb, cs),
        BlendMode::Darken => cb.min(cs),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::Difference => (cb - cs).abs(),
        BlendMode::Normal => cs,
    }
}

fn hard_light(cb: f64, cs: f64) -> f64 {
    if cs <= 0.5 { cb * 2.0 * cs } else { cb + (2.0 * cs - 1.0) - cb * (2.0 * cs - 1.0) }
}

fn soft_light(cb: f64, cs: f64) -> f64 {
    if cs <= 0.5 {
        return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    let d = if cb <= 0.25 { ((16.0 * cb - 12.0) * cb + 4.0) * cb } else { cb.sqrt() };
    cb + (2.0 * cs - 1.0) * (d - cb)
}
