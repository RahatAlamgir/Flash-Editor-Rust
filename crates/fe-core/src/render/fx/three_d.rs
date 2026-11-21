//! The 3D look (C# `ThreeDFx`): a bevel on the face (lit top-left edge, shaded bottom-right edge) and a solid side
//! behind it made by pulling the outline along a direction.

use super::plane::{Img, Window, alpha_plane, blur, clamp_byte, content_window};
use crate::imaging::ColorRgba;

const FAR: f32 = 1e9;

/// A new picture of the same size. Parts of the side that fall outside are cut off, so the picture needs room.
pub fn apply(src: &Img, ox: f64, oy: f64, color: Option<ColorRgba>, bevel: f64, bevel_size: f64) -> Img {
    let (w, h) = (src.w, src.h);
    let alpha = alpha_plane(src);
    let face = content_window(&alpha, w, h, 0.0);
    let mut out = Img::new(w, h);
    if face.is_empty() {
        return out;
    }
    let front = if bevel > 0.001 { bevelled(src, &alpha, bevel, bevel_size, face) } else { src.data.clone() };

    let depth = (ox * ox + oy * oy).sqrt().round() as usize;
    let mut side: Option<Vec<f32>> = None;
    let mut distance: Option<Vec<f32>> = None;
    let mut work = face;
    if depth >= 1 {
        let (sx, sy) = (ox.abs().ceil() as usize + 2, oy.abs().ceil() as usize + 2);
        work = face.grow(
            if ox < 0.0 { sx } else { 0 },
            if oy < 0.0 { sy } else { 0 },
            if ox > 0.0 { sx } else { 0 },
            if oy > 0.0 { sy } else { 0 },
            w,
            h,
        );
        let (s, d) = extrude(&alpha, w, h, ox, oy, depth, work);
        side = Some(s);
        distance = Some(d);
    }
    let side_color = color.unwrap_or_else(|| auto_color(src, &alpha));
    for y in work.y0..work.y1 {
        for x in work.x0..work.x1 {
            let p = y * w + x;
            let i = p * 4;
            let front_a = alpha[p];
            let side_a = side.as_ref().map_or(0.0, |s| s[p]);
            let shade = distance.as_ref().map_or(1.0, |d| 1.0 - 0.45 * (d[p] / depth.max(1) as f32).min(1.0)); // darker the deeper
            let out_a = front_a + side_a * (1.0 - front_a);
            if out_a <= 0.0 {
                continue;
            }
            let weight = side_a * (1.0 - front_a);
            let mix = |f: u8, s: u8| clamp_byte((f32::from(f) * front_a + f32::from(s) * shade * weight) / out_a);
            out.data[i] = mix(front[i], side_color.r);
            out.data[i + 1] = mix(front[i + 1], side_color.g);
            out.data[i + 2] = mix(front[i + 2], side_color.b);
            out.data[i + 3] = clamp_byte(out_a * 255.0);
        }
    }
    out
}

/// The object's own color (average of what is visible), made darker, for the side.
fn auto_color(src: &Img, alpha: &[f32]) -> ColorRgba {
    let (mut r, mut g, mut b, mut total) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for p in (0..alpha.len()).step_by(3) {
        let a = f64::from(alpha[p]);
        if a <= 0.0 {
            continue;
        }
        r += f64::from(src.data[p * 4]) * a;
        g += f64::from(src.data[p * 4 + 1]) * a;
        b += f64::from(src.data[p * 4 + 2]) * a;
        total += a;
    }
    if total <= 0.0 {
        return ColorRgba::new(60, 60, 60, 255);
    }
    let dark = 0.55;
    let c = |v: f64| (v / total * dark).round().clamp(0.0, 255.0) as u8;
    ColorRgba::new(c(r), c(g), c(b), 255)
}

