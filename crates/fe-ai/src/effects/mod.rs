//! AI background removal (C# `AiBackgroundRemovalEffect`). Removes the background with a model that finds the main
//! subject (person, animal, product...), so it also works on busy backgrounds. The sliders only change how the edge is
//! cut; the model runs once per picture.
//!
//! The model only looks at the part of the layer that has pixels: a small photo on a big empty canvas would otherwise be
//! squeezed into a corner of what the model sees, and parts of the subject touching the empty area would get lost.

use std::sync::atomic::AtomicBool;

use fe_core::effects::gaussian_in_place;
use fe_core::imaging::PixelBuffer;
use fe_core::imaging::color_math::clamp_byte;

use crate::segmentation::{SubjectSegmenter, mask_ops};

/// A copy with transparent parts filled with white (the model only understands solid colors).
fn on_white(image: &PixelBuffer) -> PixelBuffer {
    let mut result = image.clone();
    for p in result.data_mut().chunks_exact_mut(4) {
        let a = f64::from(p[3]) / 255.0;
        if a >= 1.0 {
            continue;
        }
        for c in &mut p[..3] {
            *c = clamp_byte(f64::from(*c) * a + 255.0 * (1.0 - a));
        }
        p[3] = 255;
    }
    result
}

/// The picture with its background removed. `sharpness` 0..1, `shift` -1..1 (shrink / grow the edge), `feather` in pixels.
pub fn remove_background(
    segmenter: &SubjectSegmenter,
    source: &PixelBuffer,
    sharpness: f64,
    shift: f64,
    feather: f64,
    cancel: &AtomicBool,
) -> Result<PixelBuffer, String> {
    let area = source.visible_bounds();
    if area.is_empty() {
        return Ok(source.clone());
    }
    // What the model sees: only the visible area, with see-through pixels shown on white.
    let piece = on_white(&source.crop(area));
    let mut piece_mask = segmenter.predict_mask(&piece, cancel)?;
    mask_ops::adjust(&mut piece_mask, sharpness, shift);
    if feather > 0.0 {
        gaussian_in_place(&mut piece_mask, area.width as usize, area.height as usize, 1, feather);
    }
    // Put the mask back in place; outside the visible area everything is already transparent.
    let (w, aw) = (source.width() as usize, area.width as usize);
    let mut mask = vec![0f32; w * source.height() as usize];
    for y in 0..area.height as usize {
        let to = (area.y as usize + y) * w + area.x as usize;
        mask[to..to + aw].copy_from_slice(&piece_mask[y * aw..(y + 1) * aw]);
    }
    Ok(mask_ops::apply_as_alpha(source, &mask))
}
