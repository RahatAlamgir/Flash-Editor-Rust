//! Color conversions (HSV, HSL) and rounding helpers. Same formulas as the C# `ColorMath`.

use super::ColorRgba;

/// Rounds and clamps to 0..=255 (halves round up, like the C# version).
pub fn clamp_byte(v: f64) -> u8 {
    if v <= 0.0 {
        0
    } else if v >= 255.0 {
        255
    } else {
        (v + 0.5) as u8
    }
}

/// Clamps an integer to 0..=255.
pub fn clamp_byte_i(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

/// Rounds like C# `Math.Round` (halves go to the even number). Used where C# results must match exactly.
pub fn round_even(v: f64) -> f64 {
    v.round_ties_even()
}

/// RGB (0..1) to hue (0..360), saturation and value (0..1).
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let d = max - min;
    let s = if max <= 0.0 { 0.0 } else { d / max };
    let h = if d <= 0.0 { 0.0 } else { hue_from(r, g, b, max, d) };
    (h, s, max)
}

/// Hue (degrees), saturation and value (0..1) to RGB (0..1).
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = ((h % 360.0) + 360.0) % 360.0 / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// RGB (0..1) to hue (0..360), saturation and lightness (0..1).
pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let d = max - min;
    let l = (max + min) / 2.0;
    if d <= 0.0 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    (hue_from(r, g, b, max, d), s, l)
}

/// Hue (degrees), saturation and lightness (0..1) to RGB (0..1).
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s <= 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hk = (((h % 360.0) + 360.0) % 360.0) / 360.0;
    (hue_to_channel(p, q, hk + 1.0 / 3.0), hue_to_channel(p, q, hk), hue_to_channel(p, q, hk - 1.0 / 3.0))
}

/// A color from hue (degrees), saturation and value (0..1).
pub fn from_hsv(h: f64, s: f64, v: f64, alpha: u8) -> ColorRgba {
    let (r, g, b) = hsv_to_rgb(h, s, v);
    ColorRgba::new(clamp_byte(r * 255.0), clamp_byte(g * 255.0), clamp_byte(b * 255.0), alpha)
}

fn hue_from(r: f64, g: f64, b: f64, max: f64, d: f64) -> f64 {
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h * 60.0
}

fn hue_to_channel(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 0.5 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
