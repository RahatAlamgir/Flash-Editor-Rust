//! Properties > Shape (C# `ObjectPropertiesViewModel`, shape part): fill and border colors, border width and
//! style (solid / dashed / dotted, dash and gap), for lines the bend and arrow heads, and the settings some shapes
//! have (corners, hole / bite / arms, angle, points).

use egui::{Color32, Ui};
use fe_core::imaging::ColorRgba;
use fe_core::objects::{BendStyle, ShapeData, ShapeKind, StrokeStyle};
use fe_ui_kit::widgets::number_slider;

use super::label;
use crate::app::FlashApp;
use crate::commands::objects as cmd;
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

/// Changes the shape's settings (slider drags of one setting merge into one undo step).
fn change(app: &mut FlashApp, what: &'static str, mut f: impl FnMut(&mut ShapeData)) {
    cmd::change_object(app, what, |o| {
        if let Some(mut s) = o.shape() {
            f(&mut s);
            o.set_shape(&s);
        }
    });
}

/// A color square that opens the color picker.
pub(super) fn swatch(ui: &mut Ui, color: Option<ColorRgba>, tip: &str) -> bool {
    let c = color.unwrap_or(ColorRgba::TRANSPARENT);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 22.0), egui::Sense::click());
    fe_ui_kit::widgets::paint_checkerboard(ui.painter(), rect, 4.0);
    ui.painter().rect_filled(rect, 2, Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a));
    ui.painter().rect_stroke(
        rect,
        2,
        egui::Stroke::new(1.0, fe_ui_kit::theme::colors::DIVIDER),
        egui::StrokeKind::Inside,
    );
    response.on_hover_text(tip).clicked()
}

pub(super) fn slider(
    ui: &mut Ui,
    text: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    unit: &str,
) -> bool {
    ui.horizontal(|ui| {
        label(ui, text);
        number_slider(ui, value, range, unit, 96.0, 52.0).changed()
    })
    .inner
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(s) = crate::commands::parts::target_object(app).and_then(|o| o.shape()) else { return };
    let line = s.is_line();
    let custom = s.vertices.is_some();
    ui.horizontal(|ui| {
        label(ui, "Type");
        let extra = if custom { " (custom outline)" } else { "" };
        ui.label(format!("{}{extra}", s.kind.name()));
    });
    if s.can_fill() {
        ui.horizontal(|ui| {
            let mut on = s.fill.is_some();
            if ui.add_sized([72.0, 22.0], egui::Checkbox::new(&mut on, "Fill")).changed() {
                let color = if on { Some(ColorRgba::rgb(59, 130, 246)) } else { None };
                change(app, "Fill", |d| d.fill = color);
            }
            if swatch(ui, s.fill, "Fill color") {
                app.dialog =
                    Some(Dialog::Color(ColorPicker::new(ColorTarget::ShapeFill, s.fill.unwrap_or(ColorRgba::WHITE))));
            }
        });
    }
    ui.horizontal(|ui| {
        let mut on = s.stroke.is_some();
        let text = if line { "Line" } else { "Border" };
        if ui.add_sized([72.0, 22.0], egui::Checkbox::new(&mut on, text)).changed() {
            let color = if on { Some(ColorRgba::BLACK) } else { None };
            change(app, "Border", |d| {
                d.stroke = color;
                if on && d.stroke_width <= 0.0 {
                    d.stroke_width = 4.0;
                }
            });
        }
        if swatch(ui, s.stroke, "Border / line color") {
            app.dialog =
                Some(Dialog::Color(ColorPicker::new(ColorTarget::ShapeStroke, s.stroke.unwrap_or(ColorRgba::BLACK))));
        }
    });
    let mut width = s.stroke_width;
    if slider(ui, "Width", &mut width, 0.0..=200.0, "px") {
        change(app, "Border Width", |d| d.stroke_width = width);
    }
    ui.horizontal(|ui| {
        label(ui, "Line style");
        let mut dash = s.dash;
        egui::ComboBox::from_id_salt("dash-style")
            .selected_text(match dash {
                StrokeStyle::Solid => "Solid",
                StrokeStyle::Dashed => "Dashed",
                StrokeStyle::Dotted => "Dotted",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut dash, StrokeStyle::Solid, "Solid");
                ui.selectable_value(&mut dash, StrokeStyle::Dashed, "Dashed");
                ui.selectable_value(&mut dash, StrokeStyle::Dotted, "Dotted");
            })
            .response
            .on_hover_text("Solid, dashed or dotted border / line");
        if dash != s.dash {
            change(app, "Line Style", |d| d.dash = dash);
        }
    });
    let w = s.stroke_width.max(0.1);
    if s.dash == StrokeStyle::Dashed {
        let mut len = if s.dash_length > 0.0 { s.dash_length } else { (3.0 * w).round() };
        if slider(ui, "Dash", &mut len, 1.0..=200.0, "px") {
            change(app, "Dash", |d| d.dash_length = len);
        }
    }
    if s.dash != StrokeStyle::Solid {
        let auto = if s.dash == StrokeStyle::Dashed { 2.0 * w } else { 1.2 * w };
        let mut gap = if s.dash_gap > 0.0 { s.dash_gap } else { auto.round() };
        if slider(ui, "Gap", &mut gap, 1.0..=200.0, "px") {
            change(app, "Gap", |d| d.dash_gap = gap);
        }
    }
    if line {
        let mut bend = (s.bend * 100.0).round();
        if slider(ui, "Bend", &mut bend, -100.0..=100.0, "%") {
            change(app, "Bend", |d| d.bend = bend / 100.0);
        }
        ui.horizontal(|ui| {
            label(ui, "Curve");
            let mut style = s.bend_style;
            ui.selectable_value(&mut style, BendStyle::Arc, "Arc");
            ui.selectable_value(&mut style, BendStyle::Wave, "Wave");
            if style != s.bend_style {
                change(app, "Curve", |d| d.bend_style = style);
            }
        });
    }
    if line || (s.kind == ShapeKind::Path && !s.closed) {
        ui.horizontal(|ui| {
            let (mut a, mut b) = (s.arrow_start, s.arrow_end);
            if ui.checkbox(&mut a, "Arrow at start").changed() {
                change(app, "Arrow", |d| d.arrow_start = a);
            }
            if ui.checkbox(&mut b, "Arrow at end").changed() {
                change(app, "Arrow", |d| d.arrow_end = b);
            }
        });
    }
    let has_corners = !custom && matches!(s.kind, ShapeKind::Rectangle | ShapeKind::SpeechBubble | ShapeKind::Frame);
    if has_corners {
        let mut r = s.corner_radius;
        if slider(ui, "Corners", &mut r, 0.0..=500.0, "px") {
            change(app, "Corners", |d| d.corner_radius = r);
        }
    }
    if !custom && s.kind.has_inner() {
        let mut inner = (s.inner * 100.0).round();
        if slider(ui, s.kind.inner_label(), &mut inner, 0.0..=95.0, "%") {
            change(app, "Inner", |d| d.inner = inner / 100.0);
        }
    }
    if !custom && s.kind == ShapeKind::Pie {
        let mut sweep = s.sweep;
        if slider(ui, "Angle", &mut sweep, 10.0..=360.0, "°") {
            change(app, "Angle", |d| d.sweep = sweep);
        }
    }
    if !custom && matches!(s.kind, ShapeKind::Star | ShapeKind::Polygon) {
        let mut points = s.points as f64;
        if slider(ui, "Points", &mut points, 3.0..=24.0, "") {
            change(app, "Points", |d| d.points = points as i32);
        }
    }
}
