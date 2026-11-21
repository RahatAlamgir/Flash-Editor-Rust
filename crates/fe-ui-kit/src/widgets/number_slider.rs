//! A slider with a number box and a unit next to it (C# `NumberSlider`), e.g. Opacity [----o] [100] %.
//! The value is kept to whole numbers, like the C# control.

use egui::{DragValue, Response, Ui};

use super::slider::slider;
use crate::theme::colors;

/// Draws the slider and its number box; returns the combined response (`.changed()` when the value moved).
pub fn number_slider(
    ui: &mut Ui,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
    unit: &str,
    slider_width: f32,
    box_width: f32,
) -> Response {
    ui.horizontal(|ui| {
        let slide = slider(ui, value, range.clone(), slider_width);
        if slide.changed() {
            *value = value.round();
        }
        ui.add_space(4.0);
        let number = ui.add_sized([box_width, 24.0], DragValue::new(value).range(range).speed(1.0).max_decimals(0));
        if !unit.is_empty() {
            ui.label(egui::RichText::new(unit).color(colors::SUBTLE_TEXT));
        }
        slide | number
    })
    .inner
}
