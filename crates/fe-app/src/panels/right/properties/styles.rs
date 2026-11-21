//! Properties > Styles (C# `ObjectPropertiesViewModel`, styles part): save the look of the selected object under a
//! name and put it on other objects with one click.

use egui::Ui;

use crate::app::FlashApp;
use crate::commands::{parts, styles};
use crate::dialogs::Dialog;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    if ui
        .button("Save this look…")
        .on_hover_text("Keeps the shadow, 3D, gradient, colors and other looks of this object, to use on other objects")
        .clicked()
    {
        let kind = parts::target_object(app).map(|o| o.kind().to_owned()).unwrap_or_default();
        let count = styles::list(app).len() + 1;
        app.dialog = Some(Dialog::SaveStyle(format!("{} style {count}", capitalized(&kind))));
    }
    let names: Vec<String> = styles::list(app).iter().map(|s| s.name.clone()).collect();
    if names.is_empty() {
        super::note(ui, "No saved looks yet.");
        return;
    }
    for (i, name) in names.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.button(name).on_hover_text("Put this look on the selected object").clicked() {
                styles::apply(app, i);
            }
            if ui.small_button("✕").on_hover_text("Delete this saved look").clicked() {
                styles::remove(app, i);
            }
        });
    }
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
