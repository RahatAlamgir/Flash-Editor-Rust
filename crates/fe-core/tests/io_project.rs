//! `.flash` projects: reading files saved by the C# version (tests/fixtures, made by tools/csharp-fixtures),
//! and save → open round trips. Ports the ideas of the C# `ProjectFileTests`.

mod common;

use std::path::PathBuf;

use common::*;
use fe_core::document::{AdjustmentKind, Document, Layer, LayerMask, Page};
use fe_core::imaging::{BlendMode, ColorRgba, PixelRect};
use fe_core::io::project::{self, ProjectContent};
use fe_core::paint::compositor;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures").join(name)
}

fn layer<'a>(page: &'a Page, name: &str) -> &'a Layer {
    page.doc.layers().iter().find(|l| l.name == name).unwrap_or_else(|| panic!("no layer {name}"))
}

fn round_trip(content: &ProjectContent) -> ProjectContent {
    let bytes = project::to_bytes(&content.pages, content.active_page).expect("saves");
    project::open_from(std::io::Cursor::new(bytes)).expect("opens again")
}

#[test]
fn the_csharp_features_project_opens_with_every_setting() {
    let content = project::open(fixture("features.flash")).expect("opens");
    assert_eq!(2, content.pages.len());
    assert_eq!(1, content.active_page);
    let page = &content.pages[0];
    assert_eq!(("Features", 640, 400), (page.name.as_str(), page.doc.width(), page.doc.height()));
    assert_eq!(vec![100.0, 320.5], page.vertical_guides);
    assert_eq!(vec![200.0], page.horizontal_guides);
    assert_eq!(Some("Masked"), page.doc.active_layer().map(|l| l.name.as_str()));

    let moved = layer(page, "Off canvas");
    assert_eq!(ColorRgba::rgb(220, 40, 40), moved.pixels().pixel(700, 100)); // kept outside the canvas
    assert_eq!(PixelRect::new(540, 40, 200, 120), moved.visible_bounds());

    let masked = layer(page, "Masked");
    assert_eq!((0.75, BlendMode::Multiply), (masked.opacity(), masked.blend));
    let mask = masked.mask().expect("has a mask");
    assert_eq!(0, mask.get(0, 10));
    assert_eq!(255, mask.get(500, 10));

    let off = layer(page, "Mask switched off");
    assert!(!off.mask_enabled && off.locked);
    let hidden = layer(page, "Hidden");
    assert!(!hidden.visible);
    assert_eq!(BlendMode::SoftLight, hidden.blend);

    let levels = layer(page, "Levels").adjustment.as_ref().expect("adjustment");
    assert_eq!(
        (AdjustmentKind::Levels, 12.0, 1.4, 240.0),
        (levels.kind, levels.levels_black, levels.levels_gamma, levels.output_white)
    );
    let curves = layer(page, "Curves").adjustment.as_ref().expect("adjustment");
    assert_eq!(3, curves.curve.as_ref().map_or(0, Vec::len));
    assert_eq!(Some(20.0), curves.curve_red.as_ref().map(|c| c[0].y));
    let warm = layer(page, "Warm");
    assert_eq!(0.5, warm.opacity());
    assert_eq!(Some(30.0), warm.adjustment.as_ref().map(|a| a.temperature));

    let tall = &content.pages[1];
    assert_eq!((300, 500), (tall.doc.width(), tall.doc.height()));
    assert_eq!(ColorRgba::new(7, 4, 7 ^ 9, 255 - 2), tall.doc.layers()[0].pixels().pixel(7, 9));
}

#[test]
fn the_csharp_tutorial_opens_with_its_objects_kept() {
    let content = project::open(fixture("tutorial.flash")).expect("opens");
    assert_eq!(14, content.pages.len());
    let mut kinds = std::collections::BTreeMap::new();
    for page in &content.pages {
        for layer in page.doc.layers() {
            if let Some(object) = layer.object() {
                *kinds.entry(object.kind().to_owned()).or_insert(0) += 1;
            }
        }
    }
    let expected: std::collections::BTreeMap<String, i32> =
        [("barcode", 2), ("chart", 4), ("group", 32), ("image", 21), ("qr", 2), ("shape", 127), ("text", 219)]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
    assert_eq!(expected, kinds);
    // Picture objects carry their source picture.
    let photo =
        content.pages.iter().flat_map(|p| p.doc.layers()).find(|l| l.object().is_some_and(|o| o.kind() == "image"));
    assert!(photo.is_some_and(|l| l.object().is_some_and(|o| !o.images.is_empty())));
}

#[test]
fn saving_and_opening_again_keeps_everything() {
    for name in ["features.flash", "tutorial.flash"] {
        let original = project::open(fixture(name)).expect("opens");
        let again = round_trip(&original);
        assert_eq!(original.pages.len(), again.pages.len());
        assert_eq!(original.active_page, again.active_page);
        for (a, b) in original.pages.iter().zip(&again.pages) {
            assert_eq!((&a.name, a.doc.width(), a.doc.height()), (&b.name, b.doc.width(), b.doc.height()));
            assert_eq!(a.vertical_guides, b.vertical_guides);
            assert_eq!(a.doc.active_index(), b.doc.active_index());
            assert_eq!(compositor::flatten(&a.doc), compositor::flatten(&b.doc), "page {} looks different", a.name);
            for (la, lb) in a.doc.layers().iter().zip(b.doc.layers()) {
                assert_eq!(la.name, lb.name);
                assert_eq!(la.adjustment, lb.adjustment);
                assert_eq!(la.object().map(|o| o.kind()), lb.object().map(|o| o.kind()));
                assert_eq!(la.object().map(|o| o.images.len()), lb.object().map(|o| o.images.len()));
                assert_eq!(la.visible_bounds(), lb.visible_bounds());
            }
        }
    }
}

