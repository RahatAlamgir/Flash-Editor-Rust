//! The hand-drawn parts of the Transitions and Animations tabs: the boxes for a slide transition's direction, duration and
//! advance, the list of effects to add, and the timing of the chosen animation. They change `RibbonState`; the layout then
//! applies the change to the page (`commands::presentation`).

use egui::{ComboBox, DragValue, RichText, Ui};
use fe_core::document::animation::{Effect, EffectClass, Start};
use fe_core::document::{TransitionDirection, TransitionKind};
use fe_ui_kit::theme::colors;

use super::RibbonState;

/// The ribbon button of a transition kind.
pub fn button_label(kind: TransitionKind) -> &'static str {
    match kind {
        TransitionKind::None => "No Transition",
        other => other.name(),
    }
}

/// The kind a ribbon button stands for.
pub fn kind_of(label: &str) -> Option<TransitionKind> {
    TransitionKind::ALL.into_iter().find(|k| button_label(*k) == label)
}

/// True when the button is pressed: the transition the shown page has, or the Animation pane when it is open.
pub fn is_current(state: &RibbonState, label: &str) -> bool {
    button_label(state.transition.kind) == label
}

/// Draws the boxes of a custom group.
pub fn controls(ui: &mut Ui, group: &str, state: &mut RibbonState) {
    ui.vertical(|ui| {
        ui.set_width(if group == "Animation timing" { 250.0 } else { 176.0 });
        ui.spacing_mut().item_spacing.y = 2.0;
        match group {
            "Effect options" | "Advance slide" => transition_boxes(ui, group, state),
            "Add animation" => add_box(ui, state),
            "Animation timing" => timing_boxes(ui, state),
            _ => {}
        }
    });
}

fn transition_boxes(ui: &mut Ui, group: &str, state: &mut RibbonState) {
    let t = &mut state.transition;
    let before = *t;
    if group == "Effect options" {
        ui.add_enabled_ui(t.kind.has_direction(), |ui| {
            ComboBox::from_id_salt("transition-direction").width(150.0).selected_text(t.direction.name()).show_ui(
                ui,
                |ui| {
                    for d in TransitionDirection::ALL {
                        ui.selectable_value(&mut t.direction, d, d.name());
                    }
                },
            );
        });
        ui.add_enabled_ui(t.kind != TransitionKind::None, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Duration").color(colors::SUBTLE_TEXT));
                ui.add(DragValue::new(&mut t.duration).range(0.1..=6.0).speed(0.05).fixed_decimals(1).suffix(" s"));
            });
        });
    } else {
        ui.checkbox(&mut t.on_click, "On click or key");
        ui.horizontal(|ui| {
            let mut automatic = t.after.is_some();
            if ui.checkbox(&mut automatic, "After").changed() {
                t.after = automatic.then_some(5.0);
            }
            if let Some(seconds) = t.after.as_mut() {
                ui.add(DragValue::new(seconds).range(0.5..=3600.0).speed(0.2).fixed_decimals(1).suffix(" s"));
            }
        });
    }
    if *t != before {
        state.transition_changed = true;
    }
}

/// "Add Animation": every effect, under its kind.
fn add_box(ui: &mut Ui, state: &mut RibbonState) {
    ComboBox::from_id_salt("add-animation").width(150.0).selected_text("Add Animation").show_ui(ui, |ui| {
        for class in [EffectClass::Entrance, EffectClass::Emphasis, EffectClass::Exit] {
            ui.label(RichText::new(class.name()).strong().color(colors::SUBTLE_TEXT));
            for effect in Effect::ALL.into_iter().filter(|e| e.class() == class) {
                if ui.selectable_label(false, effect.name()).clicked() {
                    state.add_effect = Some(effect);
                }
            }
        }
    });
    ui.label(RichText::new("Adds it to the selected layer.").size(11.0).color(colors::SUBTLE_TEXT));
}

/// Start, duration, delay and direction of the animation chosen in the pane.
fn timing_boxes(ui: &mut Ui, state: &mut RibbonState) {
    let Some(mut a) = state.animation else {
        ui.label(RichText::new("Choose an animation in the Animation pane.").size(11.0).color(colors::SUBTLE_TEXT));
        return;
    };
    let before = a;
    ui.horizontal(|ui| {
        ComboBox::from_id_salt("animation-start").width(112.0).selected_text(a.start.name()).show_ui(ui, |ui| {
            for s in Start::ALL {
                ui.selectable_value(&mut a.start, s, s.name());
            }
        });
        if a.effect.has_direction() {
            ComboBox::from_id_salt("animation-direction").width(92.0).selected_text(a.direction.name()).show_ui(
                ui,
                |ui| {
                    for d in TransitionDirection::ALL {
                        ui.selectable_value(&mut a.direction, d, d.name());
                    }
                },
            );
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
        state.animation = Some(a);
        state.animation_changed = true;
    }
}
