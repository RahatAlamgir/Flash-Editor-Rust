//! Port of the C# `PixelBufferTests` (buffers, hex colors, HSV) plus rectangle math.

mod common;

use common::*;
use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect, color_math};

#[test]
fn fill_sets_every_pixel() {
    let buffer = solid(7, 5, ColorRgba::new(10, 20, 30, 40));
    for y in 0..5 {
        for x in 0..7 {
            assert_eq!(ColorRgba::new(10, 20, 30, 40), buffer.pixel(x, y));
        }
    }
}

#[test]
fn crop_outside_bounds_is_transparent() {
    let crop = solid(4, 4, ColorRgba::WHITE).crop(PixelRect::new(2, 2, 4, 4));
    assert_eq!(ColorRgba::WHITE, crop.pixel(0, 0));
    assert_eq!(ColorRgba::WHITE, crop.pixel(1, 1));
    assert_eq!(ColorRgba::TRANSPARENT, crop.pixel(2, 2));
}

#[test]
fn copy_from_clips_negative_destination() {
    let source = numbered(4, 4);
    let mut target = PixelBuffer::new(4, 4);
    target.copy_all_from(&source, -1, -1);
    assert_eq!(source.pixel(1, 1), target.pixel(0, 0));
    assert_eq!(source.pixel(3, 3), target.pixel(2, 2));
    assert_eq!(ColorRgba::TRANSPARENT, target.pixel(3, 3));
}

#[test]
fn visible_bounds_finds_the_content() {
    let mut buffer = PixelBuffer::new(10, 10);
    assert!(buffer.visible_bounds().is_empty());
    buffer.set_pixel(2, 3, ColorRgba::BLACK);
    buffer.set_pixel(7, 5, ColorRgba::BLACK);
    assert_eq!(PixelRect::new(2, 3, 6, 3), buffer.visible_bounds());
}

#[test]
fn hex_round_trip() {
    let c = ColorRgba::parse_hex("#3B82F6").expect("valid hex");
    assert_eq!(ColorRgba::rgb(0x3B, 0x82, 0xF6), c);
    assert_eq!("#3B82F6", c.to_hex());
    assert_eq!(Some(ColorRgba::rgb(255, 0, 0)), ColorRgba::parse_hex("f00"));
    assert_eq!(None, ColorRgba::parse_hex("nope"));
    assert_eq!("#01020380", ColorRgba::new(1, 2, 3, 0x80).to_hex());
}

#[test]
fn hsv_round_trip() {
    let original = ColorRgba::rgb(200, 120, 40);
    let (h, s, v) = color_math::rgb_to_hsv(200.0 / 255.0, 120.0 / 255.0, 40.0 / 255.0);
    assert_eq!(original, color_math::from_hsv(h, s, v, 255));
}

#[test]
fn hsl_round_trip() {
    let (h, s, l) = color_math::rgb_to_hsl(0.8, 0.4, 0.2);
    let (r, g, b) = color_math::hsl_to_rgb(h, s, l);
    assert!((r - 0.8).abs() < 1e-9 && (g - 0.4).abs() < 1e-9 && (b - 0.2).abs() < 1e-9);
}

#[test]
fn rect_math() {
    let a = PixelRect::new(0, 0, 10, 10);
    let b = PixelRect::new(5, 5, 10, 10);
    assert_eq!(PixelRect::new(5, 5, 5, 5), a.intersect(&b));
    assert_eq!(PixelRect::new(0, 0, 15, 15), a.union(&b));
    assert!(a.intersect(&PixelRect::new(20, 20, 2, 2)).is_empty());
    assert_eq!(PixelRect::new(-1, 0, 3, 2), PixelRect::from_bounds(-0.5, 0.0, 1.2, 1.9));
}
