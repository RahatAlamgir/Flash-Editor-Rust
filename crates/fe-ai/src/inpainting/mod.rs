//! LaMa fill for Remove / Fill Selection and the Remove Brush (C# `Inpainter`). Removes things from a picture: fills the
//! selected area with what would plausibly be there. The model is loaded on first use and kept in memory. It runs on the
//! processor: this model does not run correctly on the graphics card through DirectML.
//!
//! The model works on 512 x 512 pixels. A big selection shown to it whole would have to be shrunk and enlarged again,
//! which makes the fill soft. So a big selection is filled in pieces: every piece is seen at full detail, starting where
//! most of the surroundings are known, and each next piece also sees what was filled before.

mod detail;
mod geometry;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use fe_core::imaging::{PixelBuffer, PixelRect};
use ort::value::Tensor;

use crate::models::ModelInfo;
use crate::runtime::{self, Opened};

/// The longest side of a selection that is filled in one go (at full detail); bigger ones are filled piece by piece.
pub const SINGLE_FILL_MAX: i32 = 340;
/// The size of one piece when a big selection is filled piece by piece.
pub const PIECE_SIDE: i32 = 256;

pub struct Inpainter {
    path: PathBuf,
    model_size: usize,
    session: Mutex<Option<Opened>>,
}

/// The smallest rectangle around everything set in the mask.
pub fn bounds_of(mask: &[u8], width: usize, height: usize) -> PixelRect {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (width, height, None::<usize>, 0usize);
    for y in 0..height {
        let row = &mask[y * width..(y + 1) * width];
        let Some(first) = row.iter().position(|m| *m != 0) else { continue };
        let last = row.iter().rposition(|m| *m != 0).unwrap_or(first);
        min_x = min_x.min(first);
        max_x = Some(max_x.map_or(last, |m| m.max(last)));
        min_y = min_y.min(y);
        max_y = y;
    }
    match max_x {
        None => PixelRect::EMPTY,
        Some(mx) => PixelRect::new(min_x as i32, min_y as i32, (mx - min_x + 1) as i32, (max_y - min_y + 1) as i32),
    }
}

fn any_set(mask: &[u8], width: usize, rect: PixelRect) -> bool {
    (rect.y..rect.y + rect.height)
        .any(|y| mask[y as usize * width + rect.x as usize..][..rect.width as usize].iter().any(|m| *m != 0))
}

/// The piece whose surroundings are known best (so the fill grows inward from the edge of the selection).
fn best_piece(pieces: &[PixelRect], remaining: &[u8], width: usize, height: usize) -> usize {
    let bounds = PixelRect::new(0, 0, width as i32, height as i32);
    let (mut best, mut best_score) = (0, -1.0);
    for (i, piece) in pieces.iter().enumerate() {
        let ring = piece.inflate(24).intersect(&bounds);
        let (mut known, mut count) = (0u64, 0u64);
        let mut y = ring.y;
        while y < ring.y + ring.height {
            let mut x = ring.x;
            while x < ring.x + ring.width {
                if !piece.contains(x, y) {
                    count += 1;
                    if remaining[y as usize * width + x as usize] == 0 {
                        known += 1;
                    }
                }
                x += 4;
            }
            y += 4;
        }
        let score = if count == 0 { 0.0 } else { known as f64 / count as f64 };
        if score > best_score {
            best_score = score;
            best = i;
        }
    }
    best
}

/// Paints `piece` (straight alpha) over `target` inside `area`.
fn put_over(target: &mut PixelBuffer, piece: &PixelBuffer, area: PixelRect) {
    let area = area.intersect(&target.bounds());
    let w = target.width() as usize;
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            let i = (y as usize * w + x as usize) * 4;
            let a = u32::from(piece.data()[i + 3]);
            if a == 0 {
                continue;
            }
            if a == 255 {
                let src = [piece.data()[i], piece.data()[i + 1], piece.data()[i + 2], 255];
                target.data_mut()[i..i + 4].copy_from_slice(&src);
                continue;
            }
            let ta = u32::from(target.data()[i + 3]);
            let out_a = a + ta * (255 - a) / 255;
            if out_a == 0 {
                continue;
            }
            for c in 0..3 {
                let v = (u32::from(piece.data()[i + c]) * a + u32::from(target.data()[i + c]) * ta * (255 - a) / 255)
                    / out_a;
                target.data_mut()[i + c] = v as u8;
            }
            target.data_mut()[i + 3] = out_a as u8;
        }
    }
}

impl Inpainter {
    pub fn new(model: &'static ModelInfo, path: PathBuf) -> Self {
        Self { path, model_size: model.input_size, session: Mutex::new(None) }
    }

    pub fn device_name(&self) -> String {
        self.session.lock().ok().and_then(|s| s.as_ref().map(|o| o.device.to_owned())).unwrap_or_default()
    }

