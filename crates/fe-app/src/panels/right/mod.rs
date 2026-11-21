//! The right-hand column: Properties, Layers and History at the bottom, like the C# `MainWindow.xaml` grid.
//! The bar between Properties and Layers can be dragged to give one of them more room.

mod history;
pub mod layers;
mod properties;

use egui::{CursorIcon, Id, Rect, Sense, Stroke, Ui, UiBuilder, pos2};
use fe_ui_kit::theme::{colors, metrics};

use crate::app::FlashApp;

const HISTORY_LIST: f32 = 180.0;
const SPLITTER: f32 = 4.0;
const MIN_PROPERTIES: f32 = 150.0;
const MIN_LAYERS: f32 = 200.0;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let area = ui.max_rect();
    ui.painter().vline(area.left() + 0.5, area.y_range(), Stroke::new(1.0, colors::DIVIDER));
    let history_h = metrics::PANEL_TITLE + if app.history_expanded { HISTORY_LIST } else { 0.0 };
    let rest = (area.height() - history_h - SPLITTER).max(MIN_PROPERTIES + MIN_LAYERS);
    let props_h = (rest * app.props_share).clamp(MIN_PROPERTIES, rest - MIN_LAYERS);

    let props = Rect::from_min_max(pos2(area.left() + 1.0, area.top()), pos2(area.right(), area.top() + props_h));
    let splitter =
        Rect::from_min_max(pos2(props.left(), props.bottom()), pos2(props.right(), props.bottom() + SPLITTER));
    let layers =
        Rect::from_min_max(pos2(props.left(), splitter.bottom()), pos2(props.right(), area.bottom() - history_h));
    let history = Rect::from_min_max(pos2(props.left(), layers.bottom()), pos2(props.right(), area.bottom()));

    // The divider can be dragged.
    let drag = ui.interact(splitter.expand2(egui::vec2(0.0, 2.0)), Id::unique("props-layers-splitter"), Sense::drag());
    if drag.hovered() || drag.dragged() {
        ui.set_cursor_icon(CursorIcon::ResizeVertical);
    }
    if drag.dragged() {
        app.props_share = ((props_h + drag.drag_delta().y) / rest).clamp(0.1, 0.9);
    }
    let fill = if drag.dragged() { colors::ACCENT } else { colors::DIVIDER };
    ui.painter().rect_filled(splitter, 0, fill);

    ui.scope_builder(UiBuilder::new().max_rect(props), |ui| properties::show(ui, app));
    ui.scope_builder(UiBuilder::new().max_rect(layers), |ui| layers::show(ui, app));
    ui.painter().hline(history.x_range(), history.top() + 0.5, Stroke::new(1.0, colors::DIVIDER));
    ui.scope_builder(UiBuilder::new().max_rect(history), |ui| history::show(ui, app));
}
