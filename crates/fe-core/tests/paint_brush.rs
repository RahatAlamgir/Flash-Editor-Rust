//! Port of the C# `BrushTypeTests` and `PaintingTests` (the restore strokes follow with the Refine tool), plus
//! the Mask Brush stroke.

mod common;

use common::*;
use fe_core::imaging::{ColorRgba, PixelRect, PointD, TileGrid, TileMap};
use fe_core::paint::brush::{BrushSettings, BrushType};
use fe_core::paint::bucket;
use fe_core::paint::stroke::{Effect, MaskStroke, PaintStroke};
use fe_core::selection::Selection;

const CANVAS: i32 = 201;
const CENTER: i32 = 100;

fn brush(size: f64, hardness: f64, opacity: f64, kind: BrushType) -> BrushSettings {
    BrushSettings { size, hardness, opacity, kind }
}

/// Alpha after one click in the middle of a transparent canvas, as alpha(x, y).
fn dab(kind: BrushType, size: f64, hardness: f64) -> impl Fn(i32, i32) -> u8 {
    let canvas = PixelRect::new(0, 0, CANVAS, CANVAS);
    let mut pixels = TileMap::empty();
    let mut stroke =
        PaintStroke::new(&pixels, brush(size, hardness, 1.0, kind), Effect::Paint(ColorRgba::BLACK), None, canvas);
    stroke.add_point(&mut pixels, PointD::new(CENTER as f64 + 0.5, CENTER as f64 + 0.5));
    move |x, y| pixels.pixel(x, y).a
}

fn painted(alpha: &impl Fn(i32, i32) -> u8, min: u8) -> usize {
    (0..CANVAS).flat_map(|y| (0..CANVAS).map(move |x| (x, y))).filter(|&(x, y)| alpha(x, y) >= min).count()
}

#[test]
fn a_square_brush_paints_its_corners_a_round_brush_does_not() {
    let round = dab(BrushType::Round, 80.0, 1.0);
    let square = dab(BrushType::Square, 80.0, 1.0);
    assert_eq!(0, round(CENTER + 35, CENTER + 35));
    assert_eq!(255, square(CENTER + 35, CENTER + 35));
    assert!(painted(&square, 1) as f64 > painted(&round, 1) as f64 * 1.2);
    assert_eq!(0, square(CENTER + 45, CENTER));
}

#[test]
fn a_calligraphy_brush_is_thick_one_way_and_thin_the_other() {
    let alpha = dab(BrushType::Calligraphy, 80.0, 1.0);
    assert_eq!(255, alpha(CENTER + 25, CENTER + 25));
    assert_eq!(0, alpha(CENTER + 25, CENTER - 25));
    assert!((painted(&alpha, 1) as f64) < painted(&dab(BrushType::Round, 80.0, 1.0), 1) as f64 * 0.5);
}

#[test]
fn a_pencil_has_hard_edges_and_a_slight_grain() {
    let alpha = dab(BrushType::Pencil, 80.0, 0.0); // hardness is ignored by a pencil
    assert_eq!(0, alpha(CENTER + 43, CENTER));
    let (mut solid, mut partial) = (0, 0);
    for y in CENTER - 20..=CENTER + 20 {
        for x in CENTER - 20..=CENTER + 20 {
            assert!(alpha(x, y) > 100, "inside a pencil dab every pixel is clearly painted");
            if alpha(x, y) > 235 {
                solid += 1;
            } else {
                partial += 1;
            }
        }
    }
    assert!(solid > 0 && partial > 0, "the grain makes the strength vary");
}

#[test]
fn chalk_is_speckled_with_gaps() {
    let alpha = dab(BrushType::Chalk, 80.0, 1.0);
    let (mut inside, mut empty) = (0, 0);
    for y in CENTER - 25..=CENTER + 25 {
        for x in CENTER - 25..=CENTER + 25 {
            inside += 1;
            if alpha(x, y) == 0 {
                empty += 1;
            }
        }
    }
    let share = empty as f64 / inside as f64;
    assert!((0.2..=0.6).contains(&share), "bare share {share}");
}

