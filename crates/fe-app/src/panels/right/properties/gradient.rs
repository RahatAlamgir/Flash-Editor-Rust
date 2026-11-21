//! Properties > Gradient (C# `ObjectPropertiesViewModel`, gradient part): a color that fades through 2-8 colors for
//! the fill of a shape or the color of a text; on a picture it is a see-through overlay.

use egui::Ui;
use fe_core::imaging::ColorRgba;
use fe_core::objects::{GRADIENT_PRESETS, Gradient, GradientPreset, OVERLAY_PRESETS};

use super::label;
use super::shape::{slider, swatch};
use crate::app::FlashApp;
use crate::commands::{objects as cmd, parts};
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

fn change(app: &mut FlashApp, what: &'static str, mut f: impl FnMut(&mut Gradient)) {
    cmd::change_object(app, what, |o| {
        if let Some(mut g) = o.gradient() {
            f(&mut g);
            o.set_gradient(Some(&g));
        }
    });
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(object) = parts::target_object(app) else { return };
    let kind = object.kind().to_owned();
    let is_image = kind == "image";
    let base =
        object.text().map(|t| t.color).or_else(|| object.shape().and_then(|s| s.fill)).unwrap_or(ColorRgba::WHITE);
    let current = object.gradient();

    let mut on = current.is_some();
    if ui.checkbox(&mut on, if is_image { "Color overlay" } else { "Gradient" }).changed() {
        cmd::change_object(app, "Gradient", |o| {
            let g = on.then(|| if is_image { OVERLAY_PRESETS[0].create() } else { Gradient::starting(base) });
            o.set_gradient(g.as_ref());
        });
    }
    let Some(g) = current.filter(|_| on) else { return };

    let presets: &[GradientPreset] = if is_image { &OVERLAY_PRESETS } else { &GRADIENT_PRESETS };
    ui.horizontal(|ui| {
        label(ui, "Style");
        egui::ComboBox::from_id_salt("gradient-preset").selected_text("Pick a look").width(130.0).show_ui(ui, |ui| {
            for preset in presets {
                if ui.selectable_label(false, preset.name).clicked() {
                    cmd::change_object(app, "Gradient Style", |o| o.set_gradient(Some(&preset.create())));
                }
            }
        });
    });
    for (i, (color, position)) in g.stops.iter().enumerate() {
        ui.horizontal(|ui| {
            if swatch(ui, Some(*color), "Color of this stop") {
                app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::GradientStop(i), *color)));
            }
            let mut pos = position * 100.0;
            if slider(ui, "", &mut pos, 0.0..=100.0, "%") {
                change(app, "Gradient Stop", |g| {
                    if let Some(s) = g.stops.get_mut(i) {
                        s.1 = (pos.round() / 100.0).clamp(0.0, 1.0);
                    }
                });
            }
            if g.stops.len() > Gradient::MIN_STOPS && ui.small_button("✕").on_hover_text("Remove this color").clicked()
            {
                change(app, "Remove Gradient Color", |g| *g = g.with_stop_removed(i));
            }
        });
    }
    ui.horizontal(|ui| {
        if g.stops.len() < Gradient::MAX_STOPS && ui.button("+ Add color").clicked() {
            change(app, "Add Gradient Color", |g| *g = g.with_stop_added());
        }
        if ui.button("Reverse").clicked() {
            change(app, "Reverse Gradient", |g| *g = g.reversed());
        }
    });
    let mut radial = g.radial;
    if ui.checkbox(&mut radial, "Radial (from the middle)").changed() {
        change(app, "Gradient Type", |g| g.radial = radial);
    }
    if !g.radial {
        let mut angle = g.angle;
        if slider(ui, "Angle", &mut angle, -180.0..=180.0, "°") {
            change(app, "Gradient Angle", |g| g.angle = angle.round());
        }
    }
    if let Some(shape) = object.shape()
        && shape.stroke.is_some()
    {
        let mut border = shape.gradient_border;
        if ui.checkbox(&mut border, "Also on the border").changed() {
            cmd::change_object(app, "Gradient Border", |o| {
                if let Some(mut s) = o.shape() {
                    s.gradient_border = border;
                    o.set_shape(&s);
                }
            });
        }
    }
}
