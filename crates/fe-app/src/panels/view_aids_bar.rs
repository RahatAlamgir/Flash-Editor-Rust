//! On / off buttons above the canvas: rulers, grid, guides, snapping (C# `ViewAidsBar`). The small arrow
//! next to a button opens its settings.

use egui::{Id, RichText, Stroke, Ui, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::{aid_toggle, popup_below};

use crate::app::{FlashApp, GRID_SIZES};

/// Which settings card is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AidPopup {
    Grid,
    Guides,
    Snap,
}

/// Draws the buttons right to left (the caller's layout is right-to-left).
pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    ui.add_space(6.0);
    more_arrow(ui, app, AidPopup::Snap, "What to snap to");
    aid_toggle(
        ui,
        "Magnet",
        &mut app.aids.snapping,
        "Snapping (Ctrl+Shift+;): things click into place against the lines below while you drag them",
    );
    let (rect, _) = ui.allocate_exact_size(vec2(17.0, 18.0), egui::Sense::hover());
    ui.painter().vline(rect.center().x, rect.y_range(), Stroke::new(1.0, colors::DIVIDER));
    more_arrow(ui, app, AidPopup::Guides, "Guide settings");
    aid_toggle(ui, "Guides", &mut app.aids.guides, "Show guide lines (Ctrl+;)");
    ui.add_space(2.0);
    more_arrow(ui, app, AidPopup::Grid, "Grid settings");
    aid_toggle(ui, "Grid", &mut app.aids.grid, "Grid (Ctrl+')");
    ui.add_space(2.0);
    aid_toggle(
        ui,
        "Ruler",
        &mut app.aids.rulers,
        "Rulers (Ctrl+R). Drag from a ruler onto the picture to make a guide line.",
    );
}

/// The small ▾ next to a button; opens (or closes) its settings card.
fn more_arrow(ui: &mut Ui, app: &mut FlashApp, which: AidPopup, tip: &str) {
    let (rect, response) = ui.allocate_exact_size(vec2(13.0, 26.0), egui::Sense::click());
    let open = app.aid_popup == Some(which);
    if response.hovered() || open {
        ui.painter().rect_filled(rect, 4, if open { colors::ACCENT_SOFT } else { colors::HOVER });
    }
    let c = rect.center();
    let points = vec![c + vec2(-3.0, -1.5), c + vec2(3.0, -1.5), c + vec2(0.0, 2.0)];
    ui.painter().add(egui::Shape::convex_polygon(points, colors::ICON, Stroke::NONE));
    if response.on_hover_text(tip).clicked() {
        app.aid_popup = if open { None } else { Some(which) };
    }
    if app.aid_popup == Some(which) {
        let mut still_open = true;
        popup_below(ui, Id::unique(("aid-popup", which as u8)), rect, &mut still_open, |ui| card(ui, app, which));
        if !still_open {
            app.aid_popup = None;
        }
    }
}

fn card(ui: &mut Ui, app: &mut FlashApp, which: AidPopup) {
    let aids = &mut app.aids;
    let title = |ui: &mut Ui, text: &str| ui.label(RichText::new(text).strong().color(colors::TEXT));
    match which {
        AidPopup::Grid => {
            title(ui, "Grid");
            ui.horizontal(|ui| {
                ui.label("Lines every");
                egui::ComboBox::from_id_salt("grid-size")
                    .width(70.0)
                    .selected_text(format!("{}", aids.grid_size))
                    .show_ui(ui, |ui| {
                        for size in GRID_SIZES {
                            ui.selectable_value(&mut aids.grid_size, size, format!("{size}"));
                        }
                    });
                ui.label(RichText::new("px").color(colors::SUBTLE_TEXT));
            });
            ui.add_space(6.0);
            ui.checkbox(&mut aids.snap_to_grid, "Snap to the grid");
        }
        AidPopup::Guides => {
            title(ui, "Guide lines");
            ui.set_max_width(230.0);
            ui.label(
                RichText::new(
                    "Drag from a ruler to make one. Drag a guide with the Select / Move tool to move it, or drag it back onto the ruler to remove it.",
                )
                .color(colors::SUBTLE_TEXT),
            );
            ui.add_space(6.0);
            ui.checkbox(&mut aids.snap_to_guides, "Snap to guides");
            ui.add_space(6.0);
            if ui.button("Clear all guides").clicked() {
                let page = app.projects[app.active_project].active_page_mut();
                page.vertical_guides.clear();
                page.horizontal_guides.clear();
            }
        }
        AidPopup::Snap => {
            title(ui, "Snap to");
            ui.checkbox(&mut aids.snap_to_objects, "Other objects (edges and centers)");
            ui.checkbox(&mut aids.snap_to_canvas, "Canvas edges and center");
            ui.checkbox(&mut aids.snap_to_spacing, "Equal spacing between objects (shows the gap in px)");
            ui.checkbox(&mut aids.snap_to_grid, "Grid (while the grid is shown)");
            ui.checkbox(&mut aids.snap_to_guides, "Guides (while they are shown)");
        }
    }
}
