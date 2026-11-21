//! The model fills with plausible but smooth texture. This measures how much fine detail (leaves, grain, pebbles) the
//! picture has around the filled area and sharpens the fill so that it matches, never more than a moderate amount (C#
//! `InpaintDetail`).

use fe_core::imaging::color_math::clamp_byte;
use fe_core::imaging::{PixelBuffer, PixelRect};

fn luma(d: &[u8], i: usize) -> f64 {
    0.299 * f64::from(d[i]) + 0.587 * f64::from(d[i + 1]) + 0.114 * f64::from(d[i + 2])
}

/// The average strength of fine detail in the pixels `which` selects (higher = sharper / more texture).
pub fn detail_strength(picture: &PixelBuffer, which: impl Fn(i32, i32) -> bool, region: PixelRect) -> f64 {
    let region = region.intersect(&picture.bounds());
    let (w, h) = (picture.width() as i32, picture.height() as i32);
    let d = picture.data();
    let (mut sum, mut count) = (0.0, 0u64);
    let mut y = region.y.max(1);
    while y < (h - 1).min(region.y + region.height) {
        let mut x = region.x.max(1);
        while x < (w - 1).min(region.x + region.width) {
            if which(x, y) {
                let at = |xx: i32, yy: i32| luma(d, ((yy * w + xx) * 4) as usize);
                sum += (4.0 * at(x, y) - at(x - 1, y) - at(x + 1, y) - at(x, y - 1) - at(x, y + 1)).abs();
                count += 1;
            }
            x += 2;
        }
        y += 2;
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// How much to sharpen a fill with detail strength `fill` to match surroundings with `around` (0 = leave it).
pub fn amount_for(fill: f64, around: f64) -> f64 {
    if fill <= 0.01 || around <= 0.01 {
        return 0.0;
    }
    let ratio = around / fill;
    if ratio < 1.15 { 0.0 } else { ((ratio - 1.0) * 0.9).min(1.8) }
}

/// Sharpens the filled part (`fill` has its pixels where alpha is set) with an unsharp mask. The neighbours the blur
/// sees come from `original` where nothing was filled, so the edge of the fill does not get a dark ring.
pub fn sharpen(fill: &mut PixelBuffer, original: &PixelBuffer, amount: f64) {
    if amount <= 0.01 {
        return;
    }
    let w = fill.width() as usize;
    let area = fill.visible_bounds();
    if area.is_empty() {
        return;
    }
    let region = area.inflate(4).intersect(&fill.bounds());
    // The picture as it looks with the fill on it.
    let mut mixed = original.clone();
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let i = (y as usize * w + x as usize) * 4;
            let a = u32::from(fill.data()[i + 3]);
            if a == 0 {
                continue;
            }
            for c in 0..3 {
                let v = (u32::from(fill.data()[i + c]) * a + u32::from(mixed.data()[i + c]) * (255 - a)) / 255;
                mixed.data_mut()[i + c] = v as u8;
            }
        }
    }
    let blurred = blur(&mixed, region, 2);
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let i = (y as usize * w + x as usize) * 4;
            if fill.data()[i + 3] == 0 {
                continue;
            }
            for c in 0..3 {
                let m = f64::from(mixed.data()[i + c]);
                fill.data_mut()[i + c] = clamp_byte(m + amount * (m - f64::from(blurred.data()[i + c])));
            }
        }
    }
}

/// A soft blur (box blur twice) of the colors inside the region.
fn blur(source: &PixelBuffer, region: PixelRect, radius: i32) -> PixelBuffer {
    let w = source.width() as i32;
    let h = source.height() as i32;
    let mut temp = source.clone();
    let mut result = source.clone();
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let (from, to) = ((x - radius).max(0), (x + radius).min(w - 1));
            let n = (to - from + 1) as u32;
            for c in 0..3 {
                let sum: u32 = (from..=to).map(|k| u32::from(source.data()[((y * w + k) * 4) as usize + c])).sum();
                temp.data_mut()[((y * w + x) * 4) as usize + c] = (sum / n) as u8;
            }
        }
    }
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let (from, to) = ((y - radius).max(0), (y + radius).min(h - 1));
            let n = (to - from + 1) as u32;
            for c in 0..3 {
                let sum: u32 = (from..=to).map(|k| u32::from(temp.data()[((k * w + x) * 4) as usize + c])).sum();
                result.data_mut()[((y * w + x) * 4) as usize + c] = (sum / n) as u8;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fill_with_less_detail_than_its_surroundings_is_sharpened_a_little() {
        assert_eq!(amount_for(10.0, 10.5), 0.0);
        assert!(amount_for(10.0, 20.0) > 0.5);
        assert_eq!(amount_for(1.0, 100.0), 1.8);
        assert_eq!(amount_for(0.0, 5.0), 0.0);
    }
}
