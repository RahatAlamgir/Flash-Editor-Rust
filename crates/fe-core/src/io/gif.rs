//! GIF files: one picture (256 colors) or an animation with one frame per page (C# `GifEncoder`). Every frame has its
//! own palette; transparency is fully clear or fully solid.

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};

use super::IoError;
use crate::imaging::PixelBuffer;
use crate::transform::{ResampleMode, resize};

fn failed(e: image::ImageError) -> IoError {
    IoError::CannotSave(format!("Could not save the GIF: {e}"))
}

/// A GIF of several pictures shown one after another. `delay_ms` is the time per frame; `repeat` loops forever.
/// Frames of another size are resized to the size of the first.
pub fn encode_animation(frames: &[PixelBuffer], delay_ms: u32, repeat: bool) -> Result<Vec<u8>, IoError> {
    let Some(first) = frames.first() else {
        return Err(IoError::CannotSave("An animation needs at least one frame.".into()));
    };
    let (w, h) = (first.width(), first.height());
    let mut out = Vec::new();
    {
        let mut encoder = GifEncoder::new_with_speed(&mut out, 10);
        encoder.set_repeat(if repeat { Repeat::Infinite } else { Repeat::Finite(0) }).map_err(failed)?;
        for frame in frames {
            let same = if (frame.width(), frame.height()) == (w, h) {
                frame.clone()
            } else {
                resize(frame, w, h, ResampleMode::Smooth)
            };
            let mut picture = same.into_raw();
            for p in picture.chunks_exact_mut(4) {
                // Soft transparency does not exist in GIF: clear or solid.
                p[3] = if p[3] < 128 { 0 } else { 255 };
            }
            let rgba =
                RgbaImage::from_raw(w, h, picture).ok_or_else(|| IoError::CannotSave("Bad frame size.".into()))?;
            let delay = Delay::from_numer_denom_ms(delay_ms.clamp(10, 655_350), 1);
            encoder.encode_frame(Frame::from_parts(rgba, 0, 0, delay)).map_err(failed)?;
        }
    }
    Ok(out)
}

/// A single-picture GIF.
pub fn encode_gif(image: &PixelBuffer) -> Result<Vec<u8>, IoError> {
    encode_animation(std::slice::from_ref(image), 1000, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn a_gif_reads_back_with_its_size_and_a_frame_per_page() {
        let a = PixelBuffer::filled(30, 20, ColorRgba::rgb(255, 0, 0));
        let b = PixelBuffer::filled(30, 20, ColorRgba::rgb(0, 0, 255));
        let bytes = encode_animation(&[a, b], 500, true).expect("encoded");
        assert_eq!(&bytes[..6], b"GIF89a");
        let back = crate::io::decode::load_bytes(&bytes, Some("gif")).expect("readable");
        assert_eq!((back.width(), back.height()), (30, 20));
        assert!(back.pixel(5, 5).r > 200, "the first frame is shown");
    }
}
