//! The Crop and Perspective Crop tools (C# `CropTool`, `PerspectiveCropTool`). They cut the selected layer (not the
//! canvas): the cut piece becomes a new picture layer above it and the original is hidden, never changed.

mod perspective;
mod rect;

use egui::Key;
use fe_core::document::layer_crop;

use fe_core::imaging::{PixelRect, PointD};
pub use rect::fit_aspect;

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// Crop shape presets (width : height). 0 = free.
pub const ASPECTS: [(&str, f64); 7] = [
    ("Free", 0.0),
    ("Square 1:1", 1.0),
    ("Portrait 4:5 (Instagram)", 4.0 / 5.0),
    ("Story 9:16", 9.0 / 16.0),
    ("Wide 16:9", 16.0 / 9.0),
    ("Photo 3:2", 3.0 / 2.0),
    ("Photo 4:3", 4.0 / 3.0),
];

/// What the canvas draws for the two tools.
pub enum CropOverlay {
    Rect(PixelRect),
    Quad(Vec<PointD>),
}

/// The state of both tools.
#[derive(Default)]
pub struct CropTools {
    pub rect: rect::RectCrop,
    pub aspect: usize,
    pub quad: perspective::QuadCrop,
}

impl CropTools {
    pub fn cancel(&mut self) {
        self.rect.clear();
        self.quad.clear();
    }

    /// The size text / hint of the active tool, for the options bar.
    pub fn hint(&self, tool: ToolKind) -> String {
        match tool {
            ToolKind::Crop => match self.rect.rect {
                Some(r) => {
                    format!("{} x {} px. Drag the corners and edges, then press Enter. Esc cancels.", r.width, r.height)
                }
                None => "Drag a box over the part of the layer you want to keep.".into(),
            },
            _ => self.quad.hint(),
        }
    }
}

pub fn overlay(app: &FlashApp) -> Option<CropOverlay> {
    match app.tools.active() {
        ToolKind::Crop => app.tools.crop.rect.rect.map(CropOverlay::Rect),
        ToolKind::PerspectiveCrop if !app.tools.crop.quad.points.is_empty() => {
            Some(CropOverlay::Quad(app.tools.crop.quad.points.clone()))
        }
        _ => None,
    }
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    match tool {
        ToolKind::Crop => rect::pointer(app, p),
        ToolKind::PerspectiveCrop => perspective::pointer(app, p),
        _ => {}
    }
}

/// Enter applies, Esc cancels. True when the key was used.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    match app.tools.active() {
        ToolKind::Crop if app.tools.crop.rect.rect.is_some() => match key {
            Key::Enter => {
                apply_rect(app);
                true
            }
            Key::Escape => {
                app.tools.crop.rect.clear();
                app.status = "Crop cancelled.".into();
                true
            }
            _ => false,
        },
        ToolKind::PerspectiveCrop if app.tools.crop.quad.points.len() == 4 => match key {
            Key::Enter => {
                perspective::apply(app);
                true
            }
            Key::Escape => {
                app.tools.crop.quad.clear();
                true
            }
            _ => false,
        },
        _ => false,
    }
}

/// Starts cropping the selected layer (the whole of its content is the first box).
pub fn crop_active_layer(app: &mut FlashApp) {
    app.tools.activate(ToolKind::Crop);
    app.tools.crop.cancel();
    let Some(layer) = app.doc().active_layer().filter(|l| l.visible && !l.is_adjustment()) else {
        app.status = "Select the layer or object you want to crop first.".into();
        return;
    };
    let name = layer.name.clone();
    let bounds = layer.visible_bounds().intersect(&app.doc().bounds());
    if bounds.is_empty() {
        app.status = format!("\"{name}\" is empty or outside the picture, there is nothing to crop.");
        return;
    }
    app.tools.crop.rect.rect = Some(rect::fit_aspect(bounds, ASPECTS[app.tools.crop.aspect].1));
    app.status = format!(
        "Cropping \"{name}\": drag the corners and edges to adjust the box, then press Enter. Esc cancels. The canvas keeps its size."
    );
}

/// Puts a cropped piece above the original, hides the original and selects the piece (one undo step).
fn insert_piece(app: &mut FlashApp, step: &str, original: fe_core::document::LayerId, piece: fe_core::document::Layer) {
    app.project_mut().edit(step, None, |doc| {
        let index = doc.index_of(original).map_or(doc.layers().len(), |i| i + 1);
        let id = doc.insert_layer(index, piece);
        doc.set_active(Some(id));
        if let Some(l) = doc.layer_mut(original) {
            l.visible = false;
        }
    });
    app.tools.activate(ToolKind::Move);
}

fn apply_rect(app: &mut FlashApp) {
    let Some(rect) = app.tools.crop.rect.rect else { return };
    let Some(layer) = app.doc().active_layer().filter(|l| l.visible && !l.is_adjustment()) else {
        app.status = "Select a visible layer to crop (the box stays, so you can pick one and press Enter).".into();
        return;
    };
    let (id, name) = (layer.id(), layer.name.clone());
    let Some(piece) = layer_crop::crop_layer(layer, rect, app.doc().bounds()) else {
        app.status = format!("Nothing of \"{name}\" is inside the box.");
        return;
    };
    app.tools.crop.rect.clear();
    insert_piece(app, "Crop", id, piece);
    app.status = format!(
        "Cropped to {} x {} on a new layer. \"{name}\" is hidden, not changed (show it with the eye icon). The canvas keeps its size. Ctrl+Z undoes it.",
        rect.width, rect.height
    );
}
