//! File > Export (C# `ExportDialog`): format, quality, size and an optional crop before saving a copy of the picture. The
//! file is made on a background thread (a moment after the last change) so the real file size and the real result,
//! with its compression, are shown; the window never waits for it.

use std::sync::Arc;

use egui::{Color32, ColorImage, RichText, Sense, TextureHandle, TextureOptions, vec2};
use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::io::ImageFileFormat;
use fe_core::io::export::{ExportSettings, format_file_size};
use fe_core::transform::{ResampleMode, resize};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::app::FlashApp;
use crate::commands::export;

/// The formats of the list, in the order of the C# dialog.
pub const FORMATS: [(ImageFileFormat, &str); 7] = [
    (ImageFileFormat::Png, "PNG"),
    (ImageFileFormat::Jpeg, "JPG"),
    (ImageFileFormat::Webp, "WebP"),
    (ImageFileFormat::Ico, "ICO (Windows icon)"),
    (ImageFileFormat::Bmp, "BMP"),
    (ImageFileFormat::Tiff, "TIFF"),
    (ImageFileFormat::Gif, "GIF (256 colors)"),
];

/// The side of the preview square, in screen points.
const PREVIEW: f32 = 380.0;
/// How long after the last change the file is made again.
const DEBOUNCE: f64 = 0.3;

/// The real result of the current settings.
pub struct Estimate {
    pub bytes: u64,
    /// What was written, decoded again (a small version for display).
    pub preview: PixelBuffer,
    /// The middle of the result at 100% (for "Zoom to 100%").
    pub center: PixelBuffer,
    pub settings: ExportSettings,
}

pub struct ExportForm {
    pub source: Arc<PixelBuffer>,
    pub matte: ColorRgba,
    pub suggested_name: String,
    /// Export every page of the project the same way (File > Export All Pages).
    pub all_pages: bool,
    pub format: ImageFileFormat,
    pub quality: f64,
    pub width: u32,
    pub height: u32,
    pub keep_proportions: bool,
    pub mode: ResampleMode,
    pub crop: Option<PixelRect>,
    /// The shape of the crop box: an index of `ASPECTS`, or one more for "like the picture".
    pub(super) aspect: usize,
    pub(super) drag: Option<super::export_crop::Drag>,
    /// Changes with every new setting; a finished file with an older number is ignored.
    pub version: u64,
    refresh_at: Option<f64>,
    pub busy: bool,
    pub estimate: Option<Estimate>,
    pub error: Option<String>,
    show_result: bool,
    zoom_100: bool,
    thumb: Option<TextureHandle>,
    result_texture: Option<(u64, TextureHandle)>,
}

impl ExportForm {
    pub fn new(source: PixelBuffer, format: ImageFileFormat, quality: u8, matte: ColorRgba, name: String) -> Self {
        let (width, height) = (source.width(), source.height());
        Self {
            source: Arc::new(source),
            matte,
            suggested_name: name,
            all_pages: false,
            format,
            quality: f64::from(quality),
            width,
            height,
            keep_proportions: true,
            mode: ResampleMode::Lanczos,
            crop: None,
            aspect: 0,
            drag: None,
            version: 0,
            refresh_at: Some(0.0),
            busy: false,
            estimate: None,
            error: None,
            show_result: false,
            zoom_100: false,
            thumb: None,
            result_texture: None,
        }
    }

    /// The size the saved part has before it is resized: the crop, or the whole picture.
    pub fn source_size(&self) -> (u32, u32) {
        self.crop.map_or((self.source.width(), self.source.height()), |c| (c.width as u32, c.height as u32))
    }

    pub fn settings(&self) -> ExportSettings {
        let mut s = ExportSettings::new(self.format, self.quality.round() as u8, self.width, self.height);
        s.resample = self.mode;
        s.matte = Some(self.matte);
        s.crop = self.crop;
        s
    }

    /// Something changed: make the file again soon.
    fn touch(&mut self, now: f64) {
        self.version += 1;
        self.refresh_at = Some(now + DEBOUNCE);
        self.error = None;
    }

    /// The text under the format list.
    fn format_note(&self) -> &'static str {
        match self.format {
            ImageFileFormat::Ico => {
                "Icons always contain the sizes 16 to 256 px; the size below is the source the icon sizes are made from."
            }
            ImageFileFormat::Bmp => {
                "BMP is not compressed: the file is large but exact, and transparency is kept. For a smaller file choose PNG, JPG or WebP."
            }
            ImageFileFormat::Tiff => {
                "TIFF is lossless (no quality setting) and keeps transparency: good for printing and archives. Files are larger than JPG / WebP."
            }
            ImageFileFormat::Gif => {
                "GIF can show at most 256 colors (and fully clear or solid transparency), so photos lose detail. Best for simple graphics. One picture only; File > Export as Animated GIF makes an animation."
            }
            _ => {
                "PNG is lossless (no quality setting). To make the file smaller, reduce the size below or choose JPG / WebP."
            }
        }
    }
}

