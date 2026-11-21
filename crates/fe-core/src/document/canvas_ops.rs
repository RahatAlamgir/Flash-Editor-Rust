//! Changing the canvas frame: a new canvas size with every layer shifted (C# `CanvasChangeAction.Reframe`),
//! used by Reveal All now and by Canvas Size, Crop and Trim later.

use super::{Document, LayerMask, object_ops};
use crate::imaging::{PixelBuffer, PixelRect, TILE_SIZE, TileMap};
use crate::objects::StoredObject;

/// The largest canvas side the program allows (like the C# `ImageDecoder.MaxDimension`).
pub const MAX_SIDE: i32 = 16384;

/// The canvas plus every visible pixel of every layer (what Reveal All grows the canvas to).
pub fn content_bounds(doc: &Document) -> PixelRect {
    doc.layers().iter().fold(doc.bounds(), |all, layer| all.union(&layer.visible_bounds()))
}

/// `pixels` with `mask` baked into their alpha (pixels outside the mask's canvas stay as they are, like C#).
pub fn bake_mask(pixels: &TileMap, mask: &LayerMask) -> TileMap {
    let mut baked = pixels.clone();
    let canvas = mask.bounds();
    for key in pixels.keys().collect::<Vec<_>>() {
        let tr = key.rect();
        let values = mask.tile_values(key);
        if (values.is_none() && mask.fill_value() == 255) || canvas.intersect(&tr).is_empty() {
            continue; // fully shown here
        }
        let tile = baked.tile_mut(key);
        for (i, p) in tile.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (tr.x + i as i32 % TILE_SIZE, tr.y + i as i32 / TILE_SIZE);
            let m = if !canvas.contains(x, y) { 255 } else { values.map_or(mask.fill_value(), |v| v[i]) } as u32;
            p[3] = (p[3] as u32 * m / 255) as u8;
            if p[3] == 0 {
                p.copy_from_slice(&[0; 4]);
            }
        }
        baked.drop_if_fill(key);
    }
    baked
}

/// Gives the canvas a new size and moves every layer by (shift_x, shift_y). Masks are baked into the pixels and
/// removed (like in C#); objects move with their drawing. Drops the selected area.
pub fn reframe(doc: &mut Document, width: u32, height: u32, shift_x: i32, shift_y: i32) {
    for layer in doc.layers_mut() {
        let base = match (layer.effective_mask(), layer.object()) {
            (Some(mask), None) => bake_mask(layer.pixels(), mask),
            _ => layer.pixels().clone(),
        };
        let pixels = base.offset(shift_x, shift_y);
        match layer.object().cloned() {
            Some(object) => {
                layer.set_pixels_with_object(pixels, Some(object.offset(shift_x as f64, shift_y as f64)));
            }
            None => layer.replace_pixels(pixels),
        }
        layer.restore = layer.restore.as_ref().map(|r| r.offset(shift_x, shift_y));
        let _ = layer.set_mask(None, width, height);
    }
    doc.set_canvas_size(width, height);
}

/// How an editable object follows a change of the whole picture: it gets the object and the new canvas size.
pub type ObjectFollow<'a> = &'a dyn Fn(&StoredObject, u32, u32) -> StoredObject;

/// Changes every layer with `transform` (crop, resize, turn, flip) and gives the canvas the size of the results.
/// `object` says how editable objects follow (it gets the object and the new canvas size); without it objects are
/// turned into pixels. Masks are baked in. Adjustment layers stay. Drops the selected area.
pub fn transform_all(
    doc: &mut Document,
    transform: &dyn Fn(&PixelBuffer) -> PixelBuffer,
    object: Option<ObjectFollow>,
) {
    let canvas = doc.bounds();
    let mut done: Vec<Option<PixelBuffer>> = Vec::new();
    let mut size = None;
    for layer in doc.layers() {
        if layer.is_adjustment() || (layer.object().is_some() && object.is_some()) {
            done.push(None);
            continue;
        }
        let masked = match layer.effective_mask() {
            Some(mask) => bake_mask(layer.pixels(), mask),
            None => layer.pixels().clone(),
        };
        let out = transform(&masked.to_buffer(canvas));
        size.get_or_insert((out.width(), out.height()));
        done.push(Some(out));
    }
    let (w, h) = size.unwrap_or_else(|| {
        let t = transform(&PixelBuffer::new(canvas.width as u32, canvas.height as u32));
        (t.width(), t.height())
    });
    let new_canvas = PixelRect::new(0, 0, w as i32, h as i32);
    for (layer, out) in doc.layers_mut().iter_mut().zip(done) {
        if layer.is_adjustment() {
            continue;
        }
        match (out, layer.object().cloned(), object) {
            (Some(buffer), _, _) => {
                layer.replace_pixels(TileMap::from_buffer(&buffer, 0, 0));
            }
            (None, Some(o), Some(f)) => object_ops::set_object(layer, f(&o, w, h), new_canvas),
            _ => {}
        }
        layer.restore = None;
        let _ = layer.set_mask(None, w, h);
    }
    doc.set_canvas_size(w, h);
    doc.set_selected_area(None);
}
