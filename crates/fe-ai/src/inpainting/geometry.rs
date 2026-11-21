//! The picture math around the fill model, without any AI runtime (C# `InpaintGeometry`): choosing the window around
//! the selection, widening the mask, cutting out and resizing the model input, and blending the answer back with a
//! soft edge.

use fe_core::imaging::{PixelBuffer, PixelRect};
use fe_core::transform::{ResampleMode, resize};
use rayon::prelude::*;

use crate::segmentation::mask_ops;

/// Which part of the picture the model looks at: a square, in picture coordinates (may stick out of the picture).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub left: i32,
    pub top: i32,
    pub side: i32,
}

/// A square window around the selected area. The model works on 512 x 512, so a small selection gets a 512 pixel
/// window (full detail, no resizing); a big one gets a window one and a half times its size (shrunk to fit).
pub fn plan_window(image_w: i32, image_h: i32, area: PixelRect, model_size: i32) -> Window {
    let longest = area.width.max(area.height);
    let side = model_size.max((f64::from(longest) * 1.5).ceil() as i32);
    let mut left = area.x + area.width / 2 - side / 2;
    let mut top = area.y + area.height / 2 - side / 2;
    // Keep the window on the picture when it fits there; otherwise center it (the edges are padded).
    left = if side <= image_w { left.clamp(0, image_w - side) } else { (image_w - side) / 2 };
    top = if side <= image_h { top.clamp(0, image_h - side) } else { (image_h - side) / 2 };
    Window { left, top, side }
}

/// How far the selection is widened for the model: enough to cover the soft edge of an object and thin parts the
/// selection just missed; more for big ones.
pub fn dilation_for(area: PixelRect) -> i32 {
    ((f64::from(area.width.max(area.height)) * 0.04).round_ties_even() as i32).clamp(4, 30)
}

/// The mask made bigger by `radius` pixels in every direction, inside `region` only.
pub fn dilate(mask: &[u8], width: usize, height: usize, region: PixelRect, radius: i32) -> Vec<u8> {
    let mut result = mask.to_vec();
    let bounds = PixelRect::new(0, 0, width as i32, height as i32);
    let region = region.inflate(radius).intersect(&bounds);
    if region.is_empty() || radius <= 0 {
        return result;
    }
    // A square grows in two passes (along the rows, then along the columns).
    let mut rows = vec![0u8; mask.len()];
    rows.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y = y as i32;
        if y < region.y || y >= region.y + region.height {
            return;
        }
        for x in region.x..region.x + region.width {
            let from = (x - radius).max(0) as usize;
            let to = ((x + radius) as usize).min(width - 1);
            row[x as usize] =
                mask[y as usize * width + from..=y as usize * width + to].iter().copied().max().unwrap_or(0);
        }
    });
    result.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let yi = y as i32;
        if yi < region.y || yi >= region.y + region.height {
            return;
        }
        for x in region.x..region.x + region.width {
            let from = (yi - radius).max(0) as usize;
            let to = ((yi + radius) as usize).min(height - 1);
            row[x as usize] = (from..=to).map(|k| rows[k * width + x as usize]).max().unwrap_or(0);
        }
    });
    result
}

