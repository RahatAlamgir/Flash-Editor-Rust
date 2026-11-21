//! The app's icons:
//! * line icons converted from the C# `Icons.xaml` (see `paths.rs`), drawn as SVG and tinted to any color;
//! * glyphs from the Windows icon font (see [`glyphs`]), used by the ribbon like in the C# app.
//!
//! Call [`install`] once at start-up so egui can draw SVG images.

pub mod glyphs;
mod paths;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use egui::{Color32, Image, ImageSource, Vec2, load::Bytes};

use crate::theme::metrics;

/// Lets egui load the SVG icons.
pub fn install(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}

/// The SVG path data of a line icon, by its C# name (without "Icon.").
pub fn path_data(name: &str) -> Option<&'static str> {
    paths::ICON_PATHS.binary_search_by(|(n, _)| (*n).cmp(name)).ok().map(|i| paths::ICON_PATHS[i].1)
}

/// All icon names (for tests and the icon picker).
pub fn names() -> impl Iterator<Item = &'static str> {
    paths::ICON_PATHS.iter().map(|(n, _)| *n)
}

/// The full SVG document of an icon: a white 1.4 px stroke with round ends on a 20 x 20 grid (like the
/// C# `IconPath` style). White is tinted to the wanted color when drawn.
pub fn svg_for(name: &str) -> Option<String> {
    path_data(name).map(|d| {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20"><path d="{d}" fill="none" stroke="white" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
        )
    })
}

/// An image of a line icon at `size` points, in `color`. Unknown names draw nothing (and log once in debug).
pub fn icon(name: &str, size: f32, color: Color32) -> Image<'static> {
    let source = ImageSource::Bytes { uri: format!("bytes://fe-icon/{name}.svg").into(), bytes: svg_bytes(name) };
    Image::new(source).fit_to_exact_size(Vec2::splat(size)).tint(color)
}

/// An icon at the normal 20 point size.
pub fn icon20(name: &str, color: Color32) -> Image<'static> {
    icon(name, metrics::ICON, color)
}

/// SVG bytes of each icon, made once and shared (egui caches the drawn texture by its uri).
fn svg_bytes(name: &str) -> Bytes {
    static CACHE: OnceLock<HashMap<&'static str, Arc<[u8]>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        names().filter_map(|n| svg_for(n).map(|svg| (n, Arc::from(svg.into_bytes().into_boxed_slice())))).collect()
    });
    match cache.get(name) {
        Some(bytes) => Bytes::Shared(bytes.clone()),
        None => {
            debug_assert!(false, "unknown icon {name}");
            Bytes::Static(br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"/>"#)
        }
    }
}
