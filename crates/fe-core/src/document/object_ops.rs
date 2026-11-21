//! Object layers (C# `Layer.SetObject`, `LayerCommands.Group / Ungroup / FlipLayer`, `MakePictureObject`,
//! `ConvertToPixels`, `AlignCommands`): putting a new object on a layer and drawing it again when it changes.
//!
//! A changed object is drawn by the Rust renderer when it can (shapes, pictures, groups of them). Objects the Rust
//! version cannot draw yet (text, charts, codes, shadows and other looks: Phases 9–10) keep their saved drawing,
//! which is moved, stretched or turned with them, so they keep their look.

use super::{Document, Layer, LayerId};
use crate::imaging::{PixelRect, TileMap};
use crate::objects::{GroupMember, ObjectBox, StoredObject, picture_bytes};
use crate::paint::compositor;
use crate::render;

/// True when two descriptions differ only in where the object is (box, flips, reshaped corners).
fn only_placement_changed(a: &StoredObject, b: &StoredObject) -> bool {
    let strip = |o: &StoredObject| {
        let mut j = o.json.clone();
        if let Some(m) = j.as_object_mut() {
            for k in ["Box", "FlipX", "FlipY", "Warp"] {
                m.shift_remove(k);
            }
        }
        j
    };
    strip(a) == strip(b)
}

/// The drawing of `new`. `base` is the layer's drawing of an earlier version of the object (the start of a drag),
/// used for objects the Rust version cannot draw itself.
pub fn drawing_for(new: &StoredObject, base: Option<(&TileMap, &StoredObject)>, canvas: PixelRect) -> TileMap {
    let exact = new.can_draw() && !new.has_looks();
    if !exact
        && let Some((pixels, old)) = base
        && only_placement_changed(old, new)
    {
        if old.box_() == new.box_() && old.flips() == new.flips() && old.warp() == new.warp() {
            return pixels.clone();
        }
        return render::transform_drawing(pixels, old, new, canvas);
    }
    render::render_object(new, canvas).unwrap_or_else(|| base.map(|(p, _)| p.clone()).unwrap_or_default())
}

/// Gives a layer a changed object and its new drawing (one call per change; the layer's old drawing is the base).
pub fn set_object(layer: &mut Layer, object: StoredObject, canvas: PixelRect) {
    let pixels = match layer.object() {
        Some(old) => drawing_for(&object, Some((layer.pixels(), old)), canvas),
        None => drawing_for(&object, None, canvas),
    };
    layer.set_pixels_with_object(pixels, Some(object));
}

/// A new layer holding an object, drawn.
pub fn object_layer(name: impl Into<String>, object: StoredObject, canvas: PixelRect) -> Layer {
    let mut layer = Layer::new(name);
    let pixels = drawing_for(&object, None, canvas);
    layer.set_pixels_with_object(pixels, Some(object));
    layer
}

/// A plain pixel layer as a placed picture of its visible part (C# `MakePictureObject` / `AsImage`). `None` when
/// it is empty or already an object.
pub fn as_picture(layer: &Layer) -> Option<StoredObject> {
    if layer.object().is_some() || layer.is_adjustment() {
        return None;
    }
    let bounds = layer.visible_bounds();
    if bounds.is_empty() {
        return None;
    }
    let pixels = layer.pixels().to_buffer(bounds);
    let b = ObjectBox::new(bounds.x as f64, bounds.y as f64, bounds.width as f64, bounds.height as f64, 0.0);
    Some(StoredObject::new_image(b, picture_bytes(&pixels), &layer.name))
}

/// Turns a pixel layer into a placed picture in place (its pixels stay exactly as they are). False when empty.
pub fn make_picture(doc: &mut Document, id: LayerId) -> bool {
    let Some(layer) = doc.layer_mut(id) else { return false };
    let Some(object) = as_picture(layer) else { return false };
    let pixels = layer.pixels().clone();
    layer.set_pixels_with_object(pixels, Some(object));
    true
}

