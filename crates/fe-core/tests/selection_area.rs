//! Port of the C# `SelectionTests` and `MaskSelectionTests`, plus checks for the Rust-only parts (masks that
//! only cover the selection's bounds, Bezier paths, magic wand, quick select).

use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect, PointD};
use fe_core::selection::bezier::{Anchor, path_points};
use fe_core::selection::{CombineMode, Selection, magic_wand, outline, quick_select};

#[test]
fn rectangle_selection_has_exact_bounds() {
    let s = Selection::from_rect(PixelRect::new(2, 3, 4, 5), 20, 20).unwrap();
    assert_eq!(PixelRect::new(2, 3, 4, 5), s.bounds());
    assert!(s.is_selected(2, 3));
    assert!(s.is_selected(5, 7));
    assert!(!s.is_selected(6, 7));
    assert!(s.is_rectangle());
}

#[test]
fn selection_outside_the_canvas_is_none() {
    assert!(Selection::from_rect(PixelRect::new(50, 50, 5, 5), 20, 20).is_none());
}

#[test]
fn triangle_polygon_fills_inside_only() {
    let triangle = [PointD::new(0.0, 0.0), PointD::new(10.0, 0.0), PointD::new(0.0, 10.0)];
    let s = Selection::from_polygon(&triangle, 10, 10).unwrap();
    assert!(s.is_selected(1, 1));
    assert!(!s.is_selected(8, 8));
}

#[test]
fn invert_swaps_selected_and_unselected() {
    let s = Selection::from_rect(PixelRect::new(0, 0, 5, 10), 10, 10).unwrap().invert().unwrap();
    assert!(!s.is_selected(0, 0));
    assert!(s.is_selected(5, 0));
    assert_eq!(PixelRect::new(5, 0, 5, 10), s.bounds());
    assert!(Selection::all(10, 10).invert().is_none());
}

#[test]
fn offset_moves_the_mask() {
    let moved = Selection::from_rect(PixelRect::new(0, 0, 2, 2), 10, 10).unwrap().offset(3, 4).unwrap();
    assert_eq!(PixelRect::new(3, 4, 2, 2), moved.bounds());
    // Half off the canvas: the part outside is dropped.
    let half = Selection::from_rect(PixelRect::new(0, 0, 4, 4), 10, 10).unwrap().offset(8, 0).unwrap();
    assert_eq!(PixelRect::new(8, 0, 2, 4), half.bounds());
    assert!(Selection::from_rect(PixelRect::new(0, 0, 4, 4), 10, 10).unwrap().offset(20, 0).is_none());
}

#[test]
fn square_mask_outline_is_one_four_corner_loop() {
    let mut mask = vec![0u8; 100];
    for y in 2..6 {
        for x in 3..8 {
            mask[y * 10 + x] = 255;
        }
    }
    let loops = outline::trace(&mask, PixelRect::new(0, 0, 10, 10), 128);
    assert_eq!(1, loops.len());
    assert_eq!(4, loops[0].len());
    assert_eq!(3.0, loops[0].iter().map(|p| p.x).fold(f64::MAX, f64::min));
    assert_eq!(8.0, loops[0].iter().map(|p| p.x).fold(f64::MIN, f64::max));
}

#[test]
fn ring_mask_gives_outer_and_inner_loops() {
    let mut mask = vec![0u8; 100];
    for y in 1..9 {
        for x in 1..9 {
            mask[y * 10 + x] = if (4..6).contains(&x) && (4..6).contains(&y) { 0 } else { 255 };
        }
    }
    assert_eq!(2, outline::trace(&mask, PixelRect::new(0, 0, 10, 10), 128).len());
}

#[test]
fn pixels_touching_at_a_corner_trace_cleanly() {
    // A checkerboard of 2x2: every corner is shared, so corners have two edges leaving them.
    let mask = vec![255, 0, 0, 255];
    let loops = outline::trace(&mask, PixelRect::new(0, 0, 2, 2), 128);
    let points: usize = loops.iter().map(Vec::len).sum();
    assert!(!loops.is_empty() && points >= 6);
}

#[test]
fn ellipse_selection_is_round() {
    let s = Selection::from_ellipse(PixelRect::new(0, 0, 20, 20), 20, 20).unwrap();
    assert!(s.is_selected(10, 10));
    assert!(!s.is_selected(0, 0)); // the corner is outside a circle
    assert!(s.is_selected(10, 1));
}

