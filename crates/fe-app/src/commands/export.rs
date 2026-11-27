//! File > Export and friends (C# `FileCommands.Export / ExportFor / ExportAllPages`, `PagesExportCommands`): saving copies of
//! the picture in other formats and sizes. The open project is never changed. Long work runs on a background job with a
//! progress bar and Cancel in the status bar (the same bar the AI uses).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fe_core::imaging::{ColorRgba, PixelBuffer};
use fe_core::io::ImageFileFormat;
use fe_core::io::export::{self, ExportSettings, PRESETS, format_file_size};
use fe_core::paint::compositor;

use crate::ai::job::{self, Progress};
use crate::ai::outcome::AiDone;
use crate::app::FlashApp;
use crate::app::jobs::JobDone;
use crate::dialogs::export::{Estimate, ExportForm};
use crate::dialogs::{Dialog, ExportForForm};

/// The flattened picture of every page of the project.
fn flatten_pages(app: &FlashApp) -> Vec<PixelBuffer> {
    app.project().pages().iter().map(|p| compositor::flatten(&p.doc)).collect()
}

fn project_name(app: &FlashApp) -> String {
    app.project()
        .path
        .as_ref()
        .and_then(|p| p.file_stem())
        .map_or_else(|| app.project().name.clone(), |s| s.to_string_lossy().into_owned())
}

/// The format the file dialog ended up with (the user may have typed another extension).
fn save_dialog(
    title: &str,
    name: &str,
    format: ImageFileFormat,
    folder: PathBuf,
) -> Option<(PathBuf, ImageFileFormat)> {
    let filters: [(&str, &[&str], ImageFileFormat); 7] = [
        ("PNG image", &["png"], ImageFileFormat::Png),
        ("JPEG image", &["jpg", "jpeg"], ImageFileFormat::Jpeg),
        ("WebP image", &["webp"], ImageFileFormat::Webp),
        ("Windows icon", &["ico"], ImageFileFormat::Ico),
        ("Bitmap (BMP)", &["bmp"], ImageFileFormat::Bmp),
        ("TIFF image", &["tif", "tiff"], ImageFileFormat::Tiff),
        ("GIF image (256 colors)", &["gif"], ImageFileFormat::Gif),
    ];
    let mut dialog = rfd::FileDialog::new()
        .set_title(title)
        .set_directory(folder)
        .set_file_name(format!("{name}{}", format.extension()));
    // The chosen format comes first, so it is the one shown.
    for (label, extensions, f) in
        filters.iter().filter(|(_, _, f)| *f == format).chain(filters.iter().filter(|(_, _, f)| *f != format))
    {
        let _ = f;
        dialog = dialog.add_filter(*label, extensions);
    }
    let path = dialog.save_file()?;
    let chosen = ImageFileFormat::from_path(&path).unwrap_or(format);
    let path = if ImageFileFormat::from_path(&path).is_some() {
        path
    } else {
        PathBuf::from(format!("{}{}", path.display(), chosen.extension()))
    };
    Some((path, chosen))
}

// ---------------------------------------------------------------------------------------------------- Export

/// File > Export (Ctrl+Alt+S).
pub fn open_export(app: &mut FlashApp) {
    open_form(app, false);
}

/// File > Export All Pages.
pub fn open_export_all(app: &mut FlashApp) {
    if app.project().pages().len() < 2 {
        app.status = "Export All Pages needs a project with more than one page. Use File > Export for this one.".into();
        return;
    }
    open_form(app, true);
}

fn open_form(app: &mut FlashApp, all: bool) {
    if app.dialog.is_some() {
        return;
    }
    let flat = compositor::flatten(app.doc());
    let mut form = ExportForm::new(flat, ImageFileFormat::Png, 90, ColorRgba::WHITE, project_name(app));
    form.all_pages = all;
    app.dialog = Some(Dialog::Export(Box::new(form)));
}

/// Makes the real file for the current settings on a background thread.
pub fn start_estimate(app: &mut FlashApp, ctx: &egui::Context, form: &ExportForm) {
    let (source, settings, version) = (form.source.clone(), form.settings(), form.version);
    app.jobs.spawn(ctx, move || {
        let result = estimate(&source, &settings);
        JobDone::ExportPreview { version, settings, result }
    });
}

