//! The bar under the ribbon: the active tool's icon and name, then its hint (C# `OptionsBar`).
//! Tools with settings show them here (see `tools::options_bar`).

use egui::{Align, FontFamily, FontId, Layout, RichText, Ui};
use fe_ui_kit::icons;
use fe_ui_kit::theme::{colors, fonts};

use crate::tools::{ToolState, options_bar};

pub fn show(ui: &mut Ui, tools: &mut ToolState) {
    let tool = tools.active();
    let rect = ui.max_rect();
    ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, egui::Stroke::new(1.0, colors::DIVIDER));
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(12.0);
        ui.add(icons::icon(tool.icon(), 18.0, colors::ICON));
        ui.add_space(2.0);
        let name_font = FontId::new(13.0, FontFamily::Name(fonts::SEMIBOLD.into()));
        ui.label(RichText::new(tool.name()).font(name_font).color(colors::TEXT));
        ui.add_space(16.0);
        if !options_bar::show(ui, tools) {
            ui.add(egui::Label::new(RichText::new(tool.hint()).color(colors::SUBTLE_TEXT)).truncate());
        }
    });
}
