//! One page card: the page picture (made in the background), its name (or the rename box), and a "..." button
//! that shows on hover.

use egui::{Align2, Color32, FontId, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::paint_checkerboard;

use crate::app::FlashApp;
use crate::app::thumbnails::fitted;

/// What happened on a card.
pub struct CardEvents {
    pub response: Response,
    pub menu_button: bool,
    /// Rename finished: `Some(true)` = keep the new name, `Some(false)` = cancelled.
    pub rename_done: Option<bool>,
}

/// Draws the card of page `index`, `picture` = size of the picture box.
pub fn show(ui: &mut Ui, app: &mut FlashApp, index: usize, picture: Vec2) -> CardEvents {
    let (rect, response) = ui.allocate_exact_size(vec2(picture.x, 8.0 + picture.y + 24.0), Sense::click_and_drag());
    let pic = Rect::from_min_size(rect.min + vec2(0.0, 8.0), picture);
    let project = &app.projects[app.active_project];
    let active = index == project.active_index();
    let picked = app.pages_ui.selected.contains(&index) && app.pages_ui.selected.len() > 1;
    let (border, fill) = if active {
        (colors::ACCENT, colors::CANVAS_AREA)
    } else if picked {
        (colors::SELECTION_ACCENT, colors::ACCENT_SOFT)
    } else if response.hovered() {
        (colors::SUBTLE_TEXT, colors::CANVAS_AREA)
    } else {
        (colors::DIVIDER, colors::CANVAS_AREA)
    };
    let painter = ui.painter().clone();
    painter.rect(pic, 4, fill, Stroke::new(2.0, border), egui::StrokeKind::Inside);

    let settled = !ui.input(|i| i.pointer.any_down());
    let doc = &project.pages()[index].doc;
    let inner = pic.shrink(5.0);
    let scale = (inner.width() / doc.width() as f32).min(inner.height() / doc.height() as f32);
    let shape = Rect::from_center_size(inner.center(), vec2(doc.width() as f32, doc.height() as f32) * scale);
    paint_checkerboard(&painter, shape, 4.0);
    let ctx = ui.ctx().clone();
    let max = [(picture.x * 2.0) as u32, (picture.y * 2.0) as u32];
    if let Some((texture, size)) = app.thumbnails.page(&ctx, doc, max, settled) {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(texture, fitted(size, inner), uv, Color32::WHITE);
    }

    // "..." menu button, visible on hover.
    let menu_rect = Rect::from_min_size(pos2(pic.right() - 27.0, pic.top() + 3.0), vec2(24.0, 22.0));
    let menu = ui.interact(menu_rect, ui.scope_id().with(("page-menu", index)), Sense::click());
    if response.hovered() || menu.hovered() {
        painter.rect_filled(menu_rect, 4, Color32::from_rgba_unmultiplied(0x26, 0x26, 0x26, 0xCC));
        painter.text(menu_rect.center(), Align2::CENTER_CENTER, glyphs::MORE, glyphs::font(13.0), colors::TEXT);
    }
    let menu_button = menu.on_hover_text("Page menu: rename, duplicate, move, delete").clicked();

    // Name, or the rename box.
    let name_rect = Rect::from_min_size(pos2(rect.left() + 2.0, pic.bottom() + 3.0), vec2(rect.width() - 4.0, 20.0));
    let mut rename_done = None;
    match &mut app.pages_ui.renaming {
        Some((i, text)) if *i == index => {
            let edit = ui.put(name_rect, egui::TextEdit::singleline(text).align(egui::Align2::CENTER_CENTER));
            if !edit.has_focus() && !edit.lost_focus() {
                edit.request_focus();
            }
            if edit.lost_focus() {
                rename_done = Some(!ui.input(|i| i.key_pressed(egui::Key::Escape)));
            }
        }
        _ => {
            let name = &project.pages()[index].name;
            let galley = painter.layout_no_wrap(name.clone(), FontId::proportional(12.0), colors::TEXT);
            let x = name_rect.center().x - galley.size().x.min(name_rect.width()) / 2.0;
            painter.with_clip_rect(name_rect).galley(pos2(x, name_rect.top() + 2.0), galley, colors::TEXT);
        }
    }
    CardEvents { response, menu_button, rename_done }
}

/// The "+" in the gap before a page: shows on hover and adds a blank page there.
pub fn gap_button(ui: &mut Ui, height: f32, index: usize) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(20.0, height), Sense::click());
    let rect = Rect::from_min_size(rect.min + vec2(0.0, 8.0), vec2(20.0, height - 8.0));
    if response.hovered() {
        ui.painter().rect_filled(rect, 4, colors::ACCENT_SOFT);
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, glyphs::ADD, glyphs::font(11.0), colors::TEXT);
    }
    response.on_hover_text("Add a new page here").clicked() && index < usize::MAX
}