/// Convert to Pixels: the object becomes plain pixels (its drawing stays).
pub fn convert_to_pixels(doc: &mut Document, id: LayerId) -> bool {
    let Some(layer) = doc.layer_mut(id).filter(|l| l.object().is_some()) else { return false };
    let pixels = layer.pixels().clone();
    layer.replace_pixels(pixels);
    true
}

/// Groups the visible layers among `ids` (pixel layers become pictures first). The group goes where the top one
/// was. Returns the group's id, or `None` when fewer than two layers can be grouped.
pub fn group(doc: &mut Document, ids: &[LayerId]) -> Option<LayerId> {
    let canvas = doc.bounds();
    let chosen: Vec<usize> = doc
        .layers()
        .iter()
        .enumerate()
        .filter(|(_, l)| ids.contains(&l.id()) && l.visible && !l.is_adjustment())
        .map(|(i, _)| i)
        .collect();
    let mut members = Vec::new();
    let mut drawing = TileMap::empty();
    for &i in &chosen {
        let layer = &doc.layers()[i];
        let Some(object) = layer.object().cloned().or_else(|| as_picture(layer)) else { continue };
        compositor::draw_layer_into_tiles(&mut drawing, layer);
        members.push(GroupMember { object, name: layer.name.clone(), opacity: layer.opacity() });
    }
    if members.len() < 2 {
        return None;
    }
    let group = StoredObject::new_group(&members)?;
    let mut layer = Layer::new(format!("Group ({})", members.len()));
    // A group the Rust version can draw is drawn; otherwise it keeps its parts' drawings exactly.
    let pixels = if group.can_draw() && !group.has_looks() {
        render::render_object(&group, canvas).unwrap_or(drawing)
    } else {
        drawing
    };
    layer.set_pixels_with_object(pixels, Some(group));
    let top = *chosen.last()?;
    let id = layer.id();
    let removed: Vec<LayerId> = chosen.iter().map(|&i| doc.layers()[i].id()).collect();
    let mut list: Vec<Layer> = doc.layers().to_vec();
    list.insert(top + 1, layer);
    list.retain(|l| !removed.contains(&l.id()));
    doc.set_layers(list, Some(id));
    Some(id)
}

/// Why a group cannot be taken apart yet.
#[derive(Debug, PartialEq, Eq)]
pub enum UngroupError {
    NotAGroup,
    /// A part is text, a chart or a code: its own drawing comes with Phases 9–10.
    CannotDrawPart,
}

/// Turns a group back into separate layers, exactly where its parts are now. Returns their ids.
pub fn ungroup(doc: &mut Document, id: LayerId) -> Result<Vec<LayerId>, UngroupError> {
    let canvas = doc.bounds();
    let index = doc.index_of(id).ok_or(UngroupError::NotAGroup)?;
    let group = doc.layers()[index].object().cloned().ok_or(UngroupError::NotAGroup)?;
    let members = group.current_members().ok_or(UngroupError::NotAGroup)?;
    if members.iter().any(|m| !m.object.can_draw()) {
        return Err(UngroupError::CannotDrawPart);
    }
    let mut new_layers = Vec::new();
    for m in members {
        let mut layer = object_layer(m.name, m.object, canvas);
        layer.set_opacity(m.opacity);
        new_layers.push(layer);
    }
    let ids: Vec<LayerId> = new_layers.iter().map(Layer::id).collect();
    let mut list: Vec<Layer> = doc.layers().to_vec();
    list.splice(index..=index, new_layers);
    let active = ids.last().copied();
    doc.set_layers(list, active);
    doc.set_selection(&ids, active);
    Ok(ids)
}

