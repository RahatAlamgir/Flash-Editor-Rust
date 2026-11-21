//! The Layers panel (C# `LayersPanel.xaml`): blend mode and opacity of the active layer, the four quick
//! buttons, the layer list (top layer first) and the buttons along the bottom.

mod footer;
mod header;
mod list;
mod menu;
mod part_row;
mod row;

use egui::{Rect, Stroke, Ui, UiBuilder, pos2};
use fe_core::document::LayerId;
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::panel_title_bar;

use crate::app::FlashApp;

const FOOTER_H: f32 = 32.0;

/// What the Layers panel remembers between frames.
#[derive(Default)]
pub struct LayersUi {
    /// The layer whose name is being edited, and the text so far.
    pub renaming: Option<(LayerId, String)>,
    /// Layers being dragged to a new place, and the gap they would drop into.
    pub drag: Option<(Vec<LayerId>, usize)>,
    /// The layer clicked last without Shift (start of a Shift+click range).
    pub anchor: Option<LayerId>,
    /// Groups whose parts are listed under them.
    pub expanded: std::collections::HashSet<LayerId>,
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let area = ui.max_rect();
    panel_title_bar(ui, "Layers", |_| {});
    header::mix_and_opacity(ui, app);
    header::quick_buttons(ui, app);
    let list_top = ui.cursor().top();
    ui.painter().hline(area.x_range(), list_top + 0.5, Stroke::new(1.0, colors::DIVIDER));
    let footer = Rect::from_min_max(pos2(area.left(), area.bottom() - FOOTER_H), area.max);
    let list = Rect::from_min_max(pos2(area.left(), list_top + 1.0), pos2(area.right(), footer.top()));
    ui.scope_builder(UiBuilder::new().max_rect(list), |ui| list::show(ui, app));
    ui.scope_builder(UiBuilder::new().max_rect(footer), |ui| footer::show(ui, app));
}

/// Starts editing the active layer's name (Layer tab > Rename, F2).
pub fn start_rename_active(app: &mut FlashApp) {
    if let Some(layer) = app.doc().active_layer() {
        app.layers_ui.renaming = Some((layer.id(), layer.name.clone()));
    }
}
