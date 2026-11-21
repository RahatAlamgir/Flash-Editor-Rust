//! Properties of an adjustment layer (C# `PropertiesPanel` adjustment part, `CurveEditor`): brightness, contrast,
//! saturation, hue, temperature and tint; Levels; or a tone curve per channel.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use fe_core::document::{AdjustmentKind, AdjustmentStyle, CurvePoint, curve};
use fe_ui_kit::theme::colors;

use super::note;
use super::shape::slider;
use crate::app::FlashApp;
use crate::commands::adjustments::change;

/// A slider row: label, value, range, undo name, and how the value is stored.
type Row = (&'static str, f64, f64, f64, &'static str, fn(&mut AdjustmentStyle, f64));

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(style) = app.doc().active_layer().and_then(|l| l.adjustment.clone()) else { return };
    note(ui, "This layer changes the colors of all layers below it. Turn it off with the eye, or delete it, any time.");
    ui.add_space(4.0);
    match style.kind {
        AdjustmentKind::Basic => basic(ui, app, &style),
        AdjustmentKind::Levels => levels(ui, app, &style),
        AdjustmentKind::Curves => curves(ui, app, &style),
    }
}

fn basic(ui: &mut Ui, app: &mut FlashApp, s: &AdjustmentStyle) {
    let rows: [Row; 6] = [
        ("Brightness", s.brightness, -100.0, 100.0, "Adjustment Brightness", |s, v| s.brightness = v),
        ("Contrast", s.contrast, -100.0, 100.0, "Adjustment Contrast", |s, v| s.contrast = v),
        ("Saturation", s.saturation, -100.0, 100.0, "Adjustment Saturation", |s, v| s.saturation = v),
        ("Hue", s.hue, -180.0, 180.0, "Adjustment Hue", |s, v| s.hue = v),
        ("Warmth", s.temperature, -100.0, 100.0, "Adjustment Temperature", |s, v| s.temperature = v),
        ("Tint", s.tint, -100.0, 100.0, "Adjustment Tint", |s, v| s.tint = v),
    ];
    for (label, value, min, max, what, set) in rows {
        let mut v = value;
        if slider(ui, label, &mut v, min..=max, "") {
            let v = v.round();
            change(app, what, |s| set(s, v));
        }
    }
    if ui.button("Reset").clicked() {
        change(app, "Reset Adjustment", |s| *s = AdjustmentStyle { kind: s.kind, ..AdjustmentStyle::default() });
    }
}

fn levels(ui: &mut Ui, app: &mut FlashApp, s: &AdjustmentStyle) {
    note(
        ui,
        "Input: where black and white are in the picture. Middle: brightness of the middle tones. Output: the darkest and lightest the result may be.",
    );
    let rows: [Row; 5] = [
        ("Black", s.levels_black, 0.0, 254.0, "Levels Black", |s, v| s.levels_black = v.min(s.levels_white - 1.0)),
        ("Middle", s.levels_gamma, 0.1, 5.0, "Levels Middle", |s, v| s.levels_gamma = v),
        ("White", s.levels_white, 1.0, 255.0, "Levels White", |s, v| s.levels_white = v.max(s.levels_black + 1.0)),
        ("Out black", s.output_black, 0.0, 254.0, "Levels Output Black", |s, v| {
            s.output_black = v.min(s.output_white - 1.0)
        }),
        ("Out white", s.output_white, 1.0, 255.0, "Levels Output White", |s, v| {
            s.output_white = v.max(s.output_black + 1.0)
        }),
    ];
    for (label, value, min, max, what, set) in rows {
        let mut v = value;
        if slider(ui, label, &mut v, min..=max, "") {
            let v = if label == "Middle" { (v * 100.0).round() / 100.0 } else { v.round() };
            change(app, what, |s| set(s, v));
        }
    }
    if ui.button("Reset").clicked() {
        change(app, "Reset Levels", |s| *s = AdjustmentStyle::new_levels());
    }
}

/// Which curve is edited: 0 = all colors, 1..3 = red, green, blue.
fn channel_id() -> egui::Id {
    egui::Id::NULL.with("curve-channel")
}

fn points_of(s: &AdjustmentStyle, channel: usize) -> Vec<CurvePoint> {
    let own = match channel {
        1 => &s.curve_red,
        2 => &s.curve_green,
        3 => &s.curve_blue,
        _ => &s.curve,
    };
    own.clone().filter(|p| !p.is_empty()).unwrap_or_else(|| curve::STRAIGHT.to_vec())
}

fn set_points(s: &mut AdjustmentStyle, channel: usize, points: Vec<CurvePoint>) {
    let value = if curve::is_straight(Some(&points)) { None } else { Some(points) };
    match channel {
        1 => s.curve_red = value,
        2 => s.curve_green = value,
        3 => s.curve_blue = value,
        _ => s.curve = value,
    }
}