/// The window of the picture as the model wants it: planar RGB floats 0..1 (`image`, 3 x size x size) and the hole mask
/// (`mask`, size x size, 1 = fill this). Where the window sticks out of the picture the edge pixels are repeated and
/// nothing is asked to be filled.
pub fn prepare(image: &PixelBuffer, mask: &[u8], window: Window, model_size: usize) -> (Vec<f32>, Vec<f32>) {
    let side = window.side as usize;
    let (iw, ih) = (image.width() as i64, image.height() as i64);
    let mut crop = PixelBuffer::new(side as u32, side as u32);
    let mut hole = vec![0f32; side * side];
    let src = image.data();
    crop.data_mut().par_chunks_mut(side * 4).zip(hole.par_chunks_mut(side)).enumerate().for_each(|(y, (row, hrow))| {
        let sy = i64::from(window.top) + y as i64;
        let cy = sy.clamp(0, ih - 1);
        for x in 0..side {
            let sx = i64::from(window.left) + x as i64;
            let cx = sx.clamp(0, iw - 1);
            let si = ((cy * iw + cx) * 4) as usize;
            // The picture is flattened onto white first, so see-through places are not dark holes for the model.
            let a = u32::from(src[si + 3]);
            for c in 0..3 {
                row[x * 4 + c] = ((u32::from(src[si + c]) * a + 255 * (255 - a)) / 255) as u8;
            }
            row[x * 4 + 3] = 255;
            let inside = sx >= 0 && sy >= 0 && sx < iw && sy < ih;
            hrow[x] = if inside && mask[(cy * iw + cx) as usize] > 0 { 1.0 } else { 0.0 };
        }
    });
    let resized = if side == model_size {
        crop
    } else {
        resize(&crop, model_size as u32, model_size as u32, ResampleMode::Smooth)
    };
    let hole_small =
        if side == model_size { hole } else { mask_ops::resize_mask(&hole, side, side, model_size, model_size) };
    let plane = model_size * model_size;
    let mut tensor = vec![0f32; 3 * plane];
    let mut mask01 = vec![0f32; plane];
    let d = resized.data();
    for p in 0..plane {
        let i = p * 4;
        tensor[p] = f32::from(d[i]) / 255.0;
        tensor[plane + p] = f32::from(d[i + 1]) / 255.0;
        tensor[2 * plane + p] = f32::from(d[i + 2]) / 255.0;
        mask01[p] = if hole_small[p] >= 0.25 { 1.0 } else { 0.0 }; // a little generous, so no sliver of the object is left
    }
    (tensor, mask01)
}

#[allow(clippy::too_many_arguments)]
/// A picture of the same size as the original that holds only the filled part: the model's answer where the (widened)
/// selection was, with a soft edge, and nothing anywhere else. `output` is the model's picture (size x size, planar RGB,
/// values 0..1).
pub fn compose(
    image_w: usize,
    image_h: usize,
    mask: &[u8],
    area: PixelRect,
    output: &[f32],
    window: Window,
    model_size: usize,
    feather: i32,
) -> PixelBuffer {
    let plane = model_size * model_size;
    let mut answer = PixelBuffer::new(model_size as u32, model_size as u32);
    for (p, px) in answer.data_mut().chunks_exact_mut(4).enumerate() {
        px[0] = fe_core::imaging::color_math::clamp_byte(f64::from(output[p] * 255.0));
        px[1] = fe_core::imaging::color_math::clamp_byte(f64::from(output[plane + p] * 255.0));
        px[2] = fe_core::imaging::color_math::clamp_byte(f64::from(output[2 * plane + p] * 255.0));
        px[3] = 255;
    }
    if window.side as usize != model_size {
        answer = resize(&answer, window.side as u32, window.side as u32, ResampleMode::Bicubic);
    }
    // Soft edge: the widened mask, blurred a little, decides how much of the answer shows.
    let region = area.inflate(feather + 2).intersect(&PixelRect::new(0, 0, image_w as i32, image_h as i32));
    let soft = soften(mask, image_w, image_h, region, feather);
    let mut result = PixelBuffer::new(image_w as u32, image_h as u32);
    let side = window.side as usize;
    let src = answer.data();
    result.data_mut().par_chunks_mut(image_w * 4).enumerate().for_each(|(y, row)| {
        let yi = y as i32;
        if yi < region.y || yi >= region.y + region.height {
            return;
        }
        let wy = yi - window.top;
        if wy < 0 || wy >= window.side {
            return;
        }
        for x in region.x..region.x + region.width {
            let wx = x - window.left;
            if wx < 0 || wx >= window.side {
                continue;
            }
            let alpha = soft[y * image_w + x as usize];
            if alpha == 0 {
                continue;
            }
            let si = (wy as usize * side + wx as usize) * 4;
            let di = x as usize * 4;
            row[di..di + 3].copy_from_slice(&src[si..si + 3]);
            row[di + 3] = alpha;
        }
    });
    result
}

