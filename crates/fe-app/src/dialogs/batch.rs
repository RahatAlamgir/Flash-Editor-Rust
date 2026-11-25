//! File > Batch Convert (C# `BatchDialog`): pick a folder of pictures, a format, a size limit and an output folder.

use std::path::PathBuf;

use egui::{Id, RichText, vec2};
use fe_core::io::ImageFileFormat;
use fe_core::io::batch::{BatchOptions, find_pictures};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use super::export::FORMATS;

pub struct BatchForm {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub format: ImageFileFormat,
    quality: f64,
    pub max_width: u32,
    pub max_height: u32,
    pub subfolders: bool,
    found: Option<usize>,
}

impl Default for BatchForm {
    fn default() -> Self {
        Self {
            input: None,
            output: None,
            format: ImageFileFormat::Jpeg,
            quality: 85.0,
            max_width: 0,
            max_height: 0,
            subfolders: false,
            found: None,
        }
    }
}

impl BatchForm {
    /// The options when everything needed is filled in.
    pub fn options(&self) -> Option<BatchOptions> {
        Some(BatchOptions {
            input: self.input.clone()?,
            output: self.output.clone()?,
            format: self.format,
            quality: self.quality.round() as u8,
            max_width: self.max_width,
            max_height: self.max_height,
            include_subfolders: self.subfolders,
        })
    }

    fn count(&mut self) {
        self.found = self.options().map(|o| find_pictures(&o).len());
    }
}

fn limit(ui: &mut egui::Ui, value: &mut u32) {
    let mut v = f64::from(*value);
    ui.add(
        egui::DragValue::new(&mut v)
            .range(0.0..=16384.0)
            .speed(10.0)
            .max_decimals(0)
            .custom_formatter(|n, _| if n < 1.0 { "no limit".into() } else { format!("{n:.0} px") }),
    );
    *value = v.round() as u32;
}

/// `Some(true)` = Convert, `Some(false)` = Cancel, `None` = open.
pub fn show(ctx: &egui::Context, f: &mut BatchForm) -> Option<bool> {
    let mut result = None;
    let modal = egui::Modal::new(Id::NULL.with("batch")).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.label(RichText::new("Batch Convert").heading());
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Turns every picture of a folder into another format and / or a smaller size. The originals are never changed; existing files are never overwritten.")
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        let mut recount = false;
        for (label, which) in [("Pictures in", 0), ("Save to", 1)] {
            ui.horizontal(|ui| {
                ui.label(label);
                let current = if which == 0 { &mut f.input } else { &mut f.output };
                ui.label(RichText::new(current.as_ref().map_or("(not chosen)".into(), |p| p.display().to_string())).color(colors::SUBTLE_TEXT));
                if ui.button("Browse...").clicked()
                    && let Some(folder) = rfd::FileDialog::new().set_title(label).pick_folder()
                {
                    if which == 0 && current.is_none() {
                        f.output = Some(folder.join("converted"));
                    }
                    if which == 0 {
                        f.input = Some(folder);
                    } else {
                        f.output = Some(folder);
                    }
                    recount = true;
                }
            });
        }
        ui.add_space(6.0);
        egui::Grid::new("batch-grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            ui.label("Format");
            egui::ComboBox::from_id_salt("batch-format")
                .width(200.0)
                .selected_text(FORMATS.iter().find(|(x, _)| *x == f.format).map_or("", |(_, n)| *n))
                .show_ui(ui, |ui| {
                    for (format, name) in FORMATS {
                        ui.selectable_value(&mut f.format, format, name);
                    }
                });
            ui.end_row();
            if f.format.has_quality() {
                ui.label("Quality");
                number_slider(ui, &mut f.quality, 1.0..=100.0, "%", 150.0, 54.0);
                ui.end_row();
            }
            ui.label("Largest width");
            limit(ui, &mut f.max_width);
            ui.end_row();
            ui.label("Largest height");
            limit(ui, &mut f.max_height);
            ui.end_row();
        });
        if ui.checkbox(&mut f.subfolders, "Include subfolders").changed() {
            recount = true;
        }
        ui.label(RichText::new("Pictures are made smaller to fit the limits, keeping their proportions; they are never enlarged.").size(11.5).color(colors::SUBTLE_TEXT));
        if recount {
            f.count();
        }
        if let Some(n) = f.found {
            ui.add_space(4.0);
            ui.label(format!("{n} picture{} found.", if n == 1 { "" } else { "s" }));
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let ready = f.options().is_some() && f.found != Some(0);
            if ui.add_enabled(ready, egui::Button::new("Convert").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
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
