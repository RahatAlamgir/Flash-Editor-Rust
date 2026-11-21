//! Pure helper math around the models: picture -> input tensor, raw output -> 0..1 mask, mask resizing and the user's
//! edge adjustments (C# `MaskOps`). No AI runtime needed.

use fe_core::imaging::PixelBuffer;
use fe_core::imaging::color_math::clamp_byte;
use fe_core::transform::{ResampleMode, resize};
use rayon::prelude::*;

/// The picture resized to `size` x `size` as planar RGB floats (NCHW), normalized as `(value / 255 - mean) / std`.
pub fn to_input_tensor(image: &PixelBuffer, size: usize, mean: [f32; 3], std: [f32; 3]) -> Vec<f32> {
    let resized = if image.width() as usize == size && image.height() as usize == size {
        image.clone()
    } else {
        resize(image, size as u32, size as u32, ResampleMode::Smooth)
    };
    let plane = size * size;
    let mut tensor = vec![0f32; 3 * plane];
    let d = resized.data();
    let (r, rest) = tensor.split_at_mut(plane);
    let (g, b) = rest.split_at_mut(plane);
    r.par_iter_mut().zip(g.par_iter_mut()).zip(b.par_iter_mut()).enumerate().for_each(|(p, ((r, g), b))| {
        let i = p * 4;
        *r = (f32::from(d[i]) / 255.0 - mean[0]) / std[0];
        *g = (f32::from(d[i + 1]) / 255.0 - mean[1]) / std[1];
        *b = (f32::from(d[i + 2]) / 255.0 - mean[2]) / std[2];
    });
    tensor
}

/// Stretches raw model output to 0..1 (as rembg does). A flat output (nothing found) gives zeros.
pub fn normalize(raw: &[f32]) -> Vec<f32> {
    let (min, max) = raw.iter().fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
    let range = max - min;
    if range < 1e-6 {
        return vec![0.0; raw.len()];
    }
    raw.iter().map(|v| (v - min) / range).collect()
}

/// Bilinear resize of a single-channel mask.
pub fn resize_mask(mask: &[f32], width: usize, height: usize, new_width: usize, new_height: usize) -> Vec<f32> {
    let mut result = vec![0f32; new_width * new_height];
    let (sx, sy) = (width as f64 / new_width as f64, height as f64 / new_height as f64);
    result.par_chunks_mut(new_width).enumerate().for_each(|(y, row)| {
        let fy = ((y as f64 + 0.5) * sy - 0.5).clamp(0.0, (height - 1) as f64);
        let y0 = fy as usize;
        let y1 = (y0 + 1).min(height - 1);
        let ty = (fy - y0 as f64) as f32;
        for (x, out) in row.iter_mut().enumerate() {
            let fx = ((x as f64 + 0.5) * sx - 0.5).clamp(0.0, (width - 1) as f64);
            let x0 = fx as usize;
            let x1 = (x0 + 1).min(width - 1);
            let tx = (fx - x0 as f64) as f32;
            let top = mask[y0 * width + x0] * (1.0 - tx) + mask[y0 * width + x1] * tx;
            let bottom = mask[y1 * width + x0] * (1.0 - tx) + mask[y1 * width + x1] * tx;
            *out = top * (1.0 - ty) + bottom * ty;
        }
    });
    result
}

/// Applies the user's edge settings to a 0..1 mask. `sharpness` 0 = the model's soft edges .. 1 = hard cut; `shift`
/// -1 = shrink (remove more) .. +1 = grow (keep more).
pub fn adjust(mask: &mut [f32], sharpness: f64, shift: f64) {
    let width = 1.0 - 0.96 * sharpness.clamp(0.0, 1.0);
    let center = 0.5 - 0.4 * shift.clamp(-1.0, 1.0);
    // The whole soft band stays inside 0..1: what the AI is sure about stays fully kept (1) or fully removed (0).
    let lo = (center - width / 2.0).clamp(0.0, 1.0 - width) as f32;
    let range = width as f32;
    mask.par_iter_mut().for_each(|v| *v = ((*v - lo) / range).clamp(0.0, 1.0));
}

/// The picture's alpha multiplied by the mask (0 = remove, 1 = keep).
pub fn apply_as_alpha(image: &PixelBuffer, mask: &[f32]) -> PixelBuffer {
    let mut result = image.clone();
    let w = result.width() as usize;
    result.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        for (x, p) in row.chunks_exact_mut(4).enumerate() {
            p[3] = clamp_byte(f64::from(p[3]) * f64::from(mask[y * w + x]));
        }
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_stretched_to_zero_one_and_flat_output_is_nothing() {
        assert_eq!(normalize(&[2.0, 4.0, 6.0]), vec![0.0, 0.5, 1.0]);
        assert_eq!(normalize(&[3.0, 3.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn a_sharp_edge_is_a_hard_cut_and_shrinking_removes_more() {
        let mut soft = vec![0.2f32, 0.5, 0.8];
        adjust(&mut soft, 1.0, 0.0);
        assert!(soft[0] == 0.0 && soft[2] == 1.0);
        let mut a = vec![0.45f32];
        let mut b = vec![0.45f32];
        adjust(&mut a, 0.5, -1.0);
        adjust(&mut b, 0.5, 1.0);
        assert!(a[0] < b[0]);
    }

    #[test]
    fn the_mask_becomes_the_alpha() {
        let img = PixelBuffer::filled(2, 1, fe_core::imaging::ColorRgba::rgb(10, 20, 30));
        let out = apply_as_alpha(&img, &[1.0, 0.5]);
        assert_eq!((out.pixel(0, 0).a, out.pixel(1, 0).a), (255, 128));
    }

    #[test]
    fn the_input_tensor_is_planar_and_normalized() {
        let img = PixelBuffer::filled(2, 2, fe_core::imaging::ColorRgba::rgb(255, 0, 51));
        let t = to_input_tensor(&img, 2, [0.5; 3], [1.0; 3]);
        assert_eq!(t.len(), 12);
        assert!((t[0] - 0.5).abs() < 1e-6 && (t[4] + 0.5).abs() < 1e-6 && (t[8] + 0.3).abs() < 1e-6);
    }
}
