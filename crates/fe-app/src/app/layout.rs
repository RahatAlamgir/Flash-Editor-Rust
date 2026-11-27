//! The window layout, like the C# `MainWindow.xaml`: ribbon, hint bar, status bar, toolbox, right column, and
//! in the middle the project tabs, the page strip and the canvas (or the Grid view of the pages).

use egui::{Frame, Panel, Ui};
use fe_ui_kit::theme::{colors, metrics};

use super::FlashApp;
use crate::canvas;
use crate::panels::{self, ribbon};

pub fn show(app: &mut FlashApp, ui: &mut Ui) {
    if app.launcher {
        // Start-up: just the Project Manager (it is drawn by the caller). If it was closed without opening anything (a cancelled
        // dialog, a file that would not open) it comes back; its Close button is the way into the empty editor.
        if !app.manager.open && app.dialog.is_none() && !app.status.starts_with("Opening") {
            crate::dialogs::project_manager::open(app, ui.ctx());
        }
        // Same color as the manager, which fills the window, so it reads as one window and not a box on black.
        ui.painter().rect_filled(ui.max_rect(), 0, ui.ctx().global_style().visuals.window_fill);
        return;
    }
    let plain = |fill| Frame::new().fill(fill);
    Panel::top("ribbon").frame(Frame::new()).resizable(false).show_separator_line(false).show(ui, |ui| {
        // Which buttons are dimmed is worked out first (the ribbon then borrows its own state).
        let disabled: Vec<&str> = ribbon::model::TABS
            .iter()
            .flat_map(|t| t.groups.iter().flat_map(|g| g.items.iter().map(|i| i.label)))
            .filter(|label| !panels::ribbon_enabled(app, label))
            .collect();
        let enabled = |label: &str| !disabled.contains(&label);
        crate::commands::presentation::prepare_ribbon(app);
        if let Some(action) = ribbon::show(ui, &mut app.ribbon, &mut app.aids, &enabled) {
            panels::handle_ribbon_action(app, ui.ctx(), action);
        }
        crate::commands::presentation::apply_ribbon_edit(app);
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
    // The right column folds smoothly: its width follows `side` (0 open, 1 folded). While it is moving the full column is drawn,
    // cut off by the narrowing panel, so it slides instead of being squeezed.
    let side = panels::collapse::amount(ui.ctx(), "right", app.right_collapsed);
    let right_w = panels::collapse::between(metrics::RIGHT_COLUMN_W, panels::collapse::STRIP, side);
    Panel::right("right-column").frame(plain(colors::PANEL)).exact_size(right_w).resizable(false).show(ui, |ui| {
        let panel = ui.max_rect();
        if side < 0.999 {
            let full =
                egui::Rect::from_min_max(egui::pos2(panel.right() - metrics::RIGHT_COLUMN_W, panel.top()), panel.max);
            // A child that does not report what it used: whatever is inside (a wide row, a scroll bar appearing) must never
            // make the column, and with it the canvas next to it, a few pixels wider or narrower.
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(full));
            child.set_clip_rect(panel.intersect(ui.clip_rect()));
            child.style_mut().spacing.scroll = egui::style::ScrollStyle::floating();
            panels::right::show(&mut child, app);
        }
        if panels::collapse::side_handle(ui.ctx(), panel, app.right_collapsed) {
            app.right_collapsed = !app.right_collapsed;
        }
    });
    egui::CentralPanel::no_frame().show(ui, |ui| {
        Panel::top("project-tabs").frame(plain(colors::PANEL)).exact_size(metrics::PROJECT_TAB).show(ui, |ui| {
            panels::project_tabs::show(ui, app);
        });
        let bottom = panels::collapse::amount(ui.ctx(), "pages", app.pages_collapsed);
        let strip_h = panels::collapse::between(metrics::PAGE_STRIP, panels::collapse::STRIP, bottom);
        Panel::bottom("page-strip").frame(plain(colors::PANEL)).exact_size(strip_h).show(ui, |ui| {
            let panel = ui.max_rect();
            if bottom < 0.999 {
                let full = egui::Rect::from_min_size(panel.min, egui::vec2(panel.width(), metrics::PAGE_STRIP));
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(full));
                child.set_clip_rect(panel.intersect(ui.clip_rect()));
                panels::pages::show(&mut child, app);
            } else {
                let label = format!("Pages ({})", app.project().pages().len());
                if panels::collapse::folded_bar(ui, panel, &label) {
                    app.pages_collapsed = false;
                }
            }
            if panels::collapse::bottom_handle(ui.ctx(), panel, app.pages_collapsed) {
                app.pages_collapsed = !app.pages_collapsed;
            }
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
