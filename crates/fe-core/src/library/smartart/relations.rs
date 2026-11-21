//! Relationships, a matrix and a pyramid: overlapping circles, a target, a funnel, four cells, stacked layers.

use super::{Job, WHITE};
use crate::library::maker::Maker;
use crate::objects::ShapeKind::*;

pub(super) fn venn(j: &Job) -> Maker {
    let n = j.words.len();
    let r = match n {
        2 => 30.0,
        3 => 29.0,
        _ => 25.0,
    };
    let h = if n == 2 { 62.0 } else { 100.0 };
    let mut m = j.maker(h);
    let centers: &[(f64, f64)] = match n {
        2 => &[(36.0, 31.0), (64.0, 31.0)],
        3 => &[(50.0, 31.0), (34.0, 62.0), (66.0, 62.0)],
        _ => &[(50.0, 28.0), (28.0, 50.0), (72.0, 50.0), (50.0, 72.0)],
    };
    for (i, ctr) in centers.iter().enumerate().take(n) {
        m.shape("Circle", Ellipse, ctr.0 - r, ctr.1 - r, r * 2.0, r * 2.0).fill(j.color(i).with_alpha(150));
    }
    for (i, ctr) in centers.iter().enumerate().take(n) {
        let (ox, oy) = (ctr.0 - 50.0, ctr.1 - h / 2.0);
        let len = (ox * ox + oy * oy).sqrt().max(1.0);
        let (tx, ty) = if n == 2 {
            (ctr.0 + if i == 0 { -9.0 } else { 9.0 }, ctr.1)
        } else {
            (ctr.0 + ox / len * 8.0, ctr.1 + oy / len * 8.0)
        };
        m.fit_text(&j.words[i], tx, ty, r * 1.1, r * 0.8, 7.0, WHITE, true);
    }
    m
}

pub(super) fn target(j: &Job) -> Maker {
    let n = j.words.len();
    let mut m = j.maker(100.0);
    let band = 46.0 / n as f64;
    for i in 0..n {
        let r = 50.0 - i as f64 * band;
        m.shape("Ring", Ellipse, 50.0 - r, 50.0 - r, r * 2.0, r * 2.0).fill(j.color(i));
    }
    for (i, word) in j.words.iter().enumerate().take(n) {
        let r = 50.0 - i as f64 * band;
        let y = if i == n - 1 { 50.0 } else { 50.0 - r + band / 2.0 };
        m.fit_text(word, 50.0, y, band * 3.2, band * 0.8, 7.0, WHITE, true);
    }
    m
}

pub(super) fn funnel(j: &Job) -> Maker {
    let n = j.words.len();
    let sh = 17.0;
    let mut m = j.maker(n as f64 * sh);
    let step = 62.0 / n as f64;
    for i in 0..n {
        let f = i as f64;
        let (w0, w1) = (100.0 - f * step, 100.0 - (f + 1.0) * step);
        let (y0, y1) = (f * sh, f * sh + sh - 1.2);
        m.poly(
            "Layer",
            j.color(i),
            &[(50.0 - w0 / 2.0, y0), (50.0 + w0 / 2.0, y0), (50.0 + w1 / 2.0, y1), (50.0 - w1 / 2.0, y1)],
        );
        m.fit_text(&j.words[i], 50.0, (y0 + y1) / 2.0, w1 * 0.9, sh * 0.75, 7.0, WHITE, true);
    }
    m
}

pub(super) fn matrix(j: &Job) -> Maker {
    let gap = 4.0;
    let bw = (100.0 - gap) / 2.0;
    let bh = 30.0f64.min(bw * 0.62);
    let mut m = j.maker(bh * 2.0 + gap);
    for i in 0..4 {
        let (x, y) = ((i % 2) as f64 * (bw + gap), (i / 2) as f64 * (bh + gap));
        m.shape("Cell", Rectangle, x, y, bw, bh).fill(j.color(i)).radius(4);
        m.fit_text(&j.words[i], x + bw / 2.0, y + bh / 2.0, bw * 0.84, bh * 0.76, 8.0, WHITE, true);
    }
    m
}

pub(super) fn pyramid(j: &Job) -> Maker {
    let n = j.words.len();
    let sh = 18.0;
    let mut m = j.maker(n as f64 * sh);
    for i in 0..n {
        let f = i as f64;
        let (w0, w1) = (100.0 * f / n as f64, 100.0 * (f + 1.0) / n as f64);
        let (y0, y1) = (f * sh, f * sh + sh - 1.2);
        if i == 0 {
            m.poly("Top", j.color(i), &[(50.0, y0), (50.0 + w1 / 2.0, y1), (50.0 - w1 / 2.0, y1)]);
        } else {
            m.poly(
                "Layer",
                j.color(i),
                &[(50.0 - w0 / 2.0, y0), (50.0 + w0 / 2.0, y0), (50.0 + w1 / 2.0, y1), (50.0 - w1 / 2.0, y1)],
            );
        }
        let ty = if i == 0 { y1 - sh * 0.3 } else { (y0 + y1) / 2.0 };
        m.fit_text(
            &j.words[i],
            50.0,
            ty,
            (w0 * 0.78 + if i == 0 { 8.0 } else { 0.0 }).max(10.0),
            sh * 0.7,
            7.0,
            WHITE,
            true,
        );
    }
    m
}
