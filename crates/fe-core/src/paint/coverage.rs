//! Where a stroke has painted and how strongly (C# `BrushStroke`, the dab part).
//!
//! Dabs along the mouse path are added into a coverage map by taking the maximum (never adding), so overlapping
//! dabs inside one stroke never build up darker spots and a 50 % stroke stays exactly 50 %. The map is kept in
//! the same 64 px tiles as the layers: only the tiles the brush touched cost memory.

use std::collections::HashMap;

use super::brush::{BrushSettings, BrushType};
use crate::imaging::{PixelRect, PointD, TILE_SIZE, TileKey};

const TILE_PIXELS: usize = (TILE_SIZE * TILE_SIZE) as usize;

/// The flat tip of the calligraphy brush: slanted 45 degrees, a quarter as thick as it is long.
const CHISEL_FLATNESS: f64 = 0.25;

/// The coverage (0..1) of every pixel a stroke has touched, plus the brush path that makes it.
pub struct Coverage {
    tiles: HashMap<TileKey, Vec<f32>>,
    brush: BrushSettings,
    /// Dabs are cut to this area (the canvas).
    clip: PixelRect,
    last: Option<PointD>,
    /// Distance left over from the previous segment (keeps the dab spacing even).
    carry: f64,
    random: u64,
    /// Everything touched so far.
    total: PixelRect,
}

