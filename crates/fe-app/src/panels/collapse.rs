//! Folding panels away: a handle in the middle of the panel's edge (a small pill with a chevron) that folds and unfolds the right
//! column and the page strip, and `amount`, which makes the folding glide instead of jump.

use egui::{Context, Id, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};
use fe_ui_kit::theme::colors;

/// The thickness of a folded panel.
pub const STRIP: f32 = 26.0;
/// The time a fold or unfold takes.
const SECONDS: f32 = 0.24;

/// How folded a panel is right now: 0 = fully open, 1 = fully folded, moving smoothly when `folded` changes.
pub fn amount(ctx: &Context, name: &str, folded: bool) -> f32 {
    ctx.animate_bool_with_time_and_easing(
        Id::NULL.with(("fold-amount", name)),
        folded,
        SECONDS,
        egui::emath::easing::cubic_in_out,
    )
}

/// Goes from `open` to `folded` as `amount` goes from 0 to 1.
pub fn between(open: f32, folded: f32, amount: f32) -> f32 {
    open + (folded - open) * amount
}

#[derive(Clone, Copy)]
enum Way {
    Left,
    Right,
    Up,
    Down,
}

/// A chevron pointing the given way, drawn with lines so it does not depend on a font.
fn chevron(painter: &Painter, center: Pos2, way: Way, color: egui::Color32) {
    let (a, b) = (4.0, 2.8);
    let points = match way {
        Way::Left => [pos2(center.x + b, center.y - a), pos2(center.x - b, center.y), pos2(center.x + b, center.y + a)],
        Way::Right => {
            [pos2(center.x - b, center.y - a), pos2(center.x + b, center.y), pos2(center.x - b, center.y + a)]
        }
        Way::Up => [pos2(center.x - a, center.y + b), pos2(center.x, center.y - b), pos2(center.x + a, center.y + b)],
        Way::Down => [pos2(center.x - a, center.y - b), pos2(center.x, center.y + b), pos2(center.x + a, center.y - b)],
    };
    painter.add(Shape::line(points.to_vec(), Stroke::new(1.8, color)));
}

fn pill(ui: &mut Ui, rect: Rect, id: &str, way: Way, tip: &str) -> bool {
    let response = ui.interact(rect, Id::NULL.with(("fold-handle", id)), Sense::click());
    let (fill, ink) =
        if response.hovered() { (colors::ACCENT, egui::Color32::WHITE) } else { (colors::PANEL_HEADER, colors::TEXT) };
    ui.painter().rect(rect, 5, fill, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
    chevron(ui.painter(), rect.center(), way, ink);
    response.on_hover_text(tip).clicked()
}

/// The handle of the right column, in the middle of its left edge. `folded`: the column is (nearly) folded, so it points the
/// other way. True when clicked.
pub fn side_handle(ui: &mut Ui, panel: Rect, folded: bool) -> bool {
    let rect = Rect::from_center_size(pos2(panel.left() + 8.0, panel.center().y), vec2(14.0, 48.0));
    let (way, tip) = if folded {
        (Way::Left, "Show Properties and Layers")
    } else {
        (Way::Right, "Fold Properties and Layers away")
    };
    pill(ui, rect, "side", way, tip)
}

/// The handle of the page strip, in the middle of its top edge. True when clicked.
pub fn bottom_handle(ui: &mut Ui, panel: Rect, folded: bool) -> bool {
    let rect = Rect::from_center_size(pos2(panel.center().x, panel.top() + 8.0), vec2(48.0, 14.0));
    let (way, tip) = if folded {
        (Way::Up, "Show the pages")
    } else {
        (Way::Down, "Fold the pages away (they stay in the project)")
    };
    pill(ui, rect, "bottom", way, tip)
}

/// The label on the folded page bar (clicking anywhere on the bar also unfolds it). True when clicked.
pub fn folded_bar(ui: &mut Ui, panel: Rect, label: &str) -> bool {
    let response = ui.interact(panel, Id::NULL.with("unfold-bar"), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(panel, 0, colors::HOVER);
    }
    ui.painter().text(
        pos2(panel.left() + 12.0, panel.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        colors::SUBTLE_TEXT,
    );
    response.clicked()
}
