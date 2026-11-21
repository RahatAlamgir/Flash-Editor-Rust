//! Grid and guide lines drawn over the picture (C# `AidsOverlay`).

use egui::{Color32, Painter, Rect, Stroke};
use fe_gpu::ViewTransform;

const GRID: Color32 = Color32::from_rgba_unmultiplied_const(0x38, 0xB6, 0xFF, 0x5A);
const GRID_MAJOR: Color32 = Color32::from_rgba_unmultiplied_const(0x38, 0xB6, 0xFF, 0x8C);
pub const GUIDE: Color32 = Color32::from_rgb(0x00, 0xD8, 0xE8);

/// Grid lines every `size` pixels over the page; every fifth line is stronger. When the lines would be closer
/// than 8 points, every other line is skipped (and so on).
pub fn paint_grid(painter: &Painter, view: ViewTransform, page_w: f64, page_h: f64, size: f64) {
    if size < 1.0 {
        return;
    }
    let mut step = size;
    let mut every = 1;
    while step * (view.zoom as f64) < 8.0 && every < 1024 {
        step *= 2.0;
        every *= 2;
    }
    let is_major = |p: f64| ((p / (step * 5.0)) - (p / (step * 5.0)).round()).abs() < 1e-6;
    let pen = |p: f64| Stroke::new(1.0, if is_major(p) { GRID_MAJOR } else { GRID });
    let top = view.to_screen(0.0, 0.0).y;
    let bottom = view.to_screen(0.0, page_h as f32).y;
    let left = view.to_screen(0.0, 0.0).x;
    let right = view.to_screen(page_w as f32, 0.0).x;
    let mut x = 0.0;
    while x <= page_w + 1e-6 {
        painter.vline(view.to_screen(x as f32, 0.0).x.round() + 0.5, top..=bottom, pen(x));
        x += step;
    }
    let mut y = 0.0;
    while y <= page_h + 1e-6 {
        painter.hline(left..=right, view.to_screen(0.0, y as f32).y.round() + 0.5, pen(y));
        y += step;
    }
}

/// Guide lines across the whole visible canvas area.
pub fn paint_guides(painter: &Painter, view: ViewTransform, area: Rect, vertical: &[f64], horizontal: &[f64]) {
    let stroke = Stroke::new(1.0, GUIDE);
    for &x in vertical {
        painter.vline(view.to_screen(x as f32, 0.0).x.round() + 0.5, area.y_range(), stroke);
    }
    for &y in horizontal {
        painter.hline(area.x_range(), view.to_screen(0.0, y as f32).y.round() + 0.5, stroke);
    }
}

/// Smart guides while something snaps (C# `AidsOverlay`): the lines it lined up with, and equal gaps with their size.
pub const SNAP: Color32 = Color32::from_rgb(0xFF, 0x2D, 0x9A);

pub fn paint_snap(painter: &Painter, view: ViewTransform, area: Rect, snap: &fe_core::layout::snapping::SnapResult) {
    let pen = Stroke::new(1.0, SNAP);
    for &x in &snap.lines_x {
        painter.vline(view.to_screen(x as f32, 0.0).x.round() + 0.5, area.y_range(), pen);
    }
    for &y in &snap.lines_y {
        painter.hline(area.x_range(), view.to_screen(0.0, y as f32).y.round() + 0.5, pen);
    }
    for gap in &snap.gaps {
        let (a, b) = if gap.horizontal {
            (view.to_screen(gap.from as f32, gap.cross as f32), view.to_screen(gap.to as f32, gap.cross as f32))
        } else {
            (view.to_screen(gap.cross as f32, gap.from as f32), view.to_screen(gap.cross as f32, gap.to as f32))
        };
        painter.line_segment([a, b], pen);
        let tick = if gap.horizontal { egui::vec2(0.0, 5.0) } else { egui::vec2(5.0, 0.0) };
        painter.line_segment([a - tick, a + tick], pen);
        painter.line_segment([b - tick, b + tick], pen);
        let label =
            painter.layout_no_wrap(format!("{:.0}", gap.value), egui::FontId::proportional(11.0), Color32::WHITE);
        let size = label.size() + egui::vec2(8.0, 2.0);
        let mid = egui::pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        let center =
            if gap.horizontal { mid - egui::vec2(0.0, 11.0) } else { mid + egui::vec2(14.0 + size.x / 2.0, 0.0) };
        let r = Rect::from_center_size(center, size);
        painter.rect_filled(r, 3, SNAP);
        painter.galley(r.min + egui::vec2(4.0, 1.0), label, Color32::WHITE);
    }
}
