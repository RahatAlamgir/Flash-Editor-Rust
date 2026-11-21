//! The drop shadow (C# `ShadowFx.Apply`): the object's transparency is blurred, shifted and tinted, and the
//! object is laid on top.

use super::plane::{Img, Window, alpha_plane, blur, clamp_byte, content_window, resize};
use crate::imaging::ColorRgba;

/// A new picture of the same size: the shadow of `src` with `src` on top. Parts that fall outside are cut off, so
/// the picture needs enough empty room around the object.
#[allow(clippy::too_many_arguments)]
pub fn apply(src: &Img, color: ColorRgba, ox: f64, oy: f64, blur_amount: f64, size: f64, cx: f64, cy: f64) -> Img {
    let (w, h) = (src.w, src.h);
    let mut alpha = alpha_plane(src);
    let object = content_window(&alpha, w, h, 0.0);
    let mut out = Img::new(w, h);
    if object.is_empty() {
        return out;
    }
    let (dx, dy) = (ox.round() as i64, oy.round() as i64);
    let spread = if blur_amount > 0.0 {
        ((((blur_amount * blur_amount + 1.0).sqrt() - 1.0) / 2.0).round().max(1.0) as usize) * 3 + 2
    } else {
        0
    };
    let resized = (size - 1.0).abs() > 0.005;
    let shadow = if resized {
        Window { x0: 0, y0: 0, x1: w, y1: h }
    } else {
        let (l, t) = (spread as i64 - dx, spread as i64 - dy);
        let (r, b) = (spread as i64 + dx, spread as i64 + dy);
        object.grow(l.max(0) as usize, t.max(0) as usize, r.max(0) as usize, b.max(0) as usize, w, h)
    };
    let win = object.union(&shadow);
    if resized {
        alpha = resize(&alpha, w, h, size, cx, cy);
    }
    if blur_amount > 0.0 {
        alpha = blur(&alpha, w, h, blur_amount / 2.0);
    }
    let strength = f32::from(color.a) / 255.0;
    for y in win.y0..win.y1 {
        let sy = y as i64 - dy;
        for x in win.x0..win.x1 {
            let sx = x as i64 - dx;
            let shadow_a = if sx >= 0 && sx < w as i64 && sy >= 0 && sy < h as i64 {
                alpha[sy as usize * w + sx as usize] * strength
            } else {
                0.0
            };
            let i = (y * w + x) * 4;
            let object_a = f32::from(src.data[i + 3]) / 255.0;
            let out_a = object_a + shadow_a * (1.0 - object_a);
            if out_a <= 0.0 {
                continue;
            }
            // "Over" blend of the object on the shadow, on straight colors.
            let weight = shadow_a * (1.0 - object_a);
            let mix = |obj: u8, sh: u8| clamp_byte((f32::from(obj) * object_a + f32::from(sh) * weight) / out_a);
            out.data[i] = mix(src.data[i], color.r);
            out.data[i + 1] = mix(src.data[i + 1], color.g);
            out.data[i + 2] = mix(src.data[i + 2], color.b);
            out.data[i + 3] = clamp_byte(out_a * 255.0);
        }
    }
    out
}
