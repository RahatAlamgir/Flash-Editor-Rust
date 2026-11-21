//! Crops a single layer instead of the whole canvas (C# `LayerCrop`, `CropTool.Apply`, `PerspectiveCropTool.Apply`).
//! The result is a new picture layer, so the cropped piece can still be moved and resized; the original layer is
//! kept (the tools hide it).

use super::canvas_ops::bake_mask;
use super::{Layer, object_ops};
use crate::imaging::{PixelBuffer, PixelRect, PointD};
use crate::objects::{ObjectBox, StoredObject, picture_bytes, pictures};
use crate::transform::{perspective_crop, suggested_size};

/// The layer cut down to `rect` (canvas coordinates) as a new layer named "<name> (cropped)"; `None` when nothing of
/// the layer is inside the rectangle.
pub fn crop_layer(layer: &Layer, rect: PixelRect, canvas: PixelRect) -> Option<Layer> {
    let name = format!("{} (cropped)", layer.name);
    let object = upright_picture_crop(layer, rect).or_else(|| {
        // Anything else: what the layer shows (also parts outside the canvas) inside the rectangle.
        let content = layer.pixels().to_buffer(rect);
        as_image(&content, rect, &layer.name)
    })?;
    Some(object_ops::object_layer(name, object, canvas))
}

/// An upright, unflipped picture is cropped from its original pixels: no quality loss.
fn upright_picture_crop(layer: &Layer, rect: PixelRect) -> Option<StoredObject> {
    let object = layer.object().filter(|o| o.kind() == "image")?;
    let b = object.box_().filter(|b| b.rotation == 0.0)?;
    if object.flips() != (false, false) || object.warp().is_some() {
        return None;
    }
    let (left, top) = (f64::from(rect.x).max(b.x), f64::from(rect.y).max(b.y));
    let (right, bottom) =
        (f64::from(rect.x + rect.width).min(b.x + b.width), f64::from(rect.y + rect.height).min(b.y + b.height));
    if right - left < 1.0 || bottom - top < 1.0 {
        return None;
    }
    let source = pictures::decoded(object.picture()?)?;
    let (sx, sy) = (f64::from(source.width()) / b.width, f64::from(source.height()) / b.height);
    let from = PixelRect::from_bounds((left - b.x) * sx, (top - b.y) * sy, (right - b.x) * sx, (bottom - b.y) * sy)
        .intersect(&source.bounds());
    if from.is_empty() {
        return None;
    }
    let piece = source.crop(from);
    Some(StoredObject::new_image(
        ObjectBox::new(left, top, right - left, bottom - top, 0.0),
        picture_bytes(&piece),
        object.image_name(),
    ))
}

/// The piece as a resizable picture, trimmed to its visible pixels.
fn as_image(piece: &PixelBuffer, at: PixelRect, name: &str) -> Option<StoredObject> {
    let visible = piece.visible_bounds();
    if visible.is_empty() {
        return None;
    }
    let trimmed = piece.crop(visible);
    let b = ObjectBox::new(
        f64::from(at.x + visible.x),
        f64::from(at.y + visible.y),
        f64::from(visible.width),
        f64::from(visible.height),
        0.0,
    );
    Some(StoredObject::new_image(b, picture_bytes(&trimmed), name))
}

/// What Perspective Crop made: a new layer holding the flattened area, and its size.
pub struct Straightened {
    pub layer: Layer,
    pub size: (u32, u32),
}

/// The four-sided area `quad` of the layer as a flat picture on a new layer, shrunk to fit the canvas and centered on
/// the area it came from. `None` when the four points do not make a usable shape.
pub fn perspective_layer(layer: &Layer, quad: &[PointD], canvas: PixelRect) -> Option<Straightened> {
    let (w, h) = suggested_size(quad);
    let (w, h) = (w.clamp(1, 12000), h.clamp(1, 12000));
    let masked = match layer.effective_mask() {
        Some(mask) if layer.object().is_none() => bake_mask(layer.pixels(), mask),
        _ => layer.pixels().clone(),
    };
    let flat = perspective_crop(&masked.to_buffer(canvas), quad, w, h)?;
    let (cw, ch) = (f64::from(canvas.width), f64::from(canvas.height));
    let scale = 1f64.min((cw / f64::from(w)).min(ch / f64::from(h)));
    let (bw, bh) = (f64::from(w) * scale, f64::from(h) * scale);
    let cx = quad.iter().map(|p| p.x).sum::<f64>() / 4.0;
    let cy = quad.iter().map(|p| p.y).sum::<f64>() / 4.0;
    let bx = (cx - bw / 2.0).clamp(0.0, (cw - bw).max(0.0));
    let by = (cy - bh / 2.0).clamp(0.0, (ch - bh).max(0.0));
    let name = format!("{} (straightened)", layer.name);
    let object = StoredObject::new_image(ObjectBox::new(bx, by, bw, bh, 0.0), picture_bytes(&flat), &name);
    Some(Straightened { layer: object_ops::object_layer(name, object, canvas), size: (w, h) })
}

/// The layer keeping only the selected pixels (any selection shape, soft edges kept) as a resizable picture.
/// `None` when nothing of the layer is selected.
pub fn selection_object(
    layer: &Layer,
    selection: &crate::selection::Selection,
    canvas: PixelRect,
) -> Option<StoredObject> {
    let bounds = selection.bounds().intersect(&canvas);
    if bounds.is_empty() {
        return None;
    }
    let mut piece = layer.pixels().to_buffer(bounds);
    for y in 0..bounds.height {
        for x in 0..bounds.width {
            let i = piece.index_of(x as u32, y as u32) + 3;
            let a = f64::from(piece.data()[i]) * selection.coverage(bounds.x + x, bounds.y + y);
            piece.data_mut()[i] = crate::imaging::color_math::clamp_byte(a);
        }
    }
    as_image(&piece, bounds, &layer.name)
}