/// The mask with a soft edge (a box blur of the given radius, twice), only inside `region`.
fn soften(mask: &[u8], width: usize, _height: usize, region: PixelRect, radius: i32) -> Vec<u8> {
    let mut soft = vec![0u8; mask.len()];
    let (rx, ry, rr, rb) =
        (region.x as usize, region.y as usize, (region.x + region.width) as usize, (region.y + region.height) as usize);
    if radius <= 0 {
        for y in ry..rb {
            soft[y * width + rx..y * width + rr].copy_from_slice(&mask[y * width + rx..y * width + rr]);
        }
        return soft;
    }
    let r = radius as usize;
    let mut a = vec![0f32; mask.len()];
    let mut b = vec![0f32; mask.len()];
    for y in ry..rb {
        for x in rx..rr {
            a[y * width + x] = f32::from(mask[y * width + x]) / 255.0;
        }
    }
    for _ in 0..2 {
        for y in ry..rb {
            for x in rx..rr {
                let (from, to) = (rx.max(x.saturating_sub(r)), (rr - 1).min(x + r));
                let sum: f32 = (from..=to).map(|k| a[y * width + k]).sum();
                b[y * width + x] = sum / (to - from + 1) as f32;
            }
        }
        for y in ry..rb {
            for x in rx..rr {
                let (from, to) = (ry.max(y.saturating_sub(r)), (rb - 1).min(y + r));
                let sum: f32 = (from..=to).map(|k| b[k * width + x]).sum();
                a[y * width + x] = sum / (to - from + 1) as f32;
            }
        }
    }
    for y in ry..rb {
        for x in rx..rr {
            // Inside the original mask the answer must show completely; the blur only softens the outside edge.
            let i = y * width + x;
            let own = f32::from(mask[i]) / 255.0 * if a[i] > 0.5 { 1.0 } else { 0.0 };
            soft[i] = (a[i].max(own) * 255.0).round_ties_even().clamp(0.0, 255.0) as u8;
        }
    }
    soft
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_selection_gets_a_model_sized_window_on_the_picture() {
        let w = plan_window(1000, 800, PixelRect::new(900, 20, 30, 30), 512);
        assert_eq!(w.side, 512);
        assert!(w.left >= 0 && w.left + w.side <= 1000 && w.top == 0);
    }

    #[test]
    fn a_big_selection_gets_a_window_one_and_a_half_times_its_size() {
        assert_eq!(plan_window(3000, 3000, PixelRect::new(100, 100, 800, 400), 512).side, 1200);
    }

    #[test]
    fn dilation_grows_with_the_selection_within_limits() {
        assert_eq!(dilation_for(PixelRect::new(0, 0, 10, 10)), 4);
        assert_eq!(dilation_for(PixelRect::new(0, 0, 400, 100)), 16);
        assert_eq!(dilation_for(PixelRect::new(0, 0, 5000, 100)), 30);
    }

    #[test]
    fn a_mask_grows_by_the_radius_in_every_direction() {
        let mut mask = vec![0u8; 20 * 20];
        mask[10 * 20 + 10] = 255;
        let out = dilate(&mask, 20, 20, PixelRect::new(10, 10, 1, 1), 2);
        assert_eq!(out.iter().filter(|m| **m == 255).count(), 25);
        assert_eq!(out[8 * 20 + 8], 255);
        assert_eq!(out[7 * 20 + 7], 0);
    }

    #[test]
    fn the_answer_only_shows_inside_the_mask_with_a_soft_edge() {
        let (w, h) = (40usize, 40usize);
        let mut mask = vec![0u8; w * h];
        for y in 15..25 {
            for x in 15..25 {
                mask[y * w + x] = 255;
            }
        }
        let big = vec![0.5f32; 3 * 40 * 40];
        let pic =
            compose(w, h, &mask, PixelRect::new(15, 15, 10, 10), &big, Window { left: 0, top: 0, side: 40 }, 40, 3);
        assert_eq!(pic.pixel(20, 20).a, 255);
        assert_eq!(pic.pixel(2, 2).a, 0);
        assert!(pic.pixel(26, 20).a > 0 && pic.pixel(26, 20).a < 255);
    }
}
