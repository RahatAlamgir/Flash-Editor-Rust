//! Copy / clear / fill inside a selection (C# `SelectionTests.Extract_and_clear_selected_pixels`), moving pixels
//! and whole layers, and Reveal All.

mod common;

use common::*;
use fe_core::document::{Layer, canvas_ops};
use fe_core::imaging::{ColorRgba, PixelRect, TileMap};
use fe_core::paint::pixel_move::{FloatingPiece, move_layer};
use fe_core::paint::selection_pixels;
use fe_core::selection::Selection;

#[test]
fn extract_and_clear_selected_pixels() {
    let mut pixels = TileMap::from_buffer(&solid(6, 6, ColorRgba::BLACK), 0, 0);
    let selection = Selection::from_rect(PixelRect::new(1, 1, 2, 2), 6, 6).unwrap();

    let copy = selection_pixels::extract(&pixels, &selection);
    assert_eq!((2, 2), (copy.width(), copy.height()));
    assert_eq!(ColorRgba::BLACK, copy.pixel(0, 0));

    selection_pixels::clear(&mut pixels, &selection);
    assert_eq!(0, pixels.pixel(1, 1).a);
    assert_eq!(255, pixels.pixel(0, 0).a);
}

#[test]
fn partly_selected_pixels_are_partly_copied_and_cleared() {
    let mut pixels = TileMap::from_buffer(&solid(4, 4, ColorRgba::BLACK), 0, 0);
    let selection = Selection::from_mask(vec![255, 128], PixelRect::new(0, 0, 2, 1), 4, 4).unwrap();
    let copy = selection_pixels::extract(&pixels, &selection);
    assert_eq!(128, copy.pixel(1, 0).a);
    selection_pixels::clear(&mut pixels, &selection);
    assert_eq!(0, pixels.pixel(0, 0).a);
    assert!(in_range(pixels.pixel(1, 0).a, 126, 128));
}

#[test]
fn fill_paints_only_the_selection_and_clearing_frees_tiles() {
    let mut pixels = TileMap::empty();
    let selection = Selection::from_rect(PixelRect::new(70, 0, 10, 10), 200, 200).unwrap();
    selection_pixels::fill(&mut pixels, &selection, ColorRgba::rgb(0, 128, 0));
    assert_eq!(ColorRgba::rgb(0, 128, 0), pixels.pixel(75, 5));
    assert_eq!(0, pixels.pixel(60, 5).a);
    assert_eq!(1, pixels.tile_count());
    selection_pixels::clear(&mut pixels, &selection);
    assert_eq!(0, pixels.tile_count());
}

#[test]
fn a_moved_piece_comes_back_whole_from_past_the_edge() {
    let mut layer = Layer::from_buffer("L", &numbered(20, 20));
    let selection = Selection::from_rect(PixelRect::new(2, 2, 4, 4), 20, 20).unwrap();
    let before = layer.pixels().pixel(3, 3);
    let piece = FloatingPiece::lift(&layer, &selection);

    // Moved far past the right edge: the hole stays, the piece is kept off the canvas.
    layer.replace_pixels(piece.pixels_at(30, 0));
    assert_eq!(0, layer.pixels().pixel(3, 3).a);
    assert_eq!(before, layer.pixels().pixel(33, 3));
    assert!(piece.selection_at(30, 0).is_none());

    // And back: whole again.
    layer.replace_pixels(piece.pixels_at(1, 1));
    assert_eq!(before, layer.pixels().pixel(4, 4));
    assert_eq!(PixelRect::new(3, 3, 4, 4), piece.selection_at(1, 1).unwrap().bounds());
}

#[test]
fn moving_a_whole_layer_keeps_hidden_pixels() {
    let mut layer = Layer::from_buffer("L", &numbered(10, 10));
    let corner = layer.pixels().pixel(9, 9);
    move_layer(&mut layer, 5, 0);
    assert_eq!(corner, layer.pixels().pixel(14, 9));
    assert!(layer.has_hidden_pixels(PixelRect::new(0, 0, 10, 10)));
    move_layer(&mut layer, -5, 0);
    assert_eq!(corner, layer.pixels().pixel(9, 9));
}

#[test]
fn split_keeps_only_the_selected_part() {
    let pixels = TileMap::from_buffer(&solid(10, 10, ColorRgba::WHITE), 0, 0);
    let selection = Selection::from_rect(PixelRect::new(0, 0, 3, 3), 10, 10).unwrap();
    let kept = selection_pixels::keep_only(&pixels, &selection);
    assert_eq!(255, kept.pixel(1, 1).a);
    assert_eq!(0, kept.pixel(5, 5).a);
}

#[test]
fn reveal_all_grows_the_canvas_to_hidden_pixels() {
    let mut layer = Layer::from_buffer("L", &solid(10, 10, ColorRgba::WHITE));
    move_layer(&mut layer, -4, 3);
    let mut doc = doc_with(10, 10, vec![layer]);
    let all = canvas_ops::content_bounds(&doc);
    assert_eq!(PixelRect::new(-4, 0, 14, 13), all);
    canvas_ops::reframe(&mut doc, all.width as u32, all.height as u32, -all.x, -all.y);
    assert_eq!((14, 13), (doc.width(), doc.height()));
    assert_eq!(255, doc.layers()[0].pixels().pixel(0, 3).a);
    assert_eq!(0, doc.layers()[0].pixels().pixel(0, 0).a);
}

#[test]
fn moving_an_object_layer_moves_its_box_and_keeps_it_an_object() {
    use fe_core::objects::StoredObject;
    let json =
        serde_json::json!({"kind": "shape", "Box": {"X": 10, "Y": 20.5, "Width": 5, "Height": 5, "Rotation": 0}});
    let mut layer = Layer::from_buffer("Shape", &solid(4, 4, ColorRgba::WHITE));
    let pixels = layer.pixels().clone();
    layer.set_pixels_with_object(pixels, Some(StoredObject { json, images: Default::default() }));
    move_layer(&mut layer, 3, -2);
    let object = layer.object().expect("still an object");
    assert_eq!(Some((13.0, 18.5, 5.0, 5.0)), object.bounds());
    assert_eq!("13", object.json["Box"]["X"].to_string()); // whole numbers stay whole, like C# writes them
    assert_eq!(255, layer.pixels().pixel(3, 0).a);
}
