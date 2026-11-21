//! Looks made from the transparency of a drawn object (C# `SurfaceFx`): an outline around it, a glow behind it and
//! a shadow inside its edges.

use super::plane::{Img, alpha_plane, blur, clamp_byte, content_window};
use crate::imaging::ColorRgba;

/// The object with a line of `width` pixels around its outside (drawn behind the object).
pub fn outline(src: &Img, color: ColorRgba, width: f64) -> Img {
    if width <= 0.0 || color.a == 0 {
        return src.clone();
    }
    let (w, h) = (src.w, src.h);
    let alpha = alpha_plane(src);
    let content = content_window(&alpha, w, h, 0.0);
    if content.is_empty() {
        return src.clone();
    }
    let grow = width.ceil() as usize + 2;
    let win = content.grow(grow, grow, grow, grow, w, h);
    let (ww, wh) = (win.x1 - win.x0, win.y1 - win.y0);

    // Distance of every pixel in the window to the nearest part of the object (that is more than half there).
    const INF: f32 = 1e12;
    let mut squared = vec![INF; ww * wh];
    for y in 0..wh {
        for x in 0..ww {
            if alpha[(win.y0 + y) * w + win.x0 + x] >= 0.5 {
                squared[y * ww + x] = 0.0;
            }
        }
    }
    distance_transform(&mut squared, ww, wh);

    let mut out = src.clone();
    let line_alpha = f32::from(color.a) / 255.0;
    for y in 0..wh {
        for x in 0..ww {
            let d = squared[y * ww + x].sqrt();
            let ring = (width as f32 + 0.5 - d).clamp(0.0, 1.0) * line_alpha;
            if ring <= 0.0 {
                continue;
            }
            let i = ((win.y0 + y) * w + win.x0 + x) * 4;
            let object_a = f32::from(out.data[i + 3]) / 255.0;
            let out_a = object_a + ring * (1.0 - object_a);
            let weight = ring * (1.0 - object_a);
            let mix = |obj: u8, c: u8| clamp_byte((f32::from(obj) * object_a + f32::from(c) * weight) / out_a);
            out.data[i] = mix(out.data[i], color.r);
            out.data[i + 1] = mix(out.data[i + 1], color.g);
            out.data[i + 2] = mix(out.data[i + 2], color.b);
            out.data[i + 3] = clamp_byte(out_a * 255.0);
        }
    }
    out
}

/// Only the glow of `src` (a soft light around its outline), without the object. Lay it under the object with `under`.
pub fn glow_layer(src: &Img, color: ColorRgba, size: f64, intensity: f64) -> Img {
    let (w, h) = (src.w, src.h);
    let mut out = Img::new(w, h);
    if size <= 0.0 || color.a == 0 {
        return out;
    }
    let alpha = alpha_plane(src);
    let content = content_window(&alpha, w, h, 0.0);
    if content.is_empty() {
        return out;
    }
    let soft = blur(&alpha, w, h, size / 2.0);
    let boost = (intensity / 50.0).max(0.1) as f32; // 100 % doubles the soft edge: strong light near the object
    let strength = f32::from(color.a) / 255.0;
    let reach = (size * 1.5).ceil() as usize + 2;
    let win = content.grow(reach, reach, reach, reach, w, h);
    for y in win.y0..win.y1 {
        for x in win.x0..win.x1 {
            let a = (soft[y * w + x] * boost).min(1.0) * strength;
            if a <= 0.0 {
                continue;
            }
            let i = (y * w + x) * 4;
            out.data[i..i + 4].copy_from_slice(&[color.r, color.g, color.b, clamp_byte(a * 255.0)]);
        }
    }
    out
}

