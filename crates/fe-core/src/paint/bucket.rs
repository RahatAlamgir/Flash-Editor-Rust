//! Paint bucket (C# `FloodFill.Fill`): fills the area of similar color around a pixel.

use crate::imaging::{ColorRgba, PixelRect, TILE_SIZE, TileKey, TileMap, over};
use crate::selection::{Selection, magic_wand};

/// Fills the pixels similar to the one at `at` with `color`, inside the canvas `canvas`. `tolerance` 0..255;
/// `contiguous`: only the connected area. A selection limits the fill (and a click outside it fills nothing
/// when `contiguous`). Returns the changed rectangle.
pub fn fill(
    pixels: &mut TileMap,
    canvas: PixelRect,
    at: (i32, i32),
    color: ColorRgba,
    tolerance: u8,
    contiguous: bool,
    selection: Option<&Selection>,
) -> PixelRect {
    let (x, y) = at;
    if selection.is_some_and(|s| !s.is_selected(x, y)) && contiguous {
        return PixelRect::EMPTY;
    }
    let image = pixels.to_buffer(canvas);
    let mask = magic_wand::similar_area(&image, x - canvas.x, y - canvas.y, tolerance, contiguous);
    let w = canvas.width;
    let alpha = color.a as f64 / 255.0;
    let mut changed = PixelRect::EMPTY;
    for key in TileKey::covering(canvas) {
        let tr = key.rect();
        let part = tr.intersect(&canvas);
        let hit = |px: i32, py: i32| mask[((py - canvas.y) * w + (px - canvas.x)) as usize] != 0;
        if !(part.y..part.bottom()).any(|py| (part.x..part.right()).any(|px| hit(px, py))) {
            continue;
        }
        let tile = pixels.tile_mut(key);
        for py in part.y..part.bottom() {
            for px in part.x..part.right() {
                if !hit(px, py) {
                    continue;
                }
                let strength = selection.map_or(1.0, |s| s.coverage(px, py));
                if strength <= 0.0 {
                    continue;
                }
                let i = (((py - tr.y) * TILE_SIZE + (px - tr.x)) * 4) as usize;
                let dst = &mut tile[i..i + 4];
                if strength >= 1.0 && color.a == 255 {
                    dst.copy_from_slice(&color.to_array());
                } else {
                    over(dst, [color.r, color.g, color.b], alpha * strength);
                }
            }
        }
        changed = changed.union(&part);
        pixels.drop_if_fill(key);
    }
    changed
}
