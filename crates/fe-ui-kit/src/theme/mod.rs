//! Colors, fonts, sizes and the egui style built from them.

pub mod colors;
pub mod fonts;
pub mod metrics;
mod style;

/// Applies the Flash Editor look (fonts, colors, spacing) to an egui context. Call once at start-up.
pub fn apply(ctx: &egui::Context) {
    fonts::install(ctx);
    style::apply(ctx);
}