fn smooth_step(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A repeatable pseudo-random number 0..1 for a pixel position (same pixel, same value, every time).
fn grain(x: i32, y: i32) -> f64 {
    let mut h = (x as u32).wrapping_mul(374_761_393).wrapping_add((y as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xFFFF) as f64 / 65535.0
}

/// Distance in the slanted flat tip's own shape (a thin ellipse turned 45 degrees).
fn chisel(dx: f64, dy: f64) -> f64 {
    let (c, s) = (std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2);
    let along = dx * c + dy * s;
    let across = -dx * s + dy * c;
    (along * along + across * across / (CHISEL_FLATNESS * CHISEL_FLATNESS)).sqrt()
}

impl Coverage {
    /// A new, empty stroke with this brush, painting only inside `clip`.
    pub fn new(brush: BrushSettings, clip: PixelRect) -> Self {
        static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0x9E37_79B9_7F4A_7C15);
        let random = SEED.fetch_add(0x9E37_79B9, std::sync::atomic::Ordering::Relaxed) | 1;
        Self { tiles: HashMap::new(), brush, clip, last: None, carry: 0.0, random, total: PixelRect::EMPTY }
    }

    pub fn brush(&self) -> &BrushSettings {
        &self.brush
    }

    /// Everything the stroke has touched so far.
    pub fn total(&self) -> PixelRect {
        self.total
    }

    /// The coverage of one tile (`None` = not touched).
    pub fn tile(&self, key: TileKey) -> Option<&[f32]> {
        self.tiles.get(&key).map(Vec::as_slice)
    }

    /// The coverage at a pixel.
    pub fn at(&self, x: i32, y: i32) -> f32 {
        let (key, lx, ly) = TileKey::containing(x, y);
        self.tiles.get(&key).map_or(0.0, |t| t[ly * TILE_SIZE as usize + lx])
    }

    /// Adds the next mouse position: dabs every 12 % of the brush size along the way. Returns the area that
    /// changed.
    pub fn add_point(&mut self, point: PointD) -> PixelRect {
        let spacing = (self.brush.size * 0.12).max(1.0);
        let mut dirty = PixelRect::EMPTY;
        match self.last {
            None => {
                dirty = self.stamp(point);
                self.carry = 0.0;
            }
            Some(last) => {
                let distance = last.distance_to(&point);
                let mut travelled = spacing - self.carry;
                while travelled <= distance {
                    let t = travelled / distance;
                    let at = PointD::new(last.x + (point.x - last.x) * t, last.y + (point.y - last.y) * t);
                    dirty = dirty.union(&self.stamp(at));
                    travelled += spacing;
                }
                self.carry = distance - (travelled - spacing);
            }
        }
        self.last = Some(point);
        self.total = self.total.union(&dirty);
        dirty
    }

    /// Raises the coverage in `rect` with `value(x, y)` (anything at or below 0 is skipped).
    fn raise(&mut self, rect: PixelRect, mut value: impl FnMut(i32, i32) -> f64) {
        for key in TileKey::covering(rect) {
            let tr = key.rect();
            let part = tr.intersect(&rect);
            let mut tile: Option<&mut Vec<f32>> = self.tiles.get_mut(&key);
            for y in part.y..part.bottom() {
                for x in part.x..part.right() {
                    let c = value(x, y);
                    if c <= 0.0 {
                        continue;
                    }
                    if tile.is_none() {
                        tile = Some(self.tiles.entry(key).or_insert_with(|| vec![0.0; TILE_PIXELS]));
                    }
                    let Some(t) = tile.as_deref_mut() else { continue };
                    let i = ((y - tr.y) * TILE_SIZE + (x - tr.x)) as usize;
                    if c as f32 > t[i] {
                        t[i] = c as f32;
                    }
                }
            }
        }
    }

    /// One dab of the brush type at `center`.
    fn stamp(&mut self, center: PointD) -> PixelRect {
        let radius = self.brush.radius();
        let kind = self.brush.kind;
        if matches!(kind, BrushType::Spray | BrushType::Splatter) {
            return self.stamp_scatter(center, radius, kind);
        }
        // How far from the center a dab reaches (a square sticks out into its corners).
        let reach = if kind == BrushType::Square { radius * 1.45 } else { radius };
        let rect = PixelRect::from_bounds(
            center.x - reach - 1.0,
            center.y - reach - 1.0,
            center.x + reach + 1.0,
            center.y + reach + 1.0,
        )
        .intersect(&self.clip);
        if rect.is_empty() {
            return rect;
        }
        let hardness = if kind == BrushType::Pencil { 1.0 } else { self.brush.hardness.clamp(0.0, 1.0) };
        let inner = radius * hardness;
        let falloff = (radius - inner).max(0.0001);
        self.raise(rect, |x, y| {
            let (dx, dy) = (x as f64 + 0.5 - center.x, y as f64 + 0.5 - center.y);
            // "Distance" from the center in the shape of the tip.
            let d = match kind {
                BrushType::Square => dx.abs().max(dy.abs()),
                BrushType::Calligraphy => chisel(dx, dy),
                _ => (dx * dx + dy * dy).sqrt(),
            };
            if d >= radius + 0.5 {
                return 0.0;
            }
            // Smooth falloff between the hard core and the radius, plus a 1 px anti-aliased rim.
            let soft = if d <= inner { 1.0 } else { 1.0 - smooth_step((d - inner) / falloff) };
            let mut c = soft * (radius + 0.5 - d).clamp(0.0, 1.0);
            // Texture: the grain depends on the pixel (not the dab), so overlapping dabs never change it.
            match kind {
                BrushType::Pencil => c *= 0.62 + 0.38 * grain(x, y),
                BrushType::Chalk => c *= if grain(x, y) > 0.38 { 0.55 + 0.45 * grain(y, x) } else { 0.0 },
                _ => {}
            }
            c
        });
        rect
    }

    fn next_random(&mut self) -> f64 {
        // xorshift64*
        self.random ^= self.random >> 12;
        self.random ^= self.random << 25;
        self.random ^= self.random >> 27;
        (self.random.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Spray (a mist of small dots, denser in the middle) and Splatter (a few blobs of different sizes).
    fn stamp_scatter(&mut self, center: PointD, radius: f64, kind: BrushType) -> PixelRect {
        let reach = if kind == BrushType::Splatter { radius * 1.6 } else { radius };
        let rect = PixelRect::from_bounds(
            center.x - reach - 2.0,
            center.y - reach - 2.0,
            center.x + reach + 2.0,
            center.y + reach + 2.0,
        )
        .intersect(&self.clip);
        if rect.is_empty() {
            return rect;
        }
        if kind == BrushType::Spray {
            let dots = ((radius * 2.2) as i32).clamp(6, 220);
            let dot_radius = if radius > 60.0 { 1.2 } else { 0.7 };
            for _ in 0..dots {
                let angle = self.next_random() * std::f64::consts::TAU;
                let distance = radius * self.next_random().sqrt();
                let strength = 0.45 + 0.55 * (1.0 - distance / radius); // fainter towards the edge
                self.blob(center.x + angle.cos() * distance, center.y + angle.sin() * distance, dot_radius, strength);
            }
        } else {
            let blobs = ((radius * 0.18) as i32).max(2);
            for _ in 0..blobs {
                let angle = self.next_random() * std::f64::consts::TAU;
                let distance = reach * self.next_random().sqrt();
                let size = (radius * (0.05 + 0.3 * self.next_random().powi(3))).max(0.8); // mostly small, a few big
                self.blob(center.x + angle.cos() * distance, center.y + angle.sin() * distance, size, 1.0);
            }
        }
        rect
    }

    /// A small round dot with a soft edge.
    fn blob(&mut self, cx: f64, cy: f64, radius: f64, strength: f64) {
        let area = PixelRect::from_bounds(cx - radius - 1.0, cy - radius - 1.0, cx + radius + 1.0, cy + radius + 1.0)
            .intersect(&self.clip);
        self.raise(area, |x, y| {
            let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
            (radius + 0.5 - d).clamp(0.0, 1.0) * strength
        });
    }
}
