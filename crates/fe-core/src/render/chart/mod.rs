//! Drawing a chart as vectors in its own box (C# `ChartRenderer`): the title, then the chart of its kind inside the
//! free area.

mod bars;
mod columns;
mod funnel;
mod pie;
mod progress;
mod radial;

use tiny_skia::{Pixmap, Transform};

use super::label::Label;
use super::vector::Canvas;
use crate::imaging::ColorRgba;
use crate::objects::{ChartData, ChartItem, ChartKind, ObjectBox};

const FONT: &str = "Segoe UI";

/// A rectangle in box space.
#[derive(Clone, Copy)]
pub(super) struct Area {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Area {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

/// Everything the chart kinds share.
pub(super) struct Ctx<'a, 'b> {
    pub c: &'a mut Canvas<'b>,
    pub chart: &'a ChartData,
    pub font: f64,
    pub ink: ColorRgba,
    pub faint: ColorRgba,
}

impl Ctx<'_, '_> {
    pub fn label(&self, text: &str, size: f64, bold: bool, max_width: Option<f64>) -> Label {
        match max_width {
            Some(w) => Label::fit(text, size.max(4.0), bold, FONT, w.max(4.0)),
            None => Label::new(text, size.max(4.0), bold, FONT),
        }
    }

    pub fn width_of(&self, text: &str, size: f64) -> f64 {
        self.label(text, size, false, None).width
    }

    /// Draws text centered on x with its top at y; returns the label.
    pub fn text_centered(
        &mut self,
        text: &str,
        cx: f64,
        y: f64,
        size: f64,
        bold: bool,
        max_width: Option<f64>,
    ) -> Label {
        let l = self.label(text, size, bold, max_width);
        self.c.label(&l, cx - l.width / 2.0, y, self.ink);
        l
    }

    /// The color between slices: the chart's background, white when it has none.
    pub fn separator(&self) -> ColorRgba {
        self.chart.background.unwrap_or(ColorRgba::WHITE)
    }

    /// Colored squares with the names, one row per item (pie, donut and half donut).
    pub fn legend(&mut self, area: Area, legend_width: f64) {
        if legend_width <= 0.0 {
            return;
        }
        let font = self.font;
        let n = self.chart.items.len();
        let (x, row) = (area.right() - legend_width, font * 1.6);
        let y = area.y + (area.h - row * n as f64) / 2.0;
        for i in 0..n {
            let color = self.chart.color_of(i);
            self.c.round_rect(x, y + row * i as f64 + row * 0.18, font * 0.9, font * 0.9, 2.0, color);
            let l = self.label(&self.chart.items[i].label, font * 0.95, false, Some(legend_width - font * 1.4));
            self.c.label(&l, x + font * 1.3, y + row * i as f64 + (row - l.height) / 2.0, self.ink);
        }
    }
}

pub(super) fn draw_chart(pm: &mut Pixmap, chart: &ChartData, b: &ObjectBox, t: Transform) {
    let mut canvas = Canvas { pm, t };
    let (width, height) = (b.width, b.height);
    if let Some(bg) = chart.background {
        canvas.rect(0.0, 0.0, width, height, bg);
    }
    let basis = width.min(height);
    let font = (basis * 0.045).clamp(7.0, 26.0) * chart.text_scale.clamp(0.4, 3.0);
    let ink = chart.ink();
    let faint = ink.with_alpha(60);
    let mut cx = Ctx { c: &mut canvas, chart, font, ink, faint };
    let mut top = 0.0;
    if !chart.title.trim().is_empty() {
        let l = cx.text_centered(&chart.title, width / 2.0, height * 0.03, font * 1.5, true, Some(width * 0.92));
        top = height * 0.03 + l.height + height * 0.02;
    }
    let area = Area::new(width * 0.04, top, width * 0.92, (height - top - height * 0.03).max(10.0));
    if chart.items.is_empty() {
        let l = cx.label("No numbers yet", font, false, None);
        cx.c.label(&l, (width - l.width) / 2.0, area.y + area.h / 2.0 - l.height / 2.0, faint);
        return;
    }
    match chart.kind {
        ChartKind::Pie => pie::pie(&mut cx, area, 0.0),
        ChartKind::Donut => pie::pie(&mut cx, area, 0.55),
        ChartKind::Bars => bars::bars(&mut cx, area),
        ChartKind::Line => columns::columns(&mut cx, area, columns::Mode::Line),
        ChartKind::Area => columns::columns(&mut cx, area, columns::Mode::Area),
        ChartKind::Lollipop => columns::columns(&mut cx, area, columns::Mode::Lollipop),
        ChartKind::HalfDonut => pie::half_donut(&mut cx, area),
        ChartKind::RadialBars => radial::radial_bars(&mut cx, area),
        ChartKind::Progress => progress::progress(&mut cx, area),
        ChartKind::Funnel => funnel::funnel(&mut cx, area),
        ChartKind::Columns => columns::columns(&mut cx, area, columns::Mode::Columns),
    }
}

/// A neat range for the numbers: it includes zero, and the lines are a round step apart.
pub(super) fn scale(items: &[ChartItem]) -> (f64, f64, f64) {
    let lo = items.iter().map(|i| i.value).fold(0.0, f64::min);
    let mut hi = items.iter().map(|i| i.value).fold(0.0, f64::max);
    if hi - lo < 1e-9 {
        hi = lo + 1.0;
    }
    let raw = (hi - lo) / 4.0;
    let magnitude = 10f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let step = (if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    }) * magnitude;
    ((lo / step).floor() * step, (hi / step).ceil() * step, step)
}

/// A percentage like the C# `0` format (rounded half away from zero).
pub(super) fn percent(value: f64, total: f64) -> String {
    format!("{}%", (value / total * 100.0).round() as i64)
}
