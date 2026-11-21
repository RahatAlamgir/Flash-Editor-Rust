//! One project card: preview, name, pages and size, date. Double-click opens it; right-click shows its menu.

use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::paint_checkerboard;

use super::entry::Entry;

pub const CARD: Vec2 = Vec2::new(214.0, 206.0);
const PICTURE_H: f32 = 128.0;

/// What the user did with a card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardAction {
    Open,
    Rename,
    ShowInExplorer,
    RemoveFromList,
    Delete,
}

pub fn show(
    ui: &mut Ui,
    entry: &Entry,
    texture: Option<&egui::TextureHandle>,
    renaming: Option<&mut String>,
) -> Option<CardAction> {
    let (rect, response) = ui.allocate_exact_size(CARD, Sense::click());
    let painter = ui.painter().clone();
    let fill = if response.hovered() { colors::HOVER } else { colors::PANEL_HEADER };
    painter.rect(rect, 6, fill, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
    let picture = Rect::from_min_size(rect.min + vec2(7.0, 7.0), vec2(CARD.x - 14.0, PICTURE_H));
    painter.rect_filled(picture, 4, colors::CANVAS_AREA);
    if let Some(texture) = texture {
        let size = texture.size_vec2();
        let scale = (picture.width() / size.x).min(picture.height() / size.y);
        let shown = Rect::from_center_size(picture.center(), size * scale);
        paint_checkerboard(&painter, shown, 4.0);
        painter.image(texture.id(), shown, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    let text_left = picture.left();
    let mut y = picture.bottom() + 8.0;
    match renaming {
        Some(name) => {
            let edit = ui.put(
                Rect::from_min_size(pos2(text_left, y - 2.0), vec2(picture.width(), 22.0)),
                egui::TextEdit::singleline(name),
            );
            if !edit.has_focus() && !edit.lost_focus() {
                edit.request_focus();
            }
        }
        None => {
            let name = painter.layout(entry.name.clone(), FontId::proportional(13.0), colors::TEXT, picture.width());
            painter.with_clip_rect(Rect::from_min_size(pos2(text_left, y), vec2(picture.width(), 18.0))).galley(
                pos2(text_left, y),
                name,
                colors::TEXT,
            );
        }
    }
    y += 22.0;
    painter.text(
        pos2(text_left, y),
        Align2::LEFT_TOP,
        entry.details(),
        FontId::proportional(11.5),
        colors::SUBTLE_TEXT,
    );
    y += 17.0;
    painter.text(
        pos2(text_left, y),
        Align2::LEFT_TOP,
        entry.modified_text(),
        FontId::proportional(11.5),
        colors::SUBTLE_TEXT,
    );

    let mut action = None;
    if response.double_clicked() {
        action = Some(CardAction::Open);
    }
    let tip = format!("{}\n{}", entry.path.display(), entry.size_text());
    response.on_hover_text(tip).context_menu(|ui| {
        ui.set_min_width(200.0);
        for (text, value) in [
            ("Open", CardAction::Open),
            ("Rename", CardAction::Rename),
            ("Show in Explorer", CardAction::ShowInExplorer),
            ("Remove from list", CardAction::RemoveFromList),
            ("Delete (to the Recycle Bin)", CardAction::Delete),
        ] {
            if ui.button(text).clicked() {
                action = Some(value);
                ui.close();
            }
        }
    });
    action
}
