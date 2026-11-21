//! Port of the C# `ImageIoTests` (ICO and GIF saving follow in Phase 14).

mod common;

use common::*;
use fe_core::imaging::{ColorRgba, PixelBuffer};
use fe_core::io::{ImageFileFormat, IoError, decode, encode, format};

fn round_trip(image: &PixelBuffer, fmt: ImageFileFormat, quality: u8) -> PixelBuffer {
    let bytes = encode::encode(image, fmt, quality, None).expect("encodes");
    decode::load_bytes(&bytes, None).expect("decodes")
}

#[test]
fn png_round_trip_is_lossless_including_transparency() {
    let mut image = numbered(16, 16);
    image.set_pixel(3, 3, ColorRgba::new(10, 20, 30, 128));
    assert_eq!(image, round_trip(&image, ImageFileFormat::Png, 100));
}

#[test]
fn webp_lossless_round_trip() {
    let image = numbered(16, 16);
    let decoded = round_trip(&image, ImageFileFormat::Webp, 100);
    assert_eq!((16, 16), (decoded.width(), decoded.height()));
    assert_eq!(image.pixel(5, 7), decoded.pixel(5, 7));
}

#[test]
fn bmp_and_tiff_round_trip() {
    let image = numbered(9, 7);
    assert_eq!(image, round_trip(&image, ImageFileFormat::Bmp, 100));
    assert_eq!(image, round_trip(&image, ImageFileFormat::Tiff, 100));
}

#[test]
fn jpeg_has_no_transparency() {
    let image = PixelBuffer::new(8, 8); // fully transparent
    let p = round_trip(&image, ImageFileFormat::Jpeg, 90).pixel(4, 4);
    assert_eq!(255, p.a);
    assert!(p.r > 240); // transparent became white
}

#[test]
fn formats_are_detected_from_extension() {
    assert_eq!(Some(ImageFileFormat::Jpeg), ImageFileFormat::from_path("photo.JPEG"));
    assert_eq!(Some(ImageFileFormat::Webp), ImageFileFormat::from_path(r"c:\x\a.webp"));
    assert_eq!(Some(ImageFileFormat::Ico), ImageFileFormat::from_path("app.ico"));
    assert_eq!(None, ImageFileFormat::from_path("notes.txt"));
    assert!(format::can_open("x.HEIC"));
    assert!(!format::can_open("x.txt"));
}

#[test]
fn invalid_file_gives_a_clear_error() {
    let err = decode::load_bytes(b"this is not a picture", Some("png")).expect_err("not a picture");
    assert!(matches!(err, IoError::CannotOpen(_)));
    assert!(err.to_string().contains("PNG"));
    let heic = decode::load_bytes(b"nope", Some(".heic")).expect_err("not a picture");
    assert!(heic.to_string().contains("HEIF Image Extensions"));
}

#[test]
fn too_large_pictures_are_refused_with_a_clear_message() {
    let err = decode::check_size(20_000, 10).expect_err("too large");
    assert!(err.to_string().contains("too large"));
}

#[test]
fn save_writes_the_file_and_leaves_no_temporary_file() {
    let dir = std::env::temp_dir().join(format!("fe-io-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("out.png");
    let image = numbered(5, 5);
    encode::save(&image, &path, ImageFileFormat::Png, 100, None).expect("saves");
    assert_eq!(image, decode::load(&path).expect("loads"));
    assert!(!dir.join("out.png.tmp").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_photo_with_exif_rotation_is_turned_upright() {
    // A 2 x 1 JPEG whose EXIF says "rotate 90° clockwise": it must open as 1 x 2.
    let rotated = jpeg_with_orientation(6);
    let image = decode::load_bytes(&rotated, Some("jpg")).expect("decodes");
    assert_eq!((1, 2), (image.width(), image.height()));
}

/// Encodes a 2 x 1 JPEG and inserts an EXIF block with the given orientation.
fn jpeg_with_orientation(orientation: u8) -> Vec<u8> {
    let plain =
        encode::encode(&solid(2, 1, ColorRgba::rgb(200, 10, 10)), ImageFileFormat::Jpeg, 95, None).expect("encodes");
    // TIFF header (little endian) with one IFD entry: tag 0x0112 (orientation), type SHORT, count 1.
    let mut tiff = vec![b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0];
    tiff.extend_from_slice(&[orientation, 0, 0, 0, 0, 0, 0, 0]);
    let mut exif = b"Exif\0\0".to_vec();
    exif.extend_from_slice(&tiff);
    let len = (exif.len() + 2) as u16;
    let mut out = plain[..2].to_vec(); // SOI
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&exif);
    out.extend_from_slice(&plain[2..]);
    out
}
