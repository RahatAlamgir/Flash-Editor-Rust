//! The egui style: dark visuals with the Flash Editor colors, text sizes and spacing.

use egui::{CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

use super::colors as c;
use super::metrics;

pub(super) fn apply(ctx: &egui::Context) {
    ctx.set_visuals(visuals());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(15.0, FontFamily::Name(super::fonts::SEMIBOLD.into()))),
            (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(9.0, 2.0);
        style.spacing.interact_size.y = 24.0;
        style.spacing.menu_margin = egui::Margin::same(4);
        style.interaction.tooltip_delay = 0.4;
        style.visuals = visuals();
    });
}

fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    let radius = CornerRadius::same(metrics::RADIUS);
    v.override_text_color = None;
    v.panel_fill = c::PANEL;
    v.window_fill = c::PANEL;
    v.window_stroke = Stroke::new(1.0, c::DIVIDER);
    v.window_corner_radius = CornerRadius::same(6);
    v.menu_corner_radius = CornerRadius::same(6);
    v.extreme_bg_color = c::INPUT;
    v.faint_bg_color = c::PANEL_HEADER;
    v.code_bg_color = c::INPUT;
    v.hyperlink_color = c::ACCENT;
    v.selection.bg_fill = c::ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, c::ACCENT);
    v.handle_shape = egui::style::HandleShape::Circle;
    v.slider_trailing_fill = true;

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = c::PANEL;
    w.noninteractive.weak_bg_fill = c::PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0, c::DIVIDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, c::TEXT);

    w.inactive.bg_fill = c::PANEL_HEADER;
    w.inactive.weak_bg_fill = c::PANEL_HEADER;
    w.inactive.bg_stroke = Stroke::new(1.0, c::DIVIDER);
    w.inactive.fg_stroke = Stroke::new(1.0, c::TEXT);

    w.hovered.bg_fill = c::HOVER;
    w.hovered.weak_bg_fill = c::HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0, c::SUBTLE_TEXT);
    w.hovered.fg_stroke = Stroke::new(1.0, c::TEXT);

    w.active.bg_fill = c::ACCENT_SOFT;
    w.active.weak_bg_fill = c::ACCENT_SOFT;
    w.active.bg_stroke = Stroke::new(1.0, c::ACCENT);
    w.active.fg_stroke = Stroke::new(1.0, c::TEXT);

    w.open = w.active;
    for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        state.corner_radius = radius;
        state.expansion = 0.0;
    }
    v
}