    /// Fills the area of `image` where `mask` (0..255, picture sized) is set. Returns a picture of the same size that
    /// holds only the filled part (see-through everywhere else), or `None` when the mask is empty. `progress` gets 0..1.
    pub fn fill(
        &self,
        image: &PixelBuffer,
        mask: &[u8],
        progress: &dyn Fn(f64),
        cancel: &AtomicBool,
    ) -> Result<Option<PixelBuffer>, String> {
        let (w, h) = (image.width() as usize, image.height() as usize);
        if mask.len() != w * h {
            return Err("The mask must be as big as the picture.".into());
        }
        let area = bounds_of(mask, w, h);
        if area.is_empty() {
            return Ok(None);
        }
        if area.width.max(area.height) <= SINGLE_FILL_MAX {
            let one = self.fill_piece(image, mask, mask, area, cancel)?;
            let one = one.map(|mut f| {
                match_detail(&mut f, image, mask, area);
                f
            });
            progress(1.0);
            return Ok(one);
        }
        // Cut the selection into pieces and fill them one after another.
        let mut pieces = Vec::new();
        let mut y = area.y;
        while y < area.y + area.height {
            let mut x = area.x;
            while x < area.x + area.width {
                let rect = PixelRect::new(
                    x,
                    y,
                    PIECE_SIDE.min(area.x + area.width - x),
                    PIECE_SIDE.min(area.y + area.height - y),
                );
                if any_set(mask, w, rect) {
                    pieces.push(rect);
                }
                x += PIECE_SIDE;
            }
            y += PIECE_SIDE;
        }
        let mut work = image.clone(); // the picture as it gets filled
        let mut remaining = mask.to_vec(); // what is still empty
        let mut result = PixelBuffer::new(w as u32, h as u32);
        let total = pieces.len();
        let mut done = 0;
        while !pieces.is_empty() {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled.".into());
            }
            let rect = pieces.remove(best_piece(&pieces, &remaining, w, h));
            let mut piece_mask = vec![0u8; mask.len()];
            for y in rect.y..rect.y + rect.height {
                let from = y as usize * w + rect.x as usize;
                piece_mask[from..from + rect.width as usize]
                    .copy_from_slice(&remaining[from..from + rect.width as usize]);
            }
            let piece_area = bounds_of(&piece_mask, w, h);
            if !piece_area.is_empty() {
                // The model treats everything still empty as a hole, but only this piece of its answer is kept.
                if let Some(filled) = self.fill_piece(&work, &remaining, &piece_mask, piece_area, cancel)? {
                    let touched = piece_area.inflate(geometry::dilation_for(piece_area) + 6);
                    put_over(&mut result, &filled, touched);
                    put_over(&mut work, &filled, touched);
                }
            }
            for y in rect.y..rect.y + rect.height {
                let from = y as usize * w + rect.x as usize;
                remaining[from..from + rect.width as usize].fill(0);
            }
            done += 1;
            progress(f64::from(done) / total as f64);
        }
        match_detail(&mut result, image, mask, area);
        Ok(Some(result))
    }

    /// One model run. `hole` is everything the model must treat as empty; `keep` (inside `keep_area`) says which part of
    /// its answer is returned.
    fn fill_piece(
        &self,
        image: &PixelBuffer,
        hole: &[u8],
        keep: &[u8],
        keep_area: PixelRect,
        cancel: &AtomicBool,
    ) -> Result<Option<PixelBuffer>, String> {
        let (w, h) = (image.width() as usize, image.height() as usize);
        // The model is shown a bit more than was selected, so no edge of the removed object is left.
        let widen = geometry::dilation_for(keep_area);
        let wide_hole = geometry::dilate(hole, w, h, bounds_of(hole, w, h), widen);
        let wide_keep =
            if std::ptr::eq(hole, keep) { wide_hole.clone() } else { geometry::dilate(keep, w, h, keep_area, widen) };
        let wide_area = keep_area.inflate(widen).intersect(&PixelRect::new(0, 0, w as i32, h as i32));
        let window = geometry::plan_window(w as i32, h as i32, wide_area, self.model_size as i32);
        let (input, hole_input) = geometry::prepare(image, &wide_hole, window, self.model_size);
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled.".into());
        }
        let mut answer = self.run(input, hole_input)?;
        for v in &mut answer {
            *v /= 255.0; // the model answers with 0..255
        }
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled.".into());
        }
        Ok(Some(geometry::compose(w, h, &wide_keep, wide_area, &answer, window, self.model_size, 3)))
    }

    fn run(&self, image: Vec<f32>, mask: Vec<f32>) -> Result<Vec<f32>, String> {
        let mut guard = self.session.lock().map_err(|_| "The AI model is busy.".to_owned())?; // one run at a time
        if guard.is_none() {
            *guard = Some(runtime::open(&self.path, false)?);
        }
        let opened = guard.as_mut().ok_or("The AI model is not loaded.")?;
        let s = self.model_size;
        let img = Tensor::from_array(([1usize, 3, s, s], image)).map_err(|e| e.to_string())?;
        let msk = Tensor::from_array(([1usize, 1, s, s], mask)).map_err(|e| e.to_string())?;
        let outputs = opened
            .session
            .run(ort::inputs!["image" => img, "mask" => msk])
            .map_err(|e| format!("The AI model failed: {e}"))?;
        let (_, data) = outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        Ok(data.to_vec())
    }

    pub fn release(&self) {
        if let Ok(mut s) = self.session.lock() {
            *s = None;
        }
    }
}

/// Sharpens the fill when the picture around it has clearly more fine detail than the (smooth) fill.
fn match_detail(fill: &mut PixelBuffer, image: &PixelBuffer, mask: &[u8], area: PixelRect) {
    let w = image.width() as usize;
    let around = area.inflate(80);
    let near = area.inflate(12);
    let outside =
        detail::detail_strength(image, |x, y| mask[y as usize * w + x as usize] == 0 && !near.contains(x, y), around);
    let inside = detail::detail_strength(fill, |x, y| fill.data()[(y as usize * w + x as usize) * 4 + 3] == 255, area);
    detail::sharpen(fill, image, detail::amount_for(inside, outside));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bounds_of_a_mask_are_the_smallest_rectangle_around_it() {
        let mut mask = vec![0u8; 10 * 8];
        assert!(bounds_of(&mask, 10, 8).is_empty());
        mask[2 * 10 + 3] = 255;
        mask[5 * 10 + 7] = 1;
        assert_eq!(bounds_of(&mask, 10, 8), PixelRect::new(3, 2, 5, 4));
    }
}
