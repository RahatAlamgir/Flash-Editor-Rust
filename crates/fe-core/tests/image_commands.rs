//! Phase 11: whole-picture commands (resize, turn, flip), cropping a layer, effects on a selection, and the color
//! adjustment of an object.

use fe_core::document::canvas_ops::transform_all;
use fe_core::document::{Document, layer_crop, object_ops};
use fe_core::effects;
use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject, transforms};
use fe_core::selection::Selection;
use fe_core::transform::{self, ResampleMode};

fn shape_layer(doc: &Document, x: f64, y: f64, w: f64, h: f64) -> fe_core::document::Layer {
    let s = ShapeData::new(ShapeKind::Rectangle, Some(ColorRgba::rgb(200, 30, 30)), None, 0.0);
    object_ops::object_layer("Box", StoredObject::new_shape(ObjectBox::new(x, y, w, h, 0.0), &s), doc.bounds())
}

fn document() -> Document {
    let mut doc = Document::blank(100, 80, ColorRgba::WHITE);
    let layer = shape_layer(&doc, 10.0, 20.0, 30.0, 20.0);
    doc.push_layer(layer);
    doc
}

#[test]
fn resizing_scales_pixels_and_objects_together() {
    let mut doc = document();
    transform_all(
        &mut doc,
        &|p: &PixelBuffer| transform::resize(p, 50, 40, ResampleMode::Smooth),
        Some(&|o, _, _| o.scaled(0.5, 0.5)),
    );
    assert_eq!((doc.width(), doc.height()), (50, 40));
    let b = doc.layers()[1].object().and_then(StoredObject::box_).expect("still an object");
    assert_eq!((b.x, b.y, b.width, b.height), (5.0, 10.0, 15.0, 10.0));
    assert!(doc.layers()[1].pixels().pixel(10, 15).a > 200, "the drawing moved along");
}

#[test]
fn turning_by_90_swaps_the_sides_and_carries_objects_round() {
    let mut doc = document();
    transform_all(
        &mut doc,
        &transform::rotate_90_cw,
        Some(&|o, w, h| transforms::rotate(o, 90.0, (100.0, 80.0), (f64::from(w), f64::from(h)))),
    );
    assert_eq!((doc.width(), doc.height()), (80, 100));
    let o = doc.layers()[1].object().expect("object");
    let b = o.box_().unwrap();
    // The box centre (25, 30) of the 100 x 80 canvas goes to (80 - 30, 25) = (50, 25) of the 80 x 100 canvas.
    assert!((b.center().x - 50.0).abs() < 1e-6 && (b.center().y - 25.0).abs() < 1e-6, "{:?}", b.center());
    assert!((b.rotation - 90.0).abs() < 1e-6);
    // What the object draws is what the turned pixels show: the centre of the box is still red.
    assert!(doc.layers()[1].pixels().pixel(50, 25).r > 150);
}

#[test]
fn flipping_mirrors_an_object_in_place() {
    let o = document().layers()[1].object().cloned().unwrap();
    let f = transforms::flip_horizontal(&o, 100.0);
    assert_eq!(f.box_().unwrap().x, 100.0 - 10.0 - 30.0);
    assert_eq!(f.flips(), (true, false));
    assert_eq!(transforms::flip_vertical(&o, 80.0).box_().unwrap().y, 80.0 - 20.0 - 20.0);
}

#[test]
fn a_layer_crop_keeps_the_original_and_makes_a_picture() {
    let doc = document();
    let layer = &doc.layers()[1];
    let piece = layer_crop::crop_layer(layer, PixelRect::new(20, 25, 40, 10), doc.bounds()).expect("something inside");
    let o = piece.object().expect("a picture");
    assert_eq!(o.kind(), "image");
    let b = o.box_().unwrap();
    assert_eq!((b.x, b.y, b.width, b.height), (20.0, 25.0, 20.0, 10.0)); // only the part of the box inside
    assert!(layer_crop::crop_layer(layer, PixelRect::new(60, 60, 10, 10), doc.bounds()).is_none());
}

#[test]
fn an_effect_with_a_selection_changes_only_the_selected_area() {
    let mut src = PixelBuffer::filled(20, 20, ColorRgba::rgb(200, 100, 50));
    src.set_pixel(0, 0, ColorRgba::rgb(10, 20, 30));
    let sel = Selection::from_rect(PixelRect::new(5, 5, 10, 10), 20, 20).unwrap();
    let out = effects::apply_to_selection(&src, &effects::INVERT, &effects::INVERT.defaults(), Some(&sel));
    assert_eq!(out.pixel(0, 0), src.pixel(0, 0));
    assert_eq!(out.pixel(19, 19), src.pixel(19, 19));
    assert_eq!(out.pixel(10, 10), ColorRgba::rgb(55, 155, 205));
}

#[test]
fn the_color_adjustment_of_an_object_changes_how_it_is_drawn() {
    let doc = document();
    let mut o = doc.layers()[1].object().cloned().unwrap();
    let plain = object_ops::object_layer("a", o.clone(), doc.bounds()).pixels().pixel(20, 30);
    let tone = fe_core::document::AdjustmentStyle { hue: 120.0, ..Default::default() };
    o.set_tone(Some(&tone));
    assert_eq!(o.tone().map(|t| t.hue), Some(120.0));
    let changed = object_ops::object_layer("b", o.clone(), doc.bounds()).pixels().pixel(20, 30);
    assert!(plain.r > 150 && changed.r < 100, "red turned away: {plain:?} -> {changed:?}");
    o.set_tone(None);
    assert!(o.tone().is_none());
}
