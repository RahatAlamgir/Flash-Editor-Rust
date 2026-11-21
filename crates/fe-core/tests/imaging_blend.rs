//! Port of the C# `BlendModeTests` (blend math and blend modes in the compositor).

mod common;

use common::*;
use fe_core::imaging::{BlendMode, ColorRgba, PixelRect, blend_pixel};
use fe_core::paint::compositor;

fn mix(below: ColorRgba, above: ColorRgba, mode: BlendMode, opacity: f64) -> ColorRgba {
    let mut dst = below.to_array();
    blend_pixel(&mut dst, &above.to_array(), opacity, mode);
    ColorRgba::from_slice(&dst)
}

const fn rgb(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

#[test]
fn normal_covers_what_is_below() {
    assert_eq!(rgb(10, 20, 30), mix(rgb(200, 200, 200), rgb(10, 20, 30), BlendMode::Normal, 1.0));
}

#[test]
fn multiply_darkens_and_white_changes_nothing() {
    assert_eq!(rgb(200, 100, 50), mix(rgb(200, 100, 50), rgb(255, 255, 255), BlendMode::Multiply, 1.0));
    assert_eq!(rgb(0, 0, 0), mix(rgb(200, 100, 50), rgb(0, 0, 0), BlendMode::Multiply, 1.0));
    assert!(in_range(mix(rgb(200, 200, 200), rgb(128, 128, 128), BlendMode::Multiply, 1.0).r, 99, 101));
}

#[test]
fn screen_lightens_and_black_changes_nothing() {
    assert_eq!(rgb(200, 100, 50), mix(rgb(200, 100, 50), rgb(0, 0, 0), BlendMode::Screen, 1.0));
    assert_eq!(rgb(255, 255, 255), mix(rgb(200, 100, 50), rgb(255, 255, 255), BlendMode::Screen, 1.0));
    assert!(mix(rgb(100, 100, 100), rgb(100, 100, 100), BlendMode::Screen, 1.0).r > 100);
}

#[test]
fn darken_lighten_and_difference_pick_what_they_say() {
    let (below, above) = (rgb(100, 200, 50), rgb(150, 100, 50));
    assert_eq!(rgb(100, 100, 50), mix(below, above, BlendMode::Darken, 1.0));
    assert_eq!(rgb(150, 200, 50), mix(below, above, BlendMode::Lighten, 1.0));
    assert_eq!(rgb(50, 100, 0), mix(below, above, BlendMode::Difference, 1.0));
}

#[test]
fn overlay_and_soft_light_keep_mid_grey_and_push_contrast() {
    let grey = rgb(128, 128, 128);
    let dark = rgb(60, 60, 60);
    assert!(in_range(mix(dark, grey, BlendMode::Overlay, 1.0).r, 58, 62));
    assert!(in_range(mix(dark, grey, BlendMode::SoftLight, 1.0).r, 58, 62));
    assert!(mix(dark, rgb(200, 200, 200), BlendMode::Overlay, 1.0).r < 140);
    assert!(mix(dark, rgb(200, 200, 200), BlendMode::SoftLight, 1.0).r > 60);
    assert!(mix(dark, rgb(20, 20, 20), BlendMode::HardLight, 1.0).r < 60);
}

#[test]
fn opacity_and_transparency_are_respected() {
    assert!(in_range(mix(rgb(200, 200, 200), rgb(0, 0, 0), BlendMode::Multiply, 0.5).r, 99, 101));
    // Nothing below to blend with: just the layer.
    assert_eq!(rgb(10, 20, 30), mix(ColorRgba::TRANSPARENT, rgb(10, 20, 30), BlendMode::Multiply, 1.0));
}

#[test]
fn a_flattened_document_uses_the_blend_mode_of_each_layer() {
    let back = solid_layer("Back", 8, 8, rgb(200, 100, 50));
    let mut top = solid_layer("Top", 8, 8, rgb(128, 128, 128));
    top.blend = BlendMode::Multiply;
    let mut doc = doc_with(8, 8, vec![back, top]);
    let flat = compositor::flatten(&doc);
    assert!(in_range(flat.pixel(3, 3).r, 99, 101));
    assert_eq!(flat.pixel(3, 3), compositor::sample_merged(&doc, 3, 3));
    doc.layers_mut()[1].blend = BlendMode::Normal;
    assert_eq!(rgb(128, 128, 128), compositor::flatten(&doc).pixel(3, 3));
}

#[test]
fn redrawing_one_region_matches_a_full_flatten_and_leaves_the_rest() {
    let back = solid_layer("Back", 20, 20, rgb(200, 100, 50));
    let mut top = solid_layer("Top", 20, 20, rgb(60, 120, 240));
    top.blend = BlendMode::Screen;
    top.set_opacity(0.7);
    let doc = doc_with(20, 20, vec![back, top]);
    let full = compositor::flatten(&doc);
    let mut target = solid(20, 20, rgb(1, 2, 3));
    compositor::flatten_region(&doc, PixelRect::new(5, 5, 6, 6), &mut target);
    assert_eq!(full.pixel(7, 7), target.pixel(7, 7));
    assert_eq!(rgb(1, 2, 3), target.pixel(0, 0));
    assert_eq!(rgb(1, 2, 3), target.pixel(11, 11));
}

#[test]
fn the_blend_mode_is_copied_with_the_layer() {
    let mut layer = solid_layer("A", 6, 6, rgb(1, 2, 3));
    layer.blend = BlendMode::SoftLight;
    assert_eq!(BlendMode::SoftLight, layer.duplicate("copy").blend);
    assert_eq!("SoftLight", BlendMode::SoftLight.key()); // readable name in project files
    assert_eq!(Some(BlendMode::SoftLight), BlendMode::from_key("SoftLight"));
}
