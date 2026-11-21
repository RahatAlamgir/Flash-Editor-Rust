//! One row of the layer list: eye (show / hide), thumbnail, name (or the rename box) and padlock.

use egui::{Align2, FontId, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};
use fe_core::document::Layer;
use fe_ui_kit::icons::{self, glyphs};
use fe_ui_kit::theme::{colors, metrics};
use fe_ui_kit::widgets::paint_checkerboard;

use crate::app::thumbnails::fitted;

/// What the user did on a row.
#[derive(Default)]
pub struct RowEvents {
    pub row: Option<Response>,
    pub toggle_visible: bool,
    pub toggle_lock: bool,
    /// The mask thumbnail was clicked (switch the mask on / off).
    pub toggle_mask: bool,
    /// The little arrow of a group was clicked (show / hide its parts).
    pub toggle_expand: bool,
    /// The rename box: `Some(true)` = confirmed (Enter or click elsewhere), `Some(false)` = cancelled (Esc).
    pub rename_done: Option<bool>,
}

/// Everything the row needs to draw itself.
pub struct RowLook<'a> {
    pub layer: &'a Layer,
    pub selected: bool,
    pub thumbnail: Option<(egui::TextureId, [f32; 2])>,
    /// The mask's thumbnail, when the layer has a mask.
    pub mask: Option<(egui::TextureId, [f32; 2])>,
    pub renaming: Option<&'a mut String>,
    /// `Some(open)` for a group: the arrow in front of its name.
    pub group_open: Option<bool>,
}

pub fn show(ui: &mut Ui, look: RowLook<'_>) -> RowEvents {
    let mut events = RowEvents::default();
    let (rect, row) = ui.allocate_exact_size(vec2(ui.available_width(), metrics::LAYER_ROW), Sense::click_and_drag());
    let layer = look.layer;
    let painter = ui.painter().clone();
    if look.selected {
        painter.rect_filled(rect, 0, colors::ACCENT_SOFT);
    } else if row.hovered() {
        painter.rect_filled(rect, 0, colors::HOVER);
    }

    // Eye (visible) toggle.
    let eye = Rect::from_min_size(rect.min + vec2(6.0, 6.0), Vec2::splat(24.0));
    let eye_color = if layer.visible { colors::ICON } else { colors::ICON.gamma_multiply(0.25) };
    icons::icon("Eye", 18.0, eye_color).paint_at(ui, Rect::from_center_size(eye.center(), Vec2::splat(18.0)));
    let eye_response = ui.interact(eye, ui.scope_id().with(("eye", layer.id())), Sense::click());
    events.toggle_visible = eye_response.on_hover_text("Show / hide").clicked();

    // Thumbnail on a small checkerboard (an adjustment layer shows its icon instead).
    let thumb = Rect::from_min_size(pos2(eye.right() + 4.0, rect.center().y - 14.0), Vec2::from(metrics::LAYER_THUMB));
    paint_checkerboard(&painter, thumb, 4.0);
    if layer.is_adjustment() {
        painter.rect_filled(thumb, 0, colors::PANEL_HEADER);
        painter.text(thumb.center(), Align2::CENTER_CENTER, '\u{E793}', glyphs::font(16.0), colors::ICON);
    } else if let Some((texture, size)) = look.thumbnail {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(texture, fitted(size, thumb), uv, egui::Color32::WHITE);
    }
    painter.rect_stroke(thumb, 0, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);

    // The layer mask (click to switch it on or off), like the C# row.
    let mut text_left = thumb.right() + 8.0;
    if layer.mask().is_some() {
        let mask_rect = Rect::from_min_size(pos2(thumb.right() + 4.0, rect.center().y - 14.0), vec2(30.0, 28.0));
        painter.rect_filled(mask_rect, 0, egui::Color32::BLACK);
        if let Some((texture, size)) = look.mask {
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            let tint = if layer.mask_enabled { egui::Color32::WHITE } else { egui::Color32::from_gray(90) };
            painter.image(texture, fitted(size, mask_rect.shrink(1.0)), uv, tint);
        }
        painter.rect_stroke(mask_rect, 0, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
        if !layer.mask_enabled {
            let red = Stroke::new(1.5, egui::Color32::from_rgb(0xE5, 0x48, 0x4D));
            painter.line_segment([mask_rect.left_top(), mask_rect.right_bottom()], red);
            painter.line_segment([mask_rect.right_top(), mask_rect.left_bottom()], red);
        }
        let mask_response = ui.interact(mask_rect, ui.scope_id().with(("mask", layer.id())), Sense::click());
        events.toggle_mask = mask_response
            .on_hover_text("Layer mask. Click to switch it on or off. Paint on it with the Mask Brush (K).")
            .clicked();
        text_left = mask_rect.right() + 8.0;
    }

    // Padlock: faint when unlocked, bright when locked.
    let lock = Rect::from_min_size(pos2(rect.right() - 28.0, rect.center().y - 12.0), Vec2::splat(24.0));
    let lock_color = if layer.locked { colors::ICON } else { colors::ICON.gamma_multiply(0.2) };
    icons::icon("Lock", 18.0, lock_color).paint_at(ui, Rect::from_center_size(lock.center(), Vec2::splat(18.0)));
    let lock_response = ui.interact(lock, ui.scope_id().with(("lock", layer.id())), Sense::click());
    events.toggle_lock = lock_response.on_hover_text("Lock / unlock (Ctrl+L)").clicked();

    // A group has an arrow in front of its name that shows its parts.
    if let Some(open) = look.group_open {
        let arrow = Rect::from_min_size(pos2(text_left - 4.0, rect.center().y - 10.0), Vec2::splat(20.0));
        painter.text(
            arrow.center(),
            Align2::CENTER_CENTER,
            if open { "▾" } else { "▸" },
            FontId::proportional(14.0),
            colors::ICON,
        );
        let response = ui.interact(arrow, ui.scope_id().with(("expand", layer.id())), Sense::click());
        events.toggle_expand = response.on_hover_text("Show / hide the parts of this group").clicked();
        text_left += 16.0;
    }

    // Name, or the box to rename it.
    let name_rect = Rect::from_min_max(pos2(text_left, rect.top() + 6.0), pos2(lock.left() - 4.0, rect.bottom() - 6.0));
    match look.renaming {
        Some(text) => {
            let edit = ui.put(name_rect, egui::TextEdit::singleline(text).font(FontId::proportional(12.5)));
            if !edit.has_focus() && !edit.lost_focus() {
                edit.request_focus();
            }
            if edit.lost_focus() {
                events.rename_done = Some(!ui.input(|i| i.key_pressed(egui::Key::Escape)));
            }
        }
        None => {
            // One line only: text layers are named after their text, which may have several lines.
            let first_line = layer.name.lines().next().unwrap_or_default();
            let more = layer.name.lines().nth(1).is_some();
            let text = if more { format!("{first_line} …") } else { first_line.to_owned() };
            let mut job = egui::text::LayoutJob::single_section(
                text,
                egui::TextFormat::simple(FontId::proportional(12.5), colors::TEXT),
            );
            job.wrap = egui::text::TextWrapping::truncate_at_width(name_rect.width());
            let galley = painter.layout_job(job);
            let y = rect.center().y - galley.size().y / 2.0;
            painter.with_clip_rect(name_rect).galley(pos2(name_rect.left(), y), galley, colors::TEXT);
        }
    }
    events.row = Some(row);
    events
}