fn texture_of(ctx: &egui::Context, name: &str, buffer: &PixelBuffer) -> TextureHandle {
    let image = ColorImage::from_rgba_unmultiplied([buffer.width() as usize, buffer.height() as usize], buffer.data());
    ctx.load_texture(name, image, TextureOptions::LINEAR)
}

fn size_box(ui: &mut egui::Ui, value: &mut u32) -> bool {
    let mut v = f64::from(*value);
    let changed = ui
        .add(
            egui::DragValue::new(&mut v)
                .range(1.0..=f64::from(fe_core::io::decode::MAX_DIMENSION))
                .speed(1.0)
                .max_decimals(0),
        )
        .changed();
    *value = v.round().max(1.0) as u32;
    changed
}

/// Shows the window. `Some(true)` = OK, `Some(false)` = Cancel, `None` = open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, f: &mut ExportForm) -> Option<bool> {
    let now = ctx.input(|i| i.time);
    let mut result = None;
    let mut changed = false;
    if f.thumb.is_none() {
        let scale = (f64::from(PREVIEW * 2.0) / f64::from(f.source.width().max(f.source.height()))).min(1.0);
        let small = if scale < 1.0 {
            resize(
                &f.source,
                (f64::from(f.source.width()) * scale).max(1.0) as u32,
                (f64::from(f.source.height()) * scale).max(1.0) as u32,
                ResampleMode::Smooth,
            )
        } else {
            (*f.source).clone()
        };
        f.thumb = Some(texture_of(ctx, "export-source", &small));
    }
    let title = if f.all_pages { "Export All Pages" } else { "Export" };
    let modal = super::frame::modal(ctx, "export", |ui| {
        ui.set_width(780.0);
        ui.label(RichText::new(title).heading());
        ui.add_space(6.0);
        ui.horizontal_top(|ui| {
            // ---- the picture: drag to crop, or the saved result
            ui.vertical(|ui| {
                ui.set_width(PREVIEW);
                ui.label(
                    RichText::new(if f.show_result {
                        "The saved result"
                    } else {
                        "Crop: drag the box, its corners or its edges (optional)"
                    })
                    .color(colors::SUBTLE_TEXT),
                );
                changed |= picture_area(ui, f);
                if !f.show_result {
                    changed |= super::export_crop::controls(ui, f);
                }
                ui.horizontal(|ui| {
                    ui.checkbox(&mut f.show_result, "Show the saved result");
                    if f.show_result {
                        ui.checkbox(&mut f.zoom_100, "Zoom to 100%");
                    }
                });
            });
            ui.add_space(12.0);
            // ---- the settings
            ui.vertical(|ui| {
                ui.set_width(340.0);
                egui::Grid::new("export-grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Format");
                    egui::ComboBox::from_id_salt("export-format")
                        .width(200.0)
                        .selected_text(FORMATS.iter().find(|(x, _)| *x == f.format).map_or("PNG", |(_, n)| *n))
                        .show_ui(ui, |ui| {
                            for (format, name) in FORMATS {
                                if ui.selectable_value(&mut f.format, format, name).changed() {
                                    changed = true;
                                }
                            }
                        });
                    ui.end_row();
                    if f.format.has_quality() {
                        ui.label("Quality");
                        changed |= number_slider(ui, &mut f.quality, 1.0..=100.0, "%", 150.0, 54.0).changed();
                        f.quality = f.quality.round();
                        ui.end_row();
                    }
                });
                ui.add(egui::Label::new(RichText::new(f.format_note()).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
                ui.add_space(8.0);
                let (sw, sh) = f.source_size();
                let ratio = f64::from(sw) / f64::from(sh.max(1));
                egui::Grid::new("export-size").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Width (px)");
                    if size_box(ui, &mut f.width) {
                        if f.keep_proportions {
                            f.height = ((f64::from(f.width) / ratio).round() as u32).max(1);
                        }
                        changed = true;
                    }
                    ui.end_row();
                    ui.label("Height (px)");
                    if size_box(ui, &mut f.height) {
                        if f.keep_proportions {
                            f.width = ((f64::from(f.height) * ratio).round() as u32).max(1);
                        }
                        changed = true;
                    }
                    ui.end_row();
                    ui.label("Percent");
                    let mut pct = f64::from(f.width) / f64::from(sw.max(1)) * 100.0;
                    if ui
                        .add(egui::DragValue::new(&mut pct).range(1.0..=1600.0).speed(1.0).suffix(" %").max_decimals(1))
                        .changed()
                    {
                        f.width = ((f64::from(sw) * pct / 100.0).round() as u32).max(1);
                        f.height = ((f64::from(sh) * pct / 100.0).round() as u32).max(1);
                        changed = true;
                    }
                    ui.end_row();
                    ui.label("Method");
                    egui::ComboBox::from_id_salt("export-method").selected_text(f.mode.name()).show_ui(ui, |ui| {
                        for m in ResampleMode::ALL {
                            if ui.selectable_value(&mut f.mode, m, m.name()).changed() {
                                changed = true;
                            }
                        }
                    });
                    ui.end_row();
                });
                ui.checkbox(&mut f.keep_proportions, "Keep proportions");
                ui.add_space(10.0);
                match (&f.estimate, &f.error) {
                    (_, Some(e)) => {
                        ui.label(RichText::new("Could not encode with these settings").color(Color32::LIGHT_RED));
                        ui.label(RichText::new(e).size(11.5).color(colors::SUBTLE_TEXT));
                    }
                    (Some(est), None) => {
                        ui.label(
                            RichText::new(format!("Estimated file size: {}", format_file_size(est.bytes)))
                                .strong()
                                .color(colors::TEXT),
                        );
                        let raw = u64::from(est.settings.width) * u64::from(est.settings.height) * 4;
                        let quality = if est.settings.format.has_quality() {
                            format!(" at {}% quality", est.settings.quality)
                        } else {
                            String::new()
                        };
                        ui.label(
                            RichText::new(format!(
                                "{} x {} px, {:?}{quality}  ({:.1}% of uncompressed)",
                                est.settings.width,
                                est.settings.height,
                                est.settings.format,
                                est.bytes as f64 * 100.0 / raw.max(1) as f64
                            ))
                            .size(11.5)
                            .color(colors::SUBTLE_TEXT),
                        );
                    }
                    (None, None) => {
                        ui.label(RichText::new("Working out the file size...").color(colors::SUBTLE_TEXT));
                    }
                }
                if f.busy || f.refresh_at.is_some() {
                    ui.spinner();
                }
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let ok = egui::Button::new("Export...").fill(colors::ACCENT).min_size(vec2(100.0, 28.0));
            if ui.add(ok).clicked() {
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
        f.touch(now);
    }
    // The file is made a moment after the last change.
    if let Some(at) = f.refresh_at {
        if now >= at && result.is_none() {
            f.refresh_at = None;
            f.busy = true;
            export::start_estimate(app, ctx, f);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
    }
    result
}

/// The picture with the crop box, or the result. True when the crop changed.
fn picture_area(ui: &mut egui::Ui, f: &mut ExportForm) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(PREVIEW, PREVIEW * 0.78), Sense::click_and_drag());
    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
    if f.show_result {
        if let Some(est) = &f.estimate {
            let key = f.version * 2 + u64::from(f.zoom_100);
            if f.result_texture.as_ref().is_none_or(|(k, _)| *k != key) {
                let texture =
                    texture_of(ui.ctx(), "export-result", if f.zoom_100 { &est.center } else { &est.preview });
                f.result_texture = Some((key, texture));
            }
            if let Some((_, texture)) = &f.result_texture {
                let size = texture.size_vec2();
                let fit = if f.zoom_100 { 1.0 } else { (rect.width() / size.x).min(rect.height() / size.y).min(1.0) };
                let shown = egui::Rect::from_center_size(rect.center(), size * fit);
                fe_ui_kit::widgets::paint_checkerboard(ui.painter(), shown, 8.0);
                ui.painter().image(
                    texture.id(),
                    shown,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
        return false;
    }
    let Some(thumb) = &f.thumb else { return false };
    let (sw, sh) = (f64::from(f.source.width()), f64::from(f.source.height()));
    let fit = (f64::from(rect.width()) / sw).min(f64::from(rect.height()) / sh) as f32;
    let shown = egui::Rect::from_center_size(rect.center(), vec2(sw as f32 * fit, sh as f32 * fit));
    fe_ui_kit::widgets::paint_checkerboard(ui.painter(), shown, 8.0);
    ui.painter().image(
        thumb.id(),
        shown,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    super::export_crop::handle_box(ui, f, &response, shown, fit)
}
