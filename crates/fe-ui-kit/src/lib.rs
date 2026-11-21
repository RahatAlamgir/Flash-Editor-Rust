//! Flash Editor look and feel: theme, icons and reusable egui widgets.

pub mod icons;
pub mod theme;
pub mod widgets;

/// Sets up fonts, colors and icon loading. Call once when the window opens.
pub fn install(ctx: &egui::Context) {
    theme::apply(ctx);
    icons::install(ctx);
    // Ctrl + / Ctrl - / Ctrl 0 zoom the picture in Flash Editor, not the whole interface.
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}
