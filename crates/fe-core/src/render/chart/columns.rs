//! Columns, line, area and lollipop charts (C# `DrawColumnsOrLine`).

use tiny_skia::PathBuilder;

use super::{Area, Ctx, scale};
use crate::imaging::ColorRgba;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Columns,
    Line,
    Area,
    Lollipop,
}

pub(super) fn columns(cx: &mut Ctx, area: Area, mode: Mode) {
    let chart = cx.chart;
    let font = cx.font;
    let items = &chart.items;
    let (line, area_fill, lollipop) =
        (matches!(mode, Mode::Line | Mode::Area), mode == Mode::Area, mode == Mode::Lollipop);
    let (min, max, step) = scale(items);
    let label_room = if chart.show_labels { font * 1.9 } else { 4.0 };
    let value_room = if chart.show_grid { cx.width_of(&chart.format(max), font) + font * 0.8 } else { 4.0 };
    let plot = Area::new(
        area.x + value_room,
        area.y + font * 1.2,
        (area.w - value_room).max(10.0),
        (area.h - label_room - font * 1.2).max(10.0),
    );
    let y_of = |v: f64| plot.bottom() - (v - min) / (max - min) * plot.h;

    if chart.show_grid {
        let mut v = min;
        while v <= max + step * 0.001 {
            let y = y_of(v);
            cx.c.line((plot.x, y), (plot.right(), y), cx.faint, (font * 0.07).max(0.6), false);
            let l = cx.label(&chart.format(v), font * 0.85, false, None);
            cx.c.label(&l, plot.x - l.width - font * 0.4, y - l.height / 2.0, cx.ink);
            v += step;
        }
    }
    let zero = y_of(0.0f64.min(max).max(min));
    let slot = plot.w / items.len() as f64;
    let mut points: Vec<(f64, f64)> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let (px, y) = (plot.x + slot * (i as f64 + 0.5), y_of(item.value));
        let color = chart.color_of(i);
        if line {
            points.push((px, y));
        } else {
            let bar_w = slot * chart.bar_width.clamp(0.1, 1.0);
            if lollipop {
                let stem = (bar_w * 0.12).min(font * 0.28).max(1.5);
                cx.c.line((px, zero), (px, y), color, stem, false);
                cx.c.circle(px, y, (bar_w * 0.26).min(font * 0.9), Some(color), None);
            } else {
                let r = (bar_w * 0.12).min(6.0);
                cx.c.round_rect(px - bar_w / 2.0, y.min(zero), bar_w, (zero - y).abs().max(0.5), r, color);
            }
        }
        if chart.show_labels {
            let l = cx.label(&item.label, font * 0.9, false, Some(slot * 0.95));
            cx.c.label(&l, px - l.width / 2.0, plot.bottom() + font * 0.35, cx.ink);
        }
        if chart.show_values && !line {
            let l = cx.label(&chart.format(item.value), font * 0.85, false, None);
            let extra = if lollipop { font * 0.9 } else { 0.0 };
            let ty =
                if item.value >= 0.0 { y - l.height - if lollipop { extra } else { 1.0 } } else { y + 1.0 + extra };
            cx.c.label(&l, px - l.width / 2.0, ty, cx.ink);
        }
    }

    if line && !points.is_empty() {
        let color = chart.color_of(0);
        let width = (font * 0.22).max(1.5);
        if area_fill && points.len() > 1 {
            let mut pb = PathBuilder::new();
            pb.move_to(points[0].0 as f32, zero as f32);
            for p in &points {
                pb.line_to(p.0 as f32, p.1 as f32);
            }
            pb.line_to(points[points.len() - 1].0 as f32, zero as f32);
            pb.close();
            if let Some(p) = pb.finish() {
                cx.c.fill(&p, color.with_alpha(85));
            }
        }
        if points.len() > 1 {
            let mut pb = PathBuilder::new();
            pb.move_to(points[0].0 as f32, points[0].1 as f32);
            for p in &points[1..] {
                pb.line_to(p.0 as f32, p.1 as f32);
            }
            if let Some(p) = pb.finish() {
                cx.c.stroke(&p, color, width, true);
            }
        }
        for (i, p) in points.iter().enumerate() {
            cx.c.circle(p.0, p.1, font * 0.38, Some(ColorRgba::WHITE), Some((color, width)));
            if chart.show_values {
                let l = cx.label(&chart.format(items[i].value), font * 0.85, false, None);
                cx.c.label(&l, p.0 - l.width / 2.0, p.1 - l.height - font * 0.5, cx.ink);
            }
        }
    }
    cx.c.line((plot.x, zero), (plot.right(), zero), cx.ink, (font * 0.09).max(0.8), false);
}
