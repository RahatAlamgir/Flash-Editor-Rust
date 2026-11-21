//! Port of the C# `LayerMaskTests` (the parts that use selections, undo and project files follow in
//! Phases 4–6).

mod common;

use common::*;
use fe_core::document::{Document, Layer, LayerMask};
use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::paint::compositor;

fn red() -> Layer {
    solid_layer("Red", 20, 10, ColorRgba::rgb(255, 0, 0))
}

fn doc_of(layer: Layer) -> Document {
    doc_with(20, 10, vec![layer])
}

#[test]
fn a_hidden_part_of_a_layer_does_not_show_when_the_picture_is_flattened() {
    let mut layer = red();
    let mut mask = LayerMask::new(20, 10, 255);
    for y in 0..10 {
        for x in 0..10 {
            mask.set(x, y, 0); // hide the left half
        }
    }
    layer.set_mask(Some(mask), 20, 10).expect("right size");
    let doc = doc_of(layer);
    let flat = compositor::flatten(&doc);
    assert_eq!(0, flat.pixel(3, 3).a);
    assert_eq!(255, flat.pixel(15, 3).a);
    assert_eq!(255, doc.layers()[0].pixels().pixel(3, 3).a); // the layer itself is untouched
}

#[test]
fn a_half_visible_mask_makes_the_layer_half_see_through() {
    let mut layer = red();
    layer.set_mask(Some(LayerMask::new(20, 10, 128)), 20, 10).expect("right size");
    assert!(in_range(compositor::flatten(&doc_of(layer)).pixel(5, 5).a, 126, 130));
}

#[test]
fn a_switched_off_mask_changes_nothing() {
    let mut layer = red();
    layer.set_mask(Some(LayerMask::new(20, 10, 0)), 20, 10).expect("right size");
    layer.mask_enabled = false;
    let mut doc = doc_of(layer);
    assert_eq!(255, compositor::flatten(&doc).pixel(5, 5).a);
    doc.layers_mut()[0].mask_enabled = true;
    assert_eq!(0, compositor::flatten(&doc).pixel(5, 5).a);
}

#[test]
fn the_merged_color_sampler_sees_the_mask_too() {
    let mut layer = red();
    layer.set_mask(Some(LayerMask::new(20, 10, 0)), 20, 10).expect("right size");
    assert_eq!(0, compositor::sample_merged(&doc_of(layer), 5, 5).a);
}

#[test]
fn a_mask_must_have_the_size_of_the_canvas() {
    assert!(red().set_mask(Some(LayerMask::new(5, 5, 255)), 20, 10).is_err());
}

#[test]
fn inverting_turns_hidden_into_visible() {
    let mut mask = LayerMask::new(4, 1, 255);
    mask.set(0, 0, 0);
    mask.set(1, 0, 200);
    mask.invert();
    assert_eq!(vec![255, 55, 0, 0], mask.copy_region(mask.bounds()));
}

#[test]
fn copy_and_paste_region_restore_painted_values() {
    let mut mask = LayerMask::new(20, 10, 255);
    let region = PixelRect::new(2, 2, 4, 4);
    let before = mask.copy_region(region);
    for y in 2..6 {
        for x in 2..6 {
            mask.set(x, y, 0);
        }
    }
    mask.paste_region(region, &before);
    assert_eq!(255, mask.get(3, 3));
}

#[test]
fn a_duplicated_layer_has_its_own_copy_of_the_mask() {
    let mut layer = red();
    layer.set_mask(Some(LayerMask::new(20, 10, 100)), 20, 10).expect("right size");
    let mut copy = layer.duplicate("copy");
    copy.mask_mut().expect("has mask").set(0, 0, 7);
    assert_eq!(100, layer.mask().expect("has mask").get(0, 0));
    assert_eq!(100, copy.mask().expect("has mask").get(1, 0));
}

