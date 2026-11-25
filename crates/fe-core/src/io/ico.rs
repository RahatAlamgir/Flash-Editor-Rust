//! Windows .ico files (C# `IcoEncoder`): every size is stored as an embedded PNG; a picture that is not square is
//! centered on a transparent square so it is not stretched.

use super::{IoError, encode};
use crate::imaging::PixelBuffer;
use crate::transform::{ResampleMode, resize};

/// The sizes an icon holds.
pub const SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

fn make_square(image: &PixelBuffer) -> PixelBuffer {
    let side = image.width().max(image.height());
    if image.width() == image.height() {
        return image.clone();
    }
    let mut square = PixelBuffer::new(side, side);
    square.copy_all_from(image, ((side - image.width()) / 2) as i32, ((side - image.height()) / 2) as i32);
    square
}

/// The bytes of an .ico file with the sizes 16 to 256.
pub fn encode_ico(image: &PixelBuffer) -> Result<Vec<u8>, IoError> {
    let square = make_square(image);
    let mut pngs = Vec::new();
    for size in SIZES {
        let small = resize(&square, size, size, ResampleMode::Smooth);
        pngs.push((size, encode::encode_png(&small)?));
    }
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    out.extend_from_slice(&(pngs.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * pngs.len() as u32;
    for (size, png) in &pngs {
        let byte = if *size >= 256 { 0 } else { *size as u8 }; // 0 means 256
        out.extend_from_slice(&[byte, byte, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes()); // color planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for (_, png) in pngs {
        out.extend_from_slice(&png);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn an_icon_holds_every_size_and_reads_back() {
        let bytes = encode_ico(&PixelBuffer::filled(40, 20, ColorRgba::rgb(200, 30, 30))).expect("encoded");
        assert_eq!(&bytes[..4], &[0, 0, 1, 0]);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]) as usize, SIZES.len());
        let back = crate::io::decode::load_bytes(&bytes, Some("ico")).expect("readable");
        assert!(back.width() >= 16);
    }
}
