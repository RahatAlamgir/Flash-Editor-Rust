//! The Split Asset Sheet window (C# `SplitSheetDialog`): shows the pieces found (numbered boxes, updated as the settings
//! change) and asks whether to add them as layers and / or save them as PNG files.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{Color32, ColorImage, Id, RichText, Sense, TextureHandle, TextureOptions, vec2};
use fe_core::imaging::PixelBuffer;
use fe_core::sheets::{SheetPiece, SplitSettings};
use fe_core::transform::{ResampleMode, resize};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::app::FlashApp;
use crate::commands::split_sheet;

const STAGE: (f32, f32) = (540.0, 440.0);
const DEBOUNCE: f64 = 0.25;

pub struct SplitSheetForm {
    pub sheet: Arc<PixelBuffer>,
    pub selection: Option<Arc<Vec<u8>>>,
    gap: f64,
    min_size: f64,
    tolerance: f64,
    padding: f64,
    pub remove_background: bool,
    pub pieces: Option<Arc<Vec<SheetPiece>>>,
    pub version: u64,
    refresh_at: Option<f64>,
    pub busy: bool,
    pub error: Option<String>,
    pub as_layers: bool,
    pub hide_original: bool,
    pub save_files: bool,
    pub folder: String,
    pub name: String,
    texture: Option<TextureHandle>,
}

impl SplitSheetForm {
    pub fn new(sheet: PixelBuffer, selection: Option<Vec<u8>>, name: String, folder: Option<PathBuf>) -> Self {
        let defaults = SplitSettings::default();
        Self {
            sheet: Arc::new(sheet),
            selection: selection.map(Arc::new),
            gap: f64::from(defaults.gap),
            min_size: f64::from(defaults.min_size),
            tolerance: f64::from(defaults.tolerance),
            padding: 0.0,
            remove_background: true,
            pieces: None,
            version: 0,
            refresh_at: Some(0.0),
            busy: false,
            error: None,
            as_layers: true,
            hide_original: true,
            save_files: false,
            folder: folder.map(|f| f.display().to_string()).unwrap_or_default(),
            name,
            texture: None,
        }
    }

    pub fn settings(&self) -> SplitSettings {
        SplitSettings {
            gap: self.gap as i32,
            min_size: self.min_size as i32,
            tolerance: self.tolerance as i32,
            remove_background: self.remove_background,
            padding: self.padding as i32,
        }
    }
}

