//! Rulers along the top and left of the canvas (C# `RulerControl`): pixel numbers that follow zoom and scroll,
//! the page part a bit lighter, and a blue marker at the mouse position.

use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, pos2};

/// Width of a ruler in points.
pub const THICKNESS: f32 = 20.0;

const BACK: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x2B);
const PAGE: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
const TEXT: Color32 = Color32::from_rgb(0xB0, 0xB0, 0xB0);
const TICK: Color32 = Color32::from_rgb(0x8A, 0x8A, 0x8A);
const MINOR: Color32 = Color32::from_rgb(0x5E, 0x5E, 0x5E);
const EDGE: Color32 = Color32::from_rgb(0x4A, 0x4A, 0x4A);
const MARKER: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);
const NICE_STEPS: [f64; 15] =
    [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0, 50000.0];

/// One ruler.
pub struct Ruler {
    pub rect: Rect,
    pub vertical: bool,
    pub zoom: f64,
    /// Screen position (along the ruler, from its start) of canvas coordinate 0.
    pub offset: f64,
    /// Canvas width (or height) in pixels.
    pub page_length: f64,
    /// Mouse position along the ruler, if the mouse is over the canvas.
    pub marker: Option<f32>,
}

impl Ruler {
    pub fn paint(&self, painter: &Painter) {
        let length = if self.vertical { self.rect.height() } else { self.rect.width() } as f64;
        painter.rect_filled(self.rect, 0, BACK);
        let page_start = self.offset.max(0.0);
        let page_end = (self.offset + self.page_length * self.zoom).min(length);
        if page_end > page_start {
            painter.rect_filled(self.span(page_start, page_end, 0.0, THICKNESS), 0, PAGE);
        }
        let major = NICE_STEPS.iter().copied().find(|s| s * self.zoom >= 70.0).unwrap_or(NICE_STEPS[14]);
        let parts = parts(major);
        let minor = major / parts as f64;
        let draw_minor = parts > 1 && minor * self.zoom >= 4.0;
        let step = if draw_minor { minor } else { major };
        let first = -self.offset / self.zoom;
        let last = (length - self.offset) / self.zoom;
        let mut v = (first / step).floor() * step;
        while v <= last + step {
            let pos = (self.offset + v * self.zoom).round() + 0.5;
            if pos >= -1.0 && pos <= length + 1.0 {
                let ratio = v / major;
                if (ratio - ratio.round()).abs() < 1e-6 {
                    self.line(painter, pos, 0.0, THICKNESS as f64, TICK);
                    self.label(painter, &format!("{}", v.round() as i64), pos + 3.0);
                } else {
                    let half = parts.is_multiple_of(2) && ((ratio * 2.0) - (ratio * 2.0).round()).abs() < 1e-6;
                    let from = THICKNESS as f64 - if half { 9.0 } else { 5.0 };
                    self.line(painter, pos, from, THICKNESS as f64, MINOR);
                }
            }
            v += step;
        }
        let edge = THICKNESS - 0.5;
        if self.vertical {
            painter.vline(self.rect.left() + edge, self.rect.y_range(), Stroke::new(1.0, EDGE));
        } else {
            painter.hline(self.rect.x_range(), self.rect.top() + edge, Stroke::new(1.0, EDGE));
        }
        if let Some(m) = self.marker.filter(|m| *m >= 0.0 && (*m as f64) <= length) {
            self.line(painter, m as f64, 0.0, THICKNESS as f64, MARKER);
        }
    }

    /// A rectangle given positions along and across the ruler.
    fn span(&self, from: f64, to: f64, cross_from: f32, cross_to: f32) -> Rect {
        let r = self.rect;
        if self.vertical {
            Rect::from_min_max(
                pos2(r.left() + cross_from, r.top() + from as f32),
                pos2(r.left() + cross_to, r.top() + to as f32),
            )
        } else {
            Rect::from_min_max(
                pos2(r.left() + from as f32, r.top() + cross_from),
                pos2(r.left() + to as f32, r.top() + cross_to),
            )
        }
    }

    fn line(&self, painter: &Painter, along: f64, cross_from: f64, cross_to: f64, color: Color32) {
        let r = self.rect;
        let stroke = Stroke::new(1.0, color);
        if self.vertical {
            let y = r.top() + along as f32;
            painter.hline((r.left() + cross_from as f32)..=(r.left() + cross_to as f32), y, stroke);
        } else {
            let x = r.left() + along as f32;
            painter.vline(x, (r.top() + cross_from as f32)..=(r.top() + cross_to as f32), stroke);
        }
    }

    fn label(&self, painter: &Painter, text: &str, along: f64) {
        let font = FontId::proportional(9.0);
        if self.vertical {
            // Turned a quarter to the left, reading upward, like the C# ruler.
            let galley = painter.layout_no_wrap(text.to_owned(), font, TEXT);
            let anchor = pos2(self.rect.left() + 2.0, self.rect.top() + along as f32 + galley.size().x);
            let shape = egui::epaint::TextShape::new(anchor, galley, TEXT).with_angle(-std::f32::consts::FRAC_PI_2);
            painter.add(shape);
        } else {
            painter.text(
                pos2(self.rect.left() + along as f32, self.rect.top() + 2.0),
                Align2::LEFT_TOP,
                text,
                font,
                TEXT,
            );
        }
    }
}

/// How many small ticks a big step is divided into (C# `RulerControl.Parts`).
fn parts(major: f64) -> u32 {
    if major < 10.0 {
        return if major == 1.0 {
            1
        } else if major == 2.0 {
            2
        } else {
            5
        };
    }
    let mantissa = major / 10f64.powf(major.log10().floor());
    if mantissa < 1.5 {
        10
    } else if mantissa < 3.0 {
        4
    } else {
        5
    }
}

/// The corner square where the two rulers meet.
pub fn paint_corner(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, 0, BACK);
}
