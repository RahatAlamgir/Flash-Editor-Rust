//! The Animation tab of the Properties panel (presentations): add an animation to the selected layer, the list of every animation
//! of the slide in the order they play (click one to choose it), and the settings of the chosen one: start, direction,
//! duration, delay, order, remove. (The Animations tab of the ribbon works on the same data.)

use egui::{Align2, Color32, ComboBox, DragValue, RichText, Sense, Ui, pos2, vec2};
use fe_core::document::TransitionDirection;
use fe_core::document::animation::{Animation, Effect, EffectClass, Start, timeline};
use fe_ui_kit::theme::colors;

use super::note;
use crate::app::FlashApp;
use crate::commands::presentation;

fn dot(class: EffectClass) -> Color32 {
    match class {
        EffectClass::Entrance => Color32::from_rgb(0x34, 0xC7, 0x59),
        EffectClass::Emphasis => Color32::from_rgb(0xFF, 0xCC, 0x00),
        EffectClass::Exit => Color32::from_rgb(0xFF, 0x3B, 0x30),
    }
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let project = app.project();
    let Some(page) = project.pages().get(project.active_index()) else { return };
    let rows: Vec<(usize, String, Animation)> = page
        .animations
        .iter()
        .enumerate()
        .filter_map(|(i, a)| page.doc.layer(a.layer).map(|l| (i, l.name.clone(), *a)))
        .collect();
    let steps = timeline(&page.animations);
    let total = page.animations.len();
    let selected = app.ribbon.animation_selected.filter(|i| rows.iter().any(|(r, ..)| r == i));

    // Add.
    let mut add = None;
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("props-add-animation").width(150.0).selected_text("Add Animation").show_ui(ui, |ui| {
            for class in [EffectClass::Entrance, EffectClass::Emphasis, EffectClass::Exit] {
                ui.label(RichText::new(class.name()).strong().color(colors::SUBTLE_TEXT));
                for effect in Effect::ALL.into_iter().filter(|e| e.class() == class) {
                    if ui.selectable_label(false, effect.name()).clicked() {
                        add = Some(effect);
                    }
                }
            }
        });
        if ui.button("Preview").on_hover_text("Plays this slide's animations once").clicked() {
            crate::slideshow::start_animation_preview(app, ui.ctx());
        }
    });
    note(ui, "Adds the effect to the selected layer. In the slideshow (F5) a click plays the next step.");
    ui.add_space(6.0);

    // The list of the slide.
    ui.label(RichText::new("On this slide").strong().color(colors::SUBTLE_TEXT));
    if rows.is_empty() {
        note(ui, "No animations yet.");
    }
    let (mut choose, mut remove) = (None, None);
    for (i, name, a) in &rows {
        let step = steps.items.get(*i).map_or(0, |t| t.step);
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
        if selected == Some(*i) {
            ui.painter().rect_filled(rect, 4, colors::ACCENT_SOFT);
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 4, colors::HOVER);
        }
        let painter = ui.painter();
        painter.circle_filled(pos2(rect.left() + 10.0, rect.center().y), 5.0, dot(a.effect.class()));
        painter.text(
            pos2(rect.left() + 24.0, rect.top() + 11.0),
            Align2::LEFT_CENTER,
            name.chars().take(30).collect::<String>(),
            egui::FontId::proportional(12.5),
            colors::TEXT,
        );
        let when = if step == 0 { "by itself".to_owned() } else { format!("click {step}") };
        painter.text(
            pos2(rect.left() + 24.0, rect.top() + 27.0),
            Align2::LEFT_CENTER,
            format!("{} \u{00B7} {} \u{00B7} {when}", a.effect.name(), a.start.name()),
            egui::FontId::proportional(11.0),
            colors::SUBTLE_TEXT,
        );
        let cross = egui::Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(18.0, 18.0));
        let cross_response = ui.interact(cross, egui::Id::NULL.with(("animation-remove", *i)), Sense::click());
        ui.painter().text(
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

    // The settings of the chosen one.
    let mut changed = None;
    let mut shift = None;
    if let Some(index) = selected
        && let Some((_, name, mut a)) = rows.iter().find(|(i, ..)| *i == index).cloned()
    {
        let before = a;
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} on {}", a.effect.name(), name.chars().take(18).collect::<String>()))
                    .strong()
                    .color(colors::TEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(index + 1 < total, egui::Button::new("\u{25BC}").small())
                    .on_hover_text("Later")
                    .clicked()
                {
                    shift = Some(1);
                }
                if ui.add_enabled(index > 0, egui::Button::new("\u{25B2}").small()).on_hover_text("Earlier").clicked() {
                    shift = Some(-1);
                }
            });
        });
        ComboBox::from_id_salt("props-start").width(200.0).selected_text(a.start.name()).show_ui(ui, |ui| {
            for s in Start::ALL {
                ui.selectable_value(&mut a.start, s, s.name());
            }
        });
        if a.effect.has_direction() {
            ComboBox::from_id_salt("props-direction").width(200.0).selected_text(a.direction.name()).show_ui(
                ui,
                |ui| {
                    for d in TransitionDirection::ALL {
                        ui.selectable_value(&mut a.direction, d, d.name());
                    }
                },
            );
        }
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!a.effect.instant(), |ui| {
                ui.label(RichText::new("Duration").color(colors::SUBTLE_TEXT));
                ui.add(DragValue::new(&mut a.duration).range(0.1..=30.0).speed(0.05).fixed_decimals(1).suffix(" s"));
            });
            ui.label(RichText::new("Delay").color(colors::SUBTLE_TEXT));
            ui.add(DragValue::new(&mut a.delay).range(0.0..=60.0).speed(0.05).fixed_decimals(1).suffix(" s"));
        });
        if a != before {
            changed = Some((index, a));
        }
    } else if !rows.is_empty() {
        ui.add_space(6.0);
        note(ui, "Click an animation in the list to change its start, speed or order.");
    }

    if let Some(effect) = add {
        presentation::add_animation(app, effect);
    }
    if let Some(i) = choose {
        app.ribbon.animation_selected = Some(i);
    }
    if let Some((i, a)) = changed {
        presentation::edit_animations(app, "Animation Timing", |list| {
            if let Some(slot) = list.get_mut(i) {
                *slot = a;
            }
        });
    }
    if let Some(by) = shift {
        presentation::move_selected(app, by);
    }
    if let Some(i) = remove {
        presentation::remove(app, i);
    }
}