#[test]
fn spray_is_a_sparse_mist_inside_the_brush_size() {
    let spray = dab(BrushType::Spray, 80.0, 1.0);
    let count = painted(&spray, 1);
    assert!((3..=painted(&dab(BrushType::Round, 80.0, 1.0), 1) / 2).contains(&count));
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            if spray(x, y) > 0 {
                let d = (((x - CENTER).pow(2) + (y - CENTER).pow(2)) as f64).sqrt();
                assert!(d <= 43.0, "dot at distance {d:.1} is outside the brush");
            }
        }
    }
}

#[test]
fn splatter_makes_separate_blobs_that_can_go_slightly_past_the_brush_size() {
    let splatter = dab(BrushType::Splatter, 80.0, 1.0);
    let count = painted(&splatter, 1);
    assert!((5..=painted(&dab(BrushType::Round, 80.0, 1.0), 1)).contains(&count));
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            if splatter(x, y) > 0 {
                assert!((((x - CENTER).pow(2) + (y - CENTER).pow(2)) as f64).sqrt() <= 88.0);
            }
        }
    }
}

#[test]
fn every_type_can_erase_and_stays_inside_the_canvas_near_the_edge() {
    for kind in BrushType::ALL {
        let canvas = PixelRect::new(0, 0, 60, 60);
        let mut pixels = TileMap::from_buffer(&solid(60, 60, ColorRgba::WHITE), 0, 0);
        let mut stroke = PaintStroke::new(&pixels, brush(50.0, 1.0, 1.0, kind), Effect::Erase, None, canvas);
        stroke.add_point(&mut pixels, PointD::new(2.0, 2.0)); // half outside the canvas must not crash
        stroke.add_point(&mut pixels, PointD::new(58.0, 58.0));
        let erased = (0..60).any(|y| (0..60).any(|x| pixels.pixel(x, y).a < 255));
        assert!(erased, "{kind:?} erased nothing");
        assert_eq!(0, pixels.pixel(-3, -3).a, "{kind:?} painted outside the canvas");
    }
}

#[test]
fn overlapping_dabs_do_not_change_the_grain_of_a_pencil_stroke() {
    let canvas = PixelRect::new(0, 0, 120, 40);
    let mut pixels = TileMap::empty();
    let mut stroke = PaintStroke::new(
        &pixels,
        brush(20.0, 1.0, 1.0, BrushType::Pencil),
        Effect::Paint(ColorRgba::BLACK),
        None,
        canvas,
    );
    stroke.add_point(&mut pixels, PointD::new(20.0, 20.0));
    let after_first = pixels.clone();
    stroke.add_point(&mut pixels, PointD::new(20.5, 20.0));
    assert_eq!(after_first.pixel(20, 20), pixels.pixel(20, 20));
    assert_eq!(after_first.pixel(15, 22), pixels.pixel(15, 22));
}

#[test]
fn flood_fill_stops_at_different_colors() {
    let mut image = solid(10, 10, ColorRgba::WHITE);
    for y in 0..10 {
        image.set_pixel(5, y, ColorRgba::BLACK);
    }
    let mut pixels = TileMap::from_buffer(&image, 0, 0);
    let red = ColorRgba::rgb(255, 0, 0);
    bucket::fill(&mut pixels, PixelRect::new(0, 0, 10, 10), (1, 1), red, 10, true, None);
    assert_eq!(red, pixels.pixel(0, 9));
    assert_eq!(ColorRgba::BLACK, pixels.pixel(5, 5));
    assert_eq!(ColorRgba::WHITE, pixels.pixel(8, 8));
    bucket::fill(&mut pixels, PixelRect::new(0, 0, 10, 10), (8, 8), red, 10, false, None);
    assert_eq!(red, pixels.pixel(8, 8));
}