#[test]
fn grey_picture_round_trip_and_bake() {
    let mut mask = LayerMask::new(20, 10, 255);
    mask.set(7, 4, 33);
    let back = LayerMask::from_gray_picture(&mask.to_gray_picture());
    assert_eq!(33, back.get(7, 4));
    assert_eq!(255, back.get(0, 0));
    assert!(LayerMask::new(20, 10, 255).is_all_visible());
    assert!(!back.is_all_visible());
    let baked = mask.bake(&solid(20, 10, ColorRgba::WHITE), 0, 0);
    assert_eq!(33, baked.pixel(7, 4).a);
}

#[test]
fn an_untouched_mask_costs_no_memory() {
    assert_eq!(0, LayerMask::new(4000, 3000, 255).memory_bytes());
}

#[test]
fn a_mask_from_a_selection_shows_or_hides_the_selected_area() {
    use fe_core::selection::Selection;
    let selection = Selection::from_rect(PixelRect::new(2, 2, 6, 4), 20, 10).unwrap();
    let reveal = LayerMask::from_selection(&selection, true);
    assert_eq!(255, reveal.get(3, 3));
    assert_eq!(0, reveal.get(0, 0));
    let hide = LayerMask::from_selection(&selection, false);
    assert_eq!(0, hide.get(3, 3));
    assert_eq!(255, hide.get(0, 0));
}

#[test]
fn hiding_a_selection_on_an_existing_mask_keeps_what_was_already_hidden() {
    use fe_core::selection::Selection;
    let mut mask = LayerMask::new(20, 10, 255);
    mask.set(0, 0, 0);
    mask.apply_selection(&Selection::from_rect(PixelRect::new(5, 5, 3, 3), 20, 10).unwrap(), false);
    assert_eq!(0, mask.get(0, 0));
    assert_eq!(0, mask.get(6, 6));
    assert_eq!(255, mask.get(19, 9));
    mask.apply_selection(&Selection::from_rect(PixelRect::new(5, 5, 1, 1), 20, 10).unwrap(), true);
    assert_eq!(255, mask.get(5, 5));
}

#[test]
fn clip_to_layer_below_shows_the_layer_only_inside_the_shape_under_it() {
    use fe_core::document::mask_ops;
    let mut below = Layer::new("Shape");
    below.pixels_mut().fill_rect(PixelRect::new(0, 0, 5, 10), ColorRgba::WHITE);
    let top = red();
    let id = top.id();
    let mut doc = doc_with(20, 10, vec![below, top]);
    let names = mask_ops::clip_to_below(&mut doc, id).expect("clipped");
    assert_eq!(("Red".to_owned(), "Shape".to_owned()), names);
    let mask = doc.layer(id).unwrap().mask().unwrap();
    assert_eq!(255, mask.get(2, 2));
    assert_eq!(0, mask.get(10, 2));
    // Apply: the hidden part becomes transparent for good, the mask goes away.
    assert!(mask_ops::apply(&mut doc, id));
    let layer = doc.layer(id).unwrap();
    assert!(layer.mask().is_none());
    assert_eq!(0, layer.pixels().pixel(10, 2).a);
    assert_eq!(255, layer.pixels().pixel(2, 2).a);
}

#[test]
fn mask_commands_add_toggle_invert_and_delete() {
    use fe_core::document::mask_ops;
    let layer = red();
    let id = layer.id();
    let mut doc = doc_of(layer);
    assert!(mask_ops::add(&mut doc, id, false));
    assert!(!mask_ops::add(&mut doc, id, true), "only one mask per layer");
    assert!(mask_ops::toggle(&mut doc, id));
    assert!(!doc.layer(id).unwrap().mask_enabled);
    assert!(mask_ops::invert(&mut doc, id));
    assert_eq!(255, doc.layer(id).unwrap().mask().unwrap().get(1, 1));
    assert!(mask_ops::delete(&mut doc, id));
    assert!(doc.layer(id).unwrap().mask().is_none());
}
