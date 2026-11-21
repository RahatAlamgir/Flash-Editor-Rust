//! A small card that opens under a button and closes when the user clicks anywhere else (C# `PopupCard`).

use egui::{Area, Frame, Id, Margin, Order, Rect, Ui, pos2};

use crate::theme::colors;

/// Shows the card under `anchor` while `*open` is true; a click outside the card and the anchor closes it.
pub fn popup_below(ui: &Ui, id: Id, anchor: Rect, open: &mut bool, add_contents: impl FnOnce(&mut Ui)) {
    if !*open {
        return;
    }
    let area = Area::new(id).order(Order::Foreground).fixed_pos(pos2(anchor.left(), anchor.bottom() + 2.0)).show(
        ui.ctx(),
        |ui| {
            Frame::popup(ui.style())
                .fill(colors::PANEL)
                .stroke(egui::Stroke::new(1.0, colors::DIVIDER))
                .corner_radius(6)
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, add_contents);
        },
    );
    let clicked_outside = ui.input(|i| {
        i.pointer.any_pressed()
            && i.pointer.interact_pos().is_some_and(|p| !area.response.rect.contains(p) && !anchor.contains(p))
    });
    if clicked_outside {
        *open = false;
    }
}
