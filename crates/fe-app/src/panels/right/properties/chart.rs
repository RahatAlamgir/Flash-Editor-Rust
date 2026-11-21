//! Properties > Chart (C# `ChartEditorItems` and the chart part of `ObjectPropertiesViewModel`): the kind, the title,
//! the table of numbers with a color for each, what is shown, and the colors.

use egui::{Color32, Ui};
use fe_core::imaging::ColorRgba;
use fe_core::objects::chart::palettes;
use fe_core::objects::{ChartItem, ChartKind};

use super::label;
use super::shape::{slider, swatch};
use crate::app::FlashApp;
use crate::commands::codes::{ObjectColor, change_chart};
use crate::commands::parts;
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

/// The most rows a chart can have (like the C# parser).
const MAX_ITEMS: usize = 60;

fn pick(app: &mut FlashApp, which: ObjectColor, current: ColorRgba) {
    app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::Object(which), current)));
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(chart) = parts::target_object(app).and_then(|o| o.chart()) else { return };
    ui.horizontal_wrapped(|ui| {
        for kind in ChartKind::ALL {
            if ui.selectable_label(chart.kind == kind, kind.name()).clicked() {
                change_chart(app, "Chart Type", |d| d.kind = kind);
            }
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        label(ui, "Title");
        let mut title = chart.title.clone();
        if ui.add(egui::TextEdit::singleline(&mut title).desired_width(f32::INFINITY)).changed() {
            change_chart(app, "Chart Title", |d| d.title = title.clone());
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Numbers").strong());
    numbers(ui, app, &chart.items, &chart);
    ui.add_space(4.0);
    let mut flags = (chart.show_values, chart.show_labels, chart.show_grid);
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut flags.0, "Numbers");
        ui.checkbox(&mut flags.1, "Names");
        ui.checkbox(&mut flags.2, "Grid");
    });
    if flags != (chart.show_values, chart.show_labels, chart.show_grid) {
        change_chart(app, "Chart Display", |d| {
            d.show_values = flags.0;
            d.show_labels = flags.1;
            d.show_grid = flags.2;
        });
    }
    let mut bar = chart.bar_width * 100.0;
    if slider(ui, "Bar width", &mut bar, 20.0..=100.0, "%") {
        change_chart(app, "Chart Bar Width", |d| d.bar_width = (bar / 100.0).clamp(0.2, 1.0));
    }
    let mut scale = chart.text_scale * 100.0;
    if slider(ui, "Text size", &mut scale, 40.0..=300.0, "%") {
        change_chart(app, "Chart Text Size", |d| d.text_scale = (scale / 100.0).clamp(0.4, 3.0));
    }
    ui.horizontal(|ui| {
        label(ui, "Before");
        let mut prefix = chart.value_prefix.clone();
        if ui
            .add(egui::TextEdit::singleline(&mut prefix).desired_width(48.0))
            .on_hover_text("Written before every number, for example $")
            .changed()
        {
            change_chart(app, "Chart Prefix", |d| d.value_prefix = prefix.clone());
        }
        ui.label("After");
        let mut suffix = chart.value_suffix.clone();
        if ui
            .add(egui::TextEdit::singleline(&mut suffix).desired_width(48.0))
            .on_hover_text("Written after every number, for example % or kg")
            .changed()
        {
            change_chart(app, "Chart Suffix", |d| d.value_suffix = suffix.clone());
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Text");
        if swatch(ui, Some(chart.ink()), "Color of the text") {
            pick(app, ObjectColor::ChartText, chart.ink());
        }
        if chart.text_color.is_some() && ui.small_button("Reset").clicked() {
            change_chart(app, "Chart Text Color", |d| d.text_color = None);
        }
    });
    ui.horizontal(|ui| {
        let mut on = chart.background.is_some();
        if ui.add_sized([72.0, 22.0], egui::Checkbox::new(&mut on, "Back")).changed() {
            change_chart(app, "Chart Background", |d| d.background = on.then_some(ColorRgba::WHITE));
        }
        if swatch(ui, chart.background, "Color behind the chart") {
            pick(app, ObjectColor::ChartBackground, chart.background.unwrap_or(ColorRgba::WHITE));
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Color sets").strong());
    for (name, colors) in palettes() {
        ui.horizontal(|ui| {
            let response = ui.add_sized([70.0, 20.0], egui::Button::new(name));
            for c in colors.iter().take(6) {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2, Color32::from_rgb(c.r, c.g, c.b));
            }
            if response.clicked() {
                change_chart(app, "Chart Colors", |d| d.colors = colors.clone());
            }
        });
    }
}

/// The table: a color, a name and a number per row, and a button to remove it.
fn numbers(ui: &mut Ui, app: &mut FlashApp, items: &[ChartItem], chart: &fe_core::objects::ChartData) {
    let mut edited: Vec<ChartItem> = items.to_vec();
    let (mut remove, mut changed) = (None, false);
    for (i, item) in edited.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            let color = chart.color_of(i);
            if swatch(ui, Some(color), "Color of this bar or slice") {
                pick(app, ObjectColor::ChartItem(i), color);
            }
            changed |= ui.add(egui::TextEdit::singleline(&mut item.label).desired_width(72.0)).changed();
            let response = ui.add(egui::DragValue::new(&mut item.value).speed(0.5).max_decimals(2));
            changed |= response.changed();
            if ui.small_button("✕").on_hover_text("Remove this row").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        edited.remove(i);
        changed = true;
    }
    if edited.len() < MAX_ITEMS && ui.button("+ Add a row").clicked() {
        edited.push(ChartItem::new(&format!("Item {}", edited.len() + 1), 10.0));
        changed = true;
    }
    if changed {
        change_chart(app, "Chart Numbers", |d| d.items = edited.clone());
    }
}
