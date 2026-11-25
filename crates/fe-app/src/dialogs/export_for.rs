//! File > Export For (C# `ExportForDialog`): pick a ready-made size (a social post, a thumbnail, a print...) and how the
//! picture fits it, see a preview, and export this page or every page.

use std::sync::Arc;

use egui::{ColorImage, Id, RichText, TextureHandle, TextureOptions, vec2};
use fe_core::imaging::{ColorRgba, PixelBuffer};
use fe_core::io::ImageFileFormat;
use fe_core::io::export::{ExportFit, ExportPreset, PRESETS, compose};
use fe_core::transform::{ResampleMode, resize};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

pub struct ExportForForm {
    pub source: Arc<PixelBuffer>,
    /// A small copy for the live preview.
    small: PixelBuffer,
    pub preset: usize,
    pub fit: ExportFit,
    pub format: ImageFileFormat,
    pub quality: u8,
    pub transparent: bool,
    pub page_count: usize,
    pub all_pages: bool,
    quality_f: f64,
    texture: Option<(u64, TextureHandle)>,
    key: u64,
}

impl ExportForForm {
    pub fn new(picture: PixelBuffer, page_count: usize) -> Self {
        let scale = (400.0 / f64::from(picture.width().max(picture.height()))).min(1.0);
        let small = if scale < 1.0 {
            resize(
                &picture,
                (f64::from(picture.width()) * scale).max(1.0) as u32,
                (f64::from(picture.height()) * scale).max(1.0) as u32,
                ResampleMode::Smooth,
            )
        } else {
            picture.clone()
        };
        let preset = PRESETS.iter().position(|p| p.name == "Instagram post (square)").unwrap_or(0);
        Self {
            source: Arc::new(picture),
            small,
            preset,
            fit: ExportFit::Fit,
            format: ImageFileFormat::Png,
            quality: 90,
            transparent: true,
            page_count,
            all_pages: false,
            quality_f: 90.0,
            texture: None,
            key: 1,
        }
    }
}

const FORMATS: [(ImageFileFormat, &str); 3] = [
    (ImageFileFormat::Png, "PNG (sharp, can be see-through)"),
    (ImageFileFormat::Jpeg, "JPG (small photos)"),
    (ImageFileFormat::Webp, "WebP (small, modern)"),
];

/// `Some(true)` = Export, `Some(false)` = Cancel, `None` = open.
pub fn show(ctx: &egui::Context, f: &mut ExportForForm) -> Option<bool> {
    let mut result = None;
    let before = (f.preset, f.fit, f.format, f.transparent);
    let modal = egui::Modal::new(Id::NULL.with("export-for")).show(ctx, |ui| {
        ui.set_width(640.0);
        ui.label(RichText::new("Export For").heading());
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Make a picture of a ready-made size from this page. The project itself is not changed.")
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(340.0);
                ui.label("Size");
                egui::ComboBox::from_id_salt("export-for-preset")
                    .width(320.0)
                    .selected_text(PRESETS[f.preset].description())
                    .show_ui(ui, |ui| {
                        let mut group = "";
                        for (i, preset) in PRESETS.iter().enumerate() {
                            if preset.group != group {
                                group = preset.group;
                                ui.label(RichText::new(group).strong().color(colors::SUBTLE_TEXT));
                            }
                            ui.selectable_value(&mut f.preset, i, preset.description());
                        }
                    });
                ui.add_space(8.0);
                ui.label("When the shape is different");
                ui.radio_value(
                    &mut f.fit,
                    ExportFit::Fit,
                    "Fit: show the whole picture, fill the rest with the background",
                );
                ui.radio_value(&mut f.fit, ExportFit::Fill, "Fill: cover the whole size, cut off what does not fit");
                ui.radio_value(&mut f.fit, ExportFit::Stretch, "Stretch: change the shape of the picture");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("Format");
                    egui::ComboBox::from_id_salt("export-for-format")
                        .width(240.0)
                        .selected_text(FORMATS.iter().find(|(x, _)| *x == f.format).map_or("", |(_, n)| *n))
                        .show_ui(ui, |ui| {
                            for (format, name) in FORMATS {
                                ui.selectable_value(&mut f.format, format, name);
                            }
                        });
                });
                if f.format.has_quality() {
                    ui.horizontal(|ui| {
                        ui.label("Quality");
                        number_slider(ui, &mut f.quality_f, 10.0..=100.0, "%", 160.0, 54.0);
                        f.quality = f.quality_f.round() as u8;
                    });
                }
                if f.format != ImageFileFormat::Jpeg {
                    ui.radio_value(&mut f.transparent, true, "Keep the background see-through");
                    ui.radio_value(&mut f.transparent, false, "White background");
                } else {
                    f.transparent = false;
                    ui.label(
                        RichText::new("JPG has no transparency: the background is white.")
                            .size(11.5)
                            .color(colors::SUBTLE_TEXT),
                    );
                }
                if f.page_count > 1 {
                    ui.add_space(6.0);
                    ui.checkbox(&mut f.all_pages, format!("Export all {} pages (name-1, name-2, ...)", f.page_count));
                }
            });
            ui.add_space(12.0);
            ui.vertical(|ui| {
                let preset = &PRESETS[f.preset];
                ui.label(RichText::new(format!("{} x {} px", preset.width, preset.height)).color(colors::SUBTLE_TEXT));
                if f.texture.as_ref().is_none_or(|(k, _)| *k != f.key)
                    || before != (f.preset, f.fit, f.format, f.transparent)
                {
                    f.key += 1;
                    let scale = 240.0 / f64::from(preset.width.max(preset.height));
                    let small = ExportPreset {
                        group: preset.group,
                        name: preset.name,
                        width: ((f64::from(preset.width) * scale) as u32).max(1),
                        height: ((f64::from(preset.height) * scale) as u32).max(1),
                    };
                    let background = if f.transparent { None } else { Some(ColorRgba::WHITE) };
                    let picture = compose(&f.small, &small, f.fit, background);
                    let image = ColorImage::from_rgba_unmultiplied(
                        [picture.width() as usize, picture.height() as usize],
                        picture.data(),
                    );
                    f.texture = Some((f.key, ui.ctx().load_texture("export-for", image, TextureOptions::LINEAR)));
                }
                if let Some((_, texture)) = &f.texture {
                    let (rect, _) = ui.allocate_exact_size(texture.size_vec2(), egui::Sense::hover());
                    fe_ui_kit::widgets::paint_checkerboard(ui.painter(), rect, 8.0);
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Export...").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
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
