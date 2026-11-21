//! Paint Bucket (C# `BucketTool`): fills an area of similar color with the foreground color. Eyedropper (C#
//! `EyedropperTool`): picks a color from what you see; Alt+click or right-click picks the background color.

use fe_core::paint::{bucket, compositor};

use super::super::Pointer;
use crate::app::FlashApp;
use crate::commands::checks;

pub fn bucket(app: &mut FlashApp, p: &Pointer) {
    if !p.pressed {
        return;
    }
    let (x, y) = p.pixel();
    if !app.doc().bounds().contains(x, y) {
        return;
    }
    let Some(id) = checks::editable_layer(app) else { return };
    let options = app.tools.paint.bucket.clone();
    let color = app.foreground;
    let selection = app.doc().selected_area().cloned();
    app.project_mut().edit("Paint Bucket", None, |doc| {
        let canvas = doc.bounds();
        if let Some(layer) = doc.layer_mut(id) {
            let tolerance = options.tolerance.clamp(0.0, 255.0) as u8;
            bucket::fill(layer.pixels_mut(), canvas, (x, y), color, tolerance, options.contiguous, selection.as_ref());
        }
    });
}

pub fn eyedropper(app: &mut FlashApp, p: &Pointer) {
    if p.pressed || p.secondary_pressed {
        app.tools.paint.pick_background = p.secondary_pressed || p.mods.alt;
        app.tools.paint.picking = true;
    }
    if !app.tools.paint.picking {
        return;
    }
    if p.released || (!p.down && !p.secondary_pressed) {
        app.tools.paint.picking = false;
    }
    let (x, y) = p.pixel();
    if !app.doc().bounds().contains(x, y) {
        return;
    }
    let color = compositor::sample_merged(app.doc(), x, y);
    if color.a == 0 {
        return; // nothing visible there
    }
    let color = color.with_alpha(255);
    if app.tools.paint.pick_background {
        app.background = color;
    } else {
        app.foreground = color;
    }
    app.status = format!("Picked {}", color.to_hex());
}
