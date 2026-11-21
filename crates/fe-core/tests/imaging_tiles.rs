//! The sparse tile storage: memory use, sharing (copy on write) and pixels kept past the canvas edge.
//! Also covers the C# `OffCanvasTests` ideas (there is no separate "extended" buffer any more).

mod common;

use common::*;
use fe_core::document::{Document, Layer};
use fe_core::imaging::{ColorRgba, PixelRect, TILE_SIZE, TileGrid, TileMap};

const RED: ColorRgba = ColorRgba::rgb(255, 0, 0);

#[test]
fn an_empty_4000_x_3000_layer_uses_no_tile_memory() {
    let mut doc = Document::new(4000, 3000);
    doc.push_layer(Layer::new("Empty"));
    assert_eq!(0, doc.memory_bytes());
    // A small dot costs one tile (256 KB), not a whole 48 MB canvas.
    doc.layers_mut()[0].pixels_mut().fill_rect(PixelRect::new(100, 100, 10, 10), RED);
    assert_eq!(1, doc.layers()[0].pixels().tile_count());
    assert_eq!(TileMap::TILE_BYTES, doc.memory_bytes());
}

#[test]
fn a_full_white_background_needs_one_tile_per_256_px_square() {
    let doc = Document::blank(4000, 3000, ColorRgba::WHITE);
    let tiles = doc.layers()[0].pixels().tile_count();
    let per_side = |n: i32| (n + TILE_SIZE - 1) / TILE_SIZE;
    assert_eq!((per_side(4000) * per_side(3000)) as usize, tiles);
    assert_eq!(ColorRgba::WHITE, doc.layers()[0].pixels().pixel(3999, 2999));
    assert_eq!(0, doc.layers()[0].pixels().pixel(4000, 0).a); // outside the canvas: nothing
}

#[test]
fn a_duplicated_layer_shares_its_tiles_until_one_side_paints() {
    let original = solid_layer("A", 600, 600, RED);
    let mut copy = original.duplicate("B");
    assert!(copy.pixels().shares_all_tiles_with(original.pixels()));
    copy.pixels_mut().set_pixel(5, 5, ColorRgba::BLACK);
    assert!(!copy.pixels().shares_all_tiles_with(original.pixels()));
    assert_eq!(RED, original.pixels().pixel(5, 5)); // the original is untouched
    assert_eq!(ColorRgba::BLACK, copy.pixels().pixel(5, 5));
    // Only the painted tile was copied; the other tiles are still shared.
    let shared = original
        .pixels()
        .keys()
        .filter(|k| match (original.pixels().tile_arc(*k), copy.pixels().tile_arc(*k)) {
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => false,
        })
        .count();
    assert_eq!(original.pixels().tile_count() - 1, shared);
}

#[test]
fn pixels_past_the_canvas_edge_are_kept() {
    let mut map = TileMap::from_buffer(&solid(10, 10, RED), 6, 0); // 4 px on a 10 px canvas, 6 px past the edge
    assert_eq!(RED, map.pixel(9, 5));
    assert_eq!(RED, map.pixel(15, 5)); // hidden part still there
    assert_eq!(0, map.pixel(5, 5).a);
    assert_eq!(PixelRect::new(6, 0, 10, 10), map.visible_bounds());
    // Negative positions work too.
    map.set_pixel(-300, -2, ColorRgba::BLACK);
    assert_eq!(ColorRgba::BLACK, map.pixel(-300, -2));
}

#[test]
fn moving_out_and_back_restores_every_pixel() {
    let original = numbered(12, 8);
    let map = TileMap::from_buffer(&original, 0, 0);
    let back = map.offset(9, 0).offset(-9, 0);
    assert_eq!(original, back.to_buffer(PixelRect::new(0, 0, 12, 8)));
    // Tile-aligned moves only re-label tiles.
    let far = map.offset(TILE_SIZE * 2, -TILE_SIZE);
    assert_eq!(original.pixel(3, 4), far.pixel(3 + TILE_SIZE * 2, 4 - TILE_SIZE));
}

#[test]
fn erasing_frees_the_tile() {
    let mut map = TileMap::empty();
    map.fill_rect(PixelRect::new(0, 0, 50, 50), RED);
    assert_eq!(1, map.tile_count());
    map.clear_rect(PixelRect::new(0, 0, 50, 50));
    assert_eq!(0, map.tile_count());
}

#[test]
fn reading_and_writing_rectangles_across_tile_borders() {
    let picture = numbered(300, 270);
    let map = TileMap::from_buffer(&picture, -20, 250);
    let rect = PixelRect::new(-20, 250, 300, 270);
    assert_eq!(picture, map.to_buffer(rect));
    assert_eq!(picture.pixel(280, 10), map.pixel(260, 260));
}

#[test]
fn a_one_byte_grid_reads_its_fill_value_where_nothing_is_stored() {
    let mut grid: TileGrid<1> = TileGrid::new([255]);
    assert_eq!([255], grid.get(10_000, -5));
    grid.set(3, 3, [7]);
    assert_eq!([7], grid.get(3, 3));
    assert_eq!(1, grid.tile_count());
}
