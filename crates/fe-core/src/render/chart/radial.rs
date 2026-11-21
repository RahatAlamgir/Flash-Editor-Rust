//! Radial bars: rings that each show one value.

use std::f64::consts::{FRAC_PI_2, PI};

use super::{Area, Ctx};
use crate::render::vector::arc_path;

pub(super) fn radial_bars(cx: &mut Ctx, area: Area) {
    let chart = cx.chart;
    let font = cx.font;
    let items = &chart.items;
    let max = items.iter().map(|i| i.value.max(0.0)).fold(1e-9, f64::max);
    let legend_width = if chart.show_labels {
        let widest = items
            .iter()
            .map(|i| cx.width_of(&format!("{}  {}", i.label, chart.format(i.value)), font * 0.95))
            .fold(0.0, f64::max);
        (area.w * 0.42).min(widest + font * 2.4)
    } else {
        0.0
    };
    let gap = if legend_width > 0.0 { font } else { 0.0 };
    let outer = ((area.w - legend_width - gap).min(area.h) / 2.0).max(10.0);
    let center = (area.x + (area.w - legend_width - gap) / 2.0, area.y + area.h / 2.0);
    let n = items.len() as f64;
    let thick = (outer / (n * 1.55 + 0.6)).min(font * 1.6);
    let step = (thick * 1.45).min(outer * 0.9 / n);
    let span = PI * 1.5; // three quarters of a circle
    for (i, item) in items.iter().enumerate() {
        let r = outer - thick / 2.0 - step * i as f64;
        if r <= thick / 2.0 {
            break;
        }
        let sweep = span * (item.value.max(0.0) / max).clamp(0.001, 1.0);
        if let Some(p) = arc_path(center.0, center.1, r, -FRAC_PI_2, span) {
            cx.c.stroke(&p, cx.faint, thick, true);
        }
        if let Some(p) = arc_path(center.0, center.1, r, -FRAC_PI_2, sweep) {
            cx.c.stroke(&p, chart.color_of(i), thick, true);
        }
    }
    if legend_width > 0.0 {
        let (x, row) = (area.right() - legend_width, font * 1.6);
        let y = area.y + (area.h - row * n) / 2.0;
        for (i, item) in items.iter().enumerate() {
            let cy = y + row * i as f64;
            cx.c.circle(x + font * 0.45, cy + row / 2.0, font * 0.45, Some(chart.color_of(i)), None);
            let text = if chart.show_values {
                format!("{}  {}", item.label, chart.format(item.value))
            } else {
                item.label.clone()
            };
            let l = cx.label(&text, font * 0.95, false, Some(legend_width - font * 1.4));
            cx.c.label(&l, x + font * 1.3, cy + (row - l.height) / 2.0, cx.ink);
        }
    }
}
