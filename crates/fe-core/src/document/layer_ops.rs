//! Layer commands of the Layers panel and the Layer menu, as plain changes of a document (the app wraps each
//! one in an undo step). Same rules as the C# `LayerCommands`.

use super::{Document, Layer, LayerId, layer_order};
use crate::imaging::TileMap;
use crate::paint::compositor::draw_layer_into_tiles;

/// A new empty layer ("Layer 1", "Layer 2"...) above the active layer; it becomes active.
pub fn new_layer(doc: &mut Document) -> LayerId {
    let name = doc.next_layer_name("Layer");
    let index = doc.active_index().map_or(doc.layers().len(), |i| i + 1);
    doc.insert_layer(index, Layer::new(name))
}

/// Copies of the selected layers ("Name copy"), placed above the topmost selected one and selected.
/// Returns how many were copied.
pub fn duplicate_selected(doc: &mut Document) -> usize {
    let sources = doc.selected_ids();
    let Some(top) = sources.iter().filter_map(|id| doc.index_of(*id)).max() else { return 0 };
    let copies: Vec<Layer> =
        sources.iter().filter_map(|id| doc.layer(*id)).map(|l| l.duplicate(format!("{} copy", l.name))).collect();
    let ids: Vec<LayerId> = copies.iter().map(Layer::id).collect();
    let mut layers = doc.layers().to_vec();
    layers.splice(top + 1..top + 1, copies);
    doc.set_layers(layers, ids.last().copied());
    doc.set_selection(&ids, ids.last().copied());
    ids.len()
}

/// Deletes the selected layers that are not locked. At least one layer always stays. Returns how many were
/// deleted.
pub fn delete_selected(doc: &mut Document) -> usize {
    let mut doomed: Vec<LayerId> =
        doc.selected_ids().into_iter().filter(|id| doc.layer(*id).is_some_and(|l| !l.locked)).collect();
    if doomed.len() >= doc.layers().len() {
        doomed.remove(0); // keep the bottom one
    }
    if doomed.is_empty() {
        return 0;
    }
    let below = doomed.iter().filter_map(|id| doc.index_of(*id)).min().unwrap_or(0);
    let rest: Vec<Layer> = doc.layers().iter().filter(|l| !doomed.contains(&l.id())).cloned().collect();
    let active = rest.get(below.saturating_sub(1).min(rest.len().saturating_sub(1))).map(Layer::id);
    doc.set_layers(rest, active);
    doomed.len()
}

/// Moves the active layer one place up (`+1`) or down (`-1`). False when it cannot move.
pub fn move_active(doc: &mut Document, direction: i32) -> bool {
    let Some(index) = doc.active_index() else { return false };
    let target = index as i64 + direction as i64;
    if target < 0 || target >= doc.layers().len() as i64 {
        return false;
    }
    let active = doc.active_id();
    let mut layers = doc.layers().to_vec();
    layers.swap(index, target as usize);
    doc.set_layers(layers, active);
    true
}

/// Moves the active layer to the very top (`front`) or bottom. False when it is already there.
pub fn move_active_to_end(doc: &mut Document, front: bool) -> bool {
    let Some(index) = doc.active_index() else { return false };
    let target = if front { doc.layers().len() - 1 } else { 0 };
    if index == target {
        return false;
    }
    let active = doc.active_id();
    let mut layers = doc.layers().to_vec();
    let layer = layers.remove(index);
    layers.insert(target, layer);
    doc.set_layers(layers, active);
    true
}

/// Drops the `moved` layers into gap `gap` of the Layers panel list (0 = above the top row). False when the
/// order stays the same.
pub fn move_to_gap(doc: &mut Document, moved: &[LayerId], gap: usize) -> bool {
    let order: Vec<LayerId> = doc.layers().iter().map(Layer::id).collect();
    let Some(new_order) = layer_order::move_to_gap(&order, moved, gap) else { return false };
    let active = doc.active_id();
    let selected = doc.selected_ids();
    let layers: Vec<Layer> = new_order.iter().filter_map(|id| doc.layer(*id).cloned()).collect();
    doc.set_layers(layers, active);
    doc.set_selection(&selected, active);
    true
}

/// Paints the active layer into the one below it and removes it (Merge Down). False when there is no layer
/// below.
pub fn merge_down(doc: &mut Document) -> bool {
    let Some(index) = doc.active_index().filter(|i| *i > 0) else { return false };
    let upper = doc.layers()[index].clone();
    let lower_id = doc.layers()[index - 1].id();
    if let Some(lower) = doc.layer_mut(lower_id) {
        let mut pixels = lower.pixels().clone();
        draw_layer_into_tiles(&mut pixels, &upper);
        lower.replace_pixels(pixels);
    }
    doc.remove_layer(upper.id());
    doc.set_active(Some(lower_id));
    true
}

/// The selected layers that Merge Layers would combine: visible and not locked.
pub fn merge_candidates(doc: &Document) -> Vec<LayerId> {
    doc.selected_ids().into_iter().filter(|id| doc.layer(*id).is_some_and(|l| l.visible && !l.locked)).collect()
}

/// Merges the selected visible, unlocked layers into one "Merged" layer where the top one was. Returns how
/// many layers were merged (0 when fewer than two can be).
pub fn merge_selected(doc: &mut Document) -> usize {
    let ids = merge_candidates(doc);
    if ids.len() < 2 {
        return 0;
    }
    let mut pixels = TileMap::empty();
    for id in &ids {
        if let Some(layer) = doc.layer(*id) {
            draw_layer_into_tiles(&mut pixels, layer);
        }
    }
    let mut merged = Layer::new("Merged");
    merged.replace_pixels(pixels);
    let merged_id = merged.id();
    let top = ids.iter().filter_map(|id| doc.index_of(*id)).max().unwrap_or(0);
    let mut layers = doc.layers().to_vec();
    layers.insert(top + 1, merged);
    layers.retain(|l| !ids.contains(&l.id()));
    doc.set_layers(layers, Some(merged_id));
    ids.len()
}

/// Combines all visible layers into one "Background" layer.
pub fn flatten(doc: &mut Document) {
    let mut pixels = TileMap::empty();
    for layer in doc.layers() {
        draw_layer_into_tiles(&mut pixels, layer);
    }
    let mut flat = Layer::new("Background");
    flat.replace_pixels(pixels);
    let id = flat.id();
    doc.set_layers(vec![flat], Some(id));
}

/// Locks or unlocks every selected layer. Returns how many changed.
pub fn set_lock_of_selected(doc: &mut Document, locked: bool) -> usize {
    let mut changed = 0;
    for id in doc.selected_ids() {
        if let Some(layer) = doc.layer_mut(id)
            && layer.locked != locked
        {
            layer.locked = locked;
            changed += 1;
        }
    }
    changed
}
