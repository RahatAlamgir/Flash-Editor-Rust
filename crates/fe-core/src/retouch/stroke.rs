//! A retouching stroke and area (C# `BrushStroke` with an operation, `RetouchRegion`). Like a paint stroke the result is
//! always recomputed from the untouched pixels, so overlapping dabs never build up and the stroke is one undo step.

use rayon::prelude::*;

use super::ops::{RetouchOp, Scene};
use crate::imaging::{PixelRect, PointD, TILE_SIZE, TileKey, TileMap};
use crate::paint::brush::BrushSettings;
use crate::paint::coverage::Coverage;
use crate::selection::Selection;

pub struct RetouchStroke {
    coverage: Coverage,
    original: TileMap,
    op: RetouchOp,
    selection: Option<Selection>,
    canvas: PixelRect,
}

impl RetouchStroke {
    pub fn new(
        pixels: &TileMap,
        brush: BrushSettings,
        op: RetouchOp,
        selection: Option<Selection>,
        canvas: PixelRect,
    ) -> Self {
        Self { coverage: Coverage::new(brush, canvas), original: pixels.clone(), op, selection, canvas }
    }

    /// The pixels before the stroke.
    pub fn original(&self) -> &TileMap {
        &self.original
    }

    pub fn total(&self) -> PixelRect {
        self.coverage.total()
    }

    /// Adds the next mouse position and updates `target`. Returns the area that changed.
    pub fn add_point(&mut self, target: &mut TileMap, point: PointD) -> PixelRect {
        let dirty = self.coverage.add_point(point);
        if !dirty.is_empty() {
            self.composite(target, dirty, self.op);
        }
        dirty
    }

    /// Does everything touched so far again with another operation (Spot Healing paints a tint while the mouse is down,
    /// then heals the same area when it is released). Returns the area that changed.
    pub fn reapply(&mut self, target: &mut TileMap, op: RetouchOp) -> PixelRect {
        self.op = op;
        let total = self.coverage.total();
        if !total.is_empty() {
            self.composite(target, total, op);
        }
        total
    }

    fn composite(&self, target: &mut TileMap, rect: PixelRect, op: RetouchOp) {
        let opacity = self.coverage.brush().opacity;
        let painted = |x: i32, y: i32| self.coverage.at(x, y) > 0.0;
        let scene = Scene { original: &self.original, bounds: self.canvas, painted: &painted };
        for key in TileKey::covering(rect) {
            let Some(cov) = self.coverage.tile(key) else { continue };
            let part = key.rect().intersect(&rect);
            let tr = key.rect();
            // Rows of the tile in parallel: each pixel only reads the untouched original.
            let rows: Vec<Vec<[u8; 4]>> = (part.y..part.y + part.height)
                .into_par_iter()
                .map(|y| {
                    (part.x..part.x + part.width)
                        .map(|x| {
                            let li = ((y - tr.y) * TILE_SIZE + (x - tr.x)) as usize;
                            let mut px = self.original.pixel(x, y).to_array();
                            let mut s = f64::from(cov[li]) * opacity;
                            if let Some(sel) = &self.selection {
                                s *= sel.coverage(x, y);
                            }
                            if s > 0.0 {
                                op.apply(&scene, x, y, &mut px, s.min(1.0));
                            }
                            px
                        })
                        .collect()
                })
                .collect();
            for (dy, row) in rows.iter().enumerate() {
                for (dx, px) in row.iter().enumerate() {
                    target.set_pixel(part.x + dx as i32, part.y + dy as i32, crate::imaging::ColorRgba::from_slice(px));
                }
            }
        }
    }
}

/// Applies an operation to a whole area (the Patch tool) instead of along a brush stroke: the pixels of `rect` become
/// the result of `op` on `original` where `region` is selected (and `limit`, if given); the others are put back.
/// Returns the processed rectangle.
pub fn apply_region(
    target: &mut TileMap,
    original: &TileMap,
    op: RetouchOp,
    region: &Selection,
    limit: Option<&Selection>,
    rect: PixelRect,
    canvas: PixelRect,
) -> PixelRect {
    let rect = rect.intersect(&canvas);
    if rect.is_empty() {
        return rect;
    }
    let painted = |x: i32, y: i32| region.coverage(x, y) > 0.0;
    let scene = Scene { original, bounds: canvas, painted: &painted };
    let rows: Vec<Vec<[u8; 4]>> = (rect.y..rect.y + rect.height)
        .into_par_iter()
        .map(|y| {
            (rect.x..rect.x + rect.width)
                .map(|x| {
                    let mut px = original.pixel(x, y).to_array();
                    let s = region.coverage(x, y) * limit.map_or(1.0, |l| l.coverage(x, y));
                    if s > 0.0 {
                        op.apply(&scene, x, y, &mut px, s);
                    }
                    px
                })
                .collect()
        })
        .collect();
    for (dy, row) in rows.iter().enumerate() {
        for (dx, px) in row.iter().enumerate() {
            target.set_pixel(rect.x + dx as i32, rect.y + dy as i32, crate::imaging::ColorRgba::from_slice(px));
        }
    }
    rect
}
