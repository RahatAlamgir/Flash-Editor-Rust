//! The grey checkerboard drawn behind transparent pictures (thumbnails, page cards).

use egui::{Color32, Painter, Rect, Vec2};

const LIGHT: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
const DARK: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);

/// Fills `rect` with a checkerboard of `cell`-sized squares.
pub fn paint_checkerboard(painter: &Painter, rect: Rect, cell: f32) {
    painter.rect_filled(rect, 0, LIGHT);
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for row in 0..rows {
        for col in (row % 2..cols).step_by(2) {
            let min = rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
            let cell_rect = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            painter.rect_filled(cell_rect, 0, DARK);
        }
    }
}
