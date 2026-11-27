//! The Magic Resize window (C# `MagicResizeDialog`): the new size, from a list or your own.

use egui::{RichText, vec2};
use fe_ui_kit::theme::colors;

use super::new_project::PRESETS;

pub struct MagicResizeForm {
    pub width: u32,
    pub height: u32,
    pub all_pages: bool,
    current: (u32, u32),
    preset: usize,
    pub page_count: usize,
}

impl MagicResizeForm {
    pub fn new(width: u32, height: u32, page_count: usize) -> Self {
        Self { width, height, all_pages: false, current: (width, height), preset: 0, page_count }
    }
}

fn number(ui: &mut egui::Ui, v: &mut u32) -> bool {
    let mut f = f64::from(*v);
    let changed = ui.add(egui::DragValue::new(&mut f).range(1.0..=16384.0).speed(1.0).max_decimals(0)).changed();
    *v = f.round().max(1.0) as u32;
    changed
}

/// `Some(true)` = OK, `Some(false)` = Cancel, `None` = open.
pub fn show(ctx: &egui::Context, f: &mut MagicResizeForm) -> Option<bool> {
    let mut result = None;
    let modal = super::frame::modal(ctx, "magic-resize", |ui| {
        ui.set_width(420.0);
        ui.label(RichText::new("Magic Resize").heading());
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("This page is {} x {} px.", f.current.0, f.current.1)).color(colors::SUBTLE_TEXT),
        );
        ui.add(
            egui::Label::new(
                RichText::new("The design becomes the new size and keeps its layout: everything keeps its proportions and moves to the same relative place; a background covers the new canvas.")
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Size");
            egui::ComboBox::from_id_salt("magic-preset").width(300.0).selected_text(PRESETS[f.preset].0).show_ui(
                ui,
                |ui| {
                    for (i, (name, w, h)) in PRESETS.iter().enumerate() {
                        if ui.selectable_value(&mut f.preset, i, *name).clicked() && *w > 0 {
                            f.width = *w;
                            f.height = *h;
                        }
                    }
                },
            );
        });
        ui.horizontal(|ui| {
            ui.label("Width");
            if number(ui, &mut f.width) {
                f.preset = 0;
            }
            ui.label("Height");
            if number(ui, &mut f.height) {
                f.preset = 0;
            }
            ui.label("px");
        });
        if f.page_count > 1 {
            ui.checkbox(&mut f.all_pages, format!("Resize every page of this project ({} pages)", f.page_count));
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Resize").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(vec2(90.0, 28.0))).clicked() {
                result = Some(false);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}
