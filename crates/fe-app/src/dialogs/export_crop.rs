//! The crop of File > Export. The picture always shows a crop box with handles: drag the inside to move it, a corner or an
//! edge to resize it, or drag outside it to start a new box. Under the picture are the exact numbers (X, Y, width, height),
//! the shape (free, square, 16:9...) and a button for the whole picture. The pieces that do not need the window are plain
//! functions so they can be tested.

use egui::{Color32, CursorIcon, FontId, Pos2, Rect, Stroke, StrokeKind, vec2};
use fe_core::imaging::PixelRect;
use fe_ui_kit::theme::colors;

use super::export::ExportForm;
use crate::tools::crop::ASPECTS;

/// The smallest crop box, in picture pixels.
const MIN_SIDE: i32 = 8;
/// How close (screen points) the pointer must be to a handle or an edge to take it.
const GRAB: f32 = 9.0;
/// The size of the little square handles.
const HANDLE: f32 = 9.0;

/// Which edges a handle moves: -1 = the left / top edge, 1 = the right / bottom edge, 0 = that direction stays.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Handle {
    dx: i8,
    dy: i8,
}

/// What the pointer is doing to the crop box.
#[derive(Clone, Copy)]
pub enum Drag {
    /// A new box from this point.
    New((f64, f64)),
    /// The box moves; `from` is where the pointer took hold.
    Move { orig: PixelRect, from: (f64, f64) },
    /// One handle is pulled.
    Size { orig: PixelRect, handle: Handle },
}

const HANDLES: [Handle; 8] = [
    Handle { dx: -1, dy: -1 },
    Handle { dx: 1, dy: -1 },
    Handle { dx: -1, dy: 1 },
    Handle { dx: 1, dy: 1 },
    Handle { dx: 0, dy: -1 },
    Handle { dx: 0, dy: 1 },
    Handle { dx: -1, dy: 0 },
    Handle { dx: 1, dy: 0 },
];

/// The width : height the box must keep (0 = any), for the shape at `index` of the list (the last one is "like the picture").
pub fn ratio_of(index: usize, source: (u32, u32)) -> f64 {
    if index < ASPECTS.len() { ASPECTS[index].1 } else { f64::from(source.0) / f64::from(source.1.max(1)) }
}

pub fn aspect_name(index: usize) -> &'static str {
    if index < ASPECTS.len() { ASPECTS[index].0 } else { "Like the picture" }
}

fn handle_point(r: Rect, h: Handle) -> Pos2 {
    Pos2::new(
        r.left() + r.width() * (f32::from(h.dx) + 1.0) / 2.0,
        r.top() + r.height() * (f32::from(h.dy) + 1.0) / 2.0,
    )
}

fn cursor_of(h: Handle) -> CursorIcon {
    match (h.dx, h.dy) {
        (0, _) => CursorIcon::ResizeVertical,
        (_, 0) => CursorIcon::ResizeHorizontal,
        (a, b) if a == b => CursorIcon::ResizeNwSe,
        _ => CursorIcon::ResizeNeSw,
    }
}

/// What lies under the pointer: a handle, the inside of the box, or nothing.
fn hit(pos: Pos2, r: Rect) -> Option<Option<Handle>> {
    for h in HANDLES {
        if (handle_point(r, h) - pos).abs().max_elem() <= GRAB {
            return Some(Some(h));
        }
    }
    let near = |a: f32, b: f32| (a - b).abs() <= GRAB * 0.6;
    let (inside_x, inside_y) =
        (pos.x >= r.left() - 4.0 && pos.x <= r.right() + 4.0, pos.y >= r.top() - 4.0 && pos.y <= r.bottom() + 4.0);
    if inside_y && near(pos.x, r.left()) {
        return Some(Some(Handle { dx: -1, dy: 0 }));
    }
    if inside_y && near(pos.x, r.right()) {
        return Some(Some(Handle { dx: 1, dy: 0 }));
    }
    if inside_x && near(pos.y, r.top()) {
        return Some(Some(Handle { dx: 0, dy: -1 }));
    }
    if inside_x && near(pos.y, r.bottom()) {
        return Some(Some(Handle { dx: 0, dy: 1 }));
    }
    r.contains(pos).then_some(None)
}

