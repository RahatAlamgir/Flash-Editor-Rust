//! The Animations tab of the Layers panel (presentations): what plays on the slide, in order. Like the layers, the animations belong
//! to the page, not to one object. Add an animation to the selected layer, drag rows up or down to change the order, click one to
//! change its start, speed or direction. While this tab is open the canvas numbers the animated objects.

use egui::{Align2, Color32, ComboBox, DragValue, RichText, Sense, Stroke, Ui, pos2, vec2};
use fe_core::document::TransitionDirection;
use fe_core::document::animation::{Animation, Effect, EffectClass, Start, timeline};
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::commands::presentation;

const ROW: f32 = 36.0;
const SETTINGS_H: f32 = 96.0;

fn dot(class: EffectClass) -> Color32 {
    match class {
        EffectClass::Entrance => Color32::from_rgb(0x34, 0xC7, 0x59),
        EffectClass::Emphasis => Color32::from_rgb(0xFF, 0xCC, 0x00),
        EffectClass::Exit => Color32::from_rgb(0xFF, 0x3B, 0x30),
    }
}

fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(11.0).color(colors::SUBTLE_TEXT)).wrap());
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

    egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
        // Add.
        let mut add = None;
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("layers-add-animation").width(150.0).selected_text("Add Animation").show_ui(ui, |ui| {
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
        ui.add_space(4.0);

        // The list: drag a row to move it.
        let list_h = (ui.available_height() - if selected.is_some() { SETTINGS_H } else { 24.0 }).max(ROW);
        let (mut choose, mut remove) = (None, None);
        let mut spans: Vec<(usize, f32, f32)> = Vec::new(); // (raw index, top, bottom)
        let mut drag_started = None;
        egui::ScrollArea::vertical().id_salt("animation-list").max_height(list_h).auto_shrink([false, true]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            if rows.is_empty() {
                note(ui, "No animations on this slide yet. Select a layer, then choose Add Animation. A click in the slideshow (F5) plays the next step.");
            }
            for (position, (i, name, a)) in rows.iter().enumerate() {
                let step = steps.items.get(*i).map_or(0, |t| t.step);
                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click_and_drag());
                spans.push((*i, rect.top(), rect.bottom()));
                if selected == Some(*i) {
                    ui.painter().rect_filled(rect, 4, colors::ACCENT_SOFT);
                } else if response.hovered() {
                    ui.painter().rect_filled(rect, 4, colors::HOVER);
                }
                let painter = ui.painter();
                painter.text(
                    pos2(rect.left() + 12.0, rect.center().y),
                    Align2::CENTER_CENTER,
                    (position + 1).to_string(),
                    egui::FontId::proportional(12.0),
                    colors::SUBTLE_TEXT,
                );
                painter.circle_filled(pos2(rect.left() + 28.0, rect.center().y), 5.0, dot(a.effect.class()));
                painter.text(
                    pos2(rect.left() + 40.0, rect.top() + 11.0),
                    Align2::LEFT_CENTER,
                    name.chars().take(28).collect::<String>(),
                    egui::FontId::proportional(12.5),
                    colors::TEXT,
                );
                let when = if step == 0 { "by itself".to_owned() } else { format!("click {step}") };
                painter.text(
                    pos2(rect.left() + 40.0, rect.top() + 26.0),
                    Align2::LEFT_CENTER,
                    format!("{} \u{00B7} {} \u{00B7} {when}", a.effect.name(), a.start.name()),
                    egui::FontId::proportional(11.0),
                    colors::SUBTLE_TEXT,
                );
                let cross = egui::Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(18.0, 18.0));
                let cross_response = ui.interact(cross, egui::Id::NULL.with(("animation-remove", *i)), Sense::click());
                ui.painter().text(cross.center(), Align2::CENTER_CENTER, "\u{2715}", egui::FontId::proportional(11.0), colors::SUBTLE_TEXT);
                if cross_response.on_hover_text("Remove this animation").clicked() {
                    remove = Some(*i);
                } else if response.drag_started_by(egui::PointerButton::Primary) {
                    drag_started = Some(*i);
                } else if response.clicked() {
                    choose = Some(*i);
                }
            }
        });
        if let Some(from) = drag_started {
            app.layers_ui.anim_drag = Some((from, 0));
            choose = Some(from);
        }
        let mut moved = None;
        if let Some((from, _)) = app.layers_ui.anim_drag {
            let pointer = ui.input(|i| i.pointer.interact_pos());
            // The gap under the pointer: before the first row whose middle is below it.
            let gap = pointer.map_or(0, |p| spans.iter().take_while(|(_, top, bottom)| (top + bottom) / 2.0 < p.y).count());
            if let Some(&(_, top, _)) = spans.get(gap) {
                let x = ui.max_rect().x_range();
                ui.painter().hline(x, top - 0.5, Stroke::new(2.0, colors::ACCENT));
            } else if let Some(&(_, _, bottom)) = spans.last() {
                ui.painter().hline(ui.max_rect().x_range(), bottom + 0.5, Stroke::new(2.0, colors::ACCENT));
            }
            app.layers_ui.anim_drag = Some((from, gap));
            if !ui.input(|i| i.pointer.primary_down()) {
                app.layers_ui.anim_drag = None;
                // The gap counts listed rows; the move is in the whole list.
                let to = spans.get(gap).map_or(total, |(raw, ..)| *raw);
                moved = Some((from, to));
            }
        }

        // The settings of the chosen one.
        let mut changed = None;
        if let Some(index) = selected
            && let Some((_, name, mut a)) = rows.iter().find(|(i, ..)| *i == index).cloned()
        {
            let before = a;
            ui.separator();
            ui.label(RichText::new(format!("{} on {}", a.effect.name(), name.chars().take(22).collect::<String>())).strong().color(colors::TEXT));
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("layers-anim-start").width(112.0).selected_text(a.start.name()).show_ui(ui, |ui| {
                    for s in Start::ALL {
                        ui.selectable_value(&mut a.start, s, s.name());
                    }
                });
                if a.effect.has_direction() {
                    ComboBox::from_id_salt("layers-anim-direction").width(92.0).selected_text(a.direction.name()).show_ui(ui, |ui| {
                        for d in TransitionDirection::ALL {
                            ui.selectable_value(&mut a.direction, d, d.name());
                        }
                    });
                }
            });
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
            note(ui, "Click an animation to change it. Drag one to move it.");
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
        if let Some((from, to)) = moved {
            presentation::move_to(app, from, to);
        }
        if let Some(i) = remove {
            presentation::remove(app, i);
        }
    });
}