#[test]
fn add_and_subtract_combine_selections() {
    let a = Selection::from_rect(PixelRect::new(0, 0, 10, 10), 30, 30);
    let b = Selection::from_rect(PixelRect::new(5, 5, 10, 10), 30, 30);

    let union = Selection::combine(a.as_ref(), b.as_ref(), CombineMode::Add).unwrap();
    assert!(union.is_selected(1, 1) && union.is_selected(13, 13));
    assert_eq!(PixelRect::new(0, 0, 15, 15), union.bounds());

    let cut = Selection::combine(a.as_ref(), b.as_ref(), CombineMode::Subtract).unwrap();
    assert!(cut.is_selected(1, 1));
    assert!(!cut.is_selected(7, 7));
    assert!(!cut.outline().is_empty());

    assert!(Selection::combine(None, b.as_ref(), CombineMode::Subtract).is_none());
    assert!(Selection::combine(a.as_ref(), None, CombineMode::Add).is_some());
    let replaced = Selection::combine(a.as_ref(), b.as_ref(), CombineMode::Replace).unwrap();
    assert_eq!(PixelRect::new(5, 5, 10, 10), replaced.bounds());
}

#[test]
fn soft_mask_values_are_kept() {
    let mask = vec![0, 128, 255, 0];
    let s = Selection::from_mask(mask, PixelRect::new(2, 2, 2, 2), 10, 10).unwrap();
    assert_eq!(PixelRect::new(2, 2, 2, 2), s.bounds());
    assert_eq!(128, s.value(3, 2));
    assert!((s.coverage(2, 3) - 1.0).abs() < 1e-9);
    assert_eq!(0, s.value(9, 9));
}

#[test]
fn closed_pen_path_with_curves_selects_a_rounded_area() {
    // Four smooth points around (50, 50): close to a circle with radius 40.
    let p = |x: f64, y: f64, hx: f64, hy: f64| Anchor { point: PointD::new(x, y), out: Some(PointD::new(hx, hy)) };
    let anchors =
        [p(50.0, 10.0, 72.0, 10.0), p(90.0, 50.0, 90.0, 72.0), p(50.0, 90.0, 28.0, 90.0), p(10.0, 50.0, 10.0, 28.0)];
    let points = path_points(&anchors, true, 0.25);
    assert!(points.len() > 20);
    let s = Selection::from_polygon(&points, 100, 100).unwrap();
    assert!(s.is_selected(50, 50));
    assert!(s.is_selected(50, 12));
    assert!(!s.is_selected(14, 14)); // a corner of the box is outside the round shape
    // Corner points only: straight lines.
    let corners = [Anchor::corner(PointD::new(0.0, 0.0)), Anchor::corner(PointD::new(10.0, 0.0))];
    assert_eq!(2, path_points(&corners, false, 0.25).len());
}

fn two_colors() -> PixelBuffer {
    // Left half red, right half blue, one red pixel inside the blue half.
    let mut image = PixelBuffer::filled(10, 4, ColorRgba::rgb(255, 0, 0));
    for y in 0..4 {
        for x in 5..10 {
            image.set_pixel(x, y, ColorRgba::rgb(0, 0, 255));
        }
    }
    image.set_pixel(8, 2, ColorRgba::rgb(250, 0, 0));
    image
}

#[test]
fn magic_wand_selects_connected_similar_colors() {
    let image = two_colors();
    let mask = magic_wand::similar_area(&image, 1, 1, 32, true);
    let selected = mask.iter().filter(|m| **m == 255).count();
    assert_eq!(20, selected); // the left half only
    let everywhere = magic_wand::similar_area(&image, 1, 1, 32, false);
    assert_eq!(21, everywhere.iter().filter(|m| **m == 255).count()); // plus the lone red pixel
    let exact = magic_wand::similar_area(&image, 1, 1, 0, false);
    assert_eq!(20, exact.iter().filter(|m| **m == 255).count());
}

#[test]
fn quick_select_grows_into_similar_colors_only() {
    let image = two_colors();
    let mut mask = vec![0u8; 40];
    let changed = quick_select::add_dab(&image, &mut mask, 1.5, 1.5, 1.0, 32);
    assert!(!changed.is_empty());
    assert_eq!(255, mask[4]); // (4, 0): red, reached by growing
    assert_eq!(0, mask[5]); // (5, 0): blue
}
