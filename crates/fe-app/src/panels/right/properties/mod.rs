//! The Properties panel (C# `PropertiesPanel`): for the selected layer the Transform, Align and Shape sections
//! (`transform.rs`, `align.rs`, `shape.rs`). Every layer has the same Transform section: changing the size or
//! rotation of a plain pixel layer turns it into a movable picture first (Ctrl+Z undoes it). Text, effects and
//! the other sections arrive in Phase 9.

mod adjust;
mod adjustment;
mod align;
mod barcode;
mod chart;
mod frame;
mod gradient;
mod looks;
mod qr;
mod shape;
mod styles;
mod text;
mod text_designs;
mod transform;

use egui::{RichText, Ui};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::panel_title_bar;

use crate::app::FlashApp;

/// A label of the fixed width C# uses in the panel.
pub(super) fn label(ui: &mut Ui, text: &str) {
    ui.add_sized([58.0, 22.0], egui::Label::new(RichText::new(text).color(colors::SUBTLE_TEXT)));
}

/// A small grey explanation.
pub(super) fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(11.0).color(colors::SUBTLE_TEXT)).wrap());
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let target = crate::commands::parts::target_object(app);
    let in_part = crate::commands::parts::active_part(app).is_some();
    let title = match app.doc().active_layer() {
        Some(_) if app.doc().selected_ids().len() > 1 => format!("{} layers", app.doc().selected_ids().len()),
        Some(l) => match target.as_ref().map(|o| o.kind().to_owned()) {
            Some(k) if k == "shape" => {
                let name = target.as_ref().and_then(|o| o.shape()).map_or("Shape".into(), |s| s.kind.name().to_owned());
                if in_part { format!("{name} in group") } else { name }
            }
            Some(k) if k == "qr" => "QR code".into(),
            Some(k) => {
                let mut c = k.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            }
            None if l.is_adjustment() => "Adjustment".into(),
            None => "Pixels".into(),
        },
        None => String::new(),
    };
    panel_title_bar(ui, "Properties", |ui| {
        ui.label(RichText::new(title).size(11.0).color(colors::SUBTLE_TEXT));
    });
    let Some(layer) = app.doc().active_layer() else {
        empty_hint(ui);
        return;
    };
    if layer.is_adjustment() {
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| adjustment::show(ui, app));
        });
        return;
    }
    let locked = layer.locked;
    let pixels = layer.object().is_none();
    if app.props_tab >= TABS.len() {
        app.props_tab = 0;
    }
    tab_row(ui, app);
    let tab = TABS[app.props_tab];
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
            if locked {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("This layer is locked.").color(egui::Color32::from_rgb(0xFF, 0xB0, 0x20)));
                    if ui.button("Unlock").on_hover_text("Unlock this layer so it can be changed (Ctrl+L)").clicked() {
                        crate::commands::layers::toggle_lock(app);
                    }
                });
                return;
            }
            let kinds = selected_kinds(app);
            let all = |names: &[&str]| !kinds.is_empty() && kinds.iter().all(|k| names.contains(&k.as_str()));
            let styled = !kinds.is_empty() && !kinds.iter().any(|k| k == "pixels");
            match tab {
                "Effects" | "Adjust" => {
                    if !styled {
                        note(ui, "Effects and Adjust work on shapes, text and pictures. This layer is plain pixels.");
                    } else if tab == "Effects" {
                        egui::CollapsingHeader::new("Styles").default_open(false).show(ui, |ui| styles::show(ui, app));
                        egui::CollapsingHeader::new("Effects").default_open(true).show(ui, |ui| looks::show(ui, app));
                    } else {
                        adjust::show(ui, app);
                    }
                    return;
                }
                _ => {}
            }
            if pixels {
                note(
                    ui,
                    "This layer is plain pixels. Changing its size or rotation here turns it into a movable picture \
                     automatically (Ctrl+Z undoes it).",
                );
                ui.add_space(4.0);
            }
            egui::CollapsingHeader::new("Transform").default_open(true).show(ui, |ui| transform::show(ui, app));
            egui::CollapsingHeader::new("Align").default_open(true).show(ui, |ui| align::show(ui, app));
            // With several layers selected only the sections that fit all of them show; a change goes to all of them.
            if all(&["text"]) {
                egui::CollapsingHeader::new("Text").default_open(true).show(ui, |ui| text::show(ui, app));
                egui::CollapsingHeader::new("Text designs")
                    .default_open(false)
                    .show(ui, |ui| text_designs::show(ui, app));
            }
            if all(&["shape", "text", "image"]) {
                egui::CollapsingHeader::new("Gradient").default_open(false).show(ui, |ui| gradient::show(ui, app));
            }
            if kinds.len() == 1
                && crate::commands::parts::target_object(app).is_some_and(|o| o.picture_fill().is_some())
            {
                egui::CollapsingHeader::new("Picture in frame").default_open(true).show(ui, |ui| frame::show(ui, app));
            }
            if all(&["qr"]) {
                egui::CollapsingHeader::new("QR code").default_open(true).show(ui, |ui| qr::show(ui, app));
            }
            if all(&["barcode"]) {
                egui::CollapsingHeader::new("Barcode").default_open(true).show(ui, |ui| barcode::show(ui, app));
            }
            if all(&["chart"]) {
                egui::CollapsingHeader::new("Chart").default_open(true).show(ui, |ui| chart::show(ui, app));
            }
            if all(&["shape"]) {
                egui::CollapsingHeader::new("Shape").default_open(true).show(ui, |ui| shape::show(ui, app));
            }
        });
    });
}

/// The kinds of the objects Properties works on: the active part of a group, or every selected layer that holds
/// an object ("" for a layer that is plain pixels, which no object section fits).
fn selected_kinds(app: &FlashApp) -> Vec<String> {
    if crate::commands::parts::active_part(app).is_some() {
        return crate::commands::parts::target_object(app).map(|o| o.kind().to_owned()).into_iter().collect();
    }
    let doc = app.doc();
    let ids = doc.selected_ids();
    let ids = if ids.is_empty() { doc.active_id().into_iter().collect() } else { ids };
    ids.iter()
        .filter_map(|id| doc.layer(*id))
        .map(|l| l.object().map(|o| o.kind().to_owned()).unwrap_or_default())
        .map(|k| if k.is_empty() { "pixels".to_owned() } else { k })
        .collect()
}

fn empty_hint(ui: &mut Ui) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        note(
            ui,
            "Click a shape, text or image with the Select / Move tool (V) to change its color, border, size, rotation or \
             font here.",
        );
    });
}

/// The tabs of the column: what each holds is in `show`.
const TABS: [&str; 3] = ["Properties", "Effects", "Adjust"];

/// The row of tabs under the title bar.
fn tab_row(ui: &mut Ui, app: &mut FlashApp) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.x = 2.0;
        for (i, name) in TABS.iter().enumerate() {
            let on = app.props_tab == i;
            let text = RichText::new(*name).size(12.0).color(if on { colors::TEXT } else { colors::SUBTLE_TEXT });
            if ui.add(egui::Button::selectable(on, text)).clicked() {
                app.props_tab = i;
            }
        }
    });
    ui.add_space(2.0);
}
