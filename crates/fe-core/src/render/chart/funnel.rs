//! A funnel: one band per value, as wide as the value.

use super::{Area, Ctx};
use crate::imaging::ColorRgba;
use crate::render::vector::polygon;

pub(super) fn funnel(cx: &mut Ctx, area: Area) {
    let chart = cx.chart;
    let font = cx.font;
    let items: Vec<(usize, f64, &str)> = chart
        .items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.value > 0.0)
        .map(|(n, i)| (n, i.value, i.label.as_str()))
        .collect();
    if items.is_empty() {
        let l = cx.label("Needs positive numbers", font, false, None);
        cx.c.label(&l, area.x + (area.w - l.width) / 2.0, area.y + area.h / 2.0, ColorRgba::rgb(128, 128, 128));
        return;
    }
    let max = items.iter().map(|i| i.1).fold(0.0, f64::max);
    let row = area.h / items.len() as f64;
    let center = area.x + area.w / 2.0;
    let separator = cx.separator();
    for (i, (n, value, label)) in items.iter().enumerate() {
        let w_top = area.w * value / max;
        let w_bottom = if i + 1 < items.len() { area.w * items[i + 1].1 / max } else { w_top * 0.78 };
        let (y0, y1) = (area.y + row * i as f64, area.y + row * (i + 1) as f64);
        let corners = [
            (center - w_top / 2.0, y0),
            (center + w_top / 2.0, y0),
            (center + w_bottom / 2.0, y1),
            (center - w_bottom / 2.0, y1),
        ];
        if let Some(p) = polygon(&corners) {
            cx.c.fill(&p, chart.color_of(*n));
            cx.c.stroke(&p, separator, (font * 0.12).max(1.0), false);
        }
        let text = match (chart.show_labels, chart.show_values) {
            (true, true) => format!("{}  {}", label, chart.format(*value)),
            (true, false) => (*label).to_owned(),
            (false, true) => chart.format(*value),
            _ => String::new(),
        };
        if !text.is_empty() {
            let l = cx.label(&text, font, true, Some((w_top.min(w_bottom) * 0.92).max(20.0)));
            cx.c.label(&l, center - l.width / 2.0, (y0 + y1) / 2.0 - l.height / 2.0, ColorRgba::WHITE);
        }
    }
}