/// Mirrors a layer: an object turns over in place (it stays editable); plain pixels mirror across the canvas.
pub fn flip(doc: &mut Document, id: LayerId, horizontal: bool) -> bool {
    let canvas = doc.bounds();
    let Some(layer) = doc.layer_mut(id) else { return false };
    if let Some(mut object) = layer.object().cloned() {
        let (fx, fy) = object.flips();
        if horizontal {
            object.set_flips(!fx, fy);
        } else {
            object.set_flips(fx, !fy);
        }
        set_object(layer, object, canvas);
        return true;
    }
    let source = layer.pixels();
    let mut flipped = TileMap::empty();
    let bounds = source.tile_bounds();
    for y in bounds.y..bounds.bottom() {
        let row = source.read_rect(PixelRect::new(bounds.x, y, bounds.width, 1));
        let mut mirrored: Vec<u8> = Vec::with_capacity(row.len());
        for p in row.as_chunks::<4>().0.iter().rev() {
            mirrored.extend_from_slice(p);
        }
        if horizontal {
            let x = canvas.width - bounds.right();
            flipped.write_rect(PixelRect::new(x, y, bounds.width, 1), &mirrored);
        } else {
            flipped.write_rect(PixelRect::new(bounds.x, canvas.height - 1 - y, bounds.width, 1), &row);
        }
    }
    layer.replace_pixels(flipped);
    true
}

/// Where each layer is for aligning: an object's box area, or a pixel layer's visible area.
pub fn layer_area(layer: &Layer) -> PixelRect {
    match layer.object().and_then(StoredObject::box_) {
        Some(b) => b.bounds(0.0),
        None => layer.visible_bounds(),
    }
}

/// Which way to line up (C# `AlignKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignKind {
    Left,
    CenterHorizontally,
    Right,
    Top,
    CenterVertically,
    Bottom,
}

fn middle(a: i32, b: i32) -> i32 {
    ((a + b) as f64 / 2.0).round() as i32 // C# MidpointRounding.AwayFromZero
}

/// How far each box moves to line up: one box with the canvas, several with the area they cover together.
pub fn align_moves(boxes: &[PixelRect], canvas: PixelRect, kind: AlignKind) -> Vec<(i32, i32)> {
    let area = if boxes.len() == 1 { canvas } else { boxes.iter().fold(PixelRect::EMPTY, |a, b| a.union(b)) };
    boxes
        .iter()
        .map(|b| {
            if b.is_empty() {
                return (0, 0);
            }
            match kind {
                AlignKind::Left => (area.x - b.x, 0),
                AlignKind::Right => (area.right() - b.right(), 0),
                AlignKind::CenterHorizontally => (middle(area.x, area.right()) - middle(b.x, b.right()), 0),
                AlignKind::Top => (0, area.y - b.y),
                AlignKind::Bottom => (0, area.bottom() - b.bottom()),
                AlignKind::CenterVertically => (0, middle(area.y, area.bottom()) - middle(b.y, b.bottom())),
            }
        })
        .collect()
}

/// How far each box moves so the gaps between neighbours are equal (the outer two stay; needs three boxes).
pub fn distribute_moves(boxes: &[PixelRect], horizontal: bool) -> Vec<(i32, i32)> {
    let mut result = vec![(0, 0); boxes.len()];
    let mut order: Vec<usize> = (0..boxes.len()).filter(|&i| !boxes[i].is_empty()).collect();
    if order.len() < 3 {
        return result;
    }
    let start = |b: &PixelRect| if horizontal { b.x } else { b.y };
    let end = |b: &PixelRect| if horizontal { b.right() } else { b.bottom() };
    order.sort_by_key(|&i| start(&boxes[i]));
    let first = start(&boxes[order[0]]);
    let last = end(&boxes[*order.last().unwrap_or(&0)]);
    let total: i32 = order.iter().map(|&i| end(&boxes[i]) - start(&boxes[i])).sum();
    let gap = (last - first - total) as f64 / (order.len() - 1) as f64;
    let mut position = first as f64;
    for &i in &order {
        let size = end(&boxes[i]) - start(&boxes[i]);
        let mv = position.round_ties_even() as i32 - start(&boxes[i]);
        result[i] = if horizontal { (mv, 0) } else { (0, mv) };
        position += size as f64 + gap;
    }
    result
}

/// Moves a layer by whole pixels (objects stay objects).
pub fn move_layer(doc: &mut Document, id: LayerId, dx: i32, dy: i32) {
    if let Some(layer) = doc.layer_mut(id) {
        crate::paint::pixel_move::move_layer(layer, dx, dy);
    }
}
