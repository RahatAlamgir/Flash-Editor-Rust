//! Color tokens, copied from the C# `Themes/Colors.xaml` and `Themes/Ribbon.xaml`.
//! UI code uses only these names, never hard-coded colors.

use egui::Color32;

/// Window background and the ribbon's tab row.
pub const WINDOW: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
/// Side panels, ribbon body, status bar.
pub const PANEL: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
/// Panel section headers.
pub const PANEL_HEADER: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x2E);
/// The bar of buttons under the layer list.
pub const PANEL_FOOTER: Color32 = Color32::from_rgb(0x2B, 0x2B, 0x2B);
/// Area around the picture.
pub const CANVAS_AREA: Color32 = Color32::from_rgb(0x16, 0x16, 0x16);
/// Lines between areas.
pub const DIVIDER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
/// Hovered buttons and rows.
pub const HOVER: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
/// Active tool, sliders, focus.
pub const ACCENT: Color32 = Color32::from_rgb(0x3B, 0x82, 0xF6);
/// Selected rows and pressed buttons (`#553B82F6`).
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x3B, 0x82, 0xF6, 0x55);
/// Selection fill on the canvas (`#263D8BFF`).
pub const SELECTION_TINT: Color32 = Color32::from_rgba_unmultiplied_const(0x3D, 0x8B, 0xFF, 0x26);
/// Handles and selection outline.
pub const SELECTION_ACCENT: Color32 = Color32::from_rgb(0x4F, 0xC3, 0xFF);
/// Normal text.
pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xE6, 0xE6);
/// Hints, section titles, status bar.
pub const SUBTLE_TEXT: Color32 = Color32::from_rgb(0x9A, 0x9A, 0x9A);
/// Plain icons.
pub const ICON: Color32 = Color32::from_rgb(0xD8, 0xD8, 0xD8);
/// Slider: the part of the track up to the value (measured from the C# screenshot).
pub const SLIDER_FILL: Color32 = Color32::from_rgb(0x59, 0xBE, 0xF6);
/// Slider: the rest of the track.
pub const SLIDER_TRACK: Color32 = Color32::from_rgb(0x9A, 0x9A, 0x9A);
/// Slider: the grey ring of the knob.
pub const SLIDER_RING: Color32 = Color32::from_rgb(0x45, 0x45, 0x45);
/// Slider: the thin edge around the ring.
pub const SLIDER_RING_EDGE: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
/// Slider: the blue dot in the knob.
pub const SLIDER_DOT: Color32 = Color32::from_rgb(0x4C, 0xC2, 0xFF);
/// Text boxes and drop-down lists.
pub const INPUT: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

/// The icon colors of the "Colorful" icon style (Settings > Icon Style). Plain style uses [`ICON`].
pub mod icon {
    use egui::Color32;

    pub const BLUE: Color32 = Color32::from_rgb(0x5B, 0x9B, 0xFF);
    pub const SKY: Color32 = Color32::from_rgb(0x38, 0xBD, 0xF8);
    pub const TEAL: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);
    pub const GREEN: Color32 = Color32::from_rgb(0x4A, 0xDE, 0x80);
    pub const LIME: Color32 = Color32::from_rgb(0xA3, 0xE6, 0x35);
    pub const AMBER: Color32 = Color32::from_rgb(0xFB, 0xBF, 0x24);
    pub const ORANGE: Color32 = Color32::from_rgb(0xFB, 0x92, 0x3C);
    pub const RED: Color32 = Color32::from_rgb(0xF8, 0x71, 0x71);
    pub const PINK: Color32 = Color32::from_rgb(0xF4, 0x72, 0xB6);
    pub const PURPLE: Color32 = Color32::from_rgb(0xA7, 0x8B, 0xFA);
    pub const SLATE: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
}
