//! Frames and photo grids (C# `InsertCommands.AddFrame / PhotoGrid / FillFrames`): empty shapes to drop pictures
//! into, a collage arrangement of them, and filling the selected frames with pictures.

use fe_core::document::object_ops;
use fe_core::imaging::ColorRgba;
use fe_core::objects::{COLLAGE_LAYOUTS, ObjectBox, ShapeData, ShapeKind, StoredObject, picture_bytes};

use super::objects::add_object;
use crate::app::FlashApp;
use crate::dialogs::PhotoGridSettings;

const FRAME_COLOR: ColorRgba = ColorRgba::rgb(226, 232, 240);

/// An empty frame: a shape to drop a picture into (Insert > Fill Frames).
pub fn add_frame(app: &mut FlashApp, kind: &str) {
    let (dw, dh) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    let size = (dw.min(dh) * 0.42).max(60.0);
    let w = if matches!(kind, "Rectangle" | "Rounded") { size * 1.3 } else { size };
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - size) / 2.0).round(), w, size, 0.0);
    let mut shape = ShapeData::new(
        match kind {
            "Circle" => ShapeKind::Ellipse,
            "Heart" => ShapeKind::Heart,
            "Star" => ShapeKind::Star,
            _ => ShapeKind::Rectangle,
        },
        Some(FRAME_COLOR),
        None,
        0.0,
    );
    if kind == "Rounded" {
        shape.corner_radius = size * 0.12;
    }
    add_object(app, StoredObject::new_shape(b, &shape), "Frame", "Add Frame");
    app.status = "Added an empty frame. Choose Insert > Fill Frames to put a photo in it.".into();
}

/// Adds the frames of a collage layout to the picture (one undo step); they stay selected.
pub fn photo_grid(app: &mut FlashApp, s: &PhotoGridSettings) {
    let Some(layout) = COLLAGE_LAYOUTS.get(s.layout) else { return };
    let (dw, dh) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    let canvas = app.doc().bounds();
    let mut layers = Vec::new();
    for (i, (x, y, w, h)) in layout.place(dw, dh, s.padding, s.gap).into_iter().enumerate() {
        let mut shape = ShapeData::new(ShapeKind::Rectangle, Some(FRAME_COLOR), None, 0.0);
        shape.corner_radius = s.corner_radius.min(w.min(h) / 2.0);
        let object = StoredObject::new_shape(ObjectBox::new(x, y, w, h, 0.0), &shape);
        layers.push(object_ops::object_layer(format!("Frame {}", i + 1), object, canvas));
    }
    let count = layers.len();
    app.project_mut().edit("Add Photo Grid", None, |doc| {
        let ids: Vec<_> = layers.iter().map(|l| l.id()).collect();
        for layer in layers {
            let at = doc.layers().len();
            doc.insert_layer(at, layer);
        }
        doc.set_selection(&ids, ids.last().copied());
        doc.set_selected_area(None);
    });
    app.status =
        format!("Added {count} frames. They are selected: choose Insert > Fill Frames to drop your photos in.");
}

/// The selected shapes that can be filled with a picture, in reading order (top to bottom, then left to right).
fn selected_frames(app: &FlashApp) -> Vec<fe_core::document::LayerId> {
    let doc = app.doc();
    let mut frames: Vec<_> = doc
        .selected_ids()
        .into_iter()
        .filter_map(|id| doc.layer(id))
        .filter(|l| l.visible && !l.locked)
        .filter(|l| l.object().and_then(|o| o.shape()).is_some_and(|s| s.kind != ShapeKind::Line))
        .filter_map(|l| {
            let c = l.object()?.box_()?.center();
            Some((l.id(), (c.y / 20.0).round(), c.x))
        })
        .collect();
    frames.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)));
    frames.into_iter().map(|f| f.0).collect()
}

/// Puts pictures into the selected frames: each picture covers its frame and is clipped to the frame's shape. The
/// picture becomes part of the frame, so moving, resizing or turning the frame takes the picture along.
pub fn fill_frames(app: &mut FlashApp) {
    let frames = selected_frames(app);
    if frames.is_empty() {
        app.status = "Select one or more frames (shapes) first.".into();
        return;
    }
    let title = if frames.len() == 1 {
        "Choose a picture for the frame".to_owned()
    } else {
        format!("Choose up to {} pictures for the frames", frames.len())
    };
    let Some(files) = rfd::FileDialog::new()
        .set_title(&title)
        .add_filter("Pictures", &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tif", "tiff", "ico"])
        .pick_files()
    else {
        return;
    };
    let mut pictures = Vec::new();
    for path in files.iter().take(frames.len()) {
        if let Ok(image) = fe_core::io::decode::load(path) {
            let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Picture".into());
            pictures.push((name, picture_bytes(&image)));
        }
    }
    if pictures.is_empty() {
        app.status = "None of those pictures could be opened.".into();
        return;
    }
    let canvas = app.doc().bounds();
    let filled: Vec<_> = frames.iter().copied().take(pictures.len()).collect();
    app.project_mut().edit("Fill Frames", None, |doc| {
        for (id, (name, bytes)) in filled.iter().zip(&pictures) {
            if let Some(layer) = doc.layer_mut(*id)
                && let Some(mut object) = layer.object().cloned()
            {
                object.set_picture_fill(Some((bytes.clone(), name)));
                object_ops::set_object(layer, object, canvas);
            }
        }
        doc.set_selection(&filled, filled.last().copied());
    });
    app.status = format!(
        "Filled {} frame{}. The picture is part of the frame: move or resize the frame and the picture goes along. \
         Double-click a filled frame, then drag to move the picture inside.",
        filled.len(),
        if filled.len() == 1 { "" } else { "s" }
    );
}
