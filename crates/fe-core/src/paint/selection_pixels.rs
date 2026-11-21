//! Copy, clear and fill the pixels inside a selection (C# `SelectionPixels`). Works tile by tile, so only the
//! tiles under the selection are touched.

use crate::imaging::{ColorRgba, PixelBuffer, PixelRect, TILE_SIZE, TileKey, TileMap, color_math::clamp_byte, over};
use crate::selection::Selection;

/// Calls `f(pixel, selection value)` for every selected pixel of `pixels`. With `create`, missing (transparent)
/// tiles are created first; otherwise they are skipped. Tiles left fully transparent are dropped.
fn for_each_selected(pixels: &mut TileMap, selection: &Selection, create: bool, mut f: impl FnMut(&mut [u8], u8)) {
    let bounds = selection.bounds();
    for key in TileKey::covering(bounds) {
        if !create && pixels.tile(key).is_none() {
            continue;
        }
        let tr = key.rect();
        let part = tr.intersect(&bounds);
        let any = (part.y..part.bottom()).any(|y| (part.x..part.right()).any(|x| selection.value(x, y) > 0));
        if !any {
            continue;
        }
        let tile = pixels.tile_mut(key);
        for y in part.y..part.bottom() {
            for x in part.x..part.right() {
                let v = selection.value(x, y);
                if v > 0 {
                    let i = (((y - tr.y) * TILE_SIZE + (x - tr.x)) * 4) as usize;
                    f(&mut tile[i..i + 4], v);
                }
            }
        }
        pixels.drop_if_fill(key);
    }
}

/// The selected pixels, cropped to the selection's bounds. Pixels outside the selection are transparent;
/// partly selected pixels keep part of their alpha.
pub fn extract(pixels: &TileMap, selection: &Selection) -> PixelBuffer {
    let bounds = selection.bounds();
    let mut result = pixels.to_buffer(bounds);
    let mask = selection.mask();
    for (p, m) in result.data_mut().as_chunks_mut::<4>().0.iter_mut().zip(mask) {
        if *m < 255 {
            p[3] = clamp_byte(p[3] as f64 * (*m as f64 / 255.0));
        }
    }
    result
}

/// Makes the selected pixels transparent. Returns the changed rectangle.
pub fn clear(pixels: &mut TileMap, selection: &Selection) -> PixelRect {
    for_each_selected(pixels, selection, false, |p, v| {
        p[3] = clamp_byte(p[3] as f64 * (1.0 - v as f64 / 255.0));
        if p[3] == 0 {
            p.copy_from_slice(&[0; 4]); // fully see-through: store nothing (lets empty tiles be dropped)
        }
    });
    selection.bounds()
}

/// Paints `color` over the selected pixels (partly selected pixels get less of it). Returns the changed rectangle.
pub fn fill(pixels: &mut TileMap, selection: &Selection, color: ColorRgba) -> PixelRect {
    let alpha = color.a as f64 / 255.0;
    for_each_selected(pixels, selection, true, |p, v| over(p, [color.r, color.g, color.b], alpha * v as f64 / 255.0));
    selection.bounds()
}

/// A copy of `pixels` with only the selected part left (the rest becomes transparent), like the C# "Split".
pub fn keep_only(pixels: &TileMap, selection: &Selection) -> TileMap {
    let mut kept = TileMap::empty();
    let bounds = selection.bounds();
    for key in pixels.keys().filter(|k| !k.rect().intersect(&bounds).is_empty()).collect::<Vec<_>>() {
        kept.set_tile(key, pixels.tile_arc(key).cloned());
    }
    for key in kept.keys().collect::<Vec<_>>() {
        let tr = key.rect();
        let tile = kept.tile_mut(key);
        for y in tr.y..tr.bottom() {
            for x in tr.x..tr.right() {
                let v = selection.value(x, y);
                if v == 255 {
                    continue;
                }
                let i = (((y - tr.y) * TILE_SIZE + (x - tr.x)) * 4) as usize;
                let p = &mut tile[i..i + 4];
                p[3] = clamp_byte(p[3] as f64 * (v as f64 / 255.0));
                if p[3] == 0 {
                    p.copy_from_slice(&[0; 4]);
                }
            }
        }
        kept.drop_if_fill(key);
    }
    kept
}