/// The box after `handle` was pulled to `p` (picture pixels), kept inside `bounds` and, when `ratio` > 0, in that shape.
pub fn resize(orig: PixelRect, handle: Handle, p: (f64, f64), ratio: f64, bounds: (i32, i32)) -> PixelRect {
    let (bw, bh) = bounds;
    let min = MIN_SIDE.min(bw).min(bh);
    let (mut l, mut t, mut r, mut b) = (orig.x, orig.y, orig.x + orig.width, orig.y + orig.height);
    let (px, py) = (p.0.round() as i32, p.1.round() as i32);
    if handle.dx < 0 {
        l = px.clamp(0, r - min);
    } else if handle.dx > 0 {
        r = px.clamp(l + min, bw);
    }
    if handle.dy < 0 {
        t = py.clamp(0, b - min);
    } else if handle.dy > 0 {
        b = py.clamp(t + min, bh);
    }
    if ratio > 0.0 {
        let (cx, cy) = (orig.x + orig.width / 2, orig.y + orig.height / 2);
        let from_height = |h: i32| ((f64::from(h) * ratio).round() as i32).max(1);
        let from_width = |w: i32| ((f64::from(w) / ratio).round() as i32).max(1);
        match (handle.dx != 0, handle.dy != 0) {
            (true, true) => {
                // The corner opposite the handle stays; the width decides, then the height is cut to fit the picture.
                let mut w = r - l;
                let room_h = if handle.dy < 0 { b } else { bh - t };
                w = w.min(from_height(room_h));
                let h = from_width(w);
                if handle.dx < 0 {
                    l = r - w;
                } else {
                    r = l + w;
                }
                if handle.dy < 0 {
                    t = b - h;
                } else {
                    b = t + h;
                }
            }
            (true, false) => {
                // A side moves: the height follows around the middle.
                let room_h = 2 * cy.min(bh - cy);
                let w = (r - l).min(from_height(room_h)).max(1);
                let h = from_width(w);
                if handle.dx < 0 {
                    l = r - w;
                } else {
                    r = l + w;
                }
                t = cy - h / 2;
                b = t + h;
            }
            (false, true) => {
                let room_w = 2 * cx.min(bw - cx);
                let h = (b - t).min(from_width(room_w)).max(1);
                let w = from_height(h);
                if handle.dy < 0 {
                    t = b - h;
                } else {
                    b = t + h;
                }
                l = cx - w / 2;
                r = l + w;
            }
            (false, false) => {}
        }
    }
    PixelRect::new(l, t, r - l, b - t)
}

/// A box from the point where the drag started to `to`, in the wanted shape.
pub fn new_box(from: (f64, f64), to: (f64, f64), ratio: f64, bounds: (i32, i32)) -> PixelRect {
    let (x1, y1) = (from.0.round() as i32, from.1.round() as i32);
    let (x2, y2) = (to.0.round() as i32, to.1.round() as i32);
    let mut rect = PixelRect::from_corners(x1, y1, x2, y2);
    if ratio > 0.0 && !rect.is_empty() {
        // The corner where the drag started stays; the box is as wide as the pointer reached, as far as the picture allows.
        let right = x2 >= x1;
        let down = y2 >= y1;
        let room_w = if right { bounds.0 - x1 } else { x1 };
        let room_h = if down { bounds.1 - y1 } else { y1 };
        let w = rect.width.min(room_w).min(((f64::from(room_h) * ratio).round() as i32).max(1));
        let h = ((f64::from(w) / ratio).round() as i32).max(1);
        rect = PixelRect::new(if right { x1 } else { x1 - w }, if down { y1 } else { y1 - h }, w, h);
    }
    rect.intersect(&PixelRect::new(0, 0, bounds.0, bounds.1))
}

/// The largest box of the shape `ratio` that fits in `within`, with the same middle.
pub fn fit_shape(within: PixelRect, ratio: f64, bounds: (i32, i32)) -> PixelRect {
    if ratio <= 0.0 {
        return within;
    }
    let (w, h) = (f64::from(within.width), f64::from(within.height));
    let (nw, nh) = if w / h > ratio { (h * ratio, h) } else { (w, w / ratio) };
    let (nw, nh) = ((nw.round() as i32).max(1), (nh.round() as i32).max(1));
    let x = (within.x + (within.width - nw) / 2).clamp(0, (bounds.0 - nw).max(0));
    let y = (within.y + (within.height - nh) / 2).clamp(0, (bounds.1 - nh).max(0));
    PixelRect::new(x, y, nw, nh)
}

fn bounds_of(f: &ExportForm) -> (i32, i32) {
    (f.source.width() as i32, f.source.height() as i32)
}