/// (file size, the result shown small, the middle of the result at 100%).
type Preview = (u64, PixelBuffer, PixelBuffer);

fn estimate(source: &PixelBuffer, settings: &ExportSettings) -> Result<Preview, String> {
    let data = export::encode_with(source, settings).map_err(|e| e.to_string())?;
    // What was written, read again, so the preview shows the real compression artifacts.
    let shown = if settings.format == ImageFileFormat::Ico {
        export::resized(source, settings)
    } else {
        fe_core::io::decode::load_bytes(&data, Some(settings.format.extension().trim_start_matches('.')))
            .map_err(|e| e.to_string())?
    };
    let small = if shown.width().max(shown.height()) > 760 {
        let scale = 760.0 / f64::from(shown.width().max(shown.height()));
        fe_core::transform::resize(
            &shown,
            (f64::from(shown.width()) * scale).max(1.0) as u32,
            (f64::from(shown.height()) * scale).max(1.0) as u32,
            fe_core::transform::ResampleMode::Smooth,
        )
    } else {
        shown.clone()
    };
    let (w, h) = (shown.width().min(380), shown.height().min(300));
    let middle = shown.crop(fe_core::imaging::PixelRect::new(
        ((shown.width() - w) / 2) as i32,
        ((shown.height() - h) / 2) as i32,
        w as i32,
        h as i32,
    ));
    Ok((data.len() as u64, small, middle))
}

/// A real file size arrived: shown when it is for the newest settings.
pub fn on_preview(app: &mut FlashApp, version: u64, settings: ExportSettings, result: Result<Preview, String>) {
    let Some(Dialog::Export(mut form)) = app.dialog.take() else { return };
    if version == form.version {
        form.busy = false;
        match result {
            Ok((bytes, preview, center)) => {
                form.estimate = Some(Estimate { bytes, preview, center, settings });
                form.error = None;
            }
            Err(e) => {
                form.estimate = None;
                form.error = Some(e);
            }
        }
    }
    app.dialog = Some(Dialog::Export(form));
}

/// "Export...": asks where to save, then writes the file (every page when it is Export All Pages).
pub fn finish(app: &mut FlashApp, form: ExportForm) {
    let settings = form.settings();
    let name = if form.all_pages {
        form.suggested_name.clone()
    } else {
        format!("{}-{}x{}", form.suggested_name, settings.width, settings.height)
    };
    let Some((path, format)) = save_dialog("Export", &name, settings.format, app.settings.projects_folder()) else {
        return;
    };
    let settings = ExportSettings { format, ..settings };
    let source = form.source.clone();
    if !form.all_pages {
        job::start(app, "Exporting", move |_| {
            let result = export::save_with(&source, &path, &settings).map_err(|e| e.to_string()).map(|()| {
                let bytes = std::fs::metadata(&path).map_or(0, |m| m.len());
                format!(
                    "Exported {}: {} x {} px, {}",
                    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    settings.width,
                    settings.height,
                    format_file_size(bytes)
                )
            });
            AiDone::Task(result)
        });
        return;
    }
    // Every page is exported at the same enlargement as the one shown in the dialog.
    let images = flatten_pages(app);
    let current = app.project().active_index();
    let crop_size = settings.crop.map_or((source.width(), source.height()), |c| (c.width as u32, c.height as u32));
    let (scale_x, scale_y) =
        (f64::from(settings.width) / f64::from(crop_size.0), f64::from(settings.height) / f64::from(crop_size.1));
    job::start(app, "Exporting all pages", move |progress| {
        let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        for (i, image) in images.iter().enumerate() {
            if progress.flag().load(std::sync::atomic::Ordering::Relaxed) {
                return AiDone::Task(Err("Cancelled.".into()));
            }
            let mut each = settings.clone();
            if i == current {
                // The page that was shown keeps its crop and exact size.
            } else {
                each.crop = None;
                each.width = ((f64::from(image.width()) * scale_x).round() as u32).max(1);
                each.height = ((f64::from(image.height()) * scale_y).round() as u32).max(1);
            }
            let target = folder.join(format!("{stem}-{}{extension}", i + 1));
            if let Err(e) = export::save_with(image, target, &each) {
                return AiDone::Task(Err(e.to_string()));
            }
            progress.report((i + 1) as f64 / images.len() as f64);
        }
        AiDone::Task(Ok(format!("Exported {} pages to {}", images.len(), folder.display())))
    });
}