/// `Some(true)` = OK, `Some(false)` = Cancel, `None` = open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, f: &mut SplitSheetForm) -> Option<bool> {
    let now = ctx.input(|i| i.time);
    let mut result = None;
    let mut changed = false;
    if f.texture.is_none() {
        let scale = (f64::from(STAGE.0 * 2.0) / f64::from(f.sheet.width().max(f.sheet.height()))).min(1.0);
        let small = if scale < 1.0 {
            resize(
                &f.sheet,
                (f64::from(f.sheet.width()) * scale).max(1.0) as u32,
                (f64::from(f.sheet.height()) * scale).max(1.0) as u32,
                ResampleMode::Smooth,
            )
        } else {
            (*f.sheet).clone()
        };
        let image = ColorImage::from_rgba_unmultiplied([small.width() as usize, small.height() as usize], small.data());
        f.texture = Some(ctx.load_texture("split-sheet", image, TextureOptions::LINEAR));
    }
    let modal = egui::Modal::new(Id::NULL.with("split-sheet")).show(ctx, |ui| {
        ui.set_width(860.0);
        ui.label(RichText::new("Split Asset Sheet").heading());
        let intro = if f.selection.is_some() {
            "Using your selection: every separate selected area becomes one piece."
        } else {
            "Nothing is selected, so items are found automatically (transparent or plain background). For full control, select the items first with any selection tool, then split."
        };
        ui.add(egui::Label::new(RichText::new(intro).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(6.0);
        ui.horizontal_top(|ui| {
            stage(ui, f);
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.set_width(270.0);
                for (label, value, range, unit, enabled) in [
                    ("Gap between parts of one piece", &mut f.gap, 0.0..=100.0, " px", true),
                    ("Smallest piece", &mut f.min_size, 1.0..=200.0, " px", true),
                    ("Background tolerance", &mut f.tolerance, 1.0..=100.0, "", f.selection.is_none()),
                    ("Space around each piece", &mut f.padding, 0.0..=50.0, " px", true),
                ] {
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.label(RichText::new(label).color(colors::SUBTLE_TEXT));
                        changed |= number_slider(ui, value, range, unit, 170.0, 56.0).changed();
                        *value = value.round();
                    });
                }
                changed |= ui.checkbox(&mut f.remove_background, "Make the sheet background transparent").changed();
                ui.add_space(6.0);
                let text = match (&f.pieces, f.busy || f.refresh_at.is_some()) {
                    (_, true) => "Looking for pieces...".to_owned(),
                    (Some(p), false) => match p.len() {
                        0 => "No pieces found. Try a lower tolerance or smaller minimum size.".to_owned(),
                        1 => "1 piece found".to_owned(),
                        n => format!("{n} pieces found"),
                    },
                    (None, false) => String::new(),
                };
                ui.label(RichText::new(text).strong().color(colors::TEXT));
                ui.add_space(8.0);
                ui.checkbox(&mut f.as_layers, "Add the pieces as layers");
                ui.add_enabled(f.as_layers, egui::Checkbox::new(&mut f.hide_original, "Hide the original layers"));
                ui.checkbox(&mut f.save_files, "Save the pieces as PNG files");
                if f.save_files {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(170.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Folder");
                        ui.add(egui::TextEdit::singleline(&mut f.folder).desired_width(140.0));
                        if ui.button("...").clicked()
                            && let Some(folder) = rfd::FileDialog::new().set_title("Folder for the pieces").pick_folder()
                        {
                            f.folder = folder.display().to_string();
                        }
                    });
                }
                if let Some(e) = &f.error {
                    ui.label(RichText::new(e).color(Color32::LIGHT_RED));
                }
            });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let found = f.pieces.as_ref().is_some_and(|p| !p.is_empty());
            let valid = (f.as_layers || f.save_files)
                && (!f.save_files || (!f.folder.trim().is_empty() && !f.name.trim().is_empty() && !f.name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|'])));
            if ui.add_enabled(found && valid, egui::Button::new("Split").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
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
    if changed {
        f.version += 1;
        f.refresh_at = Some(now + DEBOUNCE);
        f.error = None;
    }
    if let Some(at) = f.refresh_at {
        if now >= at && result.is_none() {
            f.refresh_at = None;
            f.busy = true;
            split_sheet::start_search(app, ctx, f);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
    }
    result
}

/// The sheet with a numbered box around every piece.
fn stage(ui: &mut egui::Ui, f: &SplitSheetForm) {
    let (rect, _) = ui.allocate_exact_size(vec2(STAGE.0, STAGE.1), Sense::hover());
    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
    let Some(texture) = &f.texture else { return };
    let (sw, sh) = (f.sheet.width() as f32, f.sheet.height() as f32);
    let fit = (rect.width() / sw).min(rect.height() / sh);
    let shown = egui::Rect::from_center_size(rect.center(), vec2(sw * fit, sh * fit));
    fe_ui_kit::widgets::paint_checkerboard(ui.painter(), shown, 8.0);
    ui.painter().image(
        texture.id(),
        shown,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    let Some(pieces) = &f.pieces else { return };
    let blue = Color32::from_rgb(0x4F, 0xC3, 0xFF);
    for (i, piece) in pieces.iter().enumerate() {
        let b = piece.bounds;
        let r = egui::Rect::from_min_size(
            egui::pos2(shown.left() + b.x as f32 * fit, shown.top() + b.y as f32 * fit),
            vec2((b.width as f32 * fit).max(2.0), (b.height as f32 * fit).max(2.0)),
        );
        ui.painter().rect_filled(r, 0.0, blue.gamma_multiply(0.12));
        ui.painter().rect_stroke(r, 0.0, egui::Stroke::new(1.5, blue), egui::StrokeKind::Inside);
        let label = egui::Rect::from_min_size(r.min, vec2(8.0 + 7.0 * (i + 1).to_string().len() as f32, 15.0));
        ui.painter().rect_filled(label, 3.0, blue);
        ui.painter().text(
            label.center(),
            egui::Align2::CENTER_CENTER,
            (i + 1).to_string(),
            egui::FontId::proportional(11.0),
            Color32::BLACK,
        );
    }
}
