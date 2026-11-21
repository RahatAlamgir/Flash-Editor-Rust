//! Planes of 0..1 numbers (the transparency of a drawing) and the blur used by shadows and glows (C# `ShadowFx`).

use rayon::prelude::*;

/// A drawing with straight (not premultiplied) colors, RGBA.
#[derive(Clone)]
pub struct Img {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}

impl Img {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, data: vec![0; w * h * 4] }
    }
}

/// Rows and columns that hold something: [x0, x1) x [y0, y1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl Window {
    pub fn is_empty(&self) -> bool {
        self.x1 <= self.x0 || self.y1 <= self.y0
    }

    pub fn grow(&self, left: usize, top: usize, right: usize, bottom: usize, w: usize, h: usize) -> Self {
        Self {
            x0: self.x0.saturating_sub(left),
            y0: self.y0.saturating_sub(top),
            x1: (self.x1 + right).min(w),
            y1: (self.y1 + bottom).min(h),
        }
    }

    pub fn union(&self, o: &Self) -> Self {
        Self { x0: self.x0.min(o.x0), y0: self.y0.min(o.y0), x1: self.x1.max(o.x1), y1: self.y1.max(o.y1) }
    }
}

pub fn alpha_plane(img: &Img) -> Vec<f32> {
    img.data.as_chunks::<4>().0.iter().map(|p| f32::from(p[3]) / 255.0).collect()
}

/// The smallest window around everything above `threshold`.
pub fn content_window(plane: &[f32], w: usize, h: usize, threshold: f32) -> Window {
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
    for y in 0..h {
        let row = &plane[y * w..(y + 1) * w];
        let Some(first) = row.iter().position(|v| *v > threshold) else { continue };
        let last = row.iter().rposition(|v| *v > threshold).unwrap_or(first);
        x0 = x0.min(first);
        x1 = x1.max(last + 1);
        y0 = y0.min(y);
        y1 = y + 1;
    }
    if x1 == 0 { Window { x0: 0, y0: 0, x1: 0, y1: 0 } } else { Window { x0, y0, x1, y1 } }
}

pub fn clamp_byte(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Soft blur of a plane: three box blurs in each direction look like a Gaussian blur.
pub fn blur(plane: &[f32], w: usize, h: usize, sigma: f64) -> Vec<f32> {
    let radius = (((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) / 2.0).round().max(1.0) as usize;
    let mut a = plane.to_vec();
    let content = content_window(plane, w, h, 0.0);
    if content.is_empty() {
        return a;
    }
    let reach = radius * 3;
    let win = content.grow(reach, reach, reach, reach, w, h);
    let mut b = vec![0.0f32; w * h];
    for _ in 0..3 {
        box_horizontal(&a, &mut b, w, win, radius);
        box_vertical(&b, &mut a, w, win, radius);
    }
    a
}

/// Averages each value with its neighbors within `radius` along the row (outside counts as 0).
fn box_horizontal(src: &[f32], dst: &mut [f32], w: usize, win: Window, radius: usize) {
    let len = win.x1 - win.x0;
    let norm = 1.0 / (2 * radius + 1) as f32;
    dst[win.y0 * w..win.y1 * w].par_chunks_mut(w).enumerate().for_each(|(i, row)| {
        let y = win.y0 + i;
        let s = &src[y * w + win.x0..y * w + win.x1];
        let mut sum: f32 = s.iter().take(radius + 1).sum();
        for k in 0..len {
            row[win.x0 + k] = sum * norm;
            let add = k + radius + 1;
            if add < len {
                sum += s[add];
            }
            if k >= radius {
                sum -= s[k - radius];
            }
        }
    });
}

/// The same along the columns, walking down the picture row by row so memory is read in order.
fn box_vertical(src: &[f32], dst: &mut [f32], w: usize, win: Window, radius: usize) {
    let len = win.y1 - win.y0;
    let n = win.x1 - win.x0;
    let norm = 1.0 / (2 * radius + 1) as f32;
    let mut sum = vec![0.0f32; n];
    for k in 0..=radius.min(len.saturating_sub(1)) {
        let row = (win.y0 + k) * w + win.x0;
        for i in 0..n {
            sum[i] += src[row + i];
        }
    }
    for k in 0..len {
        let row = (win.y0 + k) * w + win.x0;
        for i in 0..n {
            dst[row + i] = sum[i] * norm;
        }
        let add = k + radius + 1;
        if add < len {
            let r = (win.y0 + add) * w + win.x0;
            for i in 0..n {
                sum[i] += src[r + i];
            }
        }
        if k >= radius {
            let r = (win.y0 + k - radius) * w + win.x0;
            for i in 0..n {
                sum[i] -= src[r + i];
            }
        }
    }
}

/// The plane made bigger or smaller by `size` around a point (bilinear).
pub fn resize(plane: &[f32], w: usize, h: usize, size: f64, cx: f64, cy: f64) -> Vec<f32> {
    let inverse = 1.0 / size.max(0.01);
    let sample = |x: i64, y: i64| -> f64 {
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            0.0
        } else {
            f64::from(plane[y as usize * w + x as usize])
        }
    };
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let sy = cy + (y as f64 + 0.5 - cy) * inverse - 0.5;
        let y0 = sy.floor();
        let fy = sy - y0;
        for (x, v) in row.iter_mut().enumerate() {
            let sx = cx + (x as f64 + 0.5 - cx) * inverse - 0.5;
            let x0 = sx.floor();
            let fx = sx - x0;
            let (xi, yi) = (x0 as i64, y0 as i64);
            *v = (sample(xi, yi) * (1.0 - fx) * (1.0 - fy)
                + sample(xi + 1, yi) * fx * (1.0 - fy)
                + sample(xi, yi + 1) * (1.0 - fx) * fy
                + sample(xi + 1, yi + 1) * fx * fy) as f32;
        }
    });
    out
}
