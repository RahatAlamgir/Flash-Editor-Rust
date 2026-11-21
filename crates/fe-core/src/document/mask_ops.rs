//! Layer > Layer Mask commands on a document (C# `MaskCommands`): add a mask (show all / hide all / from the
//! selection), hide the selection, invert, switch on / off, apply to the pixels, delete, and Clip to Layer Below.
//! Each returns false when it does not apply.

use super::canvas_ops::bake_mask;
use super::{Document, LayerId, LayerMask};
use crate::selection::Selection;

/// True when the layer can have a mask (exists and is not an adjustment layer).
fn maskable(doc: &Document, id: LayerId) -> bool {
    doc.layer(id).is_some_and(|l| !l.is_adjustment())
}

/// Adds a mask that shows (`reveal`) or hides everything. Not when the layer already has one.
pub fn add(doc: &mut Document, id: LayerId, reveal: bool) -> bool {
    let (w, h) = (doc.width(), doc.height());
    if !maskable(doc, id) || doc.layer(id).is_some_and(|l| l.mask().is_some()) {
        return false;
    }
    let layer = doc.layer_mut(id).expect("checked");
    let _ = layer.set_mask(Some(LayerMask::new(w, h, if reveal { 255 } else { 0 })), w, h);
    layer.mask_enabled = true;
    true
}

/// "Show selection only" starts a fresh mask from the selection; "hide selection" hides the selected area on top
/// of the existing mask (or of a new one that shows everything).
pub fn from_selection(doc: &mut Document, id: LayerId, selection: &Selection, reveal_selected: bool) -> bool {
    let (w, h) = (doc.width(), doc.height());
    if !maskable(doc, id) {
        return false;
    }
    let layer = doc.layer_mut(id).expect("checked");
    let mask = match layer.mask().cloned() {
        Some(mut existing) if !reveal_selected => {
            existing.apply_selection(selection, false);
            existing
        }
        _ => LayerMask::from_selection(selection, reveal_selected),
    };
    let _ = layer.set_mask(Some(mask), w, h);
    layer.mask_enabled = true;
    true
}

pub fn invert(doc: &mut Document, id: LayerId) -> bool {
    match doc.layer_mut(id).and_then(|l| l.mask_mut()) {
        Some(mask) => {
            mask.invert();
            true
        }
        None => false,
    }
}

/// Switches the mask off (the whole layer shows) or on again.
pub fn toggle(doc: &mut Document, id: LayerId) -> bool {
    match doc.layer_mut(id).filter(|l| l.mask().is_some()) {
        Some(layer) => {
            layer.mask_enabled = !layer.mask_enabled;
            true
        }
        None => false,
    }
}

pub fn delete(doc: &mut Document, id: LayerId) -> bool {
    let (w, h) = (doc.width(), doc.height());
    match doc.layer_mut(id).filter(|l| l.mask().is_some()) {
        Some(layer) => {
            let _ = layer.set_mask(None, w, h);
            true
        }
        None => false,
    }
}

/// Makes the mask part of the pixels (hidden parts become see-through for good) and removes it.
pub fn apply(doc: &mut Document, id: LayerId) -> bool {
    let (w, h) = (doc.width(), doc.height());
    let Some(layer) = doc.layer_mut(id) else { return false };
    let Some(mask) = layer.mask().cloned() else { return false };
    let baked = bake_mask(layer.pixels(), &mask);
    layer.replace_pixels(baked);
    let _ = layer.set_mask(None, w, h);
    true
}

/// Clip to Layer Below: a mask from the shape of the layer under it (where it has color, and how see-through it
/// is there), so the layer shows only inside that shape. An existing mask keeps working (both have to show).
/// Returns the names of the layer and the one below.
pub fn clip_to_below(doc: &mut Document, id: LayerId) -> Option<(String, String)> {
    let index = doc.index_of(id)?;
    if index == 0 || !maskable(doc, id) || doc.layers()[index - 1].is_adjustment() {
        return None;
    }
    let canvas = doc.bounds();
    let below = &doc.layers()[index - 1];
    let mut shape: Vec<u8> = below.pixels().read_rect(canvas).as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    if let Some(m) = below.effective_mask() {
        for (v, c) in shape.iter_mut().zip(m.copy_region(canvas)) {
            *v = (*v as u32 * c as u32 / 255) as u8;
        }
    }
    let below_name = below.name.clone();
    let mut clip = LayerMask::new(doc.width(), doc.height(), 0);
    clip.paste_region(canvas, &shape);
    let layer = doc.layer_mut(id)?;
    if let Some(existing) = layer.mask() {
        clip.multiply(existing);
    }
    let (w, h) = (clip.width(), clip.height());
    let _ = layer.set_mask(Some(clip), w, h);
    layer.mask_enabled = true;
    Some((layer.name.clone(), below_name))
}