/// `top` laid over `bottom` (same size).
pub fn under(top: &Img, bottom: &Img) -> Img {
    let mut out = bottom.clone();
    for (r, t) in out.data.as_chunks_mut::<4>().0.iter_mut().zip(top.data.as_chunks::<4>().0) {
        let ta = f32::from(t[3]) / 255.0;
        if ta <= 0.0 {
            continue;
        }
        let ba = f32::from(r[3]) / 255.0;
        let out_a = ta + ba * (1.0 - ta);
        let bw = ba * (1.0 - ta);
        for c in 0..3 {
            r[c] = clamp_byte((f32::from(t[c]) * ta + f32::from(r[c]) * bw) / out_a);
        }
        r[3] = clamp_byte(out_a * 255.0);
    }
    out
}

/// A shadow inside the object's edges: the empty space around (and inside) the object is blurred, shifted and
/// tinted onto the object.
pub fn inner_shadow(src: &Img, color: ColorRgba, ox: f64, oy: f64, blur_amount: f64) -> Img {
    if color.a == 0 {
        return src.clone();
    }
    let (w, h) = (src.w, src.h);
    let alpha = alpha_plane(src);
    let content = content_window(&alpha, w, h, 0.0);
    if content.is_empty() {
        return src.clone();
    }
    let (dx, dy) = (ox.round() as i64, oy.round() as i64);
    let sigma = blur_amount.max(0.0) / 2.0;
    let pad = (sigma * 3.0).ceil() as usize + dx.unsigned_abs().max(dy.unsigned_abs()) as usize + 2;
    let (pw, ph) = (content.x1 - content.x0 + 2 * pad, content.y1 - content.y0 + 2 * pad);

    // "What is not the object" in a plane that reaches beyond the object on every side (beyond counts as empty).
    let mut empty = vec![1.0f32; pw * ph];
    for y in content.y0..content.y1 {
        for x in content.x0..content.x1 {
            empty[(y - content.y0 + pad) * pw + (x - content.x0 + pad)] = 1.0 - alpha[y * w + x];
        }
    }
    let soft = if sigma > 0.0 { blur(&empty, pw, ph, sigma) } else { empty };

    let mut out = src.clone();
    let strength = f32::from(color.a) / 255.0;
    for y in content.y0..content.y1 {
        for x in content.x0..content.x1 {
            let i = (y * w + x) * 4;
            if out.data[i + 3] == 0 {
                continue;
            }
            let px = (x - content.x0 + pad) as i64 - dx;
            let py = (y - content.y0 + pad) as i64 - dy;
            let s = if px >= 0 && px < pw as i64 && py >= 0 && py < ph as i64 {
                soft[py as usize * pw + px as usize]
            } else {
                1.0
            };
            let k = s * strength; // how much of the shadow color covers this pixel
            if k <= 0.0 {
                continue;
            }
            for (c, target) in [color.r, color.g, color.b].into_iter().enumerate() {
                let v = f32::from(out.data[i + c]);
                out.data[i + c] = clamp_byte(v + (f32::from(target) - v) * k);
            }
        }
    }
    out
}

/// Felzenszwalb & Huttenlocher: squared distance to the nearest zero, in two passes (columns, then rows).
fn distance_transform(f: &mut [f32], w: usize, h: usize) {
    let mut column = vec![0.0f32; h];
    for x in 0..w {
        for y in 0..h {
            column[y] = f[y * w + x];
        }
        let d = transform_1d(&column);
        for y in 0..h {
            f[y * w + x] = d[y];
        }
    }
    for y in 0..h {
        let d = transform_1d(&f[y * w..(y + 1) * w]);
        f[y * w..(y + 1) * w].copy_from_slice(&d);
    }
}

fn transform_1d(f: &[f32]) -> Vec<f32> {
    let n = f.len();
    let mut d = vec![0.0f32; n];
    let mut v = vec![0usize; n];
    let mut z = vec![0.0f32; n + 1];
    let mut k = 0usize;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        let mut s;
        loop {
            let p = v[k];
            s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * q as f32 - 2.0 * p as f32);
            if s <= z[k] && k > 0 {
                k -= 1;
                continue;
            }
            break;
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f32::INFINITY;
    }
    k = 0;
    for (q, out) in d.iter_mut().enumerate() {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let dq = q as f32 - v[k] as f32;
        *out = dq * dq + f[v[k]];
    }
    d
}
