//! Picture file formats: which ones can be opened and saved, and helpful messages when one cannot be opened.

use std::path::Path;

/// Formats Flash Editor can save pictures in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageFileFormat {
    Png,
    Jpeg,
    Webp,
    Ico,
    Bmp,
    Tiff,
    Gif,
}

/// Extensions read directly by Flash Editor.
pub const COMMON_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "webp", "ico", "bmp", "gif", "dds"];

/// Extensions read with help from Windows (some need a free codec from the Microsoft Store). Phase 14.
pub const WINDOWS_EXTENSIONS: [&str; 17] = [
    "tif", "tiff", "jxr", "wdp", "hdp", "heic", "heif", "avif", "jxl", "dng", "cr2", "cr3", "nef", "arw", "orf", "rw2",
    "raf",
];

impl ImageFileFormat {
    /// The format for a file name, from its extension (any case).
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        match extension_of(path.as_ref()).as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "ico" => Some(Self::Ico),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }

    /// The usual extension, with the dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => ".png",
            Self::Jpeg => ".jpg",
            Self::Webp => ".webp",
            Self::Ico => ".ico",
            Self::Bmp => ".bmp",
            Self::Tiff => ".tif",
            Self::Gif => ".gif",
        }
    }

    /// True when the format has a quality setting.
    pub const fn has_quality(self) -> bool {
        matches!(self, Self::Jpeg | Self::Webp)
    }
}

/// True when the file extension is one Flash Editor tries to open as a picture.
pub fn can_open(path: impl AsRef<Path>) -> bool {
    let ext = extension_of(path.as_ref());
    COMMON_EXTENSIONS.contains(&ext.as_str()) || WINDOWS_EXTENSIONS.contains(&ext.as_str())
}

/// A message that says why a file could not be opened and what to do about it.
pub fn cannot_open_hint(extension: Option<&str>) -> String {
    let ext = extension.unwrap_or("").trim_start_matches('.').to_ascii_lowercase();
    let name = if ext.is_empty() { "This file".to_string() } else { format!("This {} file", ext.to_uppercase()) };
    match ext.as_str() {
        "heic" | "heif" => format!(
            "{name} could not be opened. Windows needs the free \"HEIF Image Extensions\" (Microsoft Store) to read HEIC photos. Install it, then try again."
        ),
        "avif" => format!(
            "{name} could not be opened. Windows needs the free \"AV1 Video Extension\" (Microsoft Store) to read AVIF images. Install it, then try again."
        ),
        "jxl" => format!(
            "{name} could not be opened. Windows needs the \"JPEG XL Image Extension\" (Microsoft Store) to read JPEG XL images."
        ),
        "dng" | "cr2" | "cr3" | "nef" | "arw" | "orf" | "rw2" | "raf" => format!(
            "{name} could not be opened. Windows needs the free \"Raw Image Extension\" (Microsoft Store) to read camera RAW files."
        ),
        _ => format!(
            "{name} is not a supported image, or it is damaged. Supported: PNG, JPG, WEBP, ICO, BMP, GIF, TIFF, JPEG XR, DDS (and HEIC, AVIF, RAW when Windows has the codec)."
        ),
    }
}

fn extension_of(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}
