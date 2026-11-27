//! Holding Alt with something selected shows how far it is from its surroundings, like Figma: on each side a line to the nearest
//! object in that direction (or to the canvas border when nothing is there) with the distance in pixels.

use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, pos2, vec2};
use fe_core::imaging::PixelRect;
use fe_gpu::ViewTransform;

use crate::app::FlashApp;

const INK: Color32 = Color32::from_rgb(0xFF, 0x2D, 0x78);

/// True when the distances should show.
pub fn wanted(ui: &egui::Ui, app: &FlashApp) -> bool {
    ui.input(|i| i.modifiers.alt) && !app.project().active_page().doc.selected_ids().is_empty()
}

pub fn paint(painter: &Painter, t: ViewTransform, app: &FlashApp) {
    let doc = &app.project().active_page().doc;
    let selected = doc.selected_ids();
    let mut chosen: Option<PixelRect> = None;
    let mut others: Vec<PixelRect> = Vec::new();
    for layer in doc.layers() {
        if !layer.visible || layer.is_adjustment() {
            continue;
        }
        let b = layer.visible_bounds();
        if b.is_empty() {
            continue;
        }
        if selected.contains(&layer.id()) {
            chosen = Some(chosen.map_or(b, |c| c.union(&b)));
        } else {
            others.push(b);
        }
    }
    let Some(s) = chosen else { return };
    let canvas = doc.bounds();
    let overlap = |a0: i32, a1: i32, b0: i32, b1: i32| (a0.max(b0), a1.min(b1));
    let mut lines: Vec<(PixelRect, bool)> = Vec::new(); // (the gap as a thin rect, horizontal)

    // Right.
    let near = others
        .iter()
        .filter(|o| o.x >= s.right() && o.y < s.bottom() && o.bottom() > s.y)
        .min_by_key(|o| o.x)
        .map(|o| (o.x, overlap(s.y, s.bottom(), o.y, o.bottom())));
    let (edge, (y0, y1)) = near.unwrap_or((canvas.right(), (s.y, s.bottom())));
    if edge > s.right() {
        lines.push((PixelRect::new(s.right(), (y0 + y1) / 2, edge - s.right(), 0), true));
    }
    // Left.
    let near = others
        .iter()
        .filter(|o| o.right() <= s.x && o.y < s.bottom() && o.bottom() > s.y)
        .max_by_key(|o| o.right())
        .map(|o| (o.right(), overlap(s.y, s.bottom(), o.y, o.bottom())));
    let (edge, (y0, y1)) = near.unwrap_or((canvas.x, (s.y, s.bottom())));
    if edge < s.x {
        lines.push((PixelRect::new(edge, (y0 + y1) / 2, s.x - edge, 0), true));
    }
    // Below.
    let near = others
        .iter()
        .filter(|o| o.y >= s.bottom() && o.x < s.right() && o.right() > s.x)
        .min_by_key(|o| o.y)
        .map(|o| (o.y, overlap(s.x, s.right(), o.x, o.right())));
    let (edge, (x0, x1)) = near.unwrap_or((canvas.bottom(), (s.x, s.right())));
    if edge > s.bottom() {
        lines.push((PixelRect::new((x0 + x1) / 2, s.bottom(), 0, edge - s.bottom()), false));
    }
    // Above.
    let near = others
        .iter()
        .filter(|o| o.bottom() <= s.y && o.x < s.right() && o.right() > s.x)
        .max_by_key(|o| o.bottom())
        .map(|o| (o.bottom(), overlap(s.x, s.right(), o.x, o.right())));
    let (edge, (x0, x1)) = near.unwrap_or((canvas.y, (s.x, s.right())));
    if edge < s.y {
        lines.push((PixelRect::new((x0 + x1) / 2, edge, 0, s.y - edge), false));
    }

    // The selection's frame.
    painter.rect_stroke(t.rect_to_screen(s), 0, Stroke::new(1.0, INK), egui::StrokeKind::Outside);
    let line = Stroke::new(1.0, INK);
    for (gap, horizontal) in lines {
        let r = t.rect_to_screen(gap);
        let (a, b, length) = if horizontal {
            (pos2(r.left(), r.top()), pos2(r.right(), r.top()), gap.width)
        } else {
            (pos2(r.left(), r.top()), pos2(r.left(), r.bottom()), gap.height)
        };
        painter.line_segment([a, b], line);
        // Little end caps.
        let cap = if horizontal { vec2(0.0, 4.0) } else { vec2(4.0, 0.0) };
        painter.line_segment([a - cap, a + cap], line);
        painter.line_segment([b - cap, b + cap], line);
        // The number, in a pill beside the middle of the line.
        let mid = a.lerp(b, 0.5);
        let galley = painter.layout_no_wrap(length.to_string(), FontId::proportional(11.5), Color32::WHITE);
        let size = galley.size() + vec2(10.0, 4.0);
        let center =
            if horizontal { mid + vec2(0.0, -size.y / 2.0 - 4.0) } else { mid + vec2(size.x / 2.0 + 4.0, 0.0) };
        let pill = Rect::from_center_size(center, size);
        painter.rect_filled(pill, 4, INK);
        painter.text(
            pill.center(),
            Align2::CENTER_CENTER,
            length.to_string(),
            FontId::proportional(11.5),
            Color32::WHITE,
        );
    }
}
