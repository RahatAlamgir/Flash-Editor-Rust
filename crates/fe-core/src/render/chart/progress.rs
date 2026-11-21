//! Progress bars: one track per value.

use super::{Area, Ctx};

pub(super) fn progress(cx: &mut Ctx, area: Area) {
    let chart = cx.chart;
    let font = cx.font;
    let items = &chart.items;
    let max = items.iter().map(|i| i.value.max(0.0)).fold(0.0, f64::max);
    let scale = if max <= 100.0 { 100.0 } else { max }; // percentages fill the track up to 100
    let row = area.h / items.len() as f64;
    let bar_h = (row * chart.bar_width.clamp(0.2, 1.0) * 0.5).min(font * 1.6);
    for (i, item) in items.iter().enumerate() {
        let top = area.y + row * i as f64;
        let fraction = (item.value.max(0.0) / scale).clamp(0.0, 1.0);
        let text_h = font * 1.3;
        let used = if chart.show_labels { text_h + bar_h + font * 0.2 } else { bar_h };
        let bar_y = top + (row - used) / 2.0 + if chart.show_labels { text_h + font * 0.2 } else { 0.0 };
        if chart.show_labels {
            let l = cx.label(&item.label, font, false, Some(area.w * 0.7));
            cx.c.label(&l, area.x, bar_y - l.height - font * 0.15, cx.ink);
        }
        if chart.show_values {
            let l = cx.label(&chart.format(item.value), font * 0.95, true, None);
            cx.c.label(&l, area.right() - l.width, bar_y - l.height - font * 0.15, cx.ink);
        }
        cx.c.round_rect(area.x, bar_y, area.w, bar_h, bar_h / 2.0, cx.faint);
        if fraction > 0.0 {
            cx.c.round_rect(area.x, bar_y, bar_h.max(area.w * fraction), bar_h, bar_h / 2.0, chart.color_of(i));
        }
    }
}