#[test]
fn fill_respects_the_selection() {
    let mut pixels = TileMap::from_buffer(&solid(10, 10, ColorRgba::WHITE), 0, 0);
    let selection = Selection::from_rect(PixelRect::new(0, 0, 3, 3), 10, 10).unwrap();
    bucket::fill(&mut pixels, PixelRect::new(0, 0, 10, 10), (1, 1), ColorRgba::BLACK, 10, true, Some(&selection));
    assert_eq!(ColorRgba::BLACK, pixels.pixel(2, 2));
    assert_eq!(ColorRgba::WHITE, pixels.pixel(4, 4));
    // A click outside the selection fills nothing.
    let changed =
        bucket::fill(&mut pixels, PixelRect::new(0, 0, 10, 10), (6, 6), ColorRgba::BLACK, 10, true, Some(&selection));
    assert!(changed.is_empty());
}

#[test]
fn overlapping_dabs_do_not_build_up_opacity() {
    let canvas = PixelRect::new(0, 0, 40, 40);
    let mut pixels = TileMap::from_buffer(&solid(40, 40, ColorRgba::WHITE), 0, 0);
    let mut stroke = PaintStroke::new(
        &pixels,
        brush(10.0, 1.0, 0.5, BrushType::Round),
        Effect::Paint(ColorRgba::BLACK),
        None,
        canvas,
    );
    for _ in 0..10 {
        stroke.add_point(&mut pixels, PointD::new(10.0, 20.0));
        stroke.add_point(&mut pixels, PointD::new(30.0, 20.0));
    }
    assert!(in_range(pixels.pixel(20, 20).r, 125, 130)); // 50 % black over white
}

#[test]
fn eraser_makes_pixels_transparent_and_the_original_is_kept() {
    let canvas = PixelRect::new(0, 0, 20, 20);
    let mut pixels = TileMap::from_buffer(&solid(20, 20, ColorRgba::WHITE), 0, 0);
    let mut stroke = PaintStroke::new(&pixels, brush(8.0, 1.0, 1.0, BrushType::Round), Effect::Erase, None, canvas);
    stroke.add_point(&mut pixels, PointD::new(10.0, 10.0));
    assert_eq!(0, pixels.pixel(10, 10).a);
    assert_eq!(255, pixels.pixel(0, 0).a);
    assert!(!stroke.total().is_empty());
    assert_eq!(255, stroke.original().pixel(10, 10).a);
}

#[test]
fn painting_inside_a_selection_only() {
    let canvas = PixelRect::new(0, 0, 40, 20);
    let selection = Selection::from_rect(PixelRect::new(0, 0, 20, 20), 40, 20).unwrap();
    let mut pixels = TileMap::empty();
    let mut stroke = PaintStroke::new(
        &pixels,
        brush(10.0, 1.0, 1.0, BrushType::Round),
        Effect::Paint(ColorRgba::BLACK),
        Some(selection),
        canvas,
    );
    stroke.add_point(&mut pixels, PointD::new(5.0, 10.0));
    stroke.add_point(&mut pixels, PointD::new(35.0, 10.0));
    assert_eq!(255, pixels.pixel(10, 10).a);
    assert_eq!(0, pixels.pixel(30, 10).a);
}

#[test]
fn the_mask_brush_moves_mask_values_towards_its_grey() {
    let canvas = PixelRect::new(0, 0, 40, 20);
    let mut mask = TileGrid::<1>::new([255]);
    let mut stroke = MaskStroke::new(&mask, brush(8.0, 1.0, 1.0, BrushType::Round), 0, None, canvas);
    stroke.add_point(&mut mask, PointD::new(10.0, 10.0));
    assert_eq!([0], mask.get(10, 10));
    assert_eq!([255], mask.get(30, 10));
    assert_eq!(128, MaskStroke::grey_of(ColorRgba::rgb(128, 128, 128)));
    let mut half = MaskStroke::new(&mask, brush(8.0, 1.0, 0.5, BrushType::Round), 255, None, canvas);
    half.add_point(&mut mask, PointD::new(10.0, 10.0));
    assert!((126..=129).contains(&mask.get(10, 10)[0]));
}