fn curves(ui: &mut Ui, app: &mut FlashApp, s: &AdjustmentStyle) {
    let mut channel: usize = ui.data(|d| d.get_temp(channel_id())).unwrap_or(0);
    ui.horizontal(|ui| {
        for (i, name) in ["All", "Red", "Green", "Blue"].iter().enumerate() {
            if ui.selectable_label(channel == i, *name).clicked() {
                channel = i;
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(channel_id(), channel));
    let size = ui.available_width().min(240.0);
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2, colors::INPUT);
    for k in 1..4 {
        let t = k as f32 / 4.0;
        let g = Stroke::new(1.0, colors::DIVIDER);
        painter.line_segment(
            [pos2(rect.left() + rect.width() * t, rect.top()), pos2(rect.left() + rect.width() * t, rect.bottom())],
            g,
        );
        painter.line_segment(
            [pos2(rect.left(), rect.top() + rect.height() * t), pos2(rect.right(), rect.top() + rect.height() * t)],
            g,
        );
    }
    let tone = match channel {
        1 => Color32::from_rgb(0xF8, 0x71, 0x71),
        2 => Color32::from_rgb(0x4A, 0xDE, 0x80),
        3 => Color32::from_rgb(0x5B, 0x9B, 0xFF),
        _ => Color32::WHITE,
    };
    let to_screen = |p: CurvePoint| {
        pos2(rect.left() + (p.x / 255.0) as f32 * rect.width(), rect.bottom() - (p.y / 255.0) as f32 * rect.height())
    };
    let to_curve = |p: Pos2| {
        CurvePoint::new(
            f64::from((p.x - rect.left()) / rect.width() * 255.0).clamp(0.0, 255.0),
            f64::from((rect.bottom() - p.y) / rect.height() * 255.0).clamp(0.0, 255.0),
        )
    };
    let points = points_of(s, channel);
    let table = curve::build_table(Some(&points));
    let line: Vec<Pos2> =
        (0..256).step_by(4).map(|x| to_screen(CurvePoint::new(x as f64, f64::from(table[x])))).collect();
    painter.add(egui::Shape::line(line, Stroke::new(1.5, tone)));
    for p in &points {
        painter.circle(to_screen(*p), 4.0, colors::PANEL, Stroke::new(1.5, tone));
    }
    // The point being dragged is remembered between frames.
    let drag_id = egui::Id::NULL.with("curve-drag");
    if let Some(pos) = response.interact_pointer_pos() {
        let nearest = points
            .iter()
            .enumerate()
            .map(|(i, p)| (i, to_screen(*p).distance(pos)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .filter(|(_, d)| *d < 9.0)
            .map(|(i, _)| i);
        if response.drag_started() || response.clicked() {
            ui.data_mut(|d| d.insert_temp(drag_id, nearest));
        }
        if response.secondary_clicked() {
            if let Some(i) = nearest {
                change(app, "Remove Curve Point", |s| {
                    set_points(s, channel, curve::without_point(&points_of(s, channel), i))
                });
            }
        } else if response.dragged() || response.clicked() {
            let held: Option<Option<usize>> = ui.data(|d| d.get_temp(drag_id));
            let target = to_curve(pos);
            match held.flatten() {
                Some(i) => {
                    // The two end points keep their place left and right.
                    let mut list = points.clone();
                    let last = list.len() - 1;
                    let x = if i == 0 || i == last {
                        list[i].x
                    } else {
                        target.x.clamp(list[i - 1].x + 1.0, list[i + 1].x - 1.0)
                    };
                    list[i] = CurvePoint::new(x.round(), target.y.round());
                    change(app, "Curve", |s| set_points(s, channel, list));
                }
                None => {
                    let list = curve::with_point(Some(&points), target);
                    change(app, "Add Curve Point", |s| set_points(s, channel, list));
                    // Keep dragging the new point.
                    let index = curve::with_point(Some(&points), target)
                        .iter()
                        .position(|p| (p.x - target.x.round()).abs() < 0.5);
                    ui.data_mut(|d| d.insert_temp(drag_id, index));
                }
            }
        }
    }
    note(ui, "Click the curve to add a point and drag it. Right-click a point to remove it.");
    ui.horizontal(|ui| {
        if ui.button("Reset this curve").clicked() {
            change(app, "Reset Curve", |s| set_points(s, channel, curve::STRAIGHT.to_vec()));
        }
        if ui.button("Reset all").clicked() {
            change(app, "Reset Curves", |s| *s = AdjustmentStyle::new_curves());
        }
    });
    let _: Rect = rect;
}
