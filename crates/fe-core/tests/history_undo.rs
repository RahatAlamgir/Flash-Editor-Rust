//! Port of the C# `HistoryTests`, `HistoryMemoryTests` and `ProjectHistoryTests` (adapted to snapshot steps).

mod common;

use common::*;
use fe_core::document::{Document, Layer, Page, Project, layer_ops};
use fe_core::history::{DEFAULT_STEPS, MAX_STEPS, MergeKey};
use fe_core::imaging::{ColorRgba, PixelRect};

const RED: ColorRgba = ColorRgba::rgb(255, 0, 0);

fn project() -> Project {
    Project::new("P", Page::new("Page 1", Document::blank(300, 200, ColorRgba::WHITE)))
}

fn two_pages() -> Project {
    let mut p = project();
    p.insert_page(1, Page::new("Page 2", Document::blank(300, 200, ColorRgba::WHITE)));
    p.set_active_index(0);
    p
}

fn paint(p: &mut Project, name: &str, x: i32) {
    p.edit(name, None, |doc| doc.layers_mut()[0].pixels_mut().set_pixel(x, 5, RED));
}

fn pixel(p: &Project, page: usize, x: i32) -> ColorRgba {
    p.pages()[page].doc.layers()[0].pixels().pixel(x, 5)
}

#[test]
fn undo_and_redo_a_pixel_change() {
    let mut p = project();
    paint(&mut p, "Brush", 3);
    assert_eq!(RED, pixel(&p, 0, 3));
    assert!(p.undo());
    assert_eq!(ColorRgba::WHITE, pixel(&p, 0, 3));
    assert!(p.redo());
    assert_eq!(RED, pixel(&p, 0, 3));
    assert!(!p.redo());
}

#[test]
fn new_work_discards_the_redo_steps() {
    let mut p = project();
    paint(&mut p, "A", 1);
    paint(&mut p, "B", 2);
    p.undo();
    assert!(p.history().can_redo());
    paint(&mut p, "C", 3);
    assert!(!p.history().can_redo());
    assert_eq!(vec!["A", "C"], p.history().steps().iter().map(|s| s.name.as_str()).collect::<Vec<_>>());
}

#[test]
fn jump_to_moves_several_steps() {
    let mut p = project();
    for x in 0..4 {
        paint(&mut p, "Brush", x);
    }
    p.jump_to(1);
    assert_eq!(RED, pixel(&p, 0, 0));
    assert_eq!(ColorRgba::WHITE, pixel(&p, 0, 1));
    p.jump_to(4);
    assert_eq!(RED, pixel(&p, 0, 3));
    p.jump_to(0);
    assert!(!p.history().can_undo());
}

#[test]
fn the_dirty_flag_follows_the_saved_position() {
    let mut p = project();
    assert!(!p.history().is_dirty());
    paint(&mut p, "A", 1);
    assert!(p.history().is_dirty());
    p.history_mut().mark_saved();
    assert!(!p.history().is_dirty());
    p.undo();
    assert!(p.history().is_dirty());
    p.redo();
    assert!(!p.history().is_dirty());
}

#[test]
fn opacity_changes_merge_into_one_step() {
    let mut p = project();
    let id = p.active_page().doc.layers()[0].id();
    let key = Some(MergeKey { layer: id, what: "opacity" });
    for o in [0.9, 0.7, 0.5] {
        p.edit("Layer Opacity", key, |doc| doc.layers_mut()[0].set_opacity(o));
    }
    assert_eq!(1, p.history().steps().len());
    p.undo();
    assert_eq!(1.0, p.active_page().doc.layers()[0].opacity());
}

#[test]
fn layer_structure_undo_brings_a_deleted_layer_back() {
    let mut p = project();
    p.edit("New Layer", None, layer_ops::new_layer);
    assert_eq!(2, p.active_page().doc.layers().len());
    p.edit("Delete Layer", None, layer_ops::delete_selected);
    assert_eq!(1, p.active_page().doc.layers().len());
    p.undo();
    assert_eq!(2, p.active_page().doc.layers().len());
    assert_eq!("Layer 1", p.active_page().doc.active_layer().map(|l| l.name.as_str()).unwrap_or_default());
}

