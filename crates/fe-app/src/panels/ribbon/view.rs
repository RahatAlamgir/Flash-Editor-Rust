//! Draws the whole ribbon: the tab row, then (unless hidden) the groups of big buttons of the chosen tab.

use egui::{Rect, Sense, Stroke, Ui, UiBuilder, Vec2, pos2};
use fe_ui_kit::theme::colors::{self, icon};
use fe_ui_kit::theme::metrics;
use fe_ui_kit::widgets::{RibbonButton, ribbon_button};

use super::model::{IconColor, TABS};
use super::{RibbonAction, RibbonState, tab_row};
use crate::app::ViewAids;

/// Draws the ribbon at the top of `ui` and returns what was clicked. `enabled(label)` says whether a button can
/// be used right now (dimmed when not).
pub fn show(
    ui: &mut Ui,
    state: &mut RibbonState,
    aids: &mut ViewAids,
    enabled: &dyn Fn(&str) -> bool,
) -> Option<RibbonAction> {
    let width = ui.available_width();
    let body_h = if state.collapsed { 0.0 } else { metrics::RIBBON_BODY };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, metrics::RIBBON_TAB_ROW + body_h), Sense::hover());
    let row = Rect::from_min_size(rect.min, Vec2::new(width, metrics::RIBBON_TAB_ROW));
    let body = Rect::from_min_max(pos2(rect.left(), row.bottom()), rect.max);

    // The body is drawn first so the selected tab can cover its top line.
    let mut action = None;
    if !state.collapsed {
        action = buttons(ui, body, state, aids, enabled);
    }
    tab_row::show(ui, row, state).or(action)
}

fn buttons(
    ui: &mut Ui,
    body: Rect,
    state: &RibbonState,
    aids: &mut ViewAids,
    enabled: &dyn Fn(&str) -> bool,
) -> Option<RibbonAction> {
    let painter = ui.painter();
    painter.rect_filled(body, 0, colors::PANEL);
    painter.hline(body.x_range(), body.top() + 0.5, Stroke::new(1.0, colors::DIVIDER));
    painter.hline(body.x_range(), body.bottom() - 0.5, Stroke::new(1.0, colors::DIVIDER));
    let tab_index = state.tab.min(TABS.len() - 1);
    let tab = &TABS[tab_index];
    let mut action = None;
    let mut child = ui.new_child(UiBuilder::new().max_rect(body.shrink2(Vec2::new(4.0, 0.0))));
    egui::ScrollArea::horizontal().id_salt("ribbon-body").show(&mut child, |ui| {
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (g, group) in tab.groups.iter().enumerate() {
                ui.add_space(3.0);
                let start = ui.cursor().left();
                for (i, item) in group.items.iter().enumerate() {
                    let checked = item.toggle && toggle_value(aids, item.label).is_some_and(|v| *v);
                    let color = if state.colorful { color_of(item.color) } else { colors::ICON };
                    let response = ribbon_button(
                        ui,
                        RibbonButton {
                            glyph: item.glyph,
                            label: item.label,
                            color,
                            enabled: enabled(item.label),
                            checked,
                        },
                    );
                    if response.clicked() {
                        if let Some(value) = toggle_value(aids, item.label) {
                            *value = !*value;
                        }
                        action = Some(RibbonAction::Button { tab: tab_index, group: g, item: i, rect: response.rect });
                    }
                }
                ui.add_space(3.0);
                let x = ui.cursor().left();
                let range = (body.top() + 4.0)..=(body.bottom() - 4.0);
                ui.painter().vline(x, range, Stroke::new(1.0, colors::DIVIDER));
                let group_rect = Rect::from_x_y_ranges(start..=x, body.y_range());
                ui.interact(group_rect, ui.scope_id().with(("group", g)), Sense::hover()).on_hover_text(group.name);
                ui.add_space(1.0);
            }
        });
    });
    action
}

/// The on / off value behind a View > Show toggle button.
fn toggle_value<'a>(aids: &'a mut ViewAids, label: &str) -> Option<&'a mut bool> {
    match label {
        "Rulers" => Some(&mut aids.rulers),
        "Grid" => Some(&mut aids.grid),
        "Guides" => Some(&mut aids.guides),
        "Snapping" => Some(&mut aids.snapping),
        _ => None,
    }
}

fn color_of(c: IconColor) -> egui::Color32 {
    match c {
        IconColor::Blue => icon::BLUE,
        IconColor::Sky => icon::SKY,
        IconColor::Teal => icon::TEAL,
        IconColor::Green => icon::GREEN,
        IconColor::Lime => icon::LIME,
        IconColor::Amber => icon::AMBER,
        IconColor::Orange => icon::ORANGE,
        IconColor::Red => icon::RED,
        IconColor::Pink => icon::PINK,
        IconColor::Purple => icon::PURPLE,
        IconColor::Slate => icon::SLATE,
        IconColor::Plain => colors::ICON,
    }
}
