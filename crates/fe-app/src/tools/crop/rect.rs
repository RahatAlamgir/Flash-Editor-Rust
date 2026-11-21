//! The Crop tool's box: drag a new one, drag inside to move it, drag a corner or edge to resize it.

use fe_core::imaging::{PixelRect, PointD};

use super::{ASPECTS, apply_rect};
use crate::app::FlashApp;
use crate::tools::Pointer;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Handle {
    NW,
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
}

impl Handle {
    /// Which sides move: -1 left / top, +1 right / bottom, 0 none.
    fn direction(self) -> (i32, i32) {
        match self {
            Handle::NW => (-1, -1),
            Handle::N => (0, -1),
            Handle::NE => (1, -1),
            Handle::E => (1, 0),
            Handle::SE => (1, 1),
            Handle::S => (0, 1),
            Handle::SW => (-1, 1),
            Handle::W => (-1, 0),
        }
    }
}

#[derive(Clone, Copy)]
enum Drag {
    New,
    Move,
    Resize(Handle),
}

#[derive(Default)]
pub struct RectCrop {
    pub rect: Option<PixelRect>,
    drag: Option<Drag>,
    start: PointD,
    start_rect: PixelRect,
}

impl RectCrop {
    pub fn clear(&mut self) {
        self.rect = None;
        self.drag = None;
    }
}

fn contains(r: PixelRect, p: PointD) -> bool {
    p.x >= f64::from(r.x)
        && p.x <= f64::from(r.x + r.width)
        && p.y >= f64::from(r.y)
        && p.y <= f64::from(r.y + r.height)
}

fn hit_handle(r: PixelRect, p: PointD, tol: f64) -> Option<Handle> {
    let (l, t, rr, b) = (f64::from(r.x), f64::from(r.y), f64::from(r.x + r.width), f64::from(r.y + r.height));
    let (cx, cy) = ((l + rr) / 2.0, (t + b) / 2.0);
    let spots = [
        (Handle::NW, l, t),
        (Handle::NE, rr, t),
        (Handle::SE, rr, b),
        (Handle::SW, l, b),
        (Handle::N, cx, t),
        (Handle::S, cx, b),
        (Handle::W, l, cy),
        (Handle::E, rr, cy),
    ];
    if let Some((h, _, _)) = spots.iter().find(|(_, x, y)| (p.x - x).abs() <= tol && (p.y - y).abs() <= tol) {
        return Some(*h);
    }
    // The edges grab too, in a thin strip.
    let near_x = p.y >= t - tol && p.y <= b + tol;
    let near_y = p.x >= l - tol && p.x <= rr + tol;
    if near_x && (p.x - l).abs() <= tol / 2.0 {
        return Some(Handle::W);
    }
    if near_x && (p.x - rr).abs() <= tol / 2.0 {
        return Some(Handle::E);
    }
    if near_y && (p.y - t).abs() <= tol / 2.0 {
        return Some(Handle::N);
    }
    if near_y && (p.y - b).abs() <= tol / 2.0 {
        return Some(Handle::S);
    }
    None
}

/// The largest box of the aspect that fits inside `rect`, centered in it (`0` = free).
pub fn fit_aspect(rect: PixelRect, ratio: f64) -> PixelRect {
    if ratio <= 0.0 || rect.is_empty() {
        return rect;
    }
    let (mut w, mut h) = (rect.width, rect.height);
    if f64::from(w) / f64::from(h) > ratio {
        w = ((f64::from(h) * ratio).round() as i32).max(1);
    } else {
        h = ((f64::from(w) / ratio).round() as i32).max(1);
    }
    PixelRect::new(rect.x + (rect.width - w) / 2, rect.y + (rect.height - h) / 2, w, h)
}

/// A new box shrunk to the aspect, keeping the corner where the drag started.
fn constrain_new(rect: PixelRect, ratio: f64, anchor: (i32, i32)) -> PixelRect {
    if ratio <= 0.0 || rect.is_empty() {
        return rect;
    }
    let (mut w, mut h) = (rect.width, rect.height);
    if f64::from(w) / f64::from(h) > ratio {
        w = ((f64::from(h) * ratio).round() as i32).max(1);
    } else {
        h = ((f64::from(w) / ratio).round() as i32).max(1);
    }
    let x = if rect.x == anchor.0 { rect.x } else { rect.x + rect.width - w };
    let y = if rect.y == anchor.1 { rect.y } else { rect.y + rect.height - h };
    PixelRect::new(x, y, w, h)
}

