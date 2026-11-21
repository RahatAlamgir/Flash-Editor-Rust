//! Moving pixels (C# `PixelMover`): the selected pixels of a layer, or a whole layer.
//!
//! Nothing is cut off at the canvas edge: pixels moved past it stay on the layer (hidden) and come back when
//! moved back. A selected piece is remembered between moves, so dragging a piece half off the canvas and back
//! again brings it back whole.

use super::selection_pixels;
use crate::document::{Layer, LayerId};
use crate::imaging::{TileKey, TileMap, over_pixel};
use crate::selection::Selection;

/// A selected piece of a layer that is being moved.
#[derive(Clone, Debug)]
pub struct FloatingPiece {
    pub layer: LayerId,
    /// The selection where the piece started.
    original: Selection,
    /// The piece itself, at its starting place (partly selected pixels keep part of their alpha).
    piece: TileMap,
    /// The layer without the piece.
    cleared: TileMap,
}

impl FloatingPiece {
    /// Lifts the selected pixels off a layer.
    pub fn lift(layer: &Layer, selection: &Selection) -> Self {
        let b = selection.bounds();
        let piece = TileMap::from_buffer(&selection_pixels::extract(layer.pixels(), selection), b.x, b.y);
        let mut cleared = layer.pixels().clone();
        selection_pixels::clear(&mut cleared, selection);
        Self { layer: layer.id(), original: selection.clone(), piece, cleared }
    }

    /// The layer's pixels with the piece moved by (dx, dy) from where it started.
    pub fn pixels_at(&self, dx: i32, dy: i32) -> TileMap {
        let mut result = self.cleared.clone();
        draw_over(&mut result, &self.piece, dx, dy);
        result
    }

    /// The selection moved along with the piece (`None` when it left the canvas completely).
    pub fn selection_at(&self, dx: i32, dy: i32) -> Option<Selection> {
        self.original.offset(dx, dy)
    }

    /// The selection where the piece started.
    pub fn original(&self) -> &Selection {
        &self.original
    }
}

/// Paints `src`, moved by (dx, dy), over `target`.
pub fn draw_over(target: &mut TileMap, src: &TileMap, dx: i32, dy: i32) {
    for key in src.keys().collect::<Vec<TileKey>>() {
        let Some(tile) = src.tile(key) else { continue };
        let rect = key.rect().offset(dx, dy);
        if TileKey::covering(rect).all(|k| target.tile(k).is_none()) {
            target.write_rect(rect, tile);
            continue;
        }
        let mut dest = target.read_rect(rect);
        for (d, s) in dest.as_chunks_mut::<4>().0.iter_mut().zip(tile.as_chunks::<4>().0) {
            if s[3] == 255 {
                d.copy_from_slice(s);
            } else if s[3] > 0 {
                over_pixel(d, s, 1.0);
            }
        }
        target.write_rect(rect, &dest);
    }
}

/// Moves a whole layer by (dx, dy). An object layer stays an object (its box moves with its drawing).
pub fn move_layer(layer: &mut Layer, dx: i32, dy: i32) {
    let pixels = layer.pixels().offset(dx, dy);
    match layer.object().cloned() {
        Some(object) => layer.set_pixels_with_object(pixels, Some(object.offset(dx as f64, dy as f64))),
        None => layer.replace_pixels(pixels),
    }
}
