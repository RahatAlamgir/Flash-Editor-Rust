//! Rectangle Select and Ellipse Select (C# `RectSelectTool`): drag to select. Shift after starting = square /
//! circle; Shift when starting = add, Alt = subtract. A plain click deselects.

use egui::Modifiers;
use fe_core::imaging::{PixelRect, PointD};
use fe_core::selection::Selection;

use super::{Preview, apply};
use crate::app::FlashApp;
use crate::tools::Pointer;

/// A rectangle or ellipse being dragged.
#[derive(Default)]
pub struct Marquee {
    start: Option<(PointD, Modifiers)>,
    end: PointD,
    shift: bool,
}

impl Marquee {
    pub fn cancel(&mut self) {
        self.start = None;
    }

    /// The rectangle from the start to `end` (a square while Shift is held after starting).
    fn rect(&self, start: PointD, start_mods: Modifiers) -> PixelRect {
        let (mut ex, mut ey) = (self.end.x, self.end.y);
        if self.shift && !start_mods.shift {
            let size = (ex - start.x).abs().max((ey - start.y).abs());
            ex = start.x + (ex - start.x).signum() * size;
            ey = start.y + (ey - start.y).signum() * size;
        }
        let r = |v: f64| v.round_ties_even() as i32;
        PixelRect::from_corners(r(start.x), r(start.y), r(ex), r(ey))
    }

    pub fn preview(&self, ellipse: bool, canvas: PixelRect) -> Option<Preview> {
        let (start, mods) = self.start?;
        let rect = self.rect(start, mods);
        Some(if ellipse { Preview::Ellipse(rect) } else { Preview::Rect(rect.intersect(&canvas)) })
    }
}

pub fn pointer(app: &mut FlashApp, p: &Pointer, ellipse: bool) {
    let canvas = app.doc().bounds();
    let m = &mut app.tools.select.marquee;
    if p.pressed {
        m.start = Some((p.pos, p.mods));
    }
    let Some((start, start_mods)) = m.start else { return };
    m.end = p.pos;
    m.shift = p.mods.shift;
    let rect = m.rect(start, start_mods);
    let rect = if ellipse { rect } else { rect.intersect(&canvas) };
    if !p.released {
        app.status = format!("{} x {}", rect.width.max(0), rect.height.max(0));
        return;
    }
    m.start = None;
    if rect.width < 1 || rect.height < 1 {
        if start_mods.is_none() {
            apply::deselect(app); // a plain click deselects
        }
        app.status = if app.doc().selected_area().is_none() { "Selection cleared".into() } else { String::new() };
        return;
    }
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let drawn = if ellipse { Selection::from_ellipse(rect, w, h) } else { Selection::from_rect(rect, w, h) };
    apply::apply(app, drawn, start_mods);
}