#[test]
fn by_default_thirty_steps_are_kept_and_at_most_sixty_can_be_chosen() {
    let mut p = project();
    assert_eq!(DEFAULT_STEPS, p.history().max_steps());
    for x in 0..35 {
        paint(&mut p, "Brush", x);
    }
    assert_eq!(30, p.history().steps().len());
    p.history_mut().set_max_steps(500);
    assert_eq!(MAX_STEPS, p.history().max_steps());
    p.history_mut().set_max_steps(5);
    assert_eq!(5, p.history().steps().len()); // lowering the limit forgets the oldest at once
}

#[test]
fn dropping_old_steps_keeps_the_unsaved_flag_right() {
    let mut p = project();
    p.history_mut().set_max_steps(2);
    paint(&mut p, "A", 1);
    p.history_mut().mark_saved();
    paint(&mut p, "B", 2);
    paint(&mut p, "C", 3); // "A" is forgotten; the saved state (after A) is now the start of the list
    assert!(p.history().is_dirty());
    p.undo();
    assert!(p.history().is_dirty());
    p.undo();
    assert!(!p.history().is_dirty());
}

#[test]
fn undo_takes_back_the_newest_step_of_the_whole_project_whichever_page_it_was_in() {
    let mut p = two_pages();
    paint(&mut p, "Page 1 brush", 1);
    p.set_active_index(1);
    paint(&mut p, "Page 2 brush", 2);
    p.set_active_index(0);
    p.undo();
    assert_eq!(1, p.active_index()); // the page of that step is shown
    assert_eq!(ColorRgba::WHITE, pixel(&p, 1, 2));
    assert_eq!(RED, pixel(&p, 0, 1)); // the other page keeps its state
}

#[test]
fn steps_of_different_pages_do_not_merge() {
    let mut p = two_pages();
    let id0 = p.pages()[0].doc.layers()[0].id();
    let key = Some(MergeKey { layer: id0, what: "opacity" });
    p.edit("Layer Opacity", key, |d| d.layers_mut()[0].set_opacity(0.5));
    p.set_active_index(1);
    paint(&mut p, "Brush", 1);
    p.set_active_index(0);
    p.edit("Layer Opacity", key, |d| d.layers_mut()[0].set_opacity(0.2));
    assert_eq!(3, p.history().steps().len());
}

#[test]
fn a_page_that_leaves_the_project_takes_its_steps_with_it() {
    let mut p = two_pages();
    paint(&mut p, "Page 1", 1);
    p.set_active_index(1);
    paint(&mut p, "Page 2", 2);
    p.remove_page(0);
    assert_eq!(1, p.history().steps().len());
    assert_eq!(0, p.history().steps()[0].page); // renumbered
    p.undo();
    assert_eq!(ColorRgba::WHITE, pixel(&p, 0, 2));
}

#[test]
fn a_step_only_costs_the_tiles_that_changed() {
    let mut p = Project::new("Big", Page::new("Page 1", Document::blank(4000, 3000, ColorRgba::WHITE)));
    paint(&mut p, "Dot", 10); // one tile of 192 changes
    let bytes = p.history().steps()[0].bytes();
    assert_eq!(2 * fe_core::imaging::TileMap::TILE_BYTES, bytes); // the old and the new copy of that tile
}

#[test]
fn whole_picture_edits_cannot_pile_up_past_the_memory_limit() {
    let mut p = Project::new("Big", Page::new("Page 1", Document::blank(1024, 1024, ColorRgba::WHITE)));
    p.history_mut().set_max_bytes(10 * 1024 * 1024);
    for i in 0..20u8 {
        p.edit("Fill", None, |doc| {
            doc.layers_mut()[0].pixels_mut().fill_rect(PixelRect::new(0, 0, 1024, 1024), ColorRgba::rgb(i, 0, 0))
        });
    }
    assert!(p.history().total_bytes() <= 10 * 1024 * 1024);
    assert!(!p.history().steps().is_empty()); // the newest step is always kept
}

#[test]
fn undo_restores_layer_settings_and_selection() {
    let mut p = project();
    p.edit("New Layer", None, |doc| {
        layer_ops::new_layer(doc);
    });
    let top = p.active_page().doc.active_id();
    p.edit("Hide Layer", None, |doc| doc.layers_mut()[1].visible = false);
    p.undo();
    assert!(p.active_page().doc.layers()[1].visible);
    assert_eq!(top, p.active_page().doc.active_id());
    let _ = Layer::new("unused");
    let _ = solid(1, 1, RED);
}
