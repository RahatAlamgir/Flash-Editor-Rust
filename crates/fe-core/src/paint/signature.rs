//! A fingerprint of everything that decides how one area of the canvas looks. Two equal fingerprints mean the
//! combined picture of that area is the same, so a cached drawing of it can be reused. Cheap: it reads tile
//! change stamps and layer settings, never pixels.

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::document::{AdjustmentStyle, CurvePoint, Document};
use crate::imaging::{PixelRect, TileKey};

/// The fingerprint of the combined picture inside `rect` (canvas pixels).
pub fn region_signature(doc: &Document, rect: PixelRect) -> u64 {
    let mut h = DefaultHasher::new();
    (doc.width(), doc.height()).hash(&mut h);
    rect.hash(&mut h);
    let keys: Vec<TileKey> = TileKey::covering(rect.intersect(&doc.bounds())).collect();
    for layer in doc.layers().iter().filter(|l| l.visible && l.opacity() > 0.0) {
        layer.id().hash(&mut h);
        layer.opacity().to_bits().hash(&mut h);
        (layer.blend as u8).hash(&mut h);
        if let Some(adjustment) = &layer.adjustment {
            hash_adjustment(adjustment, &mut h);
            continue;
        }
        for key in &keys {
            layer.pixels().tile_stamp(*key).hash(&mut h);
        }
        if let Some(mask) = layer.effective_mask() {
            mask.fill_value().hash(&mut h);
            for key in &keys {
                mask.tiles().tile_stamp(*key).hash(&mut h);
            }
        }
    }
    h.finish()
}

/// The fingerprint of one layer's own pixels and mask (for its thumbnail).
pub fn layer_signature(layer: &crate::document::Layer, canvas: PixelRect) -> u64 {
    let mut h = DefaultHasher::new();
    canvas.hash(&mut h);
    let mut keys: Vec<TileKey> = layer.pixels().keys().collect();
    keys.sort();
    for key in keys {
        (key, layer.pixels().tile_stamp(key)).hash(&mut h);
    }
    if let Some(adjustment) = &layer.adjustment {
        hash_adjustment(adjustment, &mut h);
    }
    h.finish()
}

/// The fingerprint of a whole page (for its thumbnail in the page strip).
pub fn page_signature(doc: &Document) -> u64 {
    region_signature(doc, doc.bounds())
}

fn hash_adjustment(a: &AdjustmentStyle, h: &mut DefaultHasher) {
    let numbers = [
        a.brightness,
        a.contrast,
        a.saturation,
        a.hue,
        a.levels_black,
        a.levels_white,
        a.levels_gamma,
        a.output_black,
        a.output_white,
        a.temperature,
        a.tint,
    ];
    numbers.iter().for_each(|n| n.to_bits().hash(h));
    for curve in [&a.curve, &a.curve_red, &a.curve_green, &a.curve_blue] {
        curve.as_deref().map(|points| points.iter().map(point_bits).collect::<Vec<_>>()).hash(h);
    }
}

fn point_bits(p: &CurvePoint) -> (u64, u64) {
    (p.x.to_bits(), p.y.to_bits())
}