// ------------------------------------------------------------------------------------------------ Export For

/// File > Export For...
pub fn open_export_for(app: &mut FlashApp) {
    if app.dialog.is_some() {
        return;
    }
    let flat = compositor::flatten(app.doc());
    let pages = app.project().pages().len();
    app.dialog = Some(Dialog::ExportFor(Box::new(ExportForForm::new(flat, pages))));
}

pub fn finish_for(app: &mut FlashApp, form: ExportForForm) {
    let preset = &PRESETS[form.preset];
    let name = format!("{} - {}", project_name(app), preset.name);
    let Some((path, format)) = save_dialog("Export For", &name, form.format, app.settings.projects_folder()) else {
        return;
    };
    let keeps_alpha = !matches!(format, ImageFileFormat::Jpeg | ImageFileFormat::Bmp);
    let background = if form.transparent && keeps_alpha { None } else { Some(ColorRgba::WHITE) };
    let mut settings = ExportSettings::new(format, form.quality, preset.width, preset.height);
    settings.matte = Some(ColorRgba::WHITE);
    let fit = form.fit;
    let sources: Vec<Arc<PixelBuffer>> =
        if form.all_pages { flatten_pages(app).into_iter().map(Arc::new).collect() } else { vec![form.source.clone()] };
    let preset_index = form.preset;
    job::start(app, "Exporting", move |progress| {
        let preset = &PRESETS[preset_index];
        let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        for (i, source) in sources.iter().enumerate() {
            let picture = export::compose(source, preset, fit, background);
            let target =
                if sources.len() == 1 { path.clone() } else { folder.join(format!("{stem}-{}{extension}", i + 1)) };
            if let Err(e) = export::save_with(&picture, target, &settings) {
                return AiDone::Task(Err(e.to_string()));
            }
            progress.report((i + 1) as f64 / sources.len() as f64);
        }
        AiDone::Task(Ok(if sources.len() == 1 {
            format!(
                "Exported {} for {}: {} x {} px",
                path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                preset.name,
                preset.width,
                preset.height
            )
        } else {
            format!("Exported {} pages for {} to {}", sources.len(), preset.name, folder.display())
        }))
    });
}

/// File > Export as SVG / Photoshop: saves the shown page with its layers.
pub fn export_document(app: &mut FlashApp, svg: bool) {
    let (title, label, extension) = if svg {
        ("Save as SVG", "SVG vector picture", "svg")
    } else {
        ("Save as Photoshop file", "Photoshop file", "psd")
    };
    let Some(path) = rfd::FileDialog::new()
        .set_title(title)
        .add_filter(label, &[extension])
        .set_directory(app.settings.projects_folder())
        .set_file_name(format!("{}.{extension}", project_name(app)))
        .save_file()
    else {
        return;
    };
    let doc = app.doc().clone();
    job::start(app, title, move |_| {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let result = if svg {
            fe_core::io::svg::save(&doc, &path)
                .map(|()| format!("Saved {name}. Plain shapes and text are vector; effects and pictures are embedded."))
        } else {
            fe_core::io::psd::save(&doc, &path).map(|()| {
                let layers = doc.layers().iter().filter(|l| !l.is_adjustment()).count();
                format!("Saved {name} with {layers} layers. Photoshop shows objects, text and effects as pictures.")
            })
        };
        AiDone::Task(result.map_err(|e| e.to_string()))
    });
}

// ---------------------------------------------------------------------------------------- PDF, GIF, Batch

/// What a settings window of "Export as ..." leads to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PagesKind {
    Pdf,
    Gif,
    Pptx,
}

