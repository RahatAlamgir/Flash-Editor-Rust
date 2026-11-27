//! Clicks, renaming and dragging of page cards, shared by the strip and the Grid view.

use egui::{Modifiers, Rect, Stroke, Ui};
use fe_ui_kit::theme::colors;

use super::card::CardEvents;
use super::menu;
use crate::app::FlashApp;
use crate::commands::pages as cmd;

/// Handles what happened on the card of page `index`. `in_grid`: the card is in the Grid view (not in the strip).
pub fn card_events(ui: &Ui, app: &mut FlashApp, index: usize, events: &CardEvents, in_grid: bool) {
    let response = &events.response;
    if events.menu_button {
        app.pages_ui.focused = true;
    }
    match events.rename_done {
        Some(true) => {
            if let Some((i, name)) = app.pages_ui.renaming.take() {
                cmd::rename(app, i, &name);
            }
        }
        Some(false) => app.pages_ui.renaming = None,
        None => {}
    }
    if response.double_clicked() {
        let name = app.project().pages()[index].name.clone();
        app.pages_ui.renaming = Some((index, name));
    } else if response.clicked() {
        click(app, index, ui.input(|i| i.modifiers));
    } else if response.secondary_clicked() && !app.pages_ui.selected.contains(&index) {
        click(app, index, Modifiers::NONE);
    } else if response.drag_started_by(egui::PointerButton::Primary) && in_grid == app.pages_ui.grid {
        // Only the strip or only the Grid view takes the drag: whichever is on top (the Grid view covers the canvas).
        let moved = if app.pages_ui.selected.contains(&index) { cmd::picked(app) } else { vec![index] };
        app.pages_ui.drag = Some((moved, index));
    }
    let picked = if app.pages_ui.selected.contains(&index) { cmd::picked(app) } else { vec![index] };
    let mut chosen = None;
    response.context_menu(|ui| chosen = menu::show(ui, app, &picked));
    if events.menu_button {
        app.pages_ui.selected = vec![index];
    }
    if let Some(item) = chosen {
        menu::run(app, index, &picked, item);
    }
}

fn click(app: &mut FlashApp, index: usize, mods: Modifiers) {
    app.pages_ui.focused = true;
    if mods.command {
        let selected = &mut app.pages_ui.selected;
        if let Some(pos) = selected.iter().position(|&i| i == index) {
            if selected.len() > 1 {
                selected.remove(pos);
            }
        } else {
            selected.push(index);
        }
        app.pages_ui.anchor = Some(index);
    } else if mods.shift
        && let Some(anchor) = app.pages_ui.anchor
    {
        app.pages_ui.selected = (anchor.min(index)..=anchor.max(index)).collect();
    } else {
        app.pages_ui.selected = vec![index];
        app.pages_ui.anchor = Some(index);
        cmd::show(app, index);
    }
}

/// While pages are dragged: the gap under the pointer (reading order of `cards`), a drop line, and the drop
/// when the button is released.
pub fn drag_feedback(ui: &Ui, app: &mut FlashApp, cards: &[(usize, Rect)]) {
    let Some((moved, _)) = app.pages_ui.drag.clone() else { return };
    let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) else { return };
    let gap = cards
        .iter()
        .find(|(_, r)| pointer.y < r.top() || (pointer.y <= r.bottom() && pointer.x < r.center().x))
        .map_or(cards.len(), |(i, _)| *i);
    let line_rect = cards
        .iter()
        .find(|(i, _)| *i == gap)
        .map(|(_, r)| (r.left() - 6.0, r))
        .or_else(|| cards.last().map(|(_, r)| (r.right() + 6.0, r)));
    if let Some((x, r)) = line_rect {
        ui.painter().vline(x, r.y_range(), Stroke::new(3.0, colors::ACCENT));
    }
    app.pages_ui.drag = Some((moved.clone(), gap));
    if !ui.input(|i| i.pointer.primary_down()) {
        app.pages_ui.drag = None;
        cmd::move_to_gap(app, &moved, gap);
    }
}
