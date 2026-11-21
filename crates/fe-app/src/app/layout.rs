//! The window layout, like the C# `MainWindow.xaml`: ribbon, hint bar, status bar, toolbox, right column, and
//! in the middle the project tabs, the page strip and the canvas (or the Grid view of the pages).

use egui::{Frame, Panel, Ui};
use fe_ui_kit::theme::{colors, metrics};

use super::FlashApp;
use crate::canvas;
use crate::panels::{self, ribbon};

pub fn show(app: &mut FlashApp, ui: &mut Ui) {
    let plain = |fill| Frame::new().fill(fill);
    Panel::top("ribbon").frame(Frame::new()).resizable(false).show_separator_line(false).show(ui, |ui| {
        // Which buttons are dimmed is worked out first (the ribbon then borrows its own state).
        let disabled: Vec<&str> = ribbon::model::TABS
            .iter()
            .flat_map(|t| t.groups.iter().flat_map(|g| g.items.iter().map(|i| i.label)))
            .filter(|label| !panels::ribbon_enabled(app, label))
            .collect();
        let enabled = |label: &str| !disabled.contains(&label);
        if let Some(action) = ribbon::show(ui, &mut app.ribbon, &mut app.aids, &enabled) {
            panels::handle_ribbon_action(app, ui.ctx(), action);
        }
    });
    Panel::top("hint-bar").frame(plain(colors::PANEL_HEADER)).exact_size(metrics::HINT_BAR).show(ui, |ui| {
        panels::hint_bar::show(ui, &mut app.tools);
    });
    Panel::bottom("status-bar").frame(plain(colors::PANEL)).exact_size(metrics::STATUS_BAR).show(ui, |ui| {
        panels::status_bar::show(ui, app);
    });
    Panel::left("toolbox")
        .frame(plain(colors::PANEL))
        .exact_size(metrics::TOOLBOX_W)
        .resizable(false)
        .show(ui, |ui| panels::toolbox::show(ui, app));
    Panel::right("right-column")
        .frame(plain(colors::PANEL))
        .exact_size(metrics::RIGHT_COLUMN_W)
        .resizable(false)
        .show(ui, |ui| panels::right::show(ui, app));
    egui::CentralPanel::no_frame().show(ui, |ui| {
        Panel::top("project-tabs").frame(plain(colors::PANEL)).exact_size(metrics::PROJECT_TAB).show(ui, |ui| {
            panels::project_tabs::show(ui, app);
        });
        Panel::bottom("page-strip").frame(plain(colors::PANEL)).exact_size(metrics::PAGE_STRIP).show(ui, |ui| {
            panels::pages::show(ui, app);
        });
        egui::CentralPanel::no_frame().show(ui, |ui| {
            if app.pages_ui.grid {
                panels::pages::grid::show(ui, app);
            } else {
                canvas::show(ui, app);
            }
        });
    });
}
