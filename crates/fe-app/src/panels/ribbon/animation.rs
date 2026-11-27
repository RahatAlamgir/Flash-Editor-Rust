//! The hand-drawn parts of the Animation tab: the boxes for the direction, the duration and when the slideshow moves on.
//! They change `RibbonState::transition`; the layout then applies it to the page.

use egui::{ComboBox, DragValue, RichText, Ui};
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

/// True when the button is the transition the shown page has (drawn as pressed).
pub fn is_current(state: &RibbonState, label: &str) -> bool {
    button_label(state.transition.kind) == label
}

/// Draws the boxes of a group ("Effect options" or "Advance slide").
pub fn controls(ui: &mut Ui, group: &str, state: &mut RibbonState) {
    ui.vertical(|ui| {
        ui.set_width(176.0);
        ui.spacing_mut().item_spacing.y = 2.0;
        let t = &mut state.transition;
        let before = *t;
        match group {
            "Effect options" => {
                ui.add_enabled_ui(t.kind.has_direction(), |ui| {
                    ComboBox::from_id_salt("transition-direction")
                        .width(150.0)
                        .selected_text(t.direction.name())
                        .show_ui(ui, |ui| {
                            for d in TransitionDirection::ALL {
                                ui.selectable_value(&mut t.direction, d, d.name());
                            }
                        });
                });
                ui.add_enabled_ui(t.kind != TransitionKind::None, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Duration").color(colors::SUBTLE_TEXT));
                        ui.add(
                            DragValue::new(&mut t.duration).range(0.1..=6.0).speed(0.05).fixed_decimals(1).suffix(" s"),
                        );
                    });
                });
            }
            "Advance slide" => {
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
            _ => {}
        }
        if *t != before {
            state.transition_changed = true;
        }
    });
}
