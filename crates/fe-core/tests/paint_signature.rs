//! The region fingerprint changes exactly when the look of that region can change.

mod common;

use common::*;
use fe_core::document::{AdjustmentStyle, Layer};
use fe_core::imaging::{BlendMode, ColorRgba, PixelRect};
use fe_core::paint::compositor;
use fe_core::paint::signature::region_signature;

const LEFT: PixelRect = PixelRect::new(0, 0, 256, 256);
const RIGHT: PixelRect = PixelRect::new(256, 0, 256, 256);

fn doc() -> fe_core::document::Document {
    doc_with(512, 256, vec![solid_layer("Back", 512, 256, ColorRgba::WHITE), Layer::new("Top")])
}

#[test]
fn painting_changes_only_the_painted_tile() {
    let mut d = doc();
    let (left, right) = (region_signature(&d, LEFT), region_signature(&d, RIGHT));
    d.layers_mut()[1].pixels_mut().set_pixel(10, 10, ColorRgba::BLACK);
    assert_ne!(left, region_signature(&d, LEFT));
    assert_eq!(right, region_signature(&d, RIGHT));
}

#[test]
fn layer_settings_change_every_tile() {
    let mut d = doc();
    let before = region_signature(&d, RIGHT);
    d.layers_mut()[0].blend = BlendMode::Multiply;
    let blended = region_signature(&d, RIGHT);
    assert_ne!(before, blended);
    d.layers_mut()[0].set_opacity(0.5);
    assert_ne!(blended, region_signature(&d, RIGHT));
}

#[test]
fn hiding_a_layer_or_moving_an_adjustment_slider_changes_it() {
    let mut d = doc();
    d.push_layer(Layer::adjustment("Adjust", AdjustmentStyle::default()));
    let before = region_signature(&d, LEFT);
    if let Some(a) = d.layers_mut()[2].adjustment.as_mut() {
        a.brightness = 10.0;
    }
    let adjusted = region_signature(&d, LEFT);
    assert_ne!(before, adjusted);
    d.layers_mut()[2].visible = false;
    assert_ne!(adjusted, region_signature(&d, LEFT));
}

#[test]
fn a_copy_of_the_document_has_the_same_fingerprint_until_it_changes() {
    let d = doc();
    let copy = d.clone();
    assert_eq!(region_signature(&d, LEFT), region_signature(&copy, LEFT));
}

#[test]
fn composite_region_matches_the_full_flatten() {
    let mut d = doc();
    d.layers_mut()[1].pixels_mut().fill_rect(PixelRect::new(200, 20, 100, 100), ColorRgba::new(10, 200, 30, 128));
    let full = compositor::flatten(&d);
    let part = compositor::composite_region(&d, PixelRect::new(190, 10, 400, 300)).expect("inside the canvas");
    assert_eq!((322, 246), (part.width(), part.height())); // clipped to the canvas
    assert_eq!(full.pixel(250, 50), part.pixel(60, 40));
    assert!(compositor::composite_region(&d, PixelRect::new(600, 0, 10, 10)).is_none());
}
