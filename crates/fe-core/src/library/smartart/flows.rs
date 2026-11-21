//! Lists and processes: blocks in a row, a column, steps and a timeline.

use super::{DARK, Job, SOFT, WHITE};
use crate::library::maker::Maker;
use crate::objects::ShapeKind::*;

pub(super) fn process(j: &Job) -> Maker {
    let n = j.words.len() as f64;
    let gap = 9.0;
    let bw = (100.0 - gap * (n - 1.0)) / n;
    let h = 30.0f64.min(bw * 1.1);
    let mut m = j.maker(h);
    for i in 0..j.words.len() {
        let x = i as f64 * (bw + gap);
        m.shape("Step", Rectangle, x, 0.0, bw, h).fill(j.color(i)).radius(3);
        m.fit_text(&j.words[i], x + bw / 2.0, h / 2.0, bw * 0.86, h * 0.8, 6.0, WHITE, true);
        if i + 1 < j.words.len() {
            m.shape("Arrow", Arrow, x + bw + 1.2, h / 2.0 - 3.5, gap - 2.4, 7).fill(DARK);
        }
    }
    m
}

pub(super) fn vertical_process(j: &Job) -> Maker {
    let n = j.words.len() as f64;
    let (bh, gap, bw) = (15.0, 9.0, 62.0);
    let h = n * bh + (n - 1.0) * gap;
    let mut m = j.maker(h);
    for i in 0..j.words.len() {
        let y = i as f64 * (bh + gap);
        m.shape("Step", Rectangle, 19, y, bw, bh).fill(j.color(i)).radius(3);
        m.fit_text(&j.words[i], 50.0, y + bh / 2.0, bw * 0.9, bh * 0.8, 7.0, WHITE, true);
        if i + 1 < j.words.len() {
            m.shape("Arrow", Triangle, 45, y + bh + 1.5, 10, gap - 3.0).fill(DARK).rotate(180);
        }
    }
    m
}

pub(super) fn chevrons(j: &Job) -> Maker {
    let n = j.words.len() as f64;
    let w = 100.0 / (n * 0.86 + 0.14);
    let h = 30.0f64.min(w * 0.9);
    let mut m = j.maker(h);
    for i in 0..j.words.len() {
        let x = i as f64 * w * 0.86;
        m.shape("Chevron", Chevron, x, 0.0, w, h).fill(j.color(i));
        m.fit_text(&j.words[i], x + w * 0.52, h / 2.0, w * 0.5, h * 0.7, 6.5, WHITE, true);
    }
    m
}

pub(super) fn steps(j: &Job) -> Maker {
    let n = j.words.len() as f64;
    let (bw, h) = (100.0 / n - 1.5, 62.0);
    let mut m = j.maker(h);
    for i in 0..j.words.len() {
        let bh = 20.0 + if j.words.len() == 1 { 0.0 } else { i as f64 * 40.0 / (n - 1.0) };
        let x = i as f64 * (bw + 1.5);
        m.shape("Step", Rectangle, x, h - bh, bw, bh).fill(j.color(i)).radius(2);
        m.fit_text(&j.words[i], x + bw / 2.0, h - bh / 2.0, bw * 0.88, bh * 0.7, 6.5, WHITE, true);
    }
    m
}

pub(super) fn timeline(j: &Job) -> Maker {
    let h = 56.0;
    let mut m = j.maker(h);
    m.line(50.0, h / 2.0, 100, 0, DARK, 1.6).named("Connector");
    let slot = 100.0 / j.words.len() as f64;
    for i in 0..j.words.len() {
        let cx = slot * (i as f64 + 0.5);
        m.shape("Dot", Ellipse, cx - 4.0, h / 2.0 - 4.0, 8, 8).fill(j.color(i)).stroke(WHITE, 1.2);
        let up = i % 2 == 0;
        m.line(cx, if up { h / 2.0 - 9.0 } else { h / 2.0 + 9.0 }, 10, 90, j.color(i), 0.9).named("Connector");
        m.fit_text(&j.words[i], cx, if up { 8.0 } else { h - 8.0 }, slot * 1.5, 12.0, 6.5, DARK, true);
    }
    m
}

pub(super) fn list(j: &Job) -> Maker {
    let n = j.words.len();
    let cols = if n <= 3 {
        n
    } else if n <= 4 {
        2
    } else {
        3
    };
    let rows = n.div_ceil(cols);
    let gap = 4.0;
    let bw = (100.0 - gap * (cols as f64 - 1.0)) / cols as f64;
    let bh = 26.0f64.min(bw * 0.62);
    let h = rows as f64 * bh + (rows as f64 - 1.0) * gap;
    let mut m = j.maker(h);
    for i in 0..n {
        let (x, y) = ((i % cols) as f64 * (bw + gap), (i / cols) as f64 * (bh + gap));
        m.shape("Box", Rectangle, x, y, bw, bh).fill(j.color(i)).radius(4);
        m.fit_text(&j.words[i], x + bw / 2.0, y + bh / 2.0, bw * 0.84, bh * 0.76, 7.0, WHITE, true);
    }
    m
}

pub(super) fn vertical_list(j: &Job) -> Maker {
    let n = j.words.len() as f64;
    let (rh, gap) = (13.0, 5.0);
    let h = n * rh + (n - 1.0) * gap;
    let mut m = j.maker(h);
    for i in 0..j.words.len() {
        let y = i as f64 * (rh + gap);
        m.shape("Bar", Rectangle, 6, y, 94, rh).fill(SOFT).radius(rh / 2.0);
        m.shape("Mark", Ellipse, 0.0, y - 1.0, rh + 2.0, rh + 2.0).fill(j.color(i)).stroke(WHITE, 1);
        m.fit_text(&(i + 1).to_string(), (rh + 2.0) / 2.0, y + rh / 2.0, rh * 0.7, rh * 0.7, 7.0, WHITE, true);
        m.fit_text(&j.words[i], 58.0, y + rh / 2.0, 74.0, rh * 0.8, 7.0, DARK, false);
    }
    m
}
