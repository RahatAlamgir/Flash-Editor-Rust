//! The Animation pane: the animations of the shown slide in the order they play, with the layer, the effect and when it
//! starts. Click one to choose it (its timing is then edited in the Animations tab); the cross removes it.

use egui::{Align2, Color32, RichText, Sense, vec2};
use fe_core::document::animation::EffectClass;
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::commands::presentation;
use crate::dialogs::frame::titled_frame;

fn class_color(class: EffectClass) -> Color32 {
    match class {
        EffectClass::Entrance => Color32::from_rgb(0x34, 0xC7, 0x59),
        EffectClass::Emphasis => Color32::from_rgb(0xFF, 0xCC, 0x00),
        EffectClass::Exit => Color32::from_rgb(0xFF, 0x3B, 0x30),
    }
}

/// Draws the pane when it is open (a floating window on the right).
pub fn show(ctx: &egui::Context, app: &mut FlashApp) {
    if !app.ribbon.animation_pane || !app.project().presentation {
        return;
    }
    let Some(page) = app.project().pages().get(app.project().active_index()) else { return };
    let rows: Vec<(usize, String, fe_core::document::animation::Animation)> = page
        .animations
        .iter()
        .enumerate()
        .filter_map(|(i, a)| page.doc.layer(a.layer).map(|l| (i, l.name.clone(), *a)))
        .collect();
    let timeline = fe_core::document::animation::timeline(&page.animations);
    let selected = app.ribbon.animation_selected;
    let (mut choose, mut remove, mut close) = (None, None, false);
    let mut open = true;
    egui::Window::new("Animation Pane")
        .id(egui::Id::NULL.with("animation-pane"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .frame(titled_frame(ctx))
        .anchor(Align2::RIGHT_TOP, vec2(-330.0, 150.0))
        .show(ctx, |ui| {
            ui.set_width(300.0);
            if rows.is_empty() {
                ui.label(
                    RichText::new("No animations on this slide. Select a layer, then choose Add Animation.")
                        .color(colors::SUBTLE_TEXT),
                );
            }
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for (i, name, a) in &rows {
                    let step = timeline.items.get(*i).map_or(0, |t| t.step);
                    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
                    if selected == Some(*i) {
                        ui.painter().rect_filled(rect, 4, colors::ACCENT_SOFT);
                    } else if response.hovered() {
                        ui.painter().rect_filled(rect, 4, colors::HOVER);
                    }
                    let mid = rect.center().y;
                    let painter = ui.painter();
                    painter.circle_filled(egui::pos2(rect.left() + 10.0, mid), 5.0, class_color(a.effect.class()));
                    let number = if step == 0 { "auto".to_owned() } else { format!("click {step}") };
                    painter.text(
                        egui::pos2(rect.left() + 24.0, rect.top() + 12.0),
                        Align2::LEFT_CENTER,
                        name.chars().take(34).collect::<String>(),
                        egui::FontId::proportional(12.5),
                        colors::TEXT,
                    );
                    painter.text(
                        egui::pos2(rect.left() + 24.0, rect.top() + 28.0),
                        Align2::LEFT_CENTER,
                        format!("{} \u{00B7} {} \u{00B7} {number}", a.effect.name(), a.start.name()),
                        egui::FontId::proportional(11.0),
                        colors::SUBTLE_TEXT,
                    );
                    let cross = egui::Rect::from_center_size(egui::pos2(rect.right() - 12.0, mid), vec2(18.0, 18.0));
                    let cross_response =
                        ui.interact(cross, egui::Id::NULL.with(("animation-remove", *i)), Sense::click());
                    painter.text(
                        cross.center(),
                        Align2::CENTER_CENTER,
                        "\u{2715}",
                        egui::FontId::proportional(11.0),
                        colors::SUBTLE_TEXT,
                    );
                    if cross_response.on_hover_text("Remove this animation").clicked() {
                        remove = Some(*i);
                    } else if response.clicked() {
                        choose = Some(*i);
                    }
                }
            });
            ui.add_space(6.0);
            if ui.button("Close").clicked() {
                close = true;
            }
        });
    if let Some(i) = choose {
        app.ribbon.animation_selected = Some(i);
    }
    if let Some(i) = remove {
        presentation::remove(app, i);
    }
    if !open || close {
        app.ribbon.animation_pane = false;
    }
}
