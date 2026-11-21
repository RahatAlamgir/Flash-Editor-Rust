//! One brush, eraser or mask-brush stroke, from mouse-down to mouse-up (C# `BrushStroke`).
//!
//! The result is always recomputed from the untouched pixels at the start of the stroke, using the stroke's
//! coverage (see [`Coverage`]), and only in the tiles the brush touched. The pixels before the stroke are kept for
//! free (shared tiles), so the stroke becomes one cheap undo step.

use super::brush::BrushSettings;
use super::coverage::Coverage;
use crate::imaging::{
    ColorRgba, PixelRect, PointD, TILE_SIZE, TileGrid, TileKey, TileMap, color_math::clamp_byte, over,
};
use crate::selection::Selection;

/// What a stroke does to the pixels it covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    /// Paints a color (Brush).
    Paint(ColorRgba),
    /// Makes pixels transparent (Eraser).
    Erase,
}

/// The strength of the stroke at a pixel: coverage x opacity x selection.
fn strength(cov: f32, opacity: f64, selection: Option<&Selection>, x: i32, y: i32) -> f64 {
    let s = cov as f64 * opacity;
    match selection {
        Some(sel) if s > 0.0 => s * sel.coverage(x, y),
        _ => s,
    }
}

/// Calls `f(key, part, coverage tile)` for every tile in `rect` the stroke has touched.
fn touched_tiles(coverage: &Coverage, rect: PixelRect, mut f: impl FnMut(TileKey, PixelRect, &[f32])) {
    for key in TileKey::covering(rect) {
        if let Some(cov) = coverage.tile(key) {
            f(key, key.rect().intersect(&rect), cov);
        }
    }
}

/// A Brush or Eraser stroke on a layer's pixels.
pub struct PaintStroke {
    coverage: Coverage,
    original: TileMap,
    effect: Effect,
    selection: Option<Selection>,
}

impl PaintStroke {
    /// Starts a stroke on `pixels` (a copy of them is kept; it costs nothing). `clip` is the canvas.
    pub fn new(
        pixels: &TileMap,
        brush: BrushSettings,
        effect: Effect,
        selection: Option<Selection>,
        clip: PixelRect,
    ) -> Self {
        Self { coverage: Coverage::new(brush, clip), original: pixels.clone(), effect, selection }
    }

    /// The pixels before the stroke.
    pub fn original(&self) -> &TileMap {
        &self.original
    }

    /// Everything the stroke touched so far.
    pub fn total(&self) -> PixelRect {
        self.coverage.total()
    }

    /// Adds the next mouse position and updates `target`. Returns the area that changed.
    pub fn add_point(&mut self, target: &mut TileMap, point: PointD) -> PixelRect {
        let dirty = self.coverage.add_point(point);
        if !dirty.is_empty() {
            self.composite(target, dirty);
        }
        dirty
    }

    /// Recomputes the pixels of `rect` from the starting pixels.
    fn composite(&self, target: &mut TileMap, rect: PixelRect) {
        let opacity = self.coverage.brush().opacity;
        let selection = self.selection.as_ref();
        touched_tiles(&self.coverage, rect, |key, part, cov| {
            let tr = key.rect();
            let original = self.original.tile(key);
            let tile = target.tile_mut(key);
            for y in part.y..part.bottom() {
                for x in part.x..part.right() {
                    let li = ((y - tr.y) * TILE_SIZE + (x - tr.x)) as usize;
                    let i = li * 4;
                    let src: [u8; 4] = original.map_or([0; 4], |o| [o[i], o[i + 1], o[i + 2], o[i + 3]]);
                    let dst = &mut tile[i..i + 4];
                    dst.copy_from_slice(&src);
                    let s = strength(cov[li], opacity, selection, x, y);
                    if s <= 0.0 {
                        continue;
                    }
                    match self.effect {
                        Effect::Paint(c) => over(dst, [c.r, c.g, c.b], s * c.a as f64 / 255.0),
                        Effect::Erase => {
                            dst[3] = clamp_byte(src[3] as f64 * (1.0 - s));
                            if dst[3] == 0 {
                                dst.copy_from_slice(&[0; 4]);
                            }
                        }
                    }
                }
            }
            target.drop_if_fill(key);
        });
    }
}

/// A Mask Brush stroke: moves mask values towards `value` (0 = hide, 255 = show).
pub struct MaskStroke {
    coverage: Coverage,
    original: TileGrid<1>,
    value: u8,
    selection: Option<Selection>,
}

impl MaskStroke {
    pub fn new(
        mask: &TileGrid<1>,
        brush: BrushSettings,
        value: u8,
        selection: Option<Selection>,
        clip: PixelRect,
    ) -> Self {
        Self { coverage: Coverage::new(brush, clip), original: mask.clone(), value, selection }
    }

    /// The grey a color paints on a mask (C#: 0.299 R + 0.587 G + 0.114 B).
    pub fn grey_of(color: ColorRgba) -> u8 {
        (0.299 * color.r as f64 + 0.587 * color.g as f64 + 0.114 * color.b as f64).round_ties_even().clamp(0.0, 255.0)
            as u8
    }

    pub fn total(&self) -> PixelRect {
        self.coverage.total()
    }

    /// Adds the next mouse position and updates `mask`. Returns the area that changed.
    pub fn add_point(&mut self, mask: &mut TileGrid<1>, point: PointD) -> PixelRect {
        let dirty = self.coverage.add_point(point);
        if dirty.is_empty() {
            return dirty;
        }
        let opacity = self.coverage.brush().opacity;
        let selection = self.selection.as_ref();
        let fill = self.original.fill_value()[0];
        touched_tiles(&self.coverage, dirty, |key, part, cov| {
            let tr = key.rect();
            let original = self.original.tile(key);
            let tile = mask.tile_mut(key);
            for y in part.y..part.bottom() {
                for x in part.x..part.right() {
                    let i = ((y - tr.y) * TILE_SIZE + (x - tr.x)) as usize;
                    let src = original.map_or(fill, |o| o[i]);
                    let s = strength(cov[i], opacity, selection, x, y).min(1.0);
                    tile[i] = if s <= 0.0 { src } else { clamp_byte(self.value as f64 * s + src as f64 * (1.0 - s)) };
                }
            }
            mask.drop_if_fill(key);
        });
        dirty
    }
}
