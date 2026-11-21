//! Flip and rotate (C# `ImageTransforms`).

use rayon::prelude::*;

use super::resample::sample_bilinear;
use crate::imaging::PixelBuffer;

pub fn flip_horizontal(src: &PixelBuffer) -> PixelBuffer {
    let mut out = PixelBuffer::new(src.width(), src.height());
    let w = src.width() as usize;
    out.data_mut().par_chunks_mut(w * 4).zip(src.data().par_chunks(w * 4)).for_each(|(d, s)| {
        for x in 0..w {
            d[(w - 1 - x) * 4..(w - x) * 4].copy_from_slice(&s[x * 4..x * 4 + 4]);
        }
    });
    out
}

pub fn flip_vertical(src: &PixelBuffer) -> PixelBuffer {
    let mut out = PixelBuffer::new(src.width(), src.height());
    let h = src.height();
    for y in 0..h {
        out.row_mut(h - 1 - y).copy_from_slice(src.row(y));
    }
    out
}

/// 90° clockwise (width and height swap).
pub fn rotate_90_cw(src: &PixelBuffer) -> PixelBuffer {
    let (w, h) = (src.width(), src.height());
    let mut out = PixelBuffer::new(h, w);
    for y in 0..h {
        for x in 0..w {
            out.set_pixel(h - 1 - y, x, src.pixel(x, y));
        }
    }
    out
}

/// 90° counter-clockwise (width and height swap).
pub fn rotate_90_ccw(src: &PixelBuffer) -> PixelBuffer {
    let (w, h) = (src.width(), src.height());
    let mut out = PixelBuffer::new(h, w);
    for y in 0..h {
        for x in 0..w {
            out.set_pixel(y, w - 1 - x, src.pixel(x, y));
        }
    }
    out
}

pub fn rotate_180(src: &PixelBuffer) -> PixelBuffer {
    flip_vertical(&flip_horizontal(src))
}

/// The size of the canvas needed to hold a picture of the given size turned by `degrees`.
pub fn rotated_size(width: u32, height: u32, degrees: f64) -> (u32, u32) {
    let rad = degrees.to_radians();
    let (cos, sin) = (rad.cos().abs(), rad.sin().abs());
    let nw = (f64::from(width) * cos + f64::from(height) * sin - 1e-6).ceil() as u32;
    let nh = (f64::from(width) * sin + f64::from(height) * cos - 1e-6).ceil() as u32;
    (nw.max(1), nh.max(1))
}

/// Rotates by any angle (clockwise, degrees). The canvas grows so nothing is cut off; new corners are transparent.
pub fn rotate(src: &PixelBuffer, degrees: f64) -> PixelBuffer {
    let normalized = ((degrees % 360.0) + 360.0) % 360.0;
    if normalized == 0.0 {
        return src.clone();
    }
    if normalized == 90.0 {
        return rotate_90_cw(src);
    }
    if normalized == 180.0 {
        return rotate_180(src);
    }
    if normalized == 270.0 {
        return rotate_90_ccw(src);
    }
    let (nw, nh) = rotated_size(src.width(), src.height(), degrees);
    let mut out = PixelBuffer::new(nw, nh);
    let rad = degrees.to_radians();
    let (cos, sin) = (rad.cos(), rad.sin());
    let (scx, scy) = (f64::from(src.width()) / 2.0, f64::from(src.height()) / 2.0);
    let (dcx, dcy) = (f64::from(nw) / 2.0, f64::from(nh) / 2.0);
    out.data_mut().par_chunks_mut(nw as usize * 4).enumerate().for_each(|(y, row)| {
        let dy = y as f64 + 0.5 - dcy;
        for x in 0..nw as usize {
            let dx = x as f64 + 0.5 - dcx;
            // Inverse rotation: where in the source does this destination pixel come from?
            let sx = dx * cos + dy * sin + scx - 0.5;
            let sy = -dx * sin + dy * cos + scy - 0.5;
            row[x * 4..x * 4 + 4].copy_from_slice(&sample_bilinear(src, sx, sy));
        }
    });
    out
}
