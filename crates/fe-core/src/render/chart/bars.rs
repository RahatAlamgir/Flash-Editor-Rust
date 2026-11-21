//! Horizontal bars (C# `DrawBars`).

use super::{Area, Ctx, scale};

pub(super) fn bars(cx: &mut Ctx, area: Area) {
    let chart = cx.chart;
    let font = cx.font;
    let items = &chart.items;
    let (min, max, step) = scale(items);
    let mut label_room = if chart.show_labels {
        items.iter().map(|i| cx.width_of(&i.label, font * 0.9)).fold(0.0, f64::max) + font * 0.6
    } else {
        4.0
    };
    label_room = label_room.min(area.w * 0.4);
    let value_room =
        if chart.show_values { cx.width_of(&chart.format(max), font * 0.85) + font * 0.8 } else { font * 0.4 };
    let plot = Area::new(
        area.x + label_room,
        area.y,
        (area.w - label_room - value_room).max(10.0),
        (area.h - if chart.show_grid { font * 1.6 } else { 0.0 }).max(10.0),
    );
    let x_of = |v: f64| plot.x + (v - min) / (max - min) * plot.w;
    if chart.show_grid {
        let mut v = min;
        while v <= max + step * 0.001 {
            let x = x_of(v);
            cx.c.line((x, plot.y), (x, plot.bottom()), cx.faint, (font * 0.07).max(0.6), false);
            let l = cx.label(&chart.format(v), font * 0.85, false, None);
            cx.c.label(&l, x - l.width / 2.0, plot.bottom() + font * 0.3, cx.ink);
            v += step;
        }
    }
    let zero = x_of(0.0f64.min(max).max(min));
    let slot = plot.h / items.len() as f64;
    for (i, item) in items.iter().enumerate() {
        let cy = plot.y + slot * (i as f64 + 0.5);
        let x = x_of(item.value);
        let bar_h = slot * chart.bar_width.clamp(0.1, 1.0);
        let r = (bar_h * 0.12).min(6.0);
        cx.c.round_rect(x.min(zero), cy - bar_h / 2.0, (x - zero).abs().max(0.5), bar_h, r, chart.color_of(i));
        if chart.show_labels {
            let l = cx.label(&item.label, font * 0.9, false, Some(label_room - font * 0.4));
            cx.c.label(&l, plot.x - l.width - font * 0.4, cy - l.height / 2.0, cx.ink);
        }
        if chart.show_values {
            let l = cx.label(&chart.format(item.value), font * 0.85, false, None);
            let tx = if item.value >= 0.0 { x + font * 0.3 } else { x - l.width - font * 0.3 };
            cx.c.label(&l, tx, cy - l.height / 2.0, cx.ink);
        }
    }
    cx.c.line((zero, plot.y), (zero, plot.bottom()), cx.ink, (font * 0.09).max(0.8), false);
}
