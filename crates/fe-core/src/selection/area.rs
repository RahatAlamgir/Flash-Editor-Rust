//! `Selection`: the selected area of the canvas (C# `Selection`).
//!
//! The C# version keeps one byte for every canvas pixel. Here the mask only covers the selection's bounds
//! (everything outside is unselected), so a small selection on a huge picture costs little memory.

use super::{outline, polygon};
use crate::imaging::{PixelRect, PointD};

/// How a new selection is combined with the existing one (Shift = add, Alt = subtract).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineMode {
    Replace,
    Add,
    Subtract,
}

/// A selected area: a mask (0 = not selected, 255 = selected, values between = partly) plus outline polygons
/// for the marching ants. Never empty (constructors return `None` instead). Selections do not change once made.
#[derive(Clone, Debug)]
pub struct Selection {
    canvas_w: u32,
    canvas_h: u32,
    /// The smallest rectangle holding every selected pixel; always inside the canvas.
    bounds: PixelRect,
    /// One byte per pixel of `bounds`, row by row.
    mask: Vec<u8>,
    /// Outline polygons in canvas pixels (even-odd).
    outline: Vec<Vec<PointD>>,
}

impl Selection {
    fn canvas(w: u32, h: u32) -> PixelRect {
        PixelRect::new(0, 0, w as i32, h as i32)
    }

    /// Makes a selection from a mask over `rect`, trimming it to its bounds. `None` when nothing is selected.
    fn create(w: u32, h: u32, rect: PixelRect, mask: Vec<u8>, outline: Option<Vec<Vec<PointD>>>) -> Option<Self> {
        let bounds = mask_bounds(&mask, rect);
        if bounds.is_empty() {
            return None;
        }
        let mask = if bounds == rect { mask } else { crop(&mask, rect, bounds) };
        let outline = outline.unwrap_or_else(|| outline::trace(&mask, bounds, 128));
        Some(Self { canvas_w: w, canvas_h: h, bounds, mask, outline })
    }

    /// A rectangle (cut to the canvas).
    pub fn from_rect(rect: PixelRect, canvas_w: u32, canvas_h: u32) -> Option<Self> {
        let rect = rect.intersect(&Self::canvas(canvas_w, canvas_h));
        if rect.is_empty() {
            return None;
        }
        let mask = vec![255; rect.area() as usize];
        Self::create(canvas_w, canvas_h, rect, mask, Some(vec![rect_polygon(rect)]))
    }

    /// The whole canvas.
    pub fn all(canvas_w: u32, canvas_h: u32) -> Self {
        Self::from_rect(Self::canvas(canvas_w, canvas_h), canvas_w, canvas_h).expect("the canvas is never empty")
    }

    /// The inside of a polygon (at least 3 points; it may reach past the canvas).
    pub fn from_polygon(points: &[PointD], canvas_w: u32, canvas_h: u32) -> Option<Self> {
        if points.len() < 3 {
            return None;
        }
        let polygons = vec![points.to_vec()];
        let rect = polygon::polygons_bounds(&polygons).intersect(&Self::canvas(canvas_w, canvas_h));
        if rect.is_empty() {
            return None;
        }
        let mut mask = vec![0; rect.area() as usize];
        polygon::fill(&mut mask, rect, &polygons);
        Self::create(canvas_w, canvas_h, rect, mask, Some(polygons))
    }

    /// An ellipse (or circle) fitting `rect`, drawn as a polygon like in C#.
    pub fn from_ellipse(rect: PixelRect, canvas_w: u32, canvas_h: u32) -> Option<Self> {
        if rect.width < 1 || rect.height < 1 {
            return None;
        }
        Self::from_polygon(&ellipse_points(rect), canvas_w, canvas_h)
    }

    /// A selection from a mask covering `rect` (soft values allowed). `rect` must lie inside the canvas.
    pub fn from_mask(mask: Vec<u8>, rect: PixelRect, canvas_w: u32, canvas_h: u32) -> Option<Self> {
        assert_eq!(mask.len(), rect.area().max(0) as usize, "the mask does not match its rectangle");
        let canvas = Self::canvas(canvas_w, canvas_h);
        if rect.intersect(&canvas) != rect {
            let inside = rect.intersect(&canvas);
            if inside.is_empty() {
                return None;
            }
            return Self::create(canvas_w, canvas_h, inside, crop(&mask, rect, inside), None);
        }
        Self::create(canvas_w, canvas_h, rect, mask, None)
    }

    /// Combines two selections (`None` = nothing selected).
    pub fn combine(existing: Option<&Self>, added: Option<&Self>, mode: CombineMode) -> Option<Self> {
        let Some(existing) = existing.filter(|_| mode != CombineMode::Replace) else {
            return if mode == CombineMode::Subtract { None } else { added.cloned() };
        };
        let Some(added) = added else { return Some(existing.clone()) };
        let (w, h) = (existing.canvas_w, existing.canvas_h);
        match mode {
            CombineMode::Add => {
                let rect = existing.bounds.union(&added.bounds);
                let mut mask = existing.mask_over(rect);
                let other = added.mask_over(rect);
                mask.iter_mut().zip(&other).for_each(|(m, a)| *m = (*m).max(*a));
                Self::create(w, h, rect, mask, None)
            }
            _ => {
                let rect = existing.bounds;
                let other = added.mask_over(rect);
                let mut mask = existing.mask.clone();
                mask.iter_mut().zip(&other).for_each(|(m, a)| *m = (*m as u32 * (255 - *a as u32) / 255) as u8);
                Self::create(w, h, rect, mask, None)
            }
        }
    }

