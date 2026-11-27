//! Draws one frame of a slide transition: the old page (or black), the new page, and how far the movement has gone.

use egui::{Color32, Painter, Rect, TextureHandle, Vec2, pos2, vec2};
use fe_core::document::{Transition, TransitionKind};

const WHOLE: Rect = Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) };

/// Draws the frame at `progress` (0 = old page, 1 = new page, already eased) inside `page`.
pub fn paint(
    painter: &Painter,
    page: Rect,
    old: Option<&TextureHandle>,
    new: &TextureHandle,
    transition: &Transition,
    progress: f32,
) {
    let clip = painter.with_clip_rect(page);
    let old_at = |offset: Vec2, tint: Color32| {
        let rect = page.translate(offset);
        match old {
            Some(texture) => clip.image(texture.id(), rect, WHOLE, tint),
            None => clip.rect_filled(rect, 0, Color32::BLACK),
        };
    };
    let new_at = |rect: Rect, tint: Color32| {
        clip.image(new.id(), rect, WHOLE, tint);
    };
    let (dx, dy) = transition.direction.travel();
    // Where the new page starts: one page away from its place, on the side it comes from.
    let away = vec2(-dx * page.width(), -dy * page.height());
    let p = progress.clamp(0.0, 1.0);
    match transition.kind {
        TransitionKind::None => new_at(page, Color32::WHITE),
        TransitionKind::Fade => {
            old_at(Vec2::ZERO, Color32::WHITE);
            new_at(page, Color32::from_white_alpha((p * 255.0) as u8));
        }
        TransitionKind::Push => {
            old_at(-away * p, Color32::WHITE);
            new_at(page.translate(away * (1.0 - p)), Color32::WHITE);
        }
        TransitionKind::Cover => {
            old_at(Vec2::ZERO, Color32::WHITE);
            new_at(page.translate(away * (1.0 - p)), Color32::WHITE);
        }
        TransitionKind::Uncover => {
            new_at(page, Color32::WHITE);
            old_at(-away * p, Color32::WHITE);
        }
        TransitionKind::Wipe => {
            old_at(Vec2::ZERO, Color32::WHITE);
            let (l, t, r, b) = (page.left(), page.top(), page.right(), page.bottom());
            let revealed = if dx > 0.0 {
                Rect::from_min_max(pos2(l, t), pos2(l + page.width() * p, b))
            } else if dx < 0.0 {
                Rect::from_min_max(pos2(r - page.width() * p, t), pos2(r, b))
            } else if dy > 0.0 {
                Rect::from_min_max(pos2(l, t), pos2(r, t + page.height() * p))
            } else {
                Rect::from_min_max(pos2(l, b - page.height() * p), pos2(r, b))
            };
            let region = painter.with_clip_rect(page.intersect(revealed));
            region.image(new.id(), page, WHOLE, Color32::WHITE);
        }
        TransitionKind::Zoom => {
            old_at(Vec2::ZERO, Color32::WHITE);
            let rect = Rect::from_center_size(page.center(), page.size() * p.max(0.001));
            new_at(rect, Color32::from_white_alpha((p.sqrt() * 255.0) as u8));
        }
        TransitionKind::Split => {
            old_at(Vec2::ZERO, Color32::WHITE);
            let band = Rect::from_center_size(page.center(), vec2(page.width() * p, page.height()));
            painter.with_clip_rect(band.intersect(page)).image(new.id(), page, WHOLE, Color32::WHITE);
        }
    }
}
