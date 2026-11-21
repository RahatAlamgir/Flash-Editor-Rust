//! Straightening a tilted picture and cutting a flat rectangle out of a picture taken at an angle (C#
//! `PerspectiveTools`).

use rayon::prelude::*;

use super::resample::sample_bilinear;
use super::rotate::rotate;
use crate::imaging::{PixelBuffer, PixelRect, PointD};

/// The biggest upright rectangle that fits inside a picture of `width` x `height` turned by `degrees` around its
/// middle (so no empty corners are left).
pub fn largest_inside_rotated(width: u32, height: u32, degrees: f64) -> (u32, u32) {
    let mut a = (degrees % 180.0).abs().to_radians();
    if a > std::f64::consts::FRAC_PI_2 {
        a = std::f64::consts::PI - a;
    }
    let (sin, cos) = a.sin_cos();
    if sin < 1e-9 {
        return (width, height);
    }
    let (wf, hf) = (f64::from(width), f64::from(height));
    let width_is_longer = width >= height;
    let (long, short) = if width_is_longer { (wf, hf) } else { (hf, wf) };
    let (w, h);
    if short <= 2.0 * sin * cos * long || (sin - cos).abs() < 1e-10 {
        // The rectangle touches the long sides of the turned picture with two corners.
        let x = 0.5 * short;
        (w, h) = if width_is_longer { (x / sin, x / cos) } else { (x / cos, x / sin) };
    } else {
        let cos2 = cos * cos - sin * sin;
        w = (wf * cos - hf * sin) / cos2;
        h = (hf * cos - wf * sin) / cos2;
    }
    ((w.floor() as i64).max(1) as u32, (h.floor() as i64).max(1) as u32)
}

/// Turns the picture by `degrees` (clockwise) and, when `crop_corners`, cuts away the empty corners.
pub fn straighten(src: &PixelBuffer, degrees: f64, crop_corners: bool) -> PixelBuffer {
    let turned = rotate(src, degrees);
    if !crop_corners {
        return turned;
    }
    let (w, h) = largest_inside_rotated(src.width(), src.height(), degrees);
    // Two pixels less on every side: the very edge of a turned picture is soft (partly see-through).
    let w = (w.min(turned.width()) as i64 - 4).max(1) as u32;
    let h = (h.min(turned.height()) as i64 - 4).max(1) as u32;
    turned.crop(PixelRect::new(
        ((turned.width() - w) / 2) as i32,
        ((turned.height() - h) / 2) as i32,
        w as i32,
        h as i32,
    ))
}

/// The size of a flat rectangle that suits the four corners (top-left, top-right, bottom-right, bottom-left).
pub fn suggested_size(quad: &[PointD]) -> (u32, u32) {
    let w = (quad[0].distance_to(&quad[1]) + quad[3].distance_to(&quad[2])) / 2.0;
    let h = (quad[0].distance_to(&quad[3]) + quad[1].distance_to(&quad[2])) / 2.0;
    ((w.round_ties_even() as i64).max(1) as u32, (h.round_ties_even() as i64).max(1) as u32)
}

/// Cuts the four-sided area `quad` (top-left, top-right, bottom-right, bottom-left, in source pixels) out of the
/// picture and makes it a flat `width` x `height` rectangle. `None` when the points do not make a usable shape.
pub fn perspective_crop(src: &PixelBuffer, quad: &[PointD], width: u32, height: u32) -> Option<PixelBuffer> {
    if quad.len() != 4 || width < 1 || height < 1 {
        return None;
    }
    let (x0, y0, x1, y1, x2, y2, x3, y3) =
        (quad[0].x, quad[0].y, quad[1].x, quad[1].y, quad[2].x, quad[2].y, quad[3].x, quad[3].y);
    let (dx1, dx2, dx3) = (x1 - x2, x3 - x2, x0 - x1 + x2 - x3);
    let (dy1, dy2, dy3) = (y1 - y2, y3 - y2, y0 - y1 + y2 - y3);
    let (g, h);
    if dx3.abs() < 1e-12 && dy3.abs() < 1e-12 {
        (g, h) = (0.0, 0.0);
    } else {
        let den = dx1 * dy2 - dy1 * dx2;
        if den.abs() < 1e-12 {
            return None;
        }
        g = (dx3 * dy2 - dy3 * dx2) / den;
        h = (dx1 * dy3 - dy1 * dx3) / den;
    }
    // The map from the unit square to the quad: (x, y) = (a u + b v + c, d u + e v + f) / (g u + h v + 1)
    let (a, b, c) = (x1 - x0 + g * x1, x3 - x0 + h * x3, x0);
    let (d, e, f) = (y1 - y0 + g * y1, y3 - y0 + h * y3, y0);
    let mut out = PixelBuffer::new(width, height);
    out.data_mut().par_chunks_mut(width as usize * 4).enumerate().for_each(|(py, row)| {
        let v = (py as f64 + 0.5) / f64::from(height);
        for px in 0..width as usize {
            let u = (px as f64 + 0.5) / f64::from(width);
            let w = g * u + h * v + 1.0;
            if w.abs() < 1e-12 {
                continue;
            }
            let p = sample_bilinear(src, (a * u + b * v + c) / w - 0.5, (d * u + e * v + f) / w - 0.5);
            row[px * 4..px * 4 + 4].copy_from_slice(&p);
        }
    });
    Some(out)
}
