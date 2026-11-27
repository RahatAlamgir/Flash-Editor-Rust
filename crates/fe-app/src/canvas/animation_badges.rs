//! While the Animations tab of the Layers panel is open, every animated object on the canvas carries small numbers: the position of
//! its animations in the play order (1, 2, 3...), like PowerPoint. The chosen animation's number is highlighted.

use egui::{Align2, Color32, FontId, Painter, Rect, Stroke, pos2, vec2};
use fe_gpu::ViewTransform;
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;

/// True when the numbers should show.
pub fn wanted(app: &FlashApp) -> bool {
    app.project().presentation && app.layers_tab == 1 && !app.right_collapsed
}

pub fn paint(painter: &Painter, t: ViewTransform, app: &FlashApp, viewport: Rect) {
    let project = app.project();
    let Some(page) = project.pages().get(project.active_index()) else { return };
    let doc = &page.doc;
    let canvas = doc.bounds();
    // The animations that count, in order.
    let listed: Vec<(usize, &fe_core::document::animation::Animation)> =
        page.animations.iter().enumerate().filter(|(_, a)| doc.layer(a.layer).is_some()).collect();
    // How many numbers are already on each layer (they sit side by side).
    let mut placed: Vec<(fe_core::document::LayerId, usize)> = Vec::new();
    for (position, (raw, animation)) in listed.iter().enumerate() {
        let Some(layer) = doc.layer(animation.layer) else { continue };
        let bounds = layer.pixels().visible_bounds().intersect(&canvas);
        if bounds.is_empty() {
            continue;
        }
        let rect = t.rect_to_screen(bounds);
        let side = match placed.iter_mut().find(|(id, _)| *id == animation.layer) {
            Some((_, n)) => {
                *n += 1;
                *n - 1
            }
            None => {
                placed.push((animation.layer, 1));
                0
            }
        };
        let size = vec2(20.0, 18.0);
        // Just outside the top left corner, kept inside the view.
        let mut min = pos2(rect.left() - size.x - 3.0 + side as f32 * (size.x + 3.0), rect.top() - 1.0);
        min.x = min.x.clamp(viewport.left() + 2.0, viewport.right() - size.x - 2.0);
        min.y = min.y.clamp(viewport.top() + 2.0, viewport.bottom() - size.y - 2.0);
        let badge = Rect::from_min_size(min, size);
        let chosen = app.ribbon.animation_selected == Some(*raw);
        let fill = if chosen { colors::ACCENT } else { Color32::from_rgb(0xFF, 0xB0, 0x20) };
        let ink = if chosen { Color32::WHITE } else { Color32::from_rgb(0x20, 0x20, 0x20) };
        painter.rect(badge, 4, fill, Stroke::new(1.0, Color32::from_black_alpha(160)), egui::StrokeKind::Inside);
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            (position + 1).to_string(),
            FontId::proportional(12.0),
            ink,
        );
    }
}
