//! Draws a page at rest or while its animations run: the whole page picture when it has no animations, otherwise its parts,
//! each animated layer placed, scaled, turned, faded and cut as its effect asks.

use egui::epaint::{Mesh, Vertex};
use egui::{Color32, Painter, Rect, Shape, pos2, vec2};
use fe_core::document::TransitionDirection;
use fe_core::document::animation::look_of;

use super::pages::{PageShow, Part};

const WHOLE: Rect = Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) };

/// Draws `shown` inside `page` when `step` clicks have happened and `time` seconds have passed since the last one.
pub fn paint(painter: &Painter, page: Rect, shown: &PageShow, step: usize, time: f32) {
    let clip = painter.with_clip_rect(page);
    if shown.parts.is_empty() {
        clip.image(shown.initial.id(), page, WHOLE, Color32::WHITE);
        return;
    }
    clip.rect_filled(page, 0, Color32::WHITE);
    for part in &shown.parts {
        match part {
            Part::Still(texture) => {
                clip.image(texture.id(), page, WHOLE, Color32::WHITE);
            }
            Part::Layer { id, texture, at, opacity } => {
                let look = look_of(&shown.animations, &shown.timeline, *id, step, time);
                if !look.visible {
                    continue;
                }
                let place = Rect::from_min_size(
                    page.min + vec2(at[0] * page.width(), at[1] * page.height()),
                    vec2(at[2] * page.width(), at[3] * page.height()),
                );
                let center = place.center() + vec2(look.dx * page.width(), look.dy * page.height());
                let half = place.size() * look.scale * 0.5;
                let alpha = (look.alpha * opacity).clamp(0.0, 1.0);
                let tint = Color32::from_white_alpha((alpha * 255.0) as u8);
                // The part that is shown (a wipe uncovers the layer's own area from one edge).
                let mut cut = page;
                if let Some((from, amount)) = look.reveal {
                    cut = cut.intersect(reveal(place, from, amount));
                }
                let layer = painter.with_clip_rect(cut);
                let (sin, cos) = look.angle.sin_cos();
                let corner = |x: f32, y: f32| center + vec2(x * cos - y * sin, x * sin + y * cos);
                let uv = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)];
                let at_corner = [
                    corner(-half.x, -half.y),
                    corner(half.x, -half.y),
                    corner(half.x, half.y),
                    corner(-half.x, half.y),
                ];
                let mut mesh = Mesh::with_texture(texture.id());
                for (position, uv) in at_corner.into_iter().zip(uv) {
                    mesh.vertices.push(Vertex { pos: position, uv, color: tint });
                }
                mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
                layer.add(Shape::mesh(mesh));
            }
        }
    }
}

/// The part of `area` that a reveal of `amount` (0..1) from the edge `from` has uncovered.
fn reveal(area: Rect, from: TransitionDirection, amount: f32) -> Rect {
    let a = amount.clamp(0.0, 1.0);
    match from {
        TransitionDirection::Left => Rect::from_min_max(area.min, pos2(area.left() + area.width() * a, area.bottom())),
        TransitionDirection::Right => Rect::from_min_max(pos2(area.right() - area.width() * a, area.top()), area.max),
        TransitionDirection::Up => Rect::from_min_max(area.min, pos2(area.right(), area.top() + area.height() * a)),
        TransitionDirection::Down => Rect::from_min_max(pos2(area.left(), area.bottom() - area.height() * a), area.max),
    }
}
