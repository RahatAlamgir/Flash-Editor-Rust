//! Properties > Effects (C# `ObjectPropertiesViewModel`, effects part): drop shadow (with 10 ready-made looks),
//! glow, inner shadow, outline, 3D (5 ready-made looks) and blur. Every effect can be switched off again at any time:
//! nothing is baked in.

use egui::Ui;
use fe_core::imaging::ColorRgba;
use fe_core::objects::{
    GlowStyle, InnerShadowStyle, Looks, OutlineStyle, SHADOW_PRESETS, ShadowStyle, THREE_D_PRESETS, ThreeDStyle,
};

use super::label;
use super::shape::{slider, swatch};
use crate::app::FlashApp;
use crate::commands::{objects as cmd, parts};
use crate::dialogs::{ColorPicker, ColorTarget, Dialog, EffectColor};

/// Changes the looks of the selected object (slider drags of one setting merge into one undo step).
fn change(app: &mut FlashApp, what: &'static str, mut f: impl FnMut(&mut Looks)) {
    cmd::change_object(app, what, |o| {
        let mut looks = o.looks();
        f(&mut looks);
        o.set_looks(&looks);
    });
}

/// The checkbox that switches an effect on or off. True when the effect is on afterwards.
fn toggle(
    ui: &mut Ui,
    app: &mut FlashApp,
    text: &str,
    on: bool,
    what: &'static str,
    set: impl Fn(&mut Looks, bool),
) -> bool {
    let mut now = on;
    if ui.checkbox(&mut now, text).changed() {
        change(app, what, |l| set(l, now));
    }
    now
}

fn color_row(ui: &mut Ui, app: &mut FlashApp, text: &str, color: ColorRgba, target: EffectColor) {
    ui.horizontal(|ui| {
        label(ui, text);
        if swatch(ui, Some(ColorRgba { a: 255, ..color }), "Color") {
            app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::Effect(target), color)));
        }
    });
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(object) = parts::target_object(app) else { return };
    let looks = object.looks();
    shadow(ui, app, &looks);
    ui.separator();
    glow(ui, app, &looks);
    ui.separator();
    inner_shadow(ui, app, &looks);
    ui.separator();
    outline(ui, app, &looks);
    ui.separator();
    three_d(ui, app, &looks);
    ui.separator();
    let mut blur = looks.blur;
    if slider(ui, "Blur", &mut blur, 0.0..=50.0, "px") {
        change(app, "Blur", |l| l.blur = blur.round());
    }
}

fn shadow(ui: &mut Ui, app: &mut FlashApp, looks: &Looks) {
    let on = toggle(ui, app, "Shadow", looks.shadow.is_some(), "Shadow", |l, on| {
        l.shadow = on.then_some(ShadowStyle::DEFAULT);
    });
    let (Some(s), true) = (looks.shadow, on) else { return };
    ui.horizontal(|ui| {
        label(ui, "Style");
        egui::ComboBox::from_id_salt("shadow-preset").selected_text("Pick a look").width(130.0).show_ui(ui, |ui| {
            for preset in &SHADOW_PRESETS {
                if ui.selectable_label(false, preset.name).on_hover_text(preset.description).clicked() {
                    change(app, "Shadow Style", |l| l.shadow = Some(preset.create(l.shadow)));
                }
            }
        });
    });
    color_row(ui, app, "Color", s.color, EffectColor::Shadow);
    let mut strength = f64::from(s.color.a) / 2.55;
    if slider(ui, "Strength", &mut strength, 0.0..=100.0, "%") {
        change(app, "Shadow Strength", |l| {
            if let Some(s) = &mut l.shadow {
                s.color.a = (strength * 2.55).round() as u8;
            }
        });
    }
    let (mut distance, mut angle, mut blur, mut size) = (s.distance, s.angle, s.blur, s.size * 100.0);
    if slider(ui, "Distance", &mut distance, 0.0..=200.0, "px") {
        change(app, "Shadow Distance", |l| l.shadow.iter_mut().for_each(|s| s.distance = distance.round()));
    }
    if slider(ui, "Angle", &mut angle, -180.0..=180.0, "°") {
        change(app, "Shadow Angle", |l| l.shadow.iter_mut().for_each(|s| s.angle = angle.round()));
    }
    if slider(ui, "Blur", &mut blur, 0.0..=100.0, "px") {
        change(app, "Shadow Blur", |l| l.shadow.iter_mut().for_each(|s| s.blur = blur.round()));
    }
    if slider(ui, "Size", &mut size, 20.0..=300.0, "%") {
        change(app, "Shadow Size", |l| l.shadow.iter_mut().for_each(|s| s.size = size.round() / 100.0));
    }
}

