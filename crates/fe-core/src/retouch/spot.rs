//! Spot Healing (C# `SpotHeal`): finds a clean place near a blemish to heal it from, without the user choosing one.

use crate::imaging::{PixelRect, TileMap};

/// Looks around `area` (the painted blemish) for the place whose surroundings look most like the surroundings of the
/// blemish, and returns how far it is `(dx, dy)`, or `None` when there is room for no candidate.
pub fn find_offset(picture: &TileMap, area: PixelRect, canvas: PixelRect) -> Option<(i32, i32)> {
    if area.is_empty() {
        return None;
    }
    let size = area.width.max(area.height);
    let ring = (size / 3).clamp(4, 20);
    let around = area.inflate(ring).intersect(&canvas); // the blemish with a band of its surroundings
    let reach = f64::from(size + 2 * ring);
    let mut best = None;
    let mut best_score = f64::MAX;
    for distance in [1.1, 1.6, 2.3, 3.2] {
        for k in 0..16 {
            let angle = f64::from(k) * std::f64::consts::PI / 8.0;
            let dx = (angle.cos() * reach * distance).round_ties_even() as i32;
            let dy = (angle.sin() * reach * distance).round_ties_even() as i32;
            let moved = PixelRect::new(around.x + dx, around.y + dy, around.width, around.height);
            if moved.intersect(&canvas) != moved {
                continue; // the copied place must be on the picture
            }
            let score = band_difference(picture, around, area, dx, dy);
            if score < best_score {
                best_score = score;
                best = Some((dx, dy));
            }
        }
    }
    best
}

/// How different the band around the blemish is from the same band at the candidate place (smaller = more alike).
fn band_difference(p: &TileMap, around: PixelRect, hole: PixelRect, dx: i32, dy: i32) -> f64 {
    let (mut sum, mut count) = (0.0, 0u64);
    let mut y = around.y;
    while y < around.y + around.height {
        let mut x = around.x;
        while x < around.x + around.width {
            if !hole.contains(x, y) {
                let (a, b) = (p.pixel(x, y), p.pixel(x + dx, y + dy));
                let d = |u: u8, v: u8| (f64::from(u) - f64::from(v)).powi(2);
                sum += d(a.r, b.r) + d(a.g, b.g) + d(a.b, b.b) + d(a.a, b.a);
                count += 1;
            }
            x += 2;
        }
        y += 2;
    }
    if count == 0 { f64::MAX } else { sum / count as f64 }
}
