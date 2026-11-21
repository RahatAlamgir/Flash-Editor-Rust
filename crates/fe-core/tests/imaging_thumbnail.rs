//! Port of the C# `ThumbnailTests`.

use fe_core::imaging::{ColorRgba, PixelBuffer, create_thumbnail, fit_size};

#[test]
fn thumbnail_keeps_the_aspect_ratio_and_never_grows_the_picture() {
    assert_eq!((96, 54), fit_size(1920, 1080, 96, 96));
    assert_eq!((40, 20), fit_size(40, 20, 96, 96));
    assert_eq!((1, 96), fit_size(1, 4000, 96, 96));
}

#[test]
fn thumbnail_shows_where_the_content_is_and_its_color() {
    let mut picture = PixelBuffer::new(400, 200);
    for y in 0..100 {
        for x in 0..200 {
            picture.set_pixel(x, y, ColorRgba::rgb(255, 0, 0));
        }
    }
    let thumb = create_thumbnail(&picture, 40, 40);
    assert_eq!((40, 20), (thumb.width(), thumb.height()));
    assert_eq!(ColorRgba::rgb(255, 0, 0), thumb.pixel(5, 5));
    assert_eq!(0, thumb.pixel(30, 15).a);
}

#[test]
fn half_transparent_edges_do_not_darken_the_color() {
    let mut picture = PixelBuffer::new(10, 10);
    for y in 0..10 {
        for x in 0..5 {
            picture.set_pixel(x, y, ColorRgba::rgb(0, 255, 0));
        }
    }
    let pixel = create_thumbnail(&picture, 1, 1).pixel(0, 0);
    assert_eq!(255, pixel.g);
    assert!((120..=135).contains(&pixel.a));
}

#[test]
fn thumbnail_of_a_huge_picture_reads_only_a_few_pixels_per_thumbnail_pixel() {
    let picture = PixelBuffer::filled(6000, 4000, ColorRgba::rgb(10, 20, 30));
    let start = std::time::Instant::now();
    let thumb = create_thumbnail(&picture, 96, 96);
    let took = start.elapsed().as_millis();
    assert_eq!(ColorRgba::rgb(10, 20, 30), thumb.pixel(50, 30));
    assert!(took < 200, "took {took} ms");
}
