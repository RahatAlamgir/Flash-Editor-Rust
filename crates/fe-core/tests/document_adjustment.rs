//! Port of the C# `AdjustmentTests` and `ToneAdjustmentTests` (project-file parts follow in Phase 5).

mod common;

use common::*;
use fe_core::document::{AdjustmentKind, AdjustmentStyle, CurvePoint, Layer, curve};
use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::paint::compositor;

fn adjusted(c: ColorRgba, a: &AdjustmentStyle, strength: f64) -> ColorRgba {
    let mut p = c.to_array();
    a.apply_pixel(&mut p, strength);
    ColorRgba::from_slice(&p)
}

fn rgb(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

fn style(f: impl FnOnce(&mut AdjustmentStyle)) -> AdjustmentStyle {
    let mut s = AdjustmentStyle::default();
    f(&mut s);
    s
}

#[test]
fn a_neutral_adjustment_changes_nothing() {
    let c = rgb(12, 200, 99);
    assert_eq!(c, adjusted(c, &AdjustmentStyle::default(), 1.0));
    assert!(AdjustmentStyle::default().is_neutral());
    assert!(!style(|s| s.hue = 1.0).is_neutral());
}

#[test]
fn brightness_moves_all_channels_and_contrast_pushes_away_from_grey() {
    assert!(in_range(adjusted(rgb(100, 100, 100), &style(|s| s.brightness = 20.0), 1.0).r, 124, 128));
    assert!(in_range(adjusted(rgb(100, 100, 100), &style(|s| s.brightness = -20.0), 1.0).r, 73, 76));
    assert!(adjusted(rgb(200, 200, 200), &style(|s| s.contrast = 50.0), 1.0).r > 200);
    assert!(adjusted(rgb(50, 50, 50), &style(|s| s.contrast = 50.0), 1.0).r < 50);
    assert!(in_range(adjusted(rgb(128, 128, 128), &style(|s| s.contrast = 80.0), 1.0).r, 126, 130));
    assert!(in_range(adjusted(rgb(255, 255, 255), &style(|s| s.contrast = -100.0), 1.0).r, 126, 130));
}

#[test]
fn saturation_and_hue_change_the_color_not_the_brightness_much() {
    let grey = adjusted(rgb(200, 100, 50), &style(|s| s.saturation = -100.0), 1.0);
    assert!(grey.r.abs_diff(grey.g) <= 1 && grey.g.abs_diff(grey.b) <= 1);
    let vivid = adjusted(rgb(180, 120, 100), &style(|s| s.saturation = 80.0), 1.0);
    assert!(vivid.r as i32 - vivid.b as i32 > 80);
    let turned = adjusted(rgb(255, 0, 0), &style(|s| s.hue = 120.0), 1.0);
    assert!(turned.g > 250 && turned.r < 5 && turned.b < 5);
}

#[test]
fn strength_mixes_with_the_original_and_transparent_pixels_stay_transparent() {
    assert!(in_range(adjusted(rgb(100, 100, 100), &style(|s| s.brightness = 40.0), 0.5).r, 124, 128));
    let mut buffer = PixelBuffer::new(2, 1);
    buffer.set_pixel(0, 0, rgb(100, 100, 100));
    let bounds = buffer.bounds();
    style(|s| s.brightness = 40.0).apply_buffer(&mut buffer, bounds, 1.0);
    assert!(buffer.pixel(0, 0).r > 100);
    assert_eq!(ColorRgba::TRANSPARENT, buffer.pixel(1, 0));
}

#[test]
fn an_adjustment_layer_changes_everything_below_it_and_nothing_above() {
    let back = solid_layer("Back", 6, 6, rgb(100, 100, 100));
    let adjust = Layer::adjustment("Adjust", style(|s| s.brightness = 20.0));
    let top = solid_layer("Top", 6, 6, rgb(10, 20, 30));
    let mut doc = doc_with(6, 6, vec![back, adjust, top]);
    assert_eq!(rgb(10, 20, 30), compositor::flatten(&doc).pixel(2, 2));
    doc.layers_mut()[2].visible = false;
    let flat = compositor::flatten(&doc);
    assert!(in_range(flat.pixel(2, 2).r, 124, 128));
    assert_eq!(flat.pixel(2, 2), compositor::sample_merged(&doc, 2, 2));
    let mut target = solid(6, 6, rgb(1, 2, 3));
    compositor::flatten_region(&doc, PixelRect::new(1, 1, 2, 2), &mut target);
    assert_eq!(flat.pixel(2, 2), target.pixel(2, 2));
    assert_eq!(rgb(1, 2, 3), target.pixel(5, 5));
    doc.layers_mut()[1].set_opacity(0.5); // opacity = strength
    assert!(in_range(compositor::flatten(&doc).pixel(2, 2).r, 111, 115));
    doc.layers_mut()[1].visible = false;
    assert_eq!(rgb(100, 100, 100), compositor::flatten(&doc).pixel(2, 2));
}

#[test]
fn an_adjustment_layer_is_copied_and_baked_by_merge_down() {
    let adjust = Layer::adjustment(
        "Adjust",
        style(|s| {
            s.brightness = 20.0;
            s.hue = 10.0;
        }),
    );
    assert!(adjust.is_adjustment());
    assert_eq!(adjust.adjustment, adjust.duplicate("copy").adjustment);
    let mut below = solid(6, 6, rgb(100, 100, 100));
    compositor::draw_layer(&mut below, &adjust); // what Merge Down does
    assert!(below.pixel(1, 1).r > 100);
}

#[test]
fn a_new_levels_or_curves_layer_changes_nothing() {
    assert!(AdjustmentStyle::new_levels().is_neutral());
    assert!(AdjustmentStyle::new_curves().is_neutral());
    assert_eq!(rgb(10, 120, 250), adjusted(rgb(10, 120, 250), &AdjustmentStyle::new_curves(), 1.0));
}

#[test]
fn levels_black_and_white_points_stretch_the_tones() {
    let s = style(|s| {
        s.kind = AdjustmentKind::Levels;
        s.levels_black = 50.0;
        s.levels_white = 200.0;
    });
    assert_eq!(rgb(0, 0, 0), adjusted(rgb(40, 50, 20), &s, 1.0));
    assert_eq!(rgb(255, 255, 255), adjusted(rgb(200, 220, 255), &s, 1.0));
    assert!(in_range(adjusted(rgb(125, 125, 125), &s, 1.0).r, 126, 129));
}

#[test]
fn levels_gamma_lightens_or_darkens_the_middle_tones_but_not_the_ends() {
    let lighter = style(|s| s.levels_gamma = 2.0);
    let darker = style(|s| s.levels_gamma = 0.5);
    assert!(adjusted(rgb(100, 100, 100), &lighter, 1.0).r > 100);
    assert!(adjusted(rgb(100, 100, 100), &darker, 1.0).r < 100);
    assert_eq!(0, adjusted(rgb(0, 0, 0), &lighter, 1.0).r);
    assert_eq!(255, adjusted(rgb(255, 255, 255), &darker, 1.0).r);
}

#[test]
fn levels_output_range_lifts_black_and_dims_white() {
    let s = style(|s| {
        s.output_black = 30.0;
        s.output_white = 220.0;
    });
    assert_eq!(30, adjusted(rgb(0, 0, 0), &s, 1.0).r);
    assert_eq!(220, adjusted(rgb(255, 255, 255), &s, 1.0).r);
}

#[test]
fn a_curve_through_points_bends_the_tones_smoothly() {
    let points = [CurvePoint::new(0.0, 0.0), CurvePoint::new(128.0, 180.0), CurvePoint::new(255.0, 255.0)];
    let table = curve::build_table(Some(&points));
    assert_eq!(0, table[0]);
    assert_eq!(255, table[255]);
    assert!(in_range(table[128], 178, 182));
    assert!(table.windows(2).all(|w| w[1] >= w[0]), "the curve never goes down here");
}

#[test]
fn a_curve_never_overshoots_between_its_points() {
    let steep = [
        CurvePoint::new(0.0, 0.0),
        CurvePoint::new(100.0, 10.0),
        CurvePoint::new(120.0, 250.0),
        CurvePoint::new(255.0, 255.0),
    ];
    let table = curve::build_table(Some(&steep));
    assert!(table.windows(2).all(|w| w[1] >= w[0]));
}

#[test]
fn a_curve_with_lowered_ends_makes_the_picture_flatter() {
    let s = style(|s| {
        s.kind = AdjustmentKind::Curves;
        s.curve = Some(vec![CurvePoint::new(0.0, 40.0), CurvePoint::new(255.0, 215.0)]);
    });
    assert_eq!(40, adjusted(rgb(0, 0, 0), &s, 1.0).r);
    assert_eq!(215, adjusted(rgb(255, 255, 255), &s, 1.0).r);
}

#[test]
fn a_channel_curve_changes_only_its_channel() {
    let s = style(|s| s.curve_red = Some(vec![CurvePoint::new(0.0, 100.0), CurvePoint::new(255.0, 255.0)]));
    assert_eq!(rgb(100, 50, 90), adjusted(rgb(0, 50, 90), &s, 1.0));
}

#[test]
fn points_can_be_added_and_removed_but_not_the_end_points() {
    let c = curve::with_point(None, CurvePoint::new(100.0, 150.0));
    assert_eq!(3, c.len());
    assert_eq!(CurvePoint::new(100.0, 150.0), c[1]);
    let moved = curve::with_point(Some(&c), CurvePoint::new(101.0, 160.0));
    assert_eq!(3, moved.len());
    assert_eq!(160.0, moved[1].y);
    assert_eq!(2, curve::without_point(&moved, 1).len());
    assert_eq!(3, curve::without_point(&moved, 0).len());
    assert_eq!(3, curve::without_point(&moved, 2).len());
}

#[test]
fn the_strength_of_an_adjustment_layer_still_blends_the_tone_change() {
    let s = style(|s| s.levels_gamma = 2.0);
    let half = adjusted(rgb(100, 100, 100), &s, 0.5).r;
    let full = adjusted(rgb(100, 100, 100), &s, 1.0).r;
    assert!((100..=full).contains(&half));
    assert_ne!(100, half);
}