/// Pulls the outline along the direction: the coverage of the extruded outline, and how far each pixel is from the
/// face along the way. Done by doubling, so even a very deep side needs only a handful of passes.
fn extrude(alpha: &[f32], w: usize, h: usize, ox: f64, oy: f64, depth: usize, win: Window) -> (Vec<f32>, Vec<f32>) {
    let length = (ox * ox + oy * oy).sqrt();
    let (ux, uy) = (ox / length, oy / length);
    let mut e = alpha.to_vec();
    let mut d = vec![FAR; w * h];
    let mut e2 = vec![0.0f32; w * h];
    let mut d2 = vec![FAR; w * h];
    for y in win.y0..win.y1 {
        for x in win.x0..win.x1 {
            if alpha[y * w + x] > 0.004 {
                d[y * w + x] = 0.0;
            }
        }
    }
    let mut covered = 1usize; // the face plus (covered - 1) steps along the way
    while covered * 2 <= depth + 1 {
        doubled(
            &e,
            &d,
            &mut e2,
            &mut d2,
            w,
            win,
            (ux * covered as f64).round() as i64,
            (uy * covered as f64).round() as i64,
            covered,
        );
        std::mem::swap(&mut e, &mut e2);
        std::mem::swap(&mut d, &mut d2);
        covered *= 2;
    }
    if covered < depth + 1 {
        let rest = depth + 1 - covered;
        doubled(
            &e,
            &d,
            &mut e2,
            &mut d2,
            w,
            win,
            (ux * rest as f64).round() as i64,
            (uy * rest as f64).round() as i64,
            rest,
        );
        std::mem::swap(&mut e, &mut e2);
        std::mem::swap(&mut d, &mut d2);
    }
    (e, d)
}

/// Each pixel also takes what lies `steps` steps behind it (at (dx, dy)): more coverage, a longer way.
#[allow(clippy::too_many_arguments)]
fn doubled(
    e: &[f32],
    d: &[f32],
    ne: &mut [f32],
    nd: &mut [f32],
    w: usize,
    win: Window,
    dx: i64,
    dy: i64,
    steps: usize,
) {
    let h = win.y1;
    for y in win.y0..win.y1 {
        let sy = y as i64 - dy;
        for x in win.x0..win.x1 {
            let p = y * w + x;
            let sx = x as i64 - dx;
            let (mut coverage, mut way) = (e[p], d[p]);
            if sx >= 0 && sx < w as i64 && sy >= 0 && sy < h as i64 {
                let q = sy as usize * w + sx as usize;
                coverage = coverage.max(e[q]);
                if d[q] < FAR && d[q] + (steps as f32) < way {
                    way = d[q] + steps as f32;
                }
            }
            ne[p] = coverage;
            nd[p] = way;
        }
    }
}

/// The face with a lit top-left edge and a shaded bottom-right edge, as if it were raised. The edge is found from
/// the slope of the blurred transparency.
fn bevelled(src: &Img, alpha: &[f32], strength: f64, size: f64, face: Window) -> Vec<u8> {
    let (w, h) = (src.w, src.h);
    let sigma = (size / 2.0).max(0.8);
    let soft = blur(alpha, w, h, sigma);
    let reach = sigma;
    let mut out = src.data.clone();
    let (lx, ly) = (-std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2); // the light comes from the top left
    for y in face.y0.max(1)..face.y1.min(h.saturating_sub(1)) {
        for x in face.x0.max(1)..face.x1.min(w.saturating_sub(1)) {
            let p = y * w + x;
            let i = p * 4;
            if alpha[p] <= 0.004 {
                continue;
            }
            let gx = f64::from(soft[p + 1] - soft[p - 1]) / 2.0;
            let gy = f64::from(soft[p + w] - soft[p - w]) / 2.0;
            let magnitude = (gx * gx + gy * gy).sqrt();
            if magnitude < 1e-4 {
                continue;
            }
            let lit = (-gx * lx + -gy * ly) / magnitude; // 1 on edges facing the light, -1 on the opposite ones
            let edge = (magnitude * reach * 2.4).clamp(0.0, 1.0); // 0 in flat areas, 1 right at the edge
            let amount = lit * edge * strength;
            for c in 0..3 {
                let v = f64::from(out[i + c]);
                let v = if amount > 0.0 { v + (255.0 - v) * amount * 0.85 } else { v * (1.0 + amount * 0.75) };
                out[i + c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}
