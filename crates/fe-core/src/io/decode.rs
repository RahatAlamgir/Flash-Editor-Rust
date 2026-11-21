//! Reads pictures (PNG, JPG, WEBP, BMP, GIF, ICO, TIFF, DDS) into a [`PixelBuffer`].
//! Photos are turned upright using their EXIF orientation. HEIC / AVIF / RAW via Windows come in Phase 14.

use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageReader};

use super::IoError;
use super::format::cannot_open_hint;
use crate::imaging::PixelBuffer;

/// The largest width or height the editor accepts.
pub const MAX_DIMENSION: u32 = 16384;

/// Reads a picture file.
pub fn load(path: impl AsRef<Path>) -> Result<PixelBuffer, IoError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)?;
    load_bytes(&bytes, path.extension().and_then(|e| e.to_str()))
}

/// Reads a picture from bytes. `extension` (like "heic") only improves the error message.
pub fn load_bytes(bytes: &[u8], extension: Option<&str>) -> Result<PixelBuffer, IoError> {
    let cannot = || IoError::CannotOpen(cannot_open_hint(extension));
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|_| cannot())?;
    if reader.format().is_none() {
        return Err(cannot());
    }
    reader.no_limits(); // our own size check below
    let mut decoder = reader.into_decoder().map_err(|_| cannot())?;
    let (width, height) = decoder.dimensions();
    check_size(width, height)?;
    let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut picture = DynamicImage::from_decoder(decoder).map_err(|_| cannot())?;
    picture.apply_orientation(orientation);
    let rgba = picture.into_rgba8();
    let (w, h) = rgba.dimensions();
    PixelBuffer::from_raw(w, h, rgba.into_raw()).ok_or_else(cannot)
}

/// Fails with a clear message when a picture is too large to edit.
pub fn check_size(width: u32, height: u32) -> Result<(), IoError> {
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(IoError::TooLarge { width, height, max: MAX_DIMENSION });
    }
    if width == 0 || height == 0 {
        return Err(IoError::CannotOpen("The picture has no pixels.".into()));
    }
    Ok(())
}
