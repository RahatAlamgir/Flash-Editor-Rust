//! Reshape for pictures, groups and curved shapes (C# `DrawWarped` + `QuadWarp`): the object is drawn normally on
//! a pixmap of its own, then stretched so its four corners land on the dragged corner points (a perspective warp).

use tiny_skia::{Pixmap, Transform};

use super::draw::{draw, margin_for};
use crate::imaging::PointD;
use crate::objects::{ObjectBox, StoredObject};

/// The perspective map that sends the unit square onto a quad (top-left, top-right, bottom-right, bottom-left),
/// stored as its inverse: a point of the quad back to (u, v) in the square.
pub struct QuadWarp {
    m: [f64; 9],
}

impl QuadWarp {
    /// `None` when the four points do not make a usable shape (all on one line...).
    pub fn new(q: &[PointD; 4]) -> Option<Self> {
        let [(x0, y0), (x1, y1), (x2, y2), (x3, y3)] = q.map(|p| (p.x, p.y));
        let (dx1, dx2, dx3) = (x1 - x2, x3 - x2, x0 - x1 + x2 - x3);
        let (dy1, dy2, dy3) = (y1 - y2, y3 - y2, y0 - y1 + y2 - y3);
        let den = dx1 * dy2 - dy1 * dx2;
        let (g, h) = if dx3.abs() < 1e-12 && dy3.abs() < 1e-12 {
            (0.0, 0.0) // a parallelogram: no perspective
        } else {
            if den.abs() < 1e-12 {
                return None;
            }
            ((dx3 * dy2 - dy3 * dx2) / den, (dx1 * dy3 - dy1 * dx3) / den)
        };
        let (a, b, c) = (x1 - x0 + g * x1, x3 - x0 + h * x3, x0);
        let (d, e, f) = (y1 - y0 + g * y1, y3 - y0 + h * y3, y0);
        let m = [
            e - f * h,
            c * h - b,
            b * f - c * e,
            f * g - d,
            a - c * g,
            c * d - a * f,
            d * h - e * g,
            b * g - a * h,
            a * e - b * d,
        ];
        if m[8].abs() < 1e-12 && m[6].abs() < 1e-12 && m[7].abs() < 1e-12 {
            return None;
        }
        Some(Self { m })
    }

    /// The place in the unit square that lands on (x, y); `None` beyond the horizon of the perspective.
    pub fn unmap(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let m = &self.m;
        let w = m[6] * x + m[7] * y + m[8];
        if w.abs() < 1e-12 {
            return None;
        }
        Some(((m[0] * x + m[1] * y + m[2]) / w, (m[3] * x + m[4] * y + m[5]) / w))
    }
}

/// Bilinear sample of a premultiplied pixmap (transparent outside).
fn sample(src: &Pixmap, sx: f64, sy: f64) -> [f64; 4] {
    let (w, h) = (src.width() as i64, src.height() as i64);
    let (x0, y0) = (sx.floor() as i64, sy.floor() as i64);
    let (fx, fy) = (sx - x0 as f64, sy - y0 as f64);
    let data = src.data();
    let mut out = [0.0; 4];
    for k in 0..4 {
        let (cx, cy) = (x0 + (k & 1), y0 + (k >> 1));
        if cx < 0 || cy < 0 || cx >= w || cy >= h {
            continue;
        }
        let weight = (if k & 1 == 0 { 1.0 - fx } else { fx }) * (if k >> 1 == 0 { 1.0 - fy } else { fy });
        let i = ((cy * w + cx) * 4) as usize;
        for (c, o) in out.iter_mut().enumerate() {
            *o += data[i + c] as f64 * weight;
        }
    }
    out
}

/// Draws `obj` (which has a warp) with its corners on `corners` (in the box, 0..1).
pub fn draw_warped(pm: &mut Pixmap, obj: &StoredObject, b: &ObjectBox, t: Transform, corners: &[PointD]) -> Option<()> {
    let mut plain = obj.clone();
    plain.set_warp(None);
    plain.set_looks(&crate::objects::Looks::default()); // the looks are made from the whole warped drawing
    let (w, h) = (b.width.max(1.0), b.height.max(1.0));
    let det = (t.sx * t.sy - t.kx * t.ky).abs() as f64;
    if det < 1e-9 || corners.len() != 4 {
        return draw(pm, &plain, t);
    }
    let quad: Vec<PointD> = corners.iter().map(|p| PointD::new(p.x * w, p.y * h)).collect();
    let (min_x, max_x) = quad.iter().fold((f64::MAX, f64::MIN), |(a, z), p| (a.min(p.x), z.max(p.x)));
    let (min_y, max_y) = quad.iter().fold((f64::MAX, f64::MIN), |(a, z), p| (a.min(p.y), z.max(p.y)));

    // Pixels per unit of box space: as sharp as on the canvas, more when the corners pulled it bigger.
    let margin = margin_for(&plain);
    let stretch = ((max_x - min_x) / w).max((max_y - min_y) / h).clamp(1.0, 4.0);
    let res = (det.sqrt() * stretch)
        .clamp(0.05, 16.0)
        .min(2500.0 / (w.max(h).max((max_x - min_x).max(max_y - min_y)) + 2.0 * margin));
    let pad = (margin * res).ceil() + 1.0;
    let mut src = Pixmap::new(((w * res).ceil() + 2.0 * pad) as u32, ((h * res).ceil() + 2.0 * pad) as u32)?;
    draw(&mut src, &plain, Transform::from_row(res as f32, 0.0, 0.0, res as f32, pad as f32, pad as f32))?;

    // The quad on the pixmap, and the warp from the unit square onto it.
    let map = |p: PointD| {
        let mut q = tiny_skia::Point::from_xy(p.x as f32, p.y as f32);
        t.map_point(&mut q);
        PointD::new(q.x as f64, q.y as f64)
    };
    let dest = [map(quad[0]), map(quad[1]), map(quad[2]), map(quad[3])];
    let warp = QuadWarp::new(&dest)?;
    let reach = margin * det.sqrt() + 2.0;
    let (l, r) = dest.iter().fold((f64::MAX, f64::MIN), |(a, z), p| (a.min(p.x), z.max(p.x)));
    let (tp, bt) = dest.iter().fold((f64::MAX, f64::MIN), |(a, z), p| (a.min(p.y), z.max(p.y)));
    let x0 = ((l - reach).floor().max(0.0)) as u32;
    let x1 = ((r + reach).ceil().min(pm.width() as f64)) as u32;
    let y0 = ((tp - reach).floor().max(0.0)) as u32;
    let y1 = ((bt + reach).ceil().min(pm.height() as f64)) as u32;
    let pw = pm.width() as usize;
    let data = pm.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let Some((u, v)) = warp.unmap(px as f64 + 0.5, py as f64 + 0.5) else { continue };
            let (sx, sy) = (pad + u * w * res - 0.5, pad + v * h * res - 0.5);
            if !sx.is_finite()
                || !sy.is_finite()
                || sx < -1.0
                || sy < -1.0
                || sx > src.width() as f64
                || sy > src.height() as f64
            {
                continue;
            }
            let s = sample(&src, sx, sy);
            if s[3] < 0.5 {
                continue;
            }
            let i = (py as usize * pw + px as usize) * 4;
            let keep = 1.0 - s[3] / 255.0;
            for c in 0..4 {
                data[i + c] = (s[c] + data[i + c] as f64 * keep).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Some(())
}