#[test]
fn a_new_project_round_trips_masks_restore_and_off_canvas_pixels() {
    let mut doc = Document::blank(300, 200, ColorRgba::WHITE);
    let mut top = Layer::new("Top");
    top.pixels_mut().fill_rect(PixelRect::new(250, -30, 100, 60), ColorRgba::rgb(1, 2, 3));
    let mut mask = LayerMask::new(300, 200, 255);
    mask.set(10, 10, 33);
    top.set_mask(Some(mask), 300, 200).expect("size");
    top.restore = Some(fe_core::imaging::TileMap::from_buffer(&solid(300, 200, ColorRgba::rgb(9, 9, 9)), 0, 0));
    doc.push_layer(top);
    let content = ProjectContent { pages: vec![Page::new("Page 1", doc)], active_page: 0, presentation: false };
    let again = round_trip(&content);
    let layer = &again.pages[0].doc.layers()[1];
    assert_eq!(ColorRgba::rgb(1, 2, 3), layer.pixels().pixel(320, -10));
    assert_eq!(33, layer.mask().expect("mask").get(10, 10));
    assert_eq!(ColorRgba::rgb(9, 9, 9), layer.restore.as_ref().expect("restore").pixel(5, 5));
}

#[test]
fn something_that_is_not_a_project_gives_a_clear_error() {
    let err = project::open_from(std::io::Cursor::new(b"hello".to_vec())).expect_err("not a zip");
    assert_eq!("This is not a Flash Editor project.", err.to_string());
    assert!(project::read_preview(fixture("features.flash")).is_some());
}

#[test]
fn painting_on_an_object_layer_turns_it_into_pixels() {
    let content = project::open(fixture("tutorial.flash")).expect("opens");
    let mut layer = content.pages[0].doc.layers().iter().find(|l| l.object().is_some()).expect("an object").clone();
    layer.pixels_mut().set_pixel(1, 1, ColorRgba::BLACK);
    assert!(layer.object().is_none());
}

#[test]
fn presentation_flag_and_slide_transitions_are_saved() {
    use fe_core::document::{Transition, TransitionDirection, TransitionKind};
    let mut first = Page::new("One", Document::blank(320, 180, ColorRgba::WHITE));
    first.transition = Transition {
        kind: TransitionKind::Push,
        direction: TransitionDirection::Up,
        duration: 1.5,
        on_click: false,
        after: Some(4.0),
    };
    let second = Page::new("Two", Document::blank(320, 180, ColorRgba::WHITE));
    let bytes = project::to_bytes_as(&[first, second], 0, true).expect("saves");
    let again = project::open_from(std::io::Cursor::new(bytes)).expect("opens");
    assert!(again.presentation);
    assert_eq!(TransitionKind::Push, again.pages[0].transition.kind);
    assert_eq!(TransitionDirection::Up, again.pages[0].transition.direction);
    assert_eq!(
        (1.5, false, Some(4.0)),
        (again.pages[0].transition.duration, again.pages[0].transition.on_click, again.pages[0].transition.after)
    );
    assert!(again.pages[1].transition.is_default());
    // A project that is not a presentation stays one that is not.
    let plain = project::to_bytes(&again.pages, 0).expect("saves");
    assert!(!project::open_from(std::io::Cursor::new(plain)).expect("opens").presentation);
}

#[test]
fn object_animations_are_saved_with_their_layers() {
    use fe_core::document::TransitionDirection;
    use fe_core::document::animation::{Animation, Effect, Start};
    let mut doc = Document::blank(64, 36, ColorRgba::WHITE);
    doc.push_layer(Layer::new("Second"));
    let (first, second) = (doc.layers()[0].id(), doc.layers()[1].id());
    let mut page = Page::new("One", doc);
    let mut fly = Animation::new(second, Effect::FlyIn);
    fly.start = Start::AfterPrevious;
    fly.direction = TransitionDirection::Down;
    fly.duration = 1.25;
    fly.delay = 0.5;
    page.animations = vec![Animation::new(first, Effect::FadeIn), fly];
    let bytes = project::to_bytes_as(&[page.clone()], 0, true).expect("saves");
    let again = project::open_from(std::io::Cursor::new(bytes)).expect("opens");
    let back = &again.pages[0];
    assert_eq!(2, back.animations.len());
    assert_eq!(back.doc.layers()[0].id(), back.animations[0].layer);
    assert_eq!(back.doc.layers()[1].id(), back.animations[1].layer);
    assert_eq!((Effect::FlyIn, Start::AfterPrevious), (back.animations[1].effect, back.animations[1].start));
    assert_eq!(
        (TransitionDirection::Down, 1.25, 0.5),
        (back.animations[1].direction, back.animations[1].duration, back.animations[1].delay)
    );
    // A copy of the page keeps the animations on the copied layers.
    let copy = fe_core::document::page_ops::copy_of(&page, "Copy");
    assert_eq!(2, copy.animations.len());
    assert_eq!(copy.doc.layers()[1].id(), copy.animations[1].layer);
    assert_ne!(page.doc.layers()[1].id(), copy.animations[1].layer);
}
