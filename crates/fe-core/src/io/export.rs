//! Saving a copy of the picture (C# `ExportSettings`, `ImageExporter`, `ExportPresets`): crop, resize, encode, and the
//! ready-made sizes of "Export For".

use super::{IoError, decode::MAX_DIMENSION, encode, format::ImageFileFormat};
use crate::imaging::{ColorRgba, PixelBuffer, PixelRect};
use crate::transform::{ResampleMode, resize};

/// How to write an image file: format, quality (JPG / WebP) and the size of the file. The open picture is not changed.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportSettings {
    pub format: ImageFileFormat,
    pub quality: u8,
    pub width: u32,
    pub height: u32,
    pub resample: ResampleMode,
    /// The color behind see-through parts, for formats without transparency (JPG).
    pub matte: Option<ColorRgba>,
    /// Cuts out part of the picture first; `width` / `height` are the size of that part in the file.
    pub crop: Option<PixelRect>,
}

impl ExportSettings {
    pub fn new(format: ImageFileFormat, quality: u8, width: u32, height: u32) -> Self {
        Self { format, quality, width, height, resample: ResampleMode::Lanczos, matte: None, crop: None }
    }
}

/// The picture cropped (if asked) and at the export size. The input is not changed.
pub fn resized(image: &PixelBuffer, settings: &ExportSettings) -> PixelBuffer {
    let mut source = image;
    let cropped;
    if let Some(crop) = settings.crop {
        let area = crop.intersect(&image.bounds());
        if crop != image.bounds() && !area.is_empty() {
            cropped = image.crop(area);
            source = &cropped;
        }
    }
    let w = settings.width.clamp(1, MAX_DIMENSION);
    let h = settings.height.clamp(1, MAX_DIMENSION);
    if (w, h) == (source.width(), source.height()) { source.clone() } else { resize(source, w, h, settings.resample) }
}

/// The bytes of the file.
pub fn encode_with(image: &PixelBuffer, settings: &ExportSettings) -> Result<Vec<u8>, IoError> {
    encode::encode(&resized(image, settings), settings.format, settings.quality, settings.matte)
}

/// Writes the file (through a temporary file, so a failure never damages an existing one).
pub fn save_with(
    image: &PixelBuffer,
    path: impl AsRef<std::path::Path>,
    settings: &ExportSettings,
) -> Result<(), IoError> {
    encode::save(&resized(image, settings), path, settings.format, settings.quality, settings.matte)
}

fn trim(value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.').to_owned() } else { text }
}

/// "12.3 KB", "4.56 MB"...
pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    if bytes < 1024 {
        format!("{bytes} bytes")
    } else if bytes < 1024 * 1024 {
        format!("{} KB", trim(bytes as f64 / KB, 1))
    } else {
        format!("{} MB", trim(bytes as f64 / (KB * KB), 2))
    }
}

/// How a picture is made to fit a preset size when its shape is different.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFit {
    /// The whole picture is shown, as big as it can be; the rest is filled with the background.
    Fit,
    /// The preset size is filled completely; what does not fit is cut off (centered).
    Fill,
    /// The picture is stretched to the preset size (its shape changes).
    Stretch,
}

/// A ready-made size for a use of the picture.
#[derive(Debug, PartialEq, Eq)]
pub struct ExportPreset {
    pub group: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}

impl ExportPreset {
    pub fn description(&self) -> String {
        format!("{}  ({} x {})", self.name, self.width, self.height)
    }
}

const fn p(group: &'static str, name: &'static str, width: u32, height: u32) -> ExportPreset {
    ExportPreset { group, name, width, height }
}

/// The ready-made export sizes.
pub const PRESETS: [ExportPreset; 22] = [
    p("Social media", "Instagram post (square)", 1080, 1080),
    p("Social media", "Instagram post (portrait)", 1080, 1350),
    p("Social media", "Instagram / TikTok story", 1080, 1920),
    p("Social media", "Facebook post", 1200, 630),
    p("Social media", "Facebook cover", 820, 312),
    p("Social media", "X (Twitter) post", 1600, 900),
    p("Social media", "X (Twitter) header", 1500, 500),
    p("Social media", "LinkedIn post", 1200, 627),
    p("Social media", "Pinterest pin", 1000, 1500),
    p("Video", "YouTube thumbnail", 1280, 720),
    p("Video", "YouTube channel banner", 2560, 1440),
    p("Video", "Full HD", 1920, 1080),
    p("Video", "4K Ultra HD", 3840, 2160),
    p("Print (300 dpi)", "A4 page", 2480, 3508),
    p("Print (300 dpi)", "A5 page", 1748, 2480),
    p("Print (300 dpi)", "US Letter page", 2550, 3300),
    p("Print (300 dpi)", "Photo 4 x 6 in", 1200, 1800),
    p("Print (300 dpi)", "Business card 3.5 x 2 in", 1050, 600),
    p("Web and apps", "Website banner", 1920, 600),
    p("Web and apps", "Email header", 600, 200),
    p("Web and apps", "App icon", 1024, 1024),
    p("Web and apps", "Favicon", 512, 512),
];

