//! Reads pictures through Windows' own picture codecs (WIC), like the C# version: HEIC, AVIF, JPEG XL, camera RAW and
//! JPEG XR open when Windows has the codec (some need a free extension from the Microsoft Store). Windows only.

use std::path::Path;

use super::IoError;
use super::format::cannot_open_hint;
use crate::imaging::PixelBuffer;

/// Reads a picture with the Windows codecs. `None` when this is not Windows or the codec is not there.
#[cfg(windows)]
#[allow(unsafe_code)] // calls into the Windows imaging API
pub fn load(path: &Path) -> Option<PixelBuffer> {
    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory, WICBitmapDitherTypeNone,
        WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx};
    use windows::core::HSTRING;

    // SAFETY: plain calls into COM / WIC with valid arguments; every object is released when it goes out of scope.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED); // "already started" is fine
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let name = HSTRING::from(path.as_os_str());
        let decoder =
            factory.CreateDecoderFromFilename(&name, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand).ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let converter = factory.CreateFormatConverter().ok()?;
        converter
            .Initialize(
                &frame,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .ok()?;
        let (mut w, mut h) = (0u32, 0u32);
        converter.GetSize(&mut w, &mut h).ok()?;
        if super::decode::check_size(w, h).is_err() {
            return None;
        }
        let mut data = vec![0u8; w as usize * h as usize * 4];
        converter.CopyPixels(std::ptr::null(), w * 4, &mut data).ok()?;
        PixelBuffer::from_raw(w, h, data)
    }
}

#[cfg(not(windows))]
pub fn load(_path: &Path) -> Option<PixelBuffer> {
    None
}

/// Reads a picture file through Windows, or explains what to install.
pub fn load_or_explain(path: &Path) -> Result<PixelBuffer, IoError> {
    load(path).ok_or_else(|| IoError::CannotOpen(cannot_open_hint(path.extension().and_then(|e| e.to_str()))))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn windows_reads_a_picture_file() {
        let path = std::env::temp_dir().join(format!("fe-wic-{}.png", std::process::id()));
        let picture = PixelBuffer::filled(12, 7, ColorRgba::rgb(200, 100, 50));
        crate::io::encode::save(&picture, &path, crate::io::ImageFileFormat::Png, 90, None).expect("saved");
        let back = load(&path).expect("Windows can read a PNG");
        let _ = std::fs::remove_file(&path);
        assert_eq!((back.width(), back.height()), (12, 7));
        assert_eq!(back.pixel(3, 3), ColorRgba::rgb(200, 100, 50));
    }
}
