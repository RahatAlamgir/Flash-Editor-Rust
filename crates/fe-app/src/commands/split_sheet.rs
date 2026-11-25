//! Image > Split Asset Sheet (C# `ImageCommands.SplitSheet`): splits an asset / sprite sheet into its separate items, as
//! layers (each a movable, resizable picture at its original place) and / or as PNG files.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use fe_core::document::object_ops;
use fe_core::io::batch::unique_path;
use fe_core::io::encode;
use fe_core::io::format::ImageFileFormat;
use fe_core::objects::{ObjectBox, StoredObject, picture_bytes};
use fe_core::paint::compositor;
use fe_core::sheets::SheetPiece;

use crate::ai::masks::canvas_mask;
use crate::app::FlashApp;
use crate::app::jobs::JobDone;
use crate::dialogs::{Dialog, SplitSheetForm};

/// The ribbon button: opens the window.
pub fn open(app: &mut FlashApp) {
    if app.dialog.is_some() {
        return;
    }
    let (w, h) = (app.doc().width(), app.doc().height());
    let sheet = compositor::flatten(app.doc());
    let selection = app.doc().selected_area().map(|s| canvas_mask(s, w, h));
    let name = app.project().name.clone();
    let folder = app.project().path.as_ref().and_then(|p| p.parent()).map(PathBuf::from).or_else(dirs::picture_dir);
    app.dialog = Some(Dialog::SplitSheet(Box::new(SplitSheetForm::new(sheet, selection, name, folder))));
}

/// Looks for the pieces with the current settings on a background thread.
pub fn start_search(app: &mut FlashApp, ctx: &egui::Context, form: &SplitSheetForm) {
    let (sheet, selection, settings, version) =
        (form.sheet.clone(), form.selection.clone(), form.settings(), form.version);
    app.jobs.spawn(ctx, move || {
        let result =
            fe_core::sheets::split(&sheet, &settings, selection.as_deref().map(Vec::as_slice), &AtomicBool::new(false))
                .map(Arc::new);
        JobDone::SheetPieces { version, result }
    });
}

/// The pieces arrived: shown when they are for the newest settings.
pub fn on_pieces(app: &mut FlashApp, version: u64, result: Result<Arc<Vec<SheetPiece>>, String>) {
    let Some(Dialog::SplitSheet(mut form)) = app.dialog.take() else { return };
    if version == form.version {
        form.busy = false;
        match result {
            Ok(pieces) => form.pieces = Some(pieces),
            Err(e) => form.error = Some(e),
        }
    }
    app.dialog = Some(Dialog::SplitSheet(form));
}

/// OK: saves the PNG files and / or adds the pieces as layers.
pub fn finish(app: &mut FlashApp, form: SplitSheetForm) {
    let Some(pieces) = form.pieces.clone() else { return };
    if pieces.is_empty() {
        return;
    }
    app.tools.reset_active();
    let base = form.name.trim().to_owned();
    let folder = form.save_files.then(|| PathBuf::from(form.folder.trim()));
    if let Some(folder) = &folder {
        let digits = pieces.len().to_string().len().max(2);
        let result = std::fs::create_dir_all(folder).map_err(|e| e.to_string()).and_then(|()| {
            for (i, piece) in pieces.iter().enumerate() {
                let target = unique_path(folder, &format!("{base}_{:0digits$}", i + 1), ".png");
                encode::save(&piece.pixels, target, ImageFileFormat::Png, 100, None).map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        if let Err(e) = result {
            app.status = format!("Could not save the pieces: {e}");
            return;
        }
    }
    if form.as_layers {
        add_as_layers(app, &pieces, &base, form.hide_original);
    } else {
        app.project_mut().active_page_mut().doc.set_selected_area(None);
    }
    app.status = match &folder {
        Some(f) => format!("Split into {} pieces; saved {} PNG files to {}.", pieces.len(), pieces.len(), f.display()),
        None => format!("Split into {} pieces, each on its own layer.", pieces.len()),
    };
}

fn add_as_layers(app: &mut FlashApp, pieces: &[SheetPiece], base: &str, hide_original: bool) {
    let canvas = app.doc().bounds();
    app.project_mut().edit("Split Asset Sheet", None, |doc| {
        if hide_original {
            for layer in doc.layers_mut() {
                layer.visible = false;
            }
        }
        let mut ids = Vec::new();
        for (i, piece) in pieces.iter().enumerate() {
            let b = piece.bounds;
            let name = format!("{base} {}", i + 1);
            let object = StoredObject::new_image(
                ObjectBox::new(f64::from(b.x), f64::from(b.y), f64::from(b.width), f64::from(b.height), 0.0),
                picture_bytes(&piece.pixels),
                &name,
            );
            ids.push(doc.push_layer(object_ops::object_layer(name, object, canvas)));
        }
        if let Some(last) = ids.last().copied() {
            doc.set_selection(&ids, Some(last));
        }
        doc.set_selected_area(None);
    });
}