    /// Everything that was not selected, and the other way round.
    pub fn invert(&self) -> Option<Self> {
        let canvas = Self::canvas(self.canvas_w, self.canvas_h);
        let mut mask = self.mask_over(canvas);
        mask.iter_mut().for_each(|m| *m = 255 - *m);
        // The canvas border plus the old outline, drawn with even-odd, is the inverted area.
        let mut outline = vec![rect_polygon(canvas)];
        outline.extend(self.outline.iter().cloned());
        Self::create(self.canvas_w, self.canvas_h, canvas, mask, Some(outline))
    }

    /// Moved by whole pixels. Parts moved off the canvas are dropped.
    pub fn offset(&self, dx: i32, dy: i32) -> Option<Self> {
        let moved = self.bounds.offset(dx, dy);
        let inside = moved.intersect(&Self::canvas(self.canvas_w, self.canvas_h));
        if inside.is_empty() {
            return None;
        }
        let mask = if inside == moved { self.mask.clone() } else { crop(&self.mask, moved, inside) };
        let outline = self.outline.iter().map(|poly| poly.iter().map(|p| p.offset(dx as f64, dy as f64)).collect());
        Self::create(self.canvas_w, self.canvas_h, inside, mask, Some(outline.collect()))
    }

    pub fn canvas_width(&self) -> u32 {
        self.canvas_w
    }

    pub fn canvas_height(&self) -> u32 {
        self.canvas_h
    }

    /// The smallest rectangle holding every selected pixel.
    pub fn bounds(&self) -> PixelRect {
        self.bounds
    }

    /// The mask over [`Self::bounds`], row by row.
    pub fn mask(&self) -> &[u8] {
        &self.mask
    }

    /// Outline polygons (canvas pixels, even-odd).
    pub fn outline(&self) -> &[Vec<PointD>] {
        &self.outline
    }

    /// True when the outline is one rectangle that is fully selected (drawn without a mask texture).
    pub fn is_rectangle(&self) -> bool {
        self.outline.len() == 1 && self.outline[0] == rect_polygon(self.bounds) && self.mask.iter().all(|m| *m == 255)
    }

    /// The mask value at a pixel (0..=255); 0 outside the selection.
    pub fn value(&self, x: i32, y: i32) -> u8 {
        if !self.bounds.contains(x, y) {
            return 0;
        }
        self.mask[((y - self.bounds.y) * self.bounds.width + (x - self.bounds.x)) as usize]
    }

    /// How much a pixel is selected, 0..1. Pixels outside the canvas are not selected.
    pub fn coverage(&self, x: i32, y: i32) -> f64 {
        self.value(x, y) as f64 / 255.0
    }

    pub fn is_selected(&self, x: i32, y: i32) -> bool {
        self.value(x, y) > 0
    }

    /// The mask over any rectangle (0 outside the selection).
    pub fn mask_over(&self, rect: PixelRect) -> Vec<u8> {
        if rect == self.bounds {
            return self.mask.clone();
        }
        let mut out = vec![0; rect.area().max(0) as usize];
        let part = rect.intersect(&self.bounds);
        for y in part.y..part.bottom() {
            let src = ((y - self.bounds.y) * self.bounds.width + (part.x - self.bounds.x)) as usize;
            let dst = ((y - rect.y) * rect.width + (part.x - rect.x)) as usize;
            out[dst..dst + part.width as usize].copy_from_slice(&self.mask[src..src + part.width as usize]);
        }
        out
    }

    /// Memory held by the mask.
    pub fn memory_bytes(&self) -> usize {
        self.mask.len()
    }
}

/// The four corners of a rectangle.
pub(crate) fn rect_polygon(r: PixelRect) -> Vec<PointD> {
    let (l, t, rr, b) = (r.x as f64, r.y as f64, r.right() as f64, r.bottom() as f64);
    vec![PointD::new(l, t), PointD::new(rr, t), PointD::new(rr, b), PointD::new(l, b)]
}

/// Points around an ellipse fitting `rect` (the C# step count: 24 to 720 depending on the size).
pub fn ellipse_points(rect: PixelRect) -> Vec<PointD> {
    let (cx, cy) = (rect.x as f64 + rect.width as f64 / 2.0, rect.y as f64 + rect.height as f64 / 2.0);
    let (rx, ry) = (rect.width as f64 / 2.0, rect.height as f64 / 2.0);
    let steps = ((rx + ry) * 1.5).clamp(24.0, 720.0) as usize;
    (0..steps)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / steps as f64;
            PointD::new(cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

/// The smallest rectangle holding every non-zero byte of a mask over `rect` (empty if none).
fn mask_bounds(mask: &[u8], rect: PixelRect) -> PixelRect {
    let w = rect.width.max(0) as usize;
    if w == 0 {
        return PixelRect::EMPTY;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (usize::MAX, usize::MAX, 0, 0);
    let mut any = false;
    for (y, row) in mask.chunks_exact(w).enumerate() {
        let Some(first) = row.iter().position(|v| *v != 0) else { continue };
        let last = row.iter().rposition(|v| *v != 0).unwrap_or(first);
        any = true;
        min_y = min_y.min(y);
        max_y = y;
        min_x = min_x.min(first);
        max_x = max_x.max(last);
    }
    if !any {
        return PixelRect::EMPTY;
    }
    PixelRect::new(rect.x + min_x as i32, rect.y + min_y as i32, (max_x - min_x + 1) as i32, (max_y - min_y + 1) as i32)
}

/// The part `to` (inside `from`) of a mask over `from`.
fn crop(mask: &[u8], from: PixelRect, to: PixelRect) -> Vec<u8> {
    let mut out = Vec::with_capacity(to.area().max(0) as usize);
    for y in to.y..to.bottom() {
        let start = ((y - from.y) * from.width + (to.x - from.x)) as usize;
        out.extend_from_slice(&mask[start..start + to.width as usize]);
    }
    out
}