/// A new crop (`None` or the whole picture = no crop). Sets the output size to the cropped size, like the C# dialog.
fn apply(f: &mut ExportForm, rect: PixelRect) {
    let (bw, bh) = bounds_of(f);
    if rect.x <= 0 && rect.y <= 0 && rect.width >= bw && rect.height >= bh {
        f.crop = None;
        f.width = bw as u32;
        f.height = bh as u32;
    } else {
        f.crop = Some(rect);
        f.width = rect.width as u32;
        f.height = rect.height as u32;
    }
}

/// The crop box over the picture shown in `shown` (one picture pixel = `fit` screen points). True when the crop changed.
pub(super) fn handle_box(
    ui: &mut egui::Ui,
    f: &mut ExportForm,
    response: &egui::Response,
    shown: Rect,
    fit: f32,
) -> bool {
    let (bw, bh) = bounds_of(f);
    let bounds = (bw, bh);
    let whole = PixelRect::new(0, 0, bw, bh);
    let to_screen = |x: i32, y: i32| Pos2::new(shown.left() + x as f32 * fit, shown.top() + y as f32 * fit);
    let current = f.crop.unwrap_or(whole);
    let box_rect = Rect::from_min_max(
        to_screen(current.x, current.y),
        to_screen(current.x + current.width, current.y + current.height),
    );
    let to_image = |p: Pos2| {
        (
            (f64::from(p.x - shown.left()) / f64::from(fit)).clamp(0.0, f64::from(bw)),
            (f64::from(p.y - shown.top()) / f64::from(fit)).clamp(0.0, f64::from(bh)),
        )
    };
    let ratio = ratio_of(f.aspect, (f.source.width(), f.source.height()));
    let mut changed = false;

    // What a drag would do here (also shown by the mouse cursor).
    if let Some(pos) = response.hover_pos().filter(|_| f.drag.is_none()) {
        ui.ctx().set_cursor_icon(match hit(pos, box_rect) {
            Some(Some(h)) => cursor_of(h),
            Some(None) => CursorIcon::Move,
            None => CursorIcon::Crosshair,
        });
    }
    if response.drag_started()
        && let Some(pos) = ui.input(|i| i.pointer.press_origin()).or_else(|| response.interact_pointer_pos())
    {
        // The drag starts a few points after the press: what was under the pointer at the press decides.
        f.drag = Some(match hit(pos, box_rect) {
            Some(Some(handle)) => Drag::Size { orig: current, handle },
            Some(None) => Drag::Move { orig: current, from: to_image(pos) },
            None => Drag::New(to_image(pos)),
        });
    }
    if response.dragged()
        && let (Some(drag), Some(pos)) = (f.drag, response.interact_pointer_pos())
    {
        let p = to_image(pos);
        let rect = match drag {
            Drag::New(from) => new_box(from, p, ratio, bounds),
            Drag::Move { orig, from } => {
                let x = (f64::from(orig.x) + p.0 - from.0).round() as i32;
                let y = (f64::from(orig.y) + p.1 - from.1).round() as i32;
                PixelRect::new(x.clamp(0, bw - orig.width), y.clamp(0, bh - orig.height), orig.width, orig.height)
            }
            Drag::Size { orig, handle } => resize(orig, handle, p, ratio, bounds),
        };
        if rect.width >= 2 && rect.height >= 2 && Some(rect) != f.crop {
            apply(f, rect);
            changed = true;
        }
        if let Drag::Size { handle, .. } = drag {
            ui.ctx().set_cursor_icon(cursor_of(handle));
        } else if matches!(drag, Drag::Move { .. }) {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }
    }
    if response.drag_stopped() {
        f.drag = None;
    }

    // ---- drawing
    let current = f.crop.unwrap_or(whole);
    let box_rect = Rect::from_min_max(
        to_screen(current.x, current.y),
        to_screen(current.x + current.width, current.y + current.height),
    );
    let p = ui.painter();
    let shade = Color32::from_black_alpha(140);
    for outside in [
        Rect::from_min_max(shown.min, Pos2::new(shown.max.x, box_rect.top())),
        Rect::from_min_max(Pos2::new(shown.min.x, box_rect.bottom()), shown.max),
        Rect::from_min_max(Pos2::new(shown.min.x, box_rect.top()), Pos2::new(box_rect.left(), box_rect.bottom())),
        Rect::from_min_max(Pos2::new(box_rect.right(), box_rect.top()), Pos2::new(shown.max.x, box_rect.bottom())),
    ] {
        p.rect_filled(outside, 0.0, shade);
    }
    // Thirds, to help placing things.
    let thin = Stroke::new(1.0, Color32::from_white_alpha(70));
    for i in 1..3 {
        let t = i as f32 / 3.0;
        p.line_segment(
            [
                Pos2::new(box_rect.left() + box_rect.width() * t, box_rect.top()),
                Pos2::new(box_rect.left() + box_rect.width() * t, box_rect.bottom()),
            ],
            thin,
        );
        p.line_segment(
            [
                Pos2::new(box_rect.left(), box_rect.top() + box_rect.height() * t),
                Pos2::new(box_rect.right(), box_rect.top() + box_rect.height() * t),
            ],
            thin,
        );
    }
    p.rect_stroke(box_rect, 0.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Outside);
    for h in HANDLES {
        let c = handle_point(box_rect, h);
        let square = Rect::from_center_size(c, vec2(HANDLE, HANDLE));
        p.rect(square, 1.0, Color32::WHITE, Stroke::new(1.0, Color32::from_gray(40)), StrokeKind::Inside);
    }
    // The size of the box, on a small label inside its top-left corner (or below it when it is tiny).
    let label = format!("{} x {}", current.width, current.height);
    let font = FontId::proportional(11.5);
    let galley = p.layout_no_wrap(label, font, Color32::WHITE);
    let size = galley.size() + vec2(10.0, 4.0);
    let at = if box_rect.width() > size.x + 24.0 && box_rect.height() > size.y + 24.0 {
        box_rect.left_top() + vec2(8.0, 8.0)
    } else {
        Pos2::new(box_rect.left(), (box_rect.bottom() + 6.0).min(shown.bottom() - size.y))
    };
    let pill = Rect::from_min_size(at, size);
    p.rect_filled(pill, 4.0, Color32::from_black_alpha(190));
    p.galley(pill.min + vec2(5.0, 2.0), galley, Color32::WHITE);
    changed
}

