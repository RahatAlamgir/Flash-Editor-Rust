//! The small picture in front of a layer's name that tells what the layer holds: text, shape, picture, group, QR code,
//! barcode, chart, plain pixels or an adjustment. Drawn with lines and shapes, so it does not depend on a font.

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

/// The size of the picture.
pub const SIZE: f32 = 16.0;

/// What a layer is, for the icon.
pub fn kind_of(layer: &fe_core::document::Layer) -> &'static str {
    if layer.is_adjustment() {
        return "adjustment";
    }
    match layer.object().map(|o| o.kind().to_owned()).as_deref() {
        Some("text") => "text",
        Some("shape") => "shape",
        Some("image") => "image",
        Some("group") => "group",
        Some("qr") => "qr",
        Some("barcode") => "barcode",
        Some("chart") => "chart",
        Some(_) => "shape",
        None => "pixels",
    }
}

/// The words for the tooltip.
pub fn describe(kind: &str) -> &'static str {
    match kind {
        "text" => "Text",
        "shape" => "Shape",
        "image" => "Picture",
        "group" => "Group",
        "qr" => "QR code",
        "barcode" => "Barcode",
        "chart" => "Chart",
        "adjustment" => "Adjustment layer",
        _ => "Plain pixels",
    }
}

/// Draws the icon of `kind` in the 16 x 16 square at `rect`.
pub fn paint(painter: &Painter, rect: Rect, kind: &str, color: Color32) {
    let c = rect.center();
    let line = Stroke::new(1.3, color);
    let at = |x: f32, y: f32| pos2(c.x + x, c.y + y);
    match kind {
        "text" => {
            painter.text(c, Align2::CENTER_CENTER, "T", FontId::proportional(15.0), color);
        }
        "shape" => {
            painter.rect_stroke(Rect::from_min_size(at(-7.0, -7.0), vec2(9.0, 9.0)), 2, line, egui::StrokeKind::Inside);
            painter.circle_stroke(at(2.5, 2.5), 4.5, line);
        }
        "image" => {
            painter.rect_stroke(Rect::from_center_size(c, vec2(14.0, 11.0)), 2, line, egui::StrokeKind::Inside);
            painter
                .add(Shape::line(vec![at(-6.0, 4.0), at(-2.0, -1.0), at(1.0, 2.0), at(3.0, 0.0), at(6.0, 4.0)], line));
            painter.circle_filled(at(3.5, -2.5), 1.5, color);
        }
        "group" => {
            painter.rect_stroke(Rect::from_min_size(at(-7.0, -7.0), vec2(9.0, 9.0)), 1, line, egui::StrokeKind::Inside);
            painter.rect_stroke(Rect::from_min_size(at(-2.0, -2.0), vec2(9.0, 9.0)), 1, line, egui::StrokeKind::Inside);
        }
        "qr" => {
            for (x, y) in [(-7.0, -7.0), (1.0, -7.0), (-7.0, 1.0)] {
                painter.rect_stroke(Rect::from_min_size(at(x, y), vec2(6.0, 6.0)), 0, line, egui::StrokeKind::Inside);
                painter.rect_filled(Rect::from_min_size(at(x + 2.0, y + 2.0), vec2(2.0, 2.0)), 0, color);
            }
            painter.rect_filled(Rect::from_min_size(at(2.0, 2.0), vec2(2.0, 2.0)), 0, color);
            painter.rect_filled(Rect::from_min_size(at(5.0, 5.0), vec2(2.0, 2.0)), 0, color);
        }
        "barcode" => {
            for (x, w) in [(-7.0, 1.5), (-4.5, 1.0), (-2.5, 2.0), (0.5, 1.0), (2.5, 1.5), (5.0, 2.0)] {
                painter.rect_filled(Rect::from_min_size(at(x, -6.0), vec2(w, 12.0)), 0, color);
            }
        }
        "chart" => {
            for (x, h) in [(-7.0, 6.0), (-2.5, 11.0), (2.0, 8.0)] {
                painter.rect_filled(Rect::from_min_size(at(x, 6.0 - h), vec2(4.0, h)), 0, color);
            }
        }
        "adjustment" => {
            painter.circle_stroke(c, 6.5, line);
            let half: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 12.0;
                    at(6.5 * a.cos(), 6.5 * a.sin())
                })
                .collect();
            painter.add(Shape::convex_polygon(half, color, Stroke::NONE));
        }
        _ => {
            // Plain pixels: a small checkerboard.
            for (x, y) in [(-6.0, -6.0), (0.0, 0.0)] {
                painter.rect_filled(Rect::from_min_size(at(x, y), vec2(6.0, 6.0)), 0, color);
            }
            for (x, y) in [(0.0, -6.0), (-6.0, 0.0)] {
                painter.rect_stroke(Rect::from_min_size(at(x, y), vec2(6.0, 6.0)), 0, line, egui::StrokeKind::Inside);
            }
        }
    }
}
