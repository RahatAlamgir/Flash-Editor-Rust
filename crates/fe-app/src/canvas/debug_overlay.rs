//! A small box in the corner of the canvas with drawing statistics (F12 shows / hides it). For checking speed.

use egui::{Color32, FontId, Rect, Ui, pos2};
use fe_gpu::RenderStats;

pub fn paint(ui: &Ui, viewport: Rect, stats: &RenderStats, zoom: f32) {
    let frame_ms = ui.input(|i| i.unstable_dt) * 1000.0;
    let text = format!(
        "frame {frame_ms:.1} ms | zoom {:.1}% | level {} | tiles drawn {} | combined {} in {:.1} ms | waiting {} | textures {:.1} MB",
        zoom * 100.0,
        stats.level,
        stats.drawn,
        stats.rendered,
        stats.render_ms,
        stats.pending,
        stats.texture_bytes as f64 / 1e6
    );
    let painter = ui.painter_at(viewport);
    let pos = pos2(viewport.left() + 8.0, viewport.top() + 8.0);
    let galley = painter.layout_no_wrap(text, FontId::monospace(11.0), Color32::WHITE);
    let bg = Rect::from_min_size(pos, galley.size()).expand(4.0);
    painter.rect_filled(bg, 4, Color32::from_black_alpha(180));
    painter.galley(pos, galley, Color32::WHITE);
}