/// The numbers and the shape under the picture. True when the crop changed.
pub(super) fn controls(ui: &mut egui::Ui, f: &mut ExportForm) -> bool {
    let (bw, bh) = bounds_of(f);
    let bounds = (bw, bh);
    let whole = PixelRect::new(0, 0, bw, bh);
    let mut changed = false;
    let mut c = f.crop.unwrap_or(whole);
    let ratio = ratio_of(f.aspect, (f.source.width(), f.source.height()));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let field = |ui: &mut egui::Ui, label: &str, value: &mut i32, range: std::ops::RangeInclusive<i32>| {
            ui.label(small(label));
            let mut v = f64::from(*value);
            let response = ui.add(
                egui::DragValue::new(&mut v)
                    .range(f64::from(*range.start())..=f64::from(*range.end()))
                    .speed(1.0)
                    .max_decimals(0),
            );
            *value = v.round() as i32;
            response.changed()
        };
        let (mut x, mut y, mut w, mut h) = (c.x, c.y, c.width, c.height);
        let mut edited = false;
        edited |= field(ui, "X", &mut x, 0..=(bw - MIN_SIDE.min(bw)));
        ui.add_space(4.0);
        edited |= field(ui, "Y", &mut y, 0..=(bh - MIN_SIDE.min(bh)));
        ui.add_space(4.0);
        let width_changed = field(ui, "W", &mut w, MIN_SIDE.min(bw)..=bw);
        ui.add_space(4.0);
        let height_changed = field(ui, "H", &mut h, MIN_SIDE.min(bh)..=bh);
        if width_changed && ratio > 0.0 {
            h = ((f64::from(w) / ratio).round() as i32).max(1);
        } else if height_changed && ratio > 0.0 {
            w = ((f64::from(h) * ratio).round() as i32).max(1);
        }
        if edited || width_changed || height_changed {
            // Keep the box inside the picture: it is moved first, shrunk only when it is bigger than the picture.
            let (w2, h2) = (w.min(bw), h.min(bh));
            let rect = PixelRect::new(x.clamp(0, bw - w2), y.clamp(0, bh - h2), w2, h2);
            apply(f, rect);
            changed = true;
        }
    });
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.label(small("Shape"));
        let before = f.aspect;
        egui::ComboBox::from_id_salt("export-aspect").width(150.0).selected_text(aspect_name(f.aspect)).show_ui(
            ui,
            |ui| {
                for i in 0..=ASPECTS.len() {
                    ui.selectable_value(&mut f.aspect, i, aspect_name(i));
                }
            },
        );
        if f.aspect != before {
            // The current box becomes the biggest box of the new shape that fits inside it.
            c = f.crop.unwrap_or(whole);
            let r = ratio_of(f.aspect, (f.source.width(), f.source.height()));
            if r > 0.0 {
                apply(f, fit_shape(c, r, bounds));
                changed = true;
            }
        }
        if ui
            .add_enabled(f.crop.is_some(), egui::Button::new("Whole picture"))
            .on_hover_text("Remove the crop")
            .clicked()
        {
            f.aspect = 0;
            apply(f, whole);
            changed = true;
        }
    });
    changed
}

