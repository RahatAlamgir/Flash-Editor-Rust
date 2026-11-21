//! The Image tab (C# `ImageCommands`): operations on the whole canvas, all layers at once, one undo step each.

use fe_core::document::canvas_ops::{self, transform_all};
use fe_core::effects;
use fe_core::imaging::PixelBuffer;
use fe_core::objects::transforms;
use fe_core::transform::{self, ResampleMode};

use crate::app::FlashApp;

fn done(app: &mut FlashApp, name: &str) {
    app.tools.reset_active();
    app.current_view_mut().fit();
    let (w, h) = (app.doc().width(), app.doc().height());
    app.status = format!("{name}: {w} x {h}");
}

/// Image > Image Size.
pub fn image_size(app: &mut FlashApp, width: u32, height: u32, mode: ResampleMode) {
    let (ow, oh) = (app.doc().width(), app.doc().height());
    if (width, height) == (ow, oh) {
        return;
    }
    let (sx, sy) = (f64::from(width) / f64::from(ow), f64::from(height) / f64::from(oh));
    app.project_mut().edit("Image Size", None, |doc| {
        transform_all(
            doc,
            &|p: &PixelBuffer| transform::resize(p, width, height, mode),
            Some(&|o, _, _| o.scaled(sx, sy)),
        );
    });
    done(app, "Image Size");
}

/// Image > Upscale 2x: twice the size with the sharpest method, then cleaned up (shapes and text stay editable).
pub fn upscale_2x(app: &mut FlashApp) {
    let (w, h) = (app.doc().width() * 2, app.doc().height() * 2);
    if w > canvas_ops::MAX_SIDE as u32 || h > canvas_ops::MAX_SIDE as u32 {
        app.status = format!("The result would be larger than {} px.", canvas_ops::MAX_SIDE);
        return;
    }
    let mut values = effects::ENHANCE.defaults();
    values.set("sharpen", 90.0);
    values.set("radius", 1.2);
    values.set("clarity", 20.0);
    values.set("denoise", 20.0);
    app.project_mut().edit("Upscale 2x", None, |doc| {
        transform_all(
            doc,
            &|p: &PixelBuffer| effects::ENHANCE.apply(&transform::resize(p, w, h, ResampleMode::Lanczos), &values),
            Some(&|o, _, _| o.scaled(2.0, 2.0)),
        );
    });
    done(app, "Upscale 2x");
}

fn reframe(app: &mut FlashApp, name: &str, width: u32, height: u32, shift: (i32, i32)) {
    app.project_mut().edit(name, None, |doc| canvas_ops::reframe(doc, width, height, shift.0, shift.1));
    done(app, name);
}

/// Image > Canvas Size: more or less room around the picture, no scaling. `anchor` is 0..8 (row by row).
pub fn canvas_size(app: &mut FlashApp, width: u32, height: u32, anchor: usize) {
    let (ow, oh) = (app.doc().width(), app.doc().height());
    if (width, height) == (ow, oh) {
        return;
    }
    let (ax, ay) = ((anchor % 3) as f64 / 2.0, (anchor / 3) as f64 / 2.0);
    let shift = (
        ((f64::from(width) - f64::from(ow)) * ax).round_ties_even() as i32,
        ((f64::from(height) - f64::from(oh)) * ay).round_ties_even() as i32,
    );
    reframe(app, "Canvas Size", width, height, shift);
}

/// Image > Trim Edges: cuts away fully transparent borders around the visible picture.
pub fn trim(app: &mut FlashApp) {
    let bounds = fe_core::paint::compositor::flatten(app.doc()).visible_bounds();
    if bounds.is_empty() || bounds == app.doc().bounds() {
        app.status = if bounds.is_empty() {
            "Trim: the image is completely transparent."
        } else {
            "Trim: no transparent edges to remove."
        }
        .into();
        return;
    }
    reframe(app, "Trim", bounds.width as u32, bounds.height as u32, (-bounds.x, -bounds.y));
}

fn rotate_with(app: &mut FlashApp, name: &str, degrees: f64, pixels: impl Fn(&PixelBuffer) -> PixelBuffer + Sync) {
    let old = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    app.project_mut().edit(name, None, |doc| {
        transform_all(doc, &pixels, Some(&|o, w, h| transforms::rotate(o, degrees, old, (f64::from(w), f64::from(h)))));
    });
    done(app, name);
}

/// 90° right / left or 180°.
pub fn rotate_quarter(app: &mut FlashApp, degrees: i32) {
    match degrees {
        90 => rotate_with(app, "Rotate 90° CW", 90.0, transform::rotate_90_cw),
        -90 => rotate_with(app, "Rotate 90° CCW", -90.0, transform::rotate_90_ccw),
        _ => rotate_with(app, "Rotate 180°", 180.0, transform::rotate_180),
    }
}

/// Image > Rotate any angle.
pub fn rotate_custom(app: &mut FlashApp, degrees: f64) {
    if degrees == 0.0 {
        return;
    }
    rotate_with(app, &format!("Rotate {degrees:.1}°"), degrees, move |p| transform::rotate(p, degrees));
}

/// Image > Straighten.
pub fn straighten(app: &mut FlashApp, degrees: f64, cut: bool) {
    if degrees == 0.0 {
        return;
    }
    rotate_with(app, &format!("Straighten {degrees:.1}°"), degrees, move |p| transform::straighten(p, degrees, cut));
}

/// Flips the whole picture left-right or top-bottom.
pub fn flip(app: &mut FlashApp, horizontal: bool) {
    let (w, h) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    if horizontal {
        app.project_mut().edit("Flip Horizontal", None, |doc| {
            transform_all(doc, &transform::flip_horizontal, Some(&|o, _, _| transforms::flip_horizontal(o, w)));
        });
    } else {
        app.project_mut().edit("Flip Vertical", None, |doc| {
            transform_all(doc, &transform::flip_vertical, Some(&|o, _, _| transforms::flip_vertical(o, h)));
        });
    }
    done(app, if horizontal { "Flip Horizontal" } else { "Flip Vertical" });
}

/// Layer > Crop Layer to Selection: the active layer keeps only the selected part (it becomes a movable picture).
pub fn crop_layer_to_selection(app: &mut FlashApp) {
    let Some(sel) = app.doc().selected_area().cloned() else {
        app.status = "Make a selection first.".into();
        return;
    };
    let Some(id) = super::checks::editable_layer(app) else { return };
    let canvas = app.doc().bounds();
    let Some(layer) = app.doc().layer(id) else { return };
    let name = layer.name.clone();
    let object = fe_core::document::layer_crop::selection_object(layer, &sel, canvas);
    app.project_mut().edit("Crop Layer to Selection", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            match object {
                Some(o) => fe_core::document::object_ops::set_object(l, o, canvas),
                None => l.replace_pixels(fe_core::imaging::TileMap::empty()),
            }
        }
        doc.set_selected_area(None);
    });
    app.tools.activate(crate::tools::ToolKind::Move);
    app.status = format!("Kept only the selected part of \"{name}\". It is now a movable, resizable picture.");
}
