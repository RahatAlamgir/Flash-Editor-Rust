//! Remove by Color (C# `BackgroundRemovalEffect`): removes the background by finding the main colors along the picture's
//! border and flood-filling inwards through pixels similar to them. Works best for subjects on a fairly plain background
//! (product photos, logos, portraits on studio backgrounds). Parts of the subject that touch the border with the same color
//! as the background may also be removed; lower the tolerance then.

use std::collections::VecDeque;

use rayon::prelude::*;

use super::blur::gaussian_in_place;
use super::{Effect, Param, Values, slider};
use crate::imaging::PixelBuffer;
use crate::sheets::detect_border_colors;

const PARAMS: [Param; 3] = [
    slider("tolerance", "Tolerance", 1.0, 100.0, 25.0, 1.0, "%"),
    slider("softness", "Edge softness", 0.0, 100.0, 40.0, 1.0, "%"),
    slider("feather", "Feather", 0.0, 10.0, 1.0, 0.5, "px"),
];

/// The whole picture is always looked at (the border is what counts), whatever is selected.
fn whole_picture(_: &Values) -> i32 {
    i32::MAX / 4
}

pub static REMOVE_BY_COLOR: Effect = Effect { name: "Remove Background", params: &PARAMS, padding: whole_picture, run };

fn nearest(p: &[u8], colors: &[(f64, f64, f64)]) -> f64 {
    let mut best = f64::MAX;
    for (r, g, b) in colors {
        let (dr, dg, db) = (f64::from(p[0]) - r, f64::from(p[1]) - g, f64::from(p[2]) - b);
        best = best.min(dr * dr + dg * dg + db * db);
    }
    best.sqrt()
}

fn run(source: &PixelBuffer, values: &Values) -> PixelBuffer {
    let (w, h) = (source.width() as usize, source.height() as usize);
    let background = detect_border_colors(source);
    if background.is_empty() {
        return source.clone(); // the border is already transparent
    }
    let tolerance = (values.get("tolerance") * 2.2) as f32; // in RGB distance units (0..441)
    let soft_zone = (f64::from(tolerance) * (values.get("softness") / 100.0) + 1.0) as f32;
    let feather = values.get("feather");
    // 1) Distance of every pixel to the closest background color.
    let mut distance = vec![0f32; w * h];
    distance.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, d) in row.iter_mut().enumerate() {
            let i = (y * w + x) * 4;
            let p = &source.data()[i..i + 4];
            *d = if p[3] < 8 { 0.0 } else { nearest(p, &background) as f32 };
        }
    });
    // 2) Flood fill from the border through background-like pixels.
    let mut removed = vec![0f32; w * h]; // 1 = background, 0 = keep, between = partially transparent
    let mut queue = VecDeque::new();
    let seed = |x: usize, y: usize, removed: &mut Vec<f32>, queue: &mut VecDeque<usize>| {
        let p = y * w + x;
        if removed[p] == 0.0 && distance[p] <= tolerance {
            removed[p] = 1.0;
            queue.push_back(p);
        }
    };
    for x in 0..w {
        seed(x, 0, &mut removed, &mut queue);
        seed(x, h - 1, &mut removed, &mut queue);
    }
    for y in 0..h {
        seed(0, y, &mut removed, &mut queue);
        seed(w - 1, y, &mut removed, &mut queue);
    }
    while let Some(p) = queue.pop_front() {
        let (px, py) = ((p % w) as i64, (p / w) as i64);
        for (x, y) in [(px - 1, py), (px + 1, py), (px, py - 1), (px, py + 1)] {
            if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
                continue;
            }
            let q = y as usize * w + x as usize;
            if removed[q] != 0.0 {
                continue;
            }
            let d = distance[q];
            if d <= tolerance {
                removed[q] = 1.0;
                queue.push_back(q);
            } else if d <= tolerance + soft_zone {
                // Edge pixel next to the background: partially transparent (anti-aliased cut-out).
                removed[q] = 1.0 - (d - tolerance) / soft_zone;
            }
        }
    }
    // 3) Optional feathering of the cut-out edge.
    if feather > 0.0 {
        gaussian_in_place(&mut removed, w, h, 1, feather);
    }
    // 4) Apply as transparency.
    let mut result = source.clone();
    result.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let r = removed[y * w + x];
            if r <= 0.0 {
                continue;
            }
            let a = f64::from(row[x * 4 + 3]) * f64::from(1.0 - r.min(1.0));
            row[x * 4 + 3] = a.round().clamp(0.0, 255.0) as u8;
        }
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn a_plain_background_goes_and_the_subject_stays() {
        let mut image = PixelBuffer::filled(60, 60, ColorRgba::rgb(240, 240, 240));
        for y in 20..40 {
            for x in 20..40 {
                image.set_pixel(x, y, ColorRgba::rgb(200, 20, 20));
            }
        }
        let out = REMOVE_BY_COLOR.apply(&image, &REMOVE_BY_COLOR.defaults());
        assert_eq!(out.pixel(2, 2).a, 0, "the background corner is see-through");
        assert_eq!(out.pixel(30, 30).a, 255, "the subject stays");
    }

    #[test]
    fn a_hole_the_border_cannot_reach_is_kept() {
        let mut image = PixelBuffer::filled(60, 60, ColorRgba::WHITE);
        for y in 10..50 {
            for x in 10..50 {
                image.set_pixel(x, y, ColorRgba::rgb(10, 10, 200));
            }
        }
        for y in 28..32 {
            for x in 28..32 {
                image.set_pixel(x, y, ColorRgba::WHITE); // white inside the subject
            }
        }
        let out = REMOVE_BY_COLOR.apply(&image, &REMOVE_BY_COLOR.defaults());
        assert_eq!(out.pixel(30, 30).a, 255);
        assert_eq!(out.pixel(1, 1).a, 0);
    }

    #[test]
    fn a_picture_with_a_see_through_border_is_left_alone() {
        let image = PixelBuffer::new(20, 20);
        assert_eq!(REMOVE_BY_COLOR.apply(&image, &REMOVE_BY_COLOR.defaults()), image);
    }
}