fn glow(ui: &mut Ui, app: &mut FlashApp, looks: &Looks) {
    let on = toggle(ui, app, "Glow", looks.glow.is_some(), "Glow", |l, on| l.glow = on.then_some(GlowStyle::DEFAULT));
    let (Some(g), true) = (looks.glow, on) else { return };
    color_row(ui, app, "Color", g.color, EffectColor::Glow);
    let (mut strength, mut size, mut intensity) = (f64::from(g.color.a) / 2.55, g.size, g.intensity);
    if slider(ui, "Strength", &mut strength, 0.0..=100.0, "%") {
        change(app, "Glow Strength", |l| l.glow.iter_mut().for_each(|g| g.color.a = (strength * 2.55).round() as u8));
    }
    if slider(ui, "Size", &mut size, 1.0..=100.0, "px") {
        change(app, "Glow Size", |l| l.glow.iter_mut().for_each(|g| g.size = size.round()));
    }
    if slider(ui, "Intensity", &mut intensity, 10.0..=300.0, "%") {
        change(app, "Glow Intensity", |l| l.glow.iter_mut().for_each(|g| g.intensity = intensity.round()));
    }
}

fn inner_shadow(ui: &mut Ui, app: &mut FlashApp, looks: &Looks) {
    let on = toggle(ui, app, "Inner shadow", looks.inner_shadow.is_some(), "Inner Shadow", |l, on| {
        l.inner_shadow = on.then_some(InnerShadowStyle::DEFAULT);
    });
    let (Some(s), true) = (looks.inner_shadow, on) else { return };
    color_row(ui, app, "Color", s.color, EffectColor::InnerShadow);
    let (mut strength, mut distance, mut angle, mut blur) = (f64::from(s.color.a) / 2.55, s.distance, s.angle, s.blur);
    if slider(ui, "Strength", &mut strength, 0.0..=100.0, "%") {
        change(app, "Inner Shadow Strength", |l| {
            l.inner_shadow.iter_mut().for_each(|s| s.color.a = (strength * 2.55).round() as u8);
        });
    }
    if slider(ui, "Distance", &mut distance, 0.0..=100.0, "px") {
        change(app, "Inner Shadow Distance", |l| l.inner_shadow.iter_mut().for_each(|s| s.distance = distance.round()));
    }
    if slider(ui, "Angle", &mut angle, -180.0..=180.0, "°") {
        change(app, "Inner Shadow Angle", |l| l.inner_shadow.iter_mut().for_each(|s| s.angle = angle.round()));
    }
    if slider(ui, "Blur", &mut blur, 0.0..=60.0, "px") {
        change(app, "Inner Shadow Blur", |l| l.inner_shadow.iter_mut().for_each(|s| s.blur = blur.round()));
    }
}

fn outline(ui: &mut Ui, app: &mut FlashApp, looks: &Looks) {
    let on = toggle(ui, app, "Outline", looks.outline.is_some(), "Outline", |l, on| {
        l.outline = on.then_some(OutlineStyle::DEFAULT);
    });
    let (Some(o), true) = (looks.outline, on) else { return };
    color_row(ui, app, "Color", o.color, EffectColor::Outline);
    let mut width = o.width;
    if slider(ui, "Width", &mut width, 1.0..=40.0, "px") {
        change(app, "Outline Width", |l| l.outline.iter_mut().for_each(|o| o.width = width.round()));
    }
}

fn three_d(ui: &mut Ui, app: &mut FlashApp, looks: &Looks) {
    let on =
        toggle(ui, app, "3D", looks.three_d.is_some(), "3D", |l, on| l.three_d = on.then_some(ThreeDStyle::DEFAULT));
    let (Some(t), true) = (looks.three_d, on) else { return };
    ui.horizontal(|ui| {
        label(ui, "Style");
        egui::ComboBox::from_id_salt("three-d-preset").selected_text("Pick a look").width(130.0).show_ui(ui, |ui| {
            for preset in &THREE_D_PRESETS {
                if ui.selectable_label(false, preset.name).on_hover_text(preset.description).clicked() {
                    change(app, "3D Style", |l| l.three_d = Some(preset.create(l.three_d)));
                }
            }
        });
    });
    let (mut depth, mut angle, mut bevel) = (t.depth, t.angle, t.bevel * 100.0);
    if slider(ui, "Depth", &mut depth, 0.0..=150.0, "px") {
        change(app, "3D Depth", |l| l.three_d.iter_mut().for_each(|t| t.depth = depth.round()));
    }
    if slider(ui, "Angle", &mut angle, -180.0..=180.0, "°") {
        change(app, "3D Angle", |l| l.three_d.iter_mut().for_each(|t| t.angle = angle.round()));
    }
    if slider(ui, "Bevel", &mut bevel, 0.0..=100.0, "%") {
        change(app, "3D Bevel", |l| l.three_d.iter_mut().for_each(|t| t.bevel = bevel.round() / 100.0));
    }
    ui.horizontal(|ui| {
        let mut own = t.color.is_some();
        if ui
            .checkbox(&mut own, "Side color")
            .on_hover_text("Off: the side is the object's own color, darker")
            .changed()
        {
            change(app, "3D Side Color", |l| {
                l.three_d.iter_mut().for_each(|t| t.color = own.then_some(ColorRgba::new(120, 120, 120, 255)));
            });
        }
        if let Some(c) = t.color
            && swatch(ui, Some(c), "Color of the solid side")
        {
            app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::Effect(EffectColor::ThreeD), c)));
        }
    });
}
