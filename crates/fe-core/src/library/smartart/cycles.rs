//! Cycles and hierarchies: a ring of nodes, hub and spokes, an organization chart, a horizontal tree.

use std::f64::consts::PI;

use super::{DARK, Job, SOFT, WHITE};
use crate::library::maker::Maker;
use crate::objects::ShapeKind::*;

pub(super) fn cycle(j: &Job) -> Maker {
    let n = j.words.len();
    let mut m = j.maker(100.0);
    m.shape("Ring", Ellipse, 14, 14, 72, 72).stroke(SOFT, 2);
    let r = 15.0f64.min(100.0 / n as f64 * 0.9);
    for i in 0..n {
        let a = -PI / 2.0 + i as f64 * PI * 2.0 / n as f64;
        let (cx, cy) = (50.0 + a.cos() * 36.0, 50.0 + a.sin() * 36.0);
        m.shape("Node", Ellipse, cx - r, cy - r, r * 2.0, r * 2.0).fill(j.color(i)).stroke(WHITE, 1.4);
        m.fit_text(&j.words[i], cx, cy, r * 1.55, r * 1.35, 6.0, WHITE, true);
        let mid = a + PI / n as f64;
        m.shape("Arrow", Triangle, 50.0 + mid.cos() * 36.0 - 3.5, 50.0 + mid.sin() * 36.0 - 3.5, 7, 7)
            .fill(DARK)
            .rotate(mid * 180.0 / PI + 180.0);
    }
    m
}

pub(super) fn hub(j: &Job) -> Maker {
    let mut m = j.maker(100.0);
    let spokes = j.words.len() - 1;
    let r = 14.0f64.min(100.0 / spokes as f64 * 0.85);
    let angle = |i: usize| -PI / 2.0 + i as f64 * PI * 2.0 / spokes as f64;
    for i in 0..spokes {
        let a = angle(i);
        m.line(50.0 + a.cos() * 22.0, 50.0 + a.sin() * 22.0, 22, a * 180.0 / PI, SOFT, 3).named("Connector");
    }
    m.shape("Hub", Ellipse, 31, 31, 38, 38).fill(DARK);
    m.fit_text(&j.words[0], 50.0, 50.0, 31.0, 26.0, 9.0, WHITE, true);
    for i in 0..spokes {
        let a = angle(i);
        let (cx, cy) = (50.0 + a.cos() * 36.0, 50.0 + a.sin() * 36.0);
        m.shape("Node", Ellipse, cx - r, cy - r, r * 2.0, r * 2.0).fill(j.color(i)).stroke(WHITE, 1.2);
        m.fit_text(&j.words[i + 1], cx, cy, r * 1.6, r * 1.35, 6.0, WHITE, true);
    }
    m
}

pub(super) fn org_chart(j: &Job) -> Maker {
    let kids = j.words.len() - 1;
    let k = kids as f64;
    let bw = 30.0f64.min((100.0 - 4.0 * (k - 1.0)) / k);
    let (bh, gap) = (14.0, 4.0);
    let h = 18.0 + 14.0 + 14.0;
    let mut m = j.maker(h);
    let total = k * bw + (k - 1.0) * gap;
    let start = (100.0 - total) / 2.0;
    let mid_y = bh + 9.0;
    m.line(50.0, bh + 4.5, 9, 90, DARK, 1.2).named("Connector");
    if kids > 1 {
        m.line(50.0, mid_y, total - bw, 0, DARK, 1.2).named("Connector");
    }
    for i in 0..kids {
        let cx = start + i as f64 * (bw + gap) + bw / 2.0;
        m.line(cx, mid_y + (h - bh - mid_y) / 2.0, h - bh - mid_y, 90, DARK, 1.2).named("Connector");
        m.shape("Box", Rectangle, cx - bw / 2.0, h - bh, bw, bh).fill(j.color(i + 1)).radius(3);
        m.fit_text(&j.words[i + 1], cx, h - bh / 2.0, bw * 0.88, bh * 0.78, 6.5, WHITE, true);
    }
    m.shape("Top", Rectangle, 50.0 - 19.0, 0.0, 38, bh).fill(j.color(0)).radius(3);
    m.fit_text(&j.words[0], 50.0, bh / 2.0, 34.0, bh * 0.78, 7.0, WHITE, true);
    m
}

pub(super) fn tree(j: &Job) -> Maker {
    let kids = j.words.len() - 1;
    let k = kids as f64;
    let (bh, gap, root_h) = (11.0, 4.0, 22.0);
    let h = (k * bh + (k - 1.0) * gap).max(root_h);
    let mut m = j.maker(h);
    let offset = (h - (k * bh + (k - 1.0) * gap)) / 2.0;
    m.shape("Root", Rectangle, 0.0, h / 2.0 - root_h / 2.0, 32, root_h).fill(j.color(0)).radius(3);
    m.fit_text(&j.words[0], 16.0, h / 2.0, 29.0, root_h * 0.78, 7.5, WHITE, true);
    m.line(36.0, h / 2.0, 8, 0, DARK, 1.2).named("Connector");
    if kids > 1 {
        m.line(40.0, h / 2.0, (k - 1.0) * (bh + gap), 90, DARK, 1.2).named("Connector");
    }
    for i in 0..kids {
        let y = offset + i as f64 * (bh + gap);
        m.line(44.0, y + bh / 2.0, 8, 0, DARK, 1.2).named("Connector");
        m.shape("Leaf", Rectangle, 48.0, y, 52, bh).fill(j.color(i + 1)).radius(3);
        m.fit_text(&j.words[i + 1], 74.0, y + bh / 2.0, 47.0, bh * 0.78, 6.5, WHITE, true);
    }
    m
}
