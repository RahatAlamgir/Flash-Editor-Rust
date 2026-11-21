//! The grey checkerboard behind transparent parts of the picture, drawn as one repeating texture.

use std::sync::OnceLock;

use egui::{Color32, ColorImage, Painter, Pos2, Rect, TextureHandle, TextureOptions, TextureWrapMode, pos2};

const LIGHT: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
const DARK: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);
/// Size of one checker square on screen, in points.
const CELL: f32 = 8.0;

/// Fills `rect` (screen points) with the checkerboard; `origin` is where a square starts (keeps it steady
/// while panning).
pub fn paint_checkerboard(ctx: &egui::Context, painter: &Painter, rect: Rect, origin: Pos2) {
    static TEXTURE: OnceLock<TextureHandle> = OnceLock::new();
    let texture = TEXTURE.get_or_init(|| {
        let image = ColorImage::new([2, 2], vec![LIGHT, DARK, DARK, LIGHT]);
        let options = TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            wrap_mode: TextureWrapMode::Repeat,
            mipmap_mode: None,
        };
        ctx.load_texture("fe-checkerboard", image, options)
    });
    let period = CELL * 2.0;
    let u0 = (rect.left() - origin.x) / period;
    let v0 = (rect.top() - origin.y) / period;
    let uv = Rect::from_min_max(pos2(u0, v0), pos2(u0 + rect.width() / period, v0 + rect.height() / period));
    painter.image(texture.id(), rect, uv, Color32::WHITE);
}
