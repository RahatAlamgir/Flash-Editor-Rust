//! Page commands (C# `PageCommands`) and their undo.

use fe_core::document::{Document, Page, Project, page_ops};
use fe_core::imaging::ColorRgba;

fn project(n: usize) -> Project {
    let pages = (1..=n).map(|i| Page::new(format!("P{i}"), Document::blank(100, 50, ColorRgba::WHITE))).collect();
    Project::from_pages("Test", pages, 0)
}

fn names(p: &Project) -> String {
    p.pages().iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(",")
}

#[test]
fn add_duplicate_and_delete_pages_with_undo() {
    let mut p = project(2);
    p.edit_pages("Add Page", |pages, active| page_ops::add_page(pages, active, 1));
    assert_eq!("P1,Page 3,P2", names(&p));
    assert_eq!(1, p.active_index());
    p.edit_pages("Duplicate Page", |pages, active| page_ops::duplicate_pages(pages, active, &[0]));
    assert_eq!("P1,P1 copy,Page 3,P2", names(&p));
    let deleted = p.edit_pages("Delete Pages", |pages, active| page_ops::delete_pages(pages, active, &[0, 1, 2, 3]));
    assert_eq!(3, deleted); // the first one stays
    assert_eq!("P1", names(&p));
    p.undo();
    assert_eq!("P1,P1 copy,Page 3,P2", names(&p));
    p.redo();
    assert_eq!("P1", names(&p));
}

#[test]
fn a_duplicated_page_is_a_separate_document() {
    let mut p = project(1);
    p.edit_pages("Duplicate Page", |pages, active| page_ops::duplicate_pages(pages, active, &[0]));
    assert_ne!(p.pages()[0].doc.uid(), p.pages()[1].doc.uid());
    assert!(p.pages()[0].doc.layers()[0].pixels().shares_all_tiles_with(p.pages()[1].doc.layers()[0].pixels()));
}

#[test]
fn moving_pages_keeps_their_order_and_the_shown_page() {
    let mut p = project(4);
    p.set_active_index(1); // P2
    let moved = p.edit_pages("Move Pages", |pages, active| page_ops::move_pages(pages, active, &[1, 2], 4));
    assert!(moved);
    assert_eq!("P1,P4,P2,P3", names(&p));
    assert_eq!("P2", p.active_page().name);
    let unchanged = p.edit_pages("Move Pages", |pages, active| page_ops::move_pages(pages, active, &[0], 0));
    assert!(!unchanged);
}

#[test]
fn renaming_and_pasting_pages() {
    let mut p = project(2);
    assert!(p.edit_pages("Rename Page", |pages, _| page_ops::rename_page(pages, 1, "Cover")));
    let clipboard = vec![p.pages()[1].clone()];
    p.edit_pages("Paste Pages", |pages, active| page_ops::paste_pages(pages, active, 0, &clipboard));
    assert_eq!("Cover,P1,Cover", names(&p));
    assert_eq!(0, p.active_index());
}

#[test]
fn page_steps_and_document_steps_undo_in_order() {
    let mut p = project(1);
    p.edit("Paint", None, |doc| doc.layers_mut()[0].pixels_mut().set_pixel(1, 1, ColorRgba::BLACK));
    p.edit_pages("Add Page", |pages, active| page_ops::add_page(pages, active, 1));
    p.undo();
    assert_eq!(1, p.pages().len());
    assert_eq!(ColorRgba::BLACK, p.pages()[0].doc.layers()[0].pixels().pixel(1, 1));
    p.undo();
    assert_eq!(ColorRgba::WHITE, p.pages()[0].doc.layers()[0].pixels().pixel(1, 1));
}
