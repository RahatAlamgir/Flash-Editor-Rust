//! Layer commands: new, duplicate, delete, move, merge, flatten, lock, multi-selection.

mod common;

use common::*;
use fe_core::document::{AdjustmentStyle, Document, Layer, LayerId, layer_ops};
use fe_core::imaging::{BlendMode, ColorRgba, PixelRect};
use fe_core::paint::compositor;

fn names(doc: &Document) -> Vec<&str> {
    doc.layers().iter().map(|l| l.name.as_str()).collect()
}

fn doc3() -> Document {
    doc_with(
        20,
        10,
        vec![
            solid_layer("A", 20, 10, ColorRgba::rgb(200, 0, 0)),
            solid_layer("B", 20, 10, ColorRgba::rgb(0, 200, 0)),
            solid_layer("C", 20, 10, ColorRgba::rgb(0, 0, 200)),
        ],
    )
}

fn id(doc: &Document, name: &str) -> LayerId {
    doc.layers().iter().find(|l| l.name == name).map(Layer::id).expect("layer exists")
}

#[test]
fn a_new_layer_goes_above_the_active_one_and_is_numbered() {
    let mut doc = doc3();
    doc.set_active(Some(id(&doc, "A")));
    layer_ops::new_layer(&mut doc);
    assert_eq!(vec!["A", "Layer 1", "B", "C"], names(&doc));
    assert_eq!(Some("Layer 1"), doc.active_layer().map(|l| l.name.as_str()));
}

#[test]
fn duplicating_several_layers_puts_the_copies_above_the_topmost() {
    let mut doc = doc3();
    let (a, b) = (id(&doc, "A"), id(&doc, "B"));
    doc.set_selection(&[a, b], Some(b));
    assert_eq!(2, layer_ops::duplicate_selected(&mut doc));
    assert_eq!(vec!["A", "B", "A copy", "B copy", "C"], names(&doc));
    assert_eq!(2, doc.selected_ids().len());
    // A copy shares its tiles with the original until one of them changes.
    assert!(doc.layers()[0].pixels().shares_all_tiles_with(doc.layers()[2].pixels()));
}

#[test]
fn deleting_skips_locked_layers_and_never_removes_the_last_one() {
    let mut doc = doc3();
    let all: Vec<LayerId> = doc.layers().iter().map(Layer::id).collect();
    doc.layers_mut()[1].locked = true;
    doc.set_selection(&all, None);
    assert_eq!(2, layer_ops::delete_selected(&mut doc));
    assert_eq!(vec!["B"], names(&doc));
    doc.layers_mut()[0].locked = false;
    assert_eq!(0, layer_ops::delete_selected(&mut doc)); // the last layer stays
}

#[test]
fn moving_and_bringing_to_front_or_back() {
    let mut doc = doc3();
    doc.set_active(Some(id(&doc, "A")));
    assert!(layer_ops::move_active(&mut doc, 1));
    assert_eq!(vec!["B", "A", "C"], names(&doc));
    assert!(layer_ops::move_active_to_end(&mut doc, true));
    assert_eq!(vec!["B", "C", "A"], names(&doc));
    assert!(!layer_ops::move_active(&mut doc, 1)); // already on top
    assert!(layer_ops::move_active_to_end(&mut doc, false));
    assert_eq!(vec!["A", "B", "C"], names(&doc));
}

#[test]
fn dragging_layers_into_a_gap() {
    let mut doc = doc3();
    let a = id(&doc, "A");
    assert!(layer_ops::move_to_gap(&mut doc, &[a], 0)); // to the top of the list = front
    assert_eq!(vec!["B", "C", "A"], names(&doc));
    assert!(!layer_ops::move_to_gap(&mut doc, &[a], 0));
}

#[test]
fn merge_down_paints_with_opacity_and_blend_mode() {
    let mut doc = doc_with(
        8,
        8,
        vec![
            solid_layer("Back", 8, 8, ColorRgba::rgb(200, 100, 50)),
            solid_layer("Top", 8, 8, ColorRgba::rgb(128, 128, 128)),
        ],
    );
    doc.layers_mut()[1].blend = BlendMode::Multiply;
    let expected = compositor::flatten(&doc).pixel(3, 3);
    assert!(layer_ops::merge_down(&mut doc));
    assert_eq!(vec!["Back"], names(&doc));
    assert_eq!(expected, doc.layers()[0].pixels().pixel(3, 3));
}

#[test]
fn merge_down_bakes_an_adjustment_layer() {
    let mut doc = doc_with(6, 6, vec![solid_layer("Back", 6, 6, ColorRgba::rgb(100, 100, 100))]);
    doc.push_layer(Layer::adjustment("Adjust", AdjustmentStyle { brightness: 20.0, ..AdjustmentStyle::default() }));
    assert!(layer_ops::merge_down(&mut doc));
    assert!(doc.layers()[0].pixels().pixel(1, 1).r > 100);
}

#[test]
fn merging_selected_layers_makes_one_layer_where_the_top_one_was() {
    let mut doc = doc3();
    let (a, c) = (id(&doc, "A"), id(&doc, "C"));
    doc.layers_mut()[2].set_opacity(0.5);
    doc.set_selection(&[a, c], Some(c));
    assert_eq!(2, layer_ops::merge_selected(&mut doc));
    assert_eq!(vec!["B", "Merged"], names(&doc));
    let merged = doc.layers()[1].pixels().pixel(0, 0);
    assert!(merged.b > 90 && merged.r > 90); // half blue over red
    doc.set_selection(&[id(&doc, "B")], None);
    assert_eq!(0, layer_ops::merge_selected(&mut doc)); // one layer is not enough
}

#[test]
fn flatten_keeps_only_visible_layers_and_matches_the_picture() {
    let mut doc = doc3();
    doc.layers_mut()[2].visible = false;
    doc.layers_mut()[1].pixels_mut().clear_rect(PixelRect::new(0, 0, 5, 10));
    let expected = compositor::flatten(&doc);
    layer_ops::flatten(&mut doc);
    assert_eq!(vec!["Background"], names(&doc));
    assert_eq!(expected, doc.layers()[0].pixels().to_buffer(doc.bounds()));
}

#[test]
fn locking_all_selected_layers() {
    let mut doc = doc3();
    let all: Vec<LayerId> = doc.layers().iter().map(Layer::id).collect();
    doc.set_selection(&all, None);
    assert_eq!(3, layer_ops::set_lock_of_selected(&mut doc, true));
    assert_eq!(0, layer_ops::set_lock_of_selected(&mut doc, true));
    assert!(doc.layers().iter().all(|l| l.locked));
}

#[test]
fn the_active_layer_is_always_selected() {
    let mut doc = doc3();
    let b = id(&doc, "B");
    doc.set_selection(&[], Some(b));
    assert_eq!(vec![b], doc.selected_ids());
    doc.set_active(Some(id(&doc, "C")));
    assert!(!doc.is_selected(b));
}
