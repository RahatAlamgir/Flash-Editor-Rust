//! A slider in the Windows 11 style the C# app uses: a thin rounded track, light blue up to the value,
//! and a round knob (a blue dot inside a dark grey ring) that grows a little on hover and shrinks while dragged.

use std::ops::RangeInclusive;

use egui::{Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};

use crate::theme::colors;

const TRACK_H: f32 = 4.0;
const RING_R: f32 = 10.0;
const DOT_R: f32 = 6.0;
const DOT_R_HOVER: f32 = 7.0;
const DOT_R_PRESSED: f32 = 5.0;

/// Draws the slider `width` points wide; dragging or clicking moves the value. `.changed()` when it moved.
pub fn slider(ui: &mut Ui, value: &mut f64, range: RangeInclusive<f64>, width: f32) -> Response {
    let (min, max) = (*range.start(), *range.end());
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, RING_R * 2.0 + 2.0), Sense::click_and_drag());
    // The knob's center can travel from one ring radius in from the left edge to one in from the right.
    let left = rect.left() + RING_R;
    let right = rect.right() - RING_R;
    if let Some(pointer) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked() || response.is_pointer_button_down_on())
    {
        let t = ((pointer.x - left) / (right - left)).clamp(0.0, 1.0) as f64;
        let new = min + t * (max - min);
        if new != *value {
            *value = new;
            response.mark_changed();
        }
    }
    if ui.is_rect_visible(rect) {
        let t = if max > min { ((*value - min) / (max - min)).clamp(0.0, 1.0) as f32 } else { 0.0 };
        let knob_x = left + t * (right - left);
        let y = rect.center().y;
        let painter = ui.painter();
        let track = |from: f32, to: f32| Rect::from_min_max(pos2(from, y - TRACK_H / 2.0), pos2(to, y + TRACK_H / 2.0));
        painter.rect_filled(track(left - 2.0, right + 2.0), TRACK_H / 2.0, colors::SLIDER_TRACK);
        painter.rect_filled(track(left - 2.0, knob_x), TRACK_H / 2.0, colors::SLIDER_FILL);
        let center = pos2(knob_x, y);
        painter.circle(center, RING_R, colors::SLIDER_RING, Stroke::new(1.0, colors::SLIDER_RING_EDGE));
        let dot = if response.is_pointer_button_down_on() {
            DOT_R_PRESSED
        } else if response.hovered() {
            DOT_R_HOVER
        } else {
            DOT_R
        };
        painter.circle_filled(center, dot, colors::SLIDER_DOT);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The size the slider takes for a given width (for layouts).
pub fn slider_size(width: f32) -> Vec2 {
    vec2(width, RING_R * 2.0 + 2.0)
}