fn resize(app: &FlashApp, c: &RectCrop, handle: Handle, p: PointD, ratio: f64) -> Option<PixelRect> {
    let (sx, sy) = handle.direction();
    let bounds = app.doc().bounds();
    let s = c.start_rect;
    let (mut left, mut top, mut right, mut bottom) = (s.x, s.y, s.x + s.width, s.y + s.height);
    let px = p.x.clamp(f64::from(bounds.x), f64::from(bounds.x + bounds.width)).round() as i32;
    let py = p.y.clamp(f64::from(bounds.y), f64::from(bounds.y + bounds.height)).round() as i32;
    if sx < 0 {
        left = px.min(right - 1);
    } else if sx > 0 {
        right = px.max(left + 1);
    }
    if sy < 0 {
        top = py.min(bottom - 1);
    } else if sy > 0 {
        bottom = py.max(top + 1);
    }
    if ratio > 0.0 {
        let (w, h) = (right - left, bottom - top);
        if sx != 0 && sy != 0 {
            // A corner: the width decides, the opposite corner stays.
            let h = ((f64::from(w) / ratio).round() as i32).max(1);
            if sy < 0 {
                top = bottom - h;
            } else {
                bottom = top + h;
            }
        } else if sx != 0 {
            let h = ((f64::from(w) / ratio).round() as i32).max(1);
            let mid = (s.y + s.y + s.height) / 2;
            top = mid - h / 2;
            bottom = top + h;
        } else {
            let w = ((f64::from(h) * ratio).round() as i32).max(1);
            let mid = (s.x + s.x + s.width) / 2;
            left = mid - w / 2;
            right = left + w;
        }
        // Would leave the picture: keep the last good box.
        if left < bounds.x || top < bounds.y || right > bounds.x + bounds.width || bottom > bounds.y + bounds.height {
            return None;
        }
    }
    Some(PixelRect::new(left, top, right - left, bottom - top))
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    let ratio = ASPECTS[app.tools.crop.aspect].1;
    let bounds = app.doc().bounds();
    if p.pressed {
        let c = &mut app.tools.crop.rect;
        if p.double_click && c.rect.is_some_and(|r| contains(r, p.pos)) {
            apply_rect(app);
            return;
        }
        c.start = p.pos;
        c.start_rect = c.rect.unwrap_or(PixelRect::EMPTY);
        let tol = 9.0 / p.zoom.max(0.01);
        c.drag = Some(match c.rect {
            None => Drag::New,
            Some(r) => match hit_handle(r, p.pos, tol) {
                Some(h) => Drag::Resize(h),
                None if contains(r, p.pos) => Drag::Move,
                None => Drag::New,
            },
        });
        return;
    }
    if !p.down {
        app.tools.crop.rect.drag = None;
        return;
    }
    let Some(drag) = app.tools.crop.rect.drag else { return };
    let (start, start_rect) = (app.tools.crop.rect.start, app.tools.crop.rect.start_rect);
    let new_rect = match drag {
        Drag::Move => {
            let (dx, dy) = ((p.pos.x - start.x).round() as i32, (p.pos.y - start.y).round() as i32);
            let x = (start_rect.x + dx).clamp(bounds.x, bounds.x.max(bounds.x + bounds.width - start_rect.width));
            let y = (start_rect.y + dy).clamp(bounds.y, bounds.y.max(bounds.y + bounds.height - start_rect.height));
            Some(PixelRect::new(x, y, start_rect.width, start_rect.height))
        }
        Drag::New => {
            let (sx, sy) = (start.x.round() as i32, start.y.round() as i32);
            let r = PixelRect::from_corners(sx, sy, p.pos.x.round() as i32, p.pos.y.round() as i32).intersect(&bounds);
            (!r.is_empty()).then(|| constrain_new(r, ratio, (sx, sy)))
        }
        Drag::Resize(h) => resize(app, &app.tools.crop.rect, h, p.pos, ratio),
    };
    if let Some(r) = new_rect {
        app.tools.crop.rect.rect = Some(r);
    }
}
