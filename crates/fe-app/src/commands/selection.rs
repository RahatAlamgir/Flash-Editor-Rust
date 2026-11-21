//! Edit and Select commands that use the selection (C# `EditCommands`, `LayerCommands.Split`,
//! `ImageCommands.RevealAll`): copy, cut, paste, delete and fill pixels, select all / none / inverse / layer
//! content, split to a new layer and reveal pixels moved past the canvas edge.

use fe_core::document::{Layer, canvas_ops};
use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::objects::{ObjectBox, StoredObject, picture_bytes};
use fe_core::paint::{pixel_move, selection_pixels};
use fe_core::selection::Selection;

use super::checks;
use crate::app::FlashApp;
use crate::tools::ToolKind;
use crate::tools::selecting::apply;

/// The selected area, or the whole canvas when nothing is selected.
fn selection_or_all(app: &FlashApp) -> Selection {
    let doc = app.doc();
    doc.selected_area().cloned().unwrap_or_else(|| Selection::all(doc.width(), doc.height()))
}

/// Ctrl+C / Ctrl+X: the selected pixels of the active layer (or the whole layer). With the Select / Move tool and
/// no pixel selection, the selected object layers are copied as they are (still editable when pasted).
pub fn copy(app: &mut FlashApp, cut: bool) {
    if copy_layers(app, cut) {
        return;
    }
    // Copying only reads pixels: it works on locked layers and keeps objects editable.
    let layer = if cut { checks::editable_layer(app) } else { checks::visible_layer(app) };
    let Some(id) = layer else { return };
    let selection = selection_or_all(app);
    let Some(layer) = app.doc().layer(id) else { return };
    let pixels = selection_pixels::extract(layer.pixels(), &selection);
    let (w, h) = (pixels.width(), pixels.height());
    let origin = (selection.bounds().x, selection.bounds().y);
    app.clipboard.copy_image(pixels, origin);
    if cut {
        app.project_mut().edit("Cut", None, |doc| {
            if let Some(l) = doc.layer_mut(id) {
                selection_pixels::clear(l.pixels_mut(), &selection);
            }
        });
        app.status = "Cut to clipboard".into();
    } else {
        app.status = format!("Copied {w} x {h} px");
    }
}

/// Copies (or cuts) the selected object layers as they are. False when this does not apply.
fn copy_layers(app: &mut FlashApp, cut: bool) -> bool {
    let doc = app.doc();
    if app.tools.active() != ToolKind::Move || doc.selected_area().is_some() {
        return false;
    }
    let ids = doc.selected_ids();
    let layers: Vec<Layer> = ids.iter().filter_map(|id| doc.layer(*id)).cloned().collect();
    if layers.is_empty() || layers.iter().any(|l| l.object().is_none() || l.is_adjustment()) {
        return false;
    }
    let bounds = layers.iter().fold(PixelRect::EMPTY, |b, l| b.union(&l.visible_bounds())).intersect(&doc.bounds());
    if bounds.is_empty() {
        return false;
    }
    let picture = fe_core::paint::compositor::composite_region(doc, bounds)
        .unwrap_or_else(|| layers[layers.len() - 1].pixels().to_buffer(bounds));
    let count = layers.len();
    let all = doc.layers().len();
    let locked = layers.iter().any(|l| l.locked);
    app.clipboard.copy_layers(layers, picture, (bounds.x, bounds.y));
    if !cut {
        app.status = if count == 1 {
            "Copied. Paste it here or into another picture or project.".into()
        } else {
            format!("Copied {count} objects. Paste them here or into another picture or project.")
        };
        return true;
    }
    if count >= all {
        app.status = "Copied. The picture needs at least one layer, so they were not cut.".into();
    } else if locked {
        app.status = "Copied. A locked layer cannot be cut.".into();
    } else {
        app.project_mut().edit("Cut", None, |doc| {
            for id in &ids {
                doc.remove_layer(*id);
            }
        });
        app.status = if count == 1 {
            "Cut. Paste it here or into another picture or project.".into()
        } else {
            format!("Cut {count} objects. Paste them here or into another picture or project.")
        };
    }
    true
}

/// Ctrl+V: copied layers come back as they were (shifted a little when pasted into the same picture); a picture
/// becomes a new layer where it was copied from, or in the middle.
pub fn paste(app: &mut FlashApp) {
    if let Some(layers) = app.clipboard.layers().map(<[Layer]>::to_vec) {
        paste_layers(app, layers);
        return;
    }
    let Some(clip) = app.clipboard.image() else {
        app.status = "Nothing to paste. Copy a picture or a part of one first.".into();
        return;
    };
    // A resizable picture: where it was copied from when it fits, else in the middle (scaled down to fit).
    let (w, h) = (clip.pixels.width() as f64, clip.pixels.height() as f64);
    let (doc_w, doc_h) = (app.doc().width() as f64, app.doc().height() as f64);
    let b = match clip.origin {
        Some((x, y)) if x as f64 + w <= doc_w && y as f64 + h <= doc_h => ObjectBox::new(x as f64, y as f64, w, h, 0.0),
        _ => {
            let scale = (doc_w / w).min(doc_h / h).min(1.0);
            let (w, h) = ((w * scale).max(1.0), (h * scale).max(1.0));
            ObjectBox::new((doc_w - w) / 2.0, (doc_h - h) / 2.0, w, h, 0.0)
        }
    };
    let object = StoredObject::new_image(b, picture_bytes(&clip.pixels), "Pasted");
    super::objects::add_object(app, object, "Pasted", "Paste");
    app.status = "Pasted as a new layer. Drag to move it, handles to resize or rotate.".into();
}