/// File > Export as PDF / Animated GIF: the settings window first.
pub fn open_pages_export(app: &mut FlashApp, kind: PagesKind) {
    if app.dialog.is_none() {
        app.dialog = Some(Dialog::Pages(crate::dialogs::PagesForm::new(kind)));
    }
}

/// OK in the settings window: asks where to save, then writes it.
pub fn finish_pages(app: &mut FlashApp, kind: PagesKind, values: fe_core::effects::Values) {
    let (title, label, extension) = match kind {
        PagesKind::Pdf => ("Save as PDF", "PDF document", "pdf"),
        PagesKind::Gif => ("Save as animated GIF", "GIF animation", "gif"),
        PagesKind::Pptx => ("Save as PowerPoint", "PowerPoint presentation", "pptx"),
    };
    let Some(path) = rfd::FileDialog::new()
        .set_title(title)
        .add_filter(label, &[extension])
        .set_directory(app.settings.projects_folder())
        .set_file_name(format!("{}.{extension}", project_name(app)))
        .save_file()
    else {
        return;
    };
    let pages = if kind == PagesKind::Pptx && values.on("editable") { Vec::new() } else { flatten_pages(app) };
    let documents: Vec<fe_core::document::Document> = if kind == PagesKind::Pptx && values.on("editable") {
        app.project().pages().iter().map(|p| p.doc.clone()).collect()
    } else {
        Vec::new()
    };
    let matte = ColorRgba::WHITE;
    job::start(app, title, move |_| {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let count = if documents.is_empty() { pages.len() } else { documents.len() };
        let plural = if count == 1 { "" } else { "s" };
        let result = match kind {
            PagesKind::Pdf => {
                fe_core::io::pdf::save(&pages, &path, values.get("quality") as u8, values.get("dpi"), Some(matte))
                    .map(|()| format!("Saved {name}: {count} page{plural}."))
            }
            PagesKind::Pptx => {
                let quality = values.get("quality") as u8;
                let saved = if documents.is_empty() {
                    fe_core::io::pptx::save_pictures(&pages, &path, quality, values.on("lossless"), Some(matte))
                } else {
                    let refs: Vec<&fe_core::document::Document> = documents.iter().collect();
                    fe_core::io::pptx::save_editable(
                        &refs,
                        &path,
                        fe_core::io::pptx::EditableOptions { quality, matte: Some(matte) },
                    )
                };
                saved.map(|()| {
                    format!(
                        "Saved {name}: {count} slide{plural}{}",
                        if documents.is_empty() { "." } else { " (text and shapes can be edited)." }
                    )
                })
            }
            PagesKind::Gif => {
                let ms = values.get("delay") as u32;
                fe_core::io::gif::encode_animation(&pages, ms, values.on("loop")).and_then(|bytes| std::fs::write(&path, bytes).map_err(Into::into)).map(|()| {
                    if count == 1 {
                        format!("Saved {name} (one page, so it does not move: add pages with Ctrl+Alt+N for more frames).")
                    } else {
                        format!("Saved {name}: {count} frames, {ms} ms each.")
                    }
                })
            }
        };
        AiDone::Task(result.map_err(|e| e.to_string()))
    });
}

/// File > Batch Convert...
pub fn open_batch(app: &mut FlashApp) {
    if app.dialog.is_none() {
        app.dialog = Some(Dialog::Batch(Box::default()));
    }
}

pub fn finish_batch(app: &mut FlashApp, options: fe_core::io::batch::BatchOptions) {
    let output = options.output.clone();
    job::start(app, "Converting the pictures", move |progress: &Progress| {
        let result = fe_core::io::batch::run(
            &options,
            &|done, total| progress.report(done as f64 / total.max(1) as f64),
            progress.flag(),
        );
        let mut text = format!(
            "Converted {} picture{} into {}",
            result.converted,
            if result.converted == 1 { "" } else { "s" },
            output.display()
        );
        if !result.failed.is_empty() {
            text.push_str(&format!(" ({} could not be converted)", result.failed.len()));
            let list: Vec<&str> = result.failed.iter().take(12).map(String::as_str).collect();
            text.push_str(&format!("\n\nThese pictures could not be converted:\n\n{}", list.join("\n")));
        }
        AiDone::Task(Ok(text))
    });
}
