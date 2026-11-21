//! The reference CPU compositor: combines all visible layers (opacity, blend modes, masks, adjustment layers)
//! into one picture. Used for export, thumbnails, color picking, tests and the no-GPU fallback.
//! The GPU canvas (Phase 3) must give the same result.

use rayon::prelude::*;

use crate::document::{Document, Layer, LayerMask};
use crate::imaging::{BlendMode, ColorRgba, PixelBuffer, PixelRect, TILE_SIZE, TileKey, TileMap, blend_pixel};

/// The whole canvas as one picture. Bands of rows are combined in parallel.
pub fn flatten(doc: &Document) -> PixelBuffer {
    let mut result = PixelBuffer::new(doc.width(), doc.height());
    let width = doc.width() as i32;
    let stride = result.stride();
    let band_rows = 256usize;
    result.data_mut().par_chunks_mut(stride * band_rows).enumerate().for_each(|(band, out)| {
        let y = (band * band_rows) as i32;
        let rows = (out.len() / stride) as i32;
        composite_into(doc, PixelRect::new(0, y, width, rows), out, stride);
    });
    result
}

/// The combined picture of one area (clipped to the canvas; an area fully outside gives `None`).
/// Large areas are combined in parallel bands.
pub fn composite_region(doc: &Document, rect: PixelRect) -> Option<PixelBuffer> {
    let rect = rect.intersect(&doc.bounds());
    if rect.is_empty() {
        return None;
    }
    let mut result = PixelBuffer::new(rect.width as u32, rect.height as u32);
    let stride = result.stride();
    let band_rows = 64usize;
    result.data_mut().par_chunks_mut(stride * band_rows).enumerate().for_each(|(band, out)| {
        let rows = (out.len() / stride) as i32;
        let band_rect = PixelRect::new(rect.x, rect.y + band as i32 * band_rows as i32, rect.width, rows);
        composite_into(doc, band_rect, out, stride);
    });
    Some(result)
}

/// Redraws one area of `target` (a canvas-sized picture) from the layers; the rest stays as it is.
pub fn flatten_region(doc: &Document, region: PixelRect, target: &mut PixelBuffer) {
    let region = region.intersect(&target.bounds());
    if region.is_empty() {
        return;
    }
    target.clear_region(region);
    let stride = target.stride();
    let start = target.index_of(region.x as u32, region.y as u32);
    composite_into(doc, region, &mut target.data_mut()[start..], stride);
}

/// The combined color of all layers at one canvas pixel (eyedropper "all layers").
pub fn sample_merged(doc: &Document, x: i32, y: i32) -> ColorRgba {
    if !doc.bounds().contains(x, y) {
        return ColorRgba::TRANSPARENT;
    }
    let mut pixel = [0u8; 4];
    composite_into(doc, PixelRect::new(x, y, 1, 1), &mut pixel, 4);
    ColorRgba::from_slice(&pixel)
}

/// The combined color of all layers at one canvas pixel, as RGBA bytes (no bounds check; for previews).
pub fn composite_pixel(doc: &Document, x: i32, y: i32) -> [u8; 4] {
    let mut pixel = [0u8; 4];
    composite_into(doc, PixelRect::new(x, y, 1, 1), &mut pixel, 4);
    pixel
}

/// Draws one layer onto a canvas-sized picture (what Merge Down does).
pub fn draw_layer(target: &mut PixelBuffer, layer: &Layer) {
    let bounds = target.bounds();
    if let Some(adjustment) = &layer.adjustment {
        adjustment.apply_buffer(target, bounds, layer.opacity());
        return;
    }
    let stride = target.stride();
    draw_tiles(target.data_mut(), stride, bounds, layer.pixels(), layer.opacity(), layer.blend, layer.effective_mask());
}

/// Draws one layer into a tile map (Merge Down, Merge Layers, Flatten). Works tile by tile, so it never needs
/// a canvas-sized buffer: a pixel layer touches only its own tiles; an adjustment layer changes every tile of
/// `target`.
pub fn draw_layer_into_tiles(target: &mut TileMap, layer: &Layer) {
    if !layer.visible || layer.opacity() <= 0.0 {
        return;
    }
    let stride = TILE_SIZE as usize * 4;
    if let Some(adjustment) = &layer.adjustment {
        let tones = adjustment.tone_tables();
        for key in target.keys().collect::<Vec<_>>() {
            let tile = target.tile_mut(key);
            for row in tile.chunks_exact_mut(stride) {
                adjustment.apply_row(row, layer.opacity(), tones.as_ref());
            }
        }
        return;
    }
    for key in layer.pixels().keys().collect::<Vec<_>>() {
        let tile = target.tile_mut(key);
        draw_tiles(tile, stride, key.rect(), layer.pixels(), layer.opacity(), layer.blend, layer.effective_mask());
    }
}

/// Combines every visible layer for `rect` into `out`, which starts transparent (or holds what the caller
/// cleared). `out[0]` is the top-left pixel of `rect`; rows are `stride` bytes apart.
fn composite_into(doc: &Document, rect: PixelRect, out: &mut [u8], stride: usize) {
    for layer in doc.layers().iter().filter(|l| l.visible && l.opacity() > 0.0) {
        match &layer.adjustment {
            Some(adjustment) => {
                if adjustment.is_neutral() {
                    continue;
                }
                let tones = adjustment.tone_tables();
                for row in 0..rect.height as usize {
                    let start = row * stride;
                    let pixels = &mut out[start..start + rect.width as usize * 4];
                    adjustment.apply_row(pixels, layer.opacity(), tones.as_ref());
                }
            }
            None => draw_tiles(out, stride, rect, layer.pixels(), layer.opacity(), layer.blend, layer.effective_mask()),
        }
    }
}

/// Draws the part of `source` inside `rect` onto `out` (see [`composite_into`] for the layout).
fn draw_tiles(
    out: &mut [u8],
    stride: usize,
    rect: PixelRect,
    source: &TileMap,
    opacity: f64,
    mode: BlendMode,
    mask: Option<&LayerMask>,
) {
    let plain = mask.is_none() && opacity >= 1.0 && mode == BlendMode::Normal;
    for key in TileKey::covering(rect) {
        let Some(tile) = source.tile(key) else { continue };
        let part = key.rect().intersect(&rect);
        let tile_rect = key.rect();
        let mask_tile = mask.map(|m| (m, m.tile_values(key)));
        for y in part.y..part.bottom() {
            let ty = (y - tile_rect.y) as usize;
            let out_row = (y - rect.y) as usize * stride;
            for x in part.x..part.right() {
                let tx = (x - tile_rect.x) as usize;
                let si = (ty * TILE_SIZE as usize + tx) * 4;
                let src = &tile[si..si + 4];
                if src[3] == 0 {
                    continue;
                }
                let di = out_row + (x - rect.x) as usize * 4;
                let dst = &mut out[di..di + 4];
                if plain && src[3] == 255 {
                    dst.copy_from_slice(src);
                    continue;
                }
                let mut factor = opacity;
                if let Some((m, values)) = mask_tile {
                    let coverage = if !m.bounds().contains(x, y) {
                        0
                    } else {
                        values.map_or(m.fill_value(), |v| v[ty * TILE_SIZE as usize + tx])
                    };
                    factor *= coverage as f64 / 255.0;
                    if factor <= 0.0 {
                        continue;
                    }
                }
                blend_pixel(dst, src, factor, mode);
            }
        }
    }
}
