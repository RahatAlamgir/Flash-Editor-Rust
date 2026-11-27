//! Insert > Chart: pick a kind of chart from pictures of real charts (like the Icons, Badges, Graphics and SmartArt pickers).
//! Click one to put it in the middle of the picture; the numbers and colors are changed in Properties > Chart afterwards.

use egui::{Color32, Rect, RichText, Sense, Stroke, vec2};
use fe_core::objects::ChartKind;
use fe_ui_kit::theme::colors;

use super::previews::Previews;

/// `Some(Some(kind))` = a chart was picked, `Some(None)` = closed, `None` = still open.
pub fn show(ctx: &egui::Context, previews: &mut Previews) -> Option<Option<ChartKind>> {
    let mut result = None;
    let modal = super::frame::modal(ctx, "chart-picker", |ui| {
        ui.set_width(640.0);
        ui.label(RichText::new("Chart").heading());
        ui.add_space(6.0);
        ui.label(
            RichText::new("Pick a chart. It starts with sample numbers: change them in Properties.")
                .color(colors::SUBTLE_TEXT),
        );
        ui.add_space(8.0);
        egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                for kind in ChartKind::ALL {
                    let (rect, response) = ui.allocate_exact_size(vec2(145.0, 160.0), Sense::click());
                    let fill = if response.hovered() { colors::HOVER } else { colors::PANEL_HEADER };
                    let stroke = if response.hovered() {
                        Stroke::new(1.5, colors::ACCENT)
                    } else {
                        Stroke::new(1.0, colors::DIVIDER)
                    };
                    ui.painter().rect(rect, 6, fill, stroke, egui::StrokeKind::Inside);
                    let picture = Rect::from_min_size(rect.min + vec2(10.0, 6.0), vec2(125.0, 125.0));
                    ui.painter().rect_filled(picture, 4, Color32::from_rgb(238, 238, 242));
                    if let Some(t) = previews.chart(ctx, kind) {
                        ui.painter().image(
                            t.id(),
                            picture,
                            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ui.painter().text(
                        rect.center_bottom() - vec2(0.0, 15.0),
                        egui::Align2::CENTER_CENTER,
                        kind.name(),
                        egui::FontId::proportional(12.0),
                        colors::TEXT,
                    );
                    if response.on_hover_text(kind.name()).clicked() {
                        result = Some(Some(kind));
                    }
                }
            });
        });
        ui.add_space(10.0);
        if ui.add(egui::Button::new("Close").min_size(vec2(90.0, 28.0))).clicked() {
            result = Some(None);
        }
    });
    if result.is_none() && modal.should_close() {
        result = Some(None);
    }
    result
}