fn paste_layers(app: &mut FlashApp, layers: Vec<Layer>) {
    // Pasted into the picture they came from: slightly shifted, so they are not hidden behind the originals.
    let doc = app.doc();
    let same_spot =
        layers.iter().any(|c| doc.layers().iter().any(|l| l.object().is_some() && l.object() == c.object()));
    let count = layers.len();
    app.project_mut().edit("Paste", None, |doc| {
        let mut ids = Vec::new();
        for copied in &layers {
            let mut layer = copied.duplicate(copied.name.clone());
            if same_spot {
                pixel_move::move_layer(&mut layer, 24, 24);
            }
            ids.push(doc.push_layer(layer));
        }
        let active = ids.last().copied();
        doc.set_selection(&ids, active);
        doc.set_selected_area(None);
    });
    app.tools.activate(ToolKind::Move);
    app.status = if count == 1 {
        "Pasted. It is still fully editable.".into()
    } else {
        format!("Pasted {count} objects. They are still fully editable.")
    };
}

/// Delete: clears the selected pixels. False when there is no selection.
pub fn delete_pixels(app: &mut FlashApp) -> bool {
    let Some(selection) = app.doc().selected_area().cloned() else { return false };
    let Some(id) = checks::editable_layer(app) else { return true };
    app.project_mut().edit("Delete", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            selection_pixels::clear(l.pixels_mut(), &selection);
        }
    });
    true
}

/// Fills the selection (or the whole layer) with a color.
pub fn fill(app: &mut FlashApp, color: ColorRgba) {
    let Some(id) = checks::editable_layer(app) else { return };
    let selection = selection_or_all(app);
    app.project_mut().edit("Fill", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            selection_pixels::fill(l.pixels_mut(), &selection, color);
        }
    });
}

/// Ctrl+A with a tool other than Select / Move: selects every pixel.
pub fn select_all(app: &mut FlashApp) {
    let (w, h) = (app.doc().width(), app.doc().height());
    app.project_mut().active_page_mut().doc.set_selected_area(Some(Selection::all(w, h)));
    app.status = format!("Selected all ({w} x {h} px).");
}

/// Ctrl+Shift+D / Esc.
pub fn deselect(app: &mut FlashApp) {
    apply::deselect(app);
}

/// Ctrl+Shift+I: everything that was not selected (everything when nothing was).
pub fn invert(app: &mut FlashApp) {
    let doc = app.doc();
    let inverted = match doc.selected_area() {
        Some(s) => s.invert(),
        None => Some(Selection::all(doc.width(), doc.height())),
    };
    app.project_mut().active_page_mut().doc.set_selected_area(inverted);
}

/// Select > Layer Content: every visible pixel of the active layer (partly see-through pixels partly).
pub fn layer_content(app: &mut FlashApp) {
    let doc = app.doc();
    let Some(layer) = doc.active_layer() else { return };
    let bounds = doc.bounds();
    let alpha: Vec<u8> = layer.pixels().read_rect(bounds).as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    let name = layer.name.clone();
    let selection = Selection::from_mask(alpha, bounds, doc.width(), doc.height());
    let empty = selection.is_none();
    app.project_mut().active_page_mut().doc.set_selected_area(selection);
    app.status = if empty { "The layer is empty.".into() } else { format!("Selected the content of \"{name}\".") };
}

/// Split to New Layer: the selected part moves onto a new layer above; the original keeps the rest.
pub fn split(app: &mut FlashApp) {
    let Some(selection) = app.doc().selected_area().cloned() else {
        app.status = "Select an area first, then split it onto a new layer.".into();
        return;
    };
    let Some(id) = checks::editable_layer(app) else { return };
    let mut names = (String::new(), String::new());
    app.project_mut().edit("Split", None, |doc| {
        let Some(index) = doc.index_of(id) else { return };
        let layer = &mut doc.layers_mut()[index];
        let mut piece = layer.duplicate(format!("{} (split)", layer.name));
        piece.replace_pixels(selection_pixels::keep_only(layer.pixels(), &selection));
        piece.restore = None;
        selection_pixels::clear(layer.pixels_mut(), &selection);
        names = (piece.name.clone(), layer.name.clone());
        doc.insert_layer(index + 1, piece);
        doc.set_selected_area(None);
    });
    app.status = format!(
        "Split the selected part onto a new layer \"{}\". \"{}\" is still there without it. Ctrl+Z undoes it.",
        names.0, names.1
    );
}

/// Image > Reveal All: grows the canvas so pixels moved past its edge show again.
pub fn reveal_all(app: &mut FlashApp) {
    let all = canvas_ops::content_bounds(app.doc());
    if all == app.doc().bounds() {
        app.status = "Reveal All: nothing is outside the canvas.".into();
        return;
    }
    if all.width > canvas_ops::MAX_SIDE || all.height > canvas_ops::MAX_SIDE {
        app.status = format!("The result would be larger than {} px.", canvas_ops::MAX_SIDE);
        return;
    }
    app.project_mut().edit("Reveal All", None, |doc| {
        canvas_ops::reframe(doc, all.width as u32, all.height as u32, -all.x, -all.y);
    });
    app.current_view_mut().fit();
    app.status = format!("Reveal All: {} x {}", all.width, all.height);
}