/// Makes a picture exactly `preset` sized from `source`. `background` fills the space around the picture for
/// [`ExportFit::Fit`] (`None` = see-through).
pub fn compose(
    source: &PixelBuffer,
    preset: &ExportPreset,
    fit: ExportFit,
    background: Option<ColorRgba>,
) -> PixelBuffer {
    let (w, h) = (preset.width, preset.height);
    if source.width() == 0 || source.height() == 0 {
        return PixelBuffer::new(w, h);
    }
    let (sw, sh) = (f64::from(source.width()), f64::from(source.height()));
    match fit {
        ExportFit::Stretch => resize(source, w, h, ResampleMode::Lanczos),
        ExportFit::Fill => {
            let scale = (f64::from(w) / sw).max(f64::from(h) / sh);
            let cw = ((f64::from(w) / scale).round_ties_even() as u32).clamp(1, source.width());
            let ch = ((f64::from(h) / scale).round_ties_even() as u32).clamp(1, source.height());
            let crop = PixelRect::new(
                ((source.width() - cw) / 2) as i32,
                ((source.height() - ch) / 2) as i32,
                cw as i32,
                ch as i32,
            );
            resize(&source.crop(crop), w, h, ResampleMode::Lanczos)
        }
        ExportFit::Fit => {
            let scale = (f64::from(w) / sw).min(f64::from(h) / sh);
            let fw = ((sw * scale).round_ties_even() as u32).clamp(1, w);
            let fh = ((sh * scale).round_ties_even() as u32).clamp(1, h);
            let mut canvas = PixelBuffer::new(w, h);
            if let Some(color) = background {
                canvas.fill(color);
            }
            canvas.copy_all_from(
                &resize(source, fw, fh, ResampleMode::Lanczos),
                ((w - fw) / 2) as i32,
                ((h - fh) / 2) as i32,
            );
            canvas
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_sizes_read_like_the_csharp_text() {
        assert_eq!(format_file_size(900), "900 bytes");
        assert_eq!(format_file_size(12_595), "12.3 KB");
        assert_eq!(format_file_size(4_781_507), "4.56 MB");
    }

    #[test]
    fn export_crops_then_resizes_without_touching_the_original() {
        let image = PixelBuffer::filled(100, 80, ColorRgba::rgb(10, 20, 30));
        let mut settings = ExportSettings::new(ImageFileFormat::Png, 90, 40, 30);
        settings.crop = Some(PixelRect::new(10, 10, 50, 50));
        let out = resized(&image, &settings);
        assert_eq!((out.width(), out.height()), (40, 30));
        assert_eq!((image.width(), image.height()), (100, 80));
    }

    #[test]
    fn a_preset_picture_has_exactly_the_preset_size_in_every_fit() {
        let image = PixelBuffer::filled(300, 200, ColorRgba::rgb(200, 0, 0));
        for fit in [ExportFit::Fit, ExportFit::Fill, ExportFit::Stretch] {
            let out = compose(&image, &PRESETS[2], fit, Some(ColorRgba::WHITE));
            assert_eq!((out.width(), out.height()), (1080, 1920));
        }
        let fitted = compose(&image, &PRESETS[2], ExportFit::Fit, Some(ColorRgba::WHITE));
        assert_eq!(fitted.pixel(5, 5), ColorRgba::WHITE, "the bars around the picture use the background");
        assert!(fitted.pixel(540, 960).r > 150);
    }

    #[test]
    fn jpg_and_lossy_webp_get_smaller_with_lower_quality() {
        let mut image = PixelBuffer::new(128, 128);
        for (i, p) in image.data_mut().chunks_exact_mut(4).enumerate() {
            p.copy_from_slice(&[(i * 7 % 256) as u8, (i * 13 % 256) as u8, (i * 29 % 256) as u8, 255]);
        }
        for format in [ImageFileFormat::Jpeg, ImageFileFormat::Webp] {
            let high = encode_with(&image, &ExportSettings::new(format, 95, 128, 128)).expect("encoded").len();
            let low = encode_with(&image, &ExportSettings::new(format, 30, 128, 128)).expect("encoded").len();
            assert!(low < high, "{format:?}: {low} < {high}");
        }
    }
}
