//! Properties > Text (C# `ObjectPropertiesViewModel`, text part): the words, font, size, bold / italic, alignment,
//! color, outline, line / letter / word gaps and curved text. The box follows the size of the text.

use egui::Ui;
use fe_core::imaging::ColorRgba;
use fe_core::text::{TextAlign, TextData};

use super::label;
use super::shape::{slider, swatch};
use crate::app::FlashApp;
use crate::commands::{parts, text as cmd};
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(t) = parts::target_object(app).and_then(|o| o.text()) else { return };
    ui.horizontal(|ui| {
        label(ui, "Words");
        let edit =
            ui.button("Edit text…").on_hover_text("Change the words right on the picture (or double-click the text)");
        if edit.clicked()
            && let Some(id) = app.doc().active_id()
        {
            let part = parts::active_part(app);
            crate::text_edit::begin(app, id, part, crate::text_edit::Start::All);
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Font");
        let mut family = t.family.clone();
        if crate::tools::options_bar::font_picker(ui, "text-font", &mut family, 150.0) {
            cmd::change_text(app, "Font", |d| d.family = family.clone());
        }
    });
    if !app.settings.brand_fonts.is_empty() {
        ui.horizontal_wrapped(|ui| {
            label(ui, "Brand");
            for family in app.settings.brand_fonts.clone() {
                if ui.selectable_label(t.family.eq_ignore_ascii_case(&family), &family).clicked() {
                    cmd::change_text(app, "Font", |d| d.family = family.clone());
                }
            }
        });
    }
    let mut size = t.size;
    if slider(ui, "Size", &mut size, 4.0..=300.0, "px") {
        cmd::change_text(app, "Font Size", |d| d.size = size.round().max(1.0));
    }
    ui.horizontal(|ui| {
        label(ui, "Style");
        let (mut bold, mut italic) = (t.bold, t.italic);
        if ui.toggle_value(&mut bold, "Bold").changed() {
            cmd::change_text(app, "Bold", |d| d.bold = bold);
        }
        if ui.toggle_value(&mut italic, "Italic").changed() {
            cmd::change_text(app, "Italic", |d| d.italic = italic);
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Align");
        for (text, align) in [("Left", TextAlign::Left), ("Center", TextAlign::Center), ("Right", TextAlign::Right)] {
            if ui.selectable_label(t.align == align, text).clicked() {
                cmd::change_text(app, "Align", |d| d.align = align);
            }
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Color");
        if swatch(ui, Some(t.color), "Text color") {
            app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::TextColor, t.color)));
        }
    });
    ui.horizontal(|ui| {
        let mut on = t.outline_width > 0.0 && t.outline_color.is_some();
        if ui.add_sized([72.0, 22.0], egui::Checkbox::new(&mut on, "Outline")).changed() {
            cmd::change_text(app, "Outline", |d| {
                if on {
                    d.outline_color.get_or_insert(ColorRgba::BLACK);
                    if d.outline_width <= 0.0 {
                        d.outline_width = 2.0;
                    }
                } else {
                    d.outline_width = 0.0;
                }
            });
        }
        if swatch(ui, t.outline_color, "Outline color") {
            app.dialog = Some(Dialog::Color(ColorPicker::new(
                ColorTarget::TextOutline,
                t.outline_color.unwrap_or(ColorRgba::BLACK),
            )));
        }
    });
    let mut width = t.outline_width;
    if slider(ui, "Outline", &mut width, 0.0..=30.0, "px") {
        cmd::change_text(app, "Outline Width", |d| {
            d.outline_width = width;
            if width > 0.0 {
                d.outline_color.get_or_insert(ColorRgba::BLACK);
            }
        });
    }
    spacing(ui, app, &t);
}

fn spacing(ui: &mut Ui, app: &mut FlashApp, t: &TextData) {
    let mut lines = t.line_spacing;
    if slider(ui, "Lines", &mut lines, 0.5..=3.0, "×") {
        cmd::change_text(app, "Line Spacing", |d| d.line_spacing = (lines * 20.0).round() / 20.0);
    }
    let mut letters = t.letter_spacing * 100.0;
    if slider(ui, "Letters", &mut letters, -10.0..=100.0, "%") {
        cmd::change_text(app, "Letter Spacing", |d| d.letter_spacing = letters.round() / 100.0);
    }
    let mut words = t.word_spacing * 100.0;
    if slider(ui, "Words gap", &mut words, -50.0..=300.0, "%") {
        cmd::change_text(app, "Word Spacing", |d| d.word_spacing = words.round() / 100.0);
    }
    let mut arc = t.arc;
    if slider(ui, "Curve", &mut arc, -360.0..=360.0, "°") {
        cmd::change_text(app, "Curve Text", |d| d.arc = arc.round());
    }
}
