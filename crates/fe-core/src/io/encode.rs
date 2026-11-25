//! Writes a [`PixelBuffer`] as PNG, JPG, WEBP, BMP, TIFF, ICO or GIF.

use std::io::Cursor;
use std::path::Path;

use image::codecs::{bmp::BmpEncoder, jpeg::JpegEncoder, png::PngEncoder, tiff::TiffEncoder, webp::WebPEncoder};
use image::{ExtendedColorType, ImageEncoder};

use super::{ImageFileFormat, IoError};
use crate::imaging::{ColorRgba, PixelBuffer, over_pixel};

/// Encodes a picture. `quality` (1..100) is used by JPG; `matte` is the color behind transparent areas in
/// formats without transparency (JPG), white when `None`.
///
/// `quality` also applies to WEBP (100 = lossless).
pub fn encode(
    image: &PixelBuffer,
    format: ImageFileFormat,
    quality: u8,
    matte: Option<ColorRgba>,
) -> Result<Vec<u8>, IoError> {
    let quality = quality.clamp(1, 100);
    let (w, h) = (image.width(), image.height());
    let mut out = Vec::new();
    let failed = |e: image::ImageError| IoError::CannotSave(format!("Could not save the picture: {e}"));
    match format {
        ImageFileFormat::Png => {
            PngEncoder::new(&mut out).write_image(image.data(), w, h, ExtendedColorType::Rgba8).map_err(failed)?
        }
        ImageFileFormat::Jpeg => {
            let rgb = flatten_to_rgb(image, matte.unwrap_or(ColorRgba::WHITE));
            JpegEncoder::new_with_quality(&mut out, quality)
                .write_image(&rgb, w, h, ExtendedColorType::Rgb8)
                .map_err(failed)?
        }
        ImageFileFormat::Webp => {
            if quality >= 100 {
                WebPEncoder::new_lossless(&mut out)
                    .write_image(image.data(), w, h, ExtendedColorType::Rgba8)
                    .map_err(failed)?
            } else {
                let lossy = webp::Encoder::from_rgba(image.data(), w, h).encode(f32::from(quality));
                out.extend_from_slice(&lossy);
            }
        }
        ImageFileFormat::Bmp => {
            BmpEncoder::new(&mut out).write_image(image.data(), w, h, ExtendedColorType::Rgba8).map_err(failed)?
        }
        ImageFileFormat::Tiff => {
            let mut cursor = Cursor::new(&mut out);
            TiffEncoder::new(&mut cursor).write_image(image.data(), w, h, ExtendedColorType::Rgba8).map_err(failed)?
        }
        ImageFileFormat::Ico => return super::ico::encode_ico(image),
        ImageFileFormat::Gif => return super::gif::encode_gif(image),
    }
    Ok(out)
}

/// Encodes a picture as lossless PNG (projects and the clipboard use this).
pub fn encode_png(image: &PixelBuffer) -> Result<Vec<u8>, IoError> {
    encode(image, ImageFileFormat::Png, 100, None)
}

/// Encodes and writes a picture. Writes to a temporary file first, so a failed save never damages an
/// existing file.
pub fn save(
    image: &PixelBuffer,
    path: impl AsRef<Path>,
    format: ImageFileFormat,
    quality: u8,
    matte: Option<ColorRgba>,
) -> Result<(), IoError> {
    let path = path.as_ref();
    let bytes = encode(image, format, quality, matte)?;
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    Ok(())
}

/// The picture painted over a solid color, as RGB bytes.
fn flatten_to_rgb(image: &PixelBuffer, matte: ColorRgba) -> Vec<u8> {
    let background = matte.with_alpha(255).to_array();
    let mut rgb = Vec::with_capacity(image.data().len() / 4 * 3);
    for p in image.data().as_chunks::<4>().0.iter() {
        let mut px = background;
        over_pixel(&mut px, p, 1.0);
        rgb.extend_from_slice(&px[..3]);
    }
    rgb
}
