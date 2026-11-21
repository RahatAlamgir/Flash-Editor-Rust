//! Pie, donut and half donut charts.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use tiny_skia::PathBuilder;

use super::{Area, Ctx, percent};
use crate::imaging::ColorRgba;
use crate::objects::ChartItem;
use crate::render::vector::{arc, on_circle};

/// The items with a positive value, with their place in the whole list (they keep their colors).
fn positive(items: &[ChartItem]) -> Vec<(usize, f64)> {
    items.iter().enumerate().filter(|(_, i)| i.value > 0.0).map(|(n, i)| (n, i.value)).collect()
}

fn message(cx: &mut Ctx, area: Area, text: &str) {
    let l = cx.label(text, cx.font, false, None);
    cx.c.label(&l, area.x + (area.w - l.width) / 2.0, area.y + area.h / 2.0, cx.ink);
}

fn legend_width(cx: &Ctx, share: f64) -> f64 {
    let font = cx.font;
    let widest =
        cx.chart.items.iter().filter(|i| i.value > 0.0).map(|i| cx.width_of(&i.label, font * 0.95)).fold(0.0, f64::max);
    (share).min(widest + font * 2.4)
}

pub(super) fn pie(cx: &mut Ctx, area: Area, hole: f64) {
    let chart = cx.chart;
    let font = cx.font;
    let items = positive(&chart.items);
    let total: f64 = items.iter().map(|i| i.1).sum();
    if total <= 0.0 {
        message(cx, area, "Pie charts need positive numbers");
        return;
    }
    let legend = if chart.show_labels { legend_width(cx, area.w * 0.42) } else { 0.0 };
    let gap = if legend > 0.0 { font } else { 0.0 };
    let diameter = (area.w - legend - gap).min(area.h).max(10.0);
    let center = (area.x + (area.w - legend - gap) / 2.0, area.y + area.h / 2.0);
    let r = diameter / 2.0;
    let separator = cx.separator();
    let line = (r * 0.015).max(1.0);
    let mut angle = -FRAC_PI_2;
    for (n, value) in &items {
        let sweep = value / total * TAU;
        let color = chart.color_of(*n);
        let mut pb = PathBuilder::new();
        if items.len() == 1 {
            let p = on_circle(center.0, center.1, r, angle);
            pb.move_to(p.0 as f32, p.1 as f32);
            arc(&mut pb, center.0, center.1, r, angle, TAU);
        } else {
            pb.move_to(center.0 as f32, center.1 as f32);
            let a0 = on_circle(center.0, center.1, r, angle);
            pb.line_to(a0.0 as f32, a0.1 as f32);
            arc(&mut pb, center.0, center.1, r, angle, sweep);
        }
        pb.close();
        if let Some(p) = pb.finish() {
            cx.c.fill(&p, color);
            cx.c.stroke(&p, separator, line, false);
        }
        if chart.show_values && sweep > 0.18 {
            let mid = angle + sweep / 2.0;
            let rr = r * if hole > 0.0 { (1.0 + hole) / 2.0 } else { 0.66 };
            let l = cx.label(&percent(*value, total), font * 0.95, true, None);
            let at = on_circle(center.0, center.1, rr, mid);
            cx.c.label(&l, at.0 - l.width / 2.0, at.1 - l.height / 2.0, ColorRgba::WHITE);
        }
        angle += sweep;
    }
    if hole > 0.0 {
        cx.c.circle(center.0, center.1, r * hole, Some(separator), None);
    }
    cx.legend(area, legend);
}

pub(super) fn half_donut(cx: &mut Ctx, area: Area) {
    let chart = cx.chart;
    let font = cx.font;
    let items = positive(&chart.items);
    let total: f64 = items.iter().map(|i| i.1).sum();
    if total <= 0.0 {
        message(cx, area, "Needs positive numbers");
        return;
    }
    let legend = if chart.show_labels { legend_width(cx, area.w * 0.38) } else { 0.0 };
    let gap = if legend > 0.0 { font } else { 0.0 };
    let r = ((area.w - legend - gap) / 2.0).min(area.h * 0.86).max(10.0);
    let center = (area.x + (area.w - legend - gap) / 2.0, area.y + area.h / 2.0 + r / 2.0);
    let inner = r * 0.58;
    let separator = cx.separator();
    let line = (r * 0.012).max(1.0);
    let mut angle = PI;
    for (n, value) in &items {
        let sweep = value / total * PI;
        let mut pb = PathBuilder::new();
        let p = on_circle(center.0, center.1, r, angle);
        pb.move_to(p.0 as f32, p.1 as f32);
        arc(&mut pb, center.0, center.1, r, angle, sweep);
        let q = on_circle(center.0, center.1, inner, angle + sweep);
        pb.line_to(q.0 as f32, q.1 as f32);
        arc(&mut pb, center.0, center.1, inner, angle + sweep, -sweep);
        pb.close();
        if let Some(path) = pb.finish() {
            cx.c.fill(&path, chart.color_of(*n));
            cx.c.stroke(&path, separator, line, false);
        }
        if chart.show_values && sweep > 0.22 {
            let mid = angle + sweep / 2.0;
            let l = cx.label(&percent(*value, total), font * 0.95, true, None);
            let at = on_circle(center.0, center.1, (r + inner) / 2.0, mid);
            cx.c.label(&l, at.0 - l.width / 2.0, at.1 - l.height / 2.0, ColorRgba::WHITE);
        }
        angle += sweep;
    }
    cx.legend(area, legend);
}