/// Small grey label text.
fn small(text: &str) -> egui::RichText {
    egui::RichText::new(text).size(12.0).color(colors::SUBTLE_TEXT)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: (i32, i32) = (1000, 600);

    #[test]
    fn a_corner_keeps_the_far_corner_and_stays_in_the_picture() {
        let orig = PixelRect::new(200, 100, 300, 200);
        let r = resize(orig, Handle { dx: 1, dy: 1 }, (2000.0, 2000.0), 0.0, BOUNDS);
        assert_eq!((r.x, r.y, r.x + r.width, r.y + r.height), (200, 100, 1000, 600));
        let r = resize(orig, Handle { dx: -1, dy: -1 }, (-50.0, -50.0), 0.0, BOUNDS);
        assert_eq!((r.x, r.y, r.x + r.width, r.y + r.height), (0, 0, 500, 300));
    }

    #[test]
    fn an_edge_cannot_pass_the_other_edge() {
        let orig = PixelRect::new(200, 100, 300, 200);
        let r = resize(orig, Handle { dx: -1, dy: 0 }, (900.0, 0.0), 0.0, BOUNDS);
        assert!(r.width >= MIN_SIDE && r.x + r.width == 500, "{r:?}");
    }

    #[test]
    fn a_shape_is_kept_while_resizing_and_inside_the_picture() {
        let orig = PixelRect::new(100, 100, 320, 180);
        for handle in HANDLES {
            for p in [(5.0, 5.0), (990.0, 590.0), (500.0, 300.0), (150.0, 450.0)] {
                let r = resize(orig, handle, p, 16.0 / 9.0, BOUNDS);
                assert!(
                    r.x >= 0 && r.y >= 0 && r.x + r.width <= BOUNDS.0 && r.y + r.height <= BOUNDS.1,
                    "{handle:?} {p:?} {r:?}"
                );
                let ratio = f64::from(r.width) / f64::from(r.height);
                // A box of a few pixels cannot be in exactly the right shape.
                assert!(r.width < 40 || (ratio - 16.0 / 9.0).abs() < 0.08, "{handle:?} {p:?} {r:?} {ratio}");
            }
        }
    }

    #[test]
    fn a_new_box_follows_the_pointer_in_every_direction() {
        let r = new_box((400.0, 300.0), (500.0, 360.0), 0.0, BOUNDS);
        assert_eq!((r.x, r.y, r.width, r.height), (400, 300, 100, 60));
        let r = new_box((400.0, 300.0), (350.0, 100.0), 0.0, BOUNDS);
        assert_eq!((r.x, r.y, r.width, r.height), (350, 100, 50, 200));
        let r = new_box((900.0, 500.0), (1000.0, 600.0), 1.0, BOUNDS);
        assert_eq!((r.width, r.height), (100, 100));
        let r = new_box((100.0, 100.0), (900.0, 150.0), 1.0, BOUNDS);
        assert!(r.width == r.height && r.y + r.height <= BOUNDS.1 && r.x + r.width <= BOUNDS.0, "{r:?}");
    }

    #[test]
    fn a_shape_fits_inside_the_box_around_the_same_middle() {
        let r = fit_shape(PixelRect::new(0, 0, 1000, 600), 1.0, BOUNDS);
        assert_eq!((r.x, r.y, r.width, r.height), (200, 0, 600, 600));
        let r = fit_shape(PixelRect::new(100, 100, 200, 200), 2.0, BOUNDS);
        assert_eq!((r.width, r.height), (200, 100));
        assert_eq!(fit_shape(PixelRect::new(5, 5, 50, 50), 0.0, BOUNDS), PixelRect::new(5, 5, 50, 50));
    }
}
