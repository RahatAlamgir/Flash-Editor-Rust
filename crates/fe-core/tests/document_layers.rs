//! Port of the C# `LayerOrderTests`, plus documents and projects.

use fe_core::document::layer_order::{RowSpan, gap_at, move_to_gap};
use fe_core::document::{Document, Layer, Page, Project};
use fe_core::imaging::ColorRgba;

// Bottom to top: A, B, C, D. The panel lists them top first: D, C, B, A.
const STACK: [char; 4] = ['A', 'B', 'C', 'D'];

fn names(layers: Option<Vec<char>>) -> String {
    layers.map_or("(unchanged)".into(), |l| l.into_iter().collect())
}

// A long list only has rows for the part on screen: rows 3..6 of 10, each 48 high.
const VISIBLE: [RowSpan; 4] = [
    RowSpan { index: 3, top: 0.0, bottom: 48.0 },
    RowSpan { index: 4, top: 48.0, bottom: 96.0 },
    RowSpan { index: 5, top: 96.0, bottom: 144.0 },
    RowSpan { index: 6, top: 144.0, bottom: 192.0 },
];

#[test]
fn below_the_last_visible_row_is_the_gap_after_that_row_not_the_end_of_the_whole_list() {
    assert_eq!(7, gap_at(&VISIBLE, 185.0, 10));
    assert_eq!(7, gap_at(&VISIBLE, 500.0, 10));
}

#[test]
fn the_gap_is_the_one_nearest_to_the_pointer_between_visible_rows() {
    assert_eq!(3, gap_at(&VISIBLE, 10.0, 10));
    assert_eq!(4, gap_at(&VISIBLE, 30.0, 10));
    assert_eq!(5, gap_at(&VISIBLE, 80.0, 10));
    assert_eq!(6, gap_at(&VISIBLE, 120.0, 10));
}

#[test]
fn below_the_last_row_of_a_short_list_is_the_very_end() {
    let all = [
        RowSpan { index: 0, top: 0.0, bottom: 48.0 },
        RowSpan { index: 1, top: 48.0, bottom: 96.0 },
        RowSpan { index: 2, top: 96.0, bottom: 144.0 },
    ];
    assert_eq!(3, gap_at(&all, 300.0, 3));
    assert_eq!(0, gap_at(&[], 10.0, 0));
}

#[test]
fn dragging_to_the_top_brings_the_layer_to_the_front_and_below_the_last_row_to_the_back() {
    assert_eq!("ACDB", names(move_to_gap(&STACK, &['B'], 0)));
    assert_eq!("DABC", names(move_to_gap(&STACK, &['D'], 4)));
}

#[test]
fn dragging_into_a_gap_between_two_rows() {
    assert_eq!("BCAD", names(move_to_gap(&STACK, &['A'], 1)));
}

#[test]
fn dropping_where_it_already_is_changes_nothing() {
    assert_eq!(None, move_to_gap(&STACK, &['C'], 1));
    assert_eq!(None, move_to_gap(&STACK, &['C'], 2));
    assert_eq!(None, move_to_gap(&STACK, &['D'], 0));
}

#[test]
fn several_layers_move_together_and_keep_their_order() {
    assert_eq!("BDAC", names(move_to_gap(&STACK, &['B', 'D'], 4)));
    assert_eq!("CDAB", names(move_to_gap(&STACK, &['A', 'B'], 0)));
}

#[test]
fn a_gap_past_the_end_or_a_layer_that_is_not_there_is_harmless() {
    assert_eq!("DABC", names(move_to_gap(&STACK, &['D'], 99)));
    assert_eq!(None, move_to_gap(&STACK, &['X'], 2));
}

#[test]
fn removing_the_active_layer_activates_the_one_below() {
    let mut doc = Document::blank(10, 10, ColorRgba::WHITE);
    let bottom = doc.active_id();
    let name = doc.next_layer_name("Layer");
    assert_eq!("Layer 1", name);
    let top = doc.push_layer(Layer::new(name));
    assert_eq!(Some(top), doc.active_id());
    doc.remove_layer(top);
    assert_eq!(bottom, doc.active_id());
    assert!(!doc.set_active(Some(top))); // not in the document any more
}

#[test]
fn a_project_always_keeps_one_page() {
    let mut project = Project::new("P", Page::new("Page 1", Document::new(5, 5)));
    assert!(project.remove_page(0).is_none());
    project.insert_page(1, Page::new("Page 2", Document::new(5, 5)));
    assert_eq!(1, project.active_index());
    assert_eq!("Page 1", project.remove_page(0).expect("removed").name);
    assert_eq!(0, project.active_index());
    assert_eq!("Page 2", project.active_page().name);
}
