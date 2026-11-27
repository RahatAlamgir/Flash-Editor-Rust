//! Mouse input on the canvas: wheel zoom, panning (Space or the middle button),
//! moving guides with the Select / Move tool, and pulling new guides out of the rulers.

use egui::{CursorIcon, Event, Id, Key, MouseWheelUnit, PointerButton, Pos2, Rect, Sense, Ui};
use fe_gpu::ViewTransform;

use super::GuideRef;
use crate::app::FlashApp;
use crate::tools::{Pointer, ToolKind};

/// Screen distance at which a guide can be grabbed (C# `GuideGrabDistance`).
const GUIDE_GRAB: f32 = 4.0;

/// Handles input over `viewport`; returns the pointer position when it is over the canvas.
pub fn handle(ui: &mut Ui, app: &mut FlashApp, viewport: Rect, key: (usize, usize)) -> Option<Pos2> {
    let response = ui.interact(viewport, Id::unique("canvas-viewport"), Sense::click_and_drag());
    if response.is_pointer_button_down_on() {
        response.request_focus(); // so Space pans instead of pressing the last clicked button
        app.pages_ui.focused = false; // Ctrl+C etc. act on the picture again, not on pages
    }
    if response.has_focus() {
        // Arrow keys nudge (Move tool) instead of moving the focus to another button.
        let filter = egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() };
        ui.memory_mut(|m| m.set_focus_lock_filter(response.id, filter));
    }
    let hover = response.hover_pos();
    let tool = app.tools.active();
    let space = ui.input(|i| i.key_down(Key::Space));
    let view = app.canvas.views.entry(key).or_default();
    let transform = ViewTransform { zoom: view.zoom, origin: view.origin(viewport.min) };

    // Status bar: the pixel under the mouse.
    app.cursor_text = match hover {
        Some(p) => {
            let c = transform.to_canvas(p);
            format!("{}, {} px", c.x.floor(), c.y.floor())
        }
        None => String::new(),
    };

    // Mouse wheel: zoom around the pointer, 1.2x per notch.
    if let Some(p) = hover {
        let lines: f32 = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    Event::MouseWheel { unit, delta, .. } => Some(match unit {
                        MouseWheelUnit::Line => delta.y,
                        MouseWheelUnit::Point => delta.y / 50.0,
                        MouseWheelUnit::Page => delta.y * 3.0,
                    }),
                    _ => None,
                })
                .sum()
        });
        if lines != 0.0 && tool == ToolKind::Move && app.tools.mover.picture_layer.is_some() {
            app.tools.mover.wheel += lines; // the Select / Move tool zooms the picture of the frame
        } else if lines != 0.0 {
            view.zoom_at(p - viewport.min, view.zoom * 1.2f32.powf(lines));
        }
    }

    // Panning.
    let pan_with_primary = space;
    if response.dragged_by(PointerButton::Middle) || (pan_with_primary && response.dragged_by(PointerButton::Primary)) {
        view.pan_by(response.drag_delta());
        ui.set_cursor_icon(CursorIcon::Grabbing);
    } else if pan_with_primary && response.hovered() {
        ui.set_cursor_icon(CursorIcon::Grab);
    }

    if tool == ToolKind::Move && app.aids.guides && !space {
        move_guides(ui, app, &response, viewport, transform);
    }

    // Everything else goes to the active tool (not while panning, zooming or moving a guide).
    let panning = space || response.dragged_by(PointerButton::Middle);
    let busy = app.canvas.guide_drag.is_some() || app.canvas.ruler_drag.is_some();
    let gesture =
        app.tools.mover.is_moving() || app.tools.paint.is_painting() || ui.input(|i| i.pointer.primary_down());
    if !panning
        && !busy
        && (response.hovered() || gesture)
        && let Some(pointer) = Pointer::read(ui, &response, transform)
    {
        crate::tools::pointer(app, &pointer);
    }
    if tool == ToolKind::Move
        && response.hovered()
        && !panning
        && !app.tools.mover.is_moving()
        && let Some(p) = response.hover_pos()
    {
        let c = transform.to_canvas(p);
        let pos = fe_core::imaging::PointD::new(c.x as f64, c.y as f64);
        if let Some(icon) = crate::tools::moving::hover_cursor(app, pos, transform.zoom as f64) {
            ui.set_cursor_icon(icon);
        }
    }
    if crate::tools::uses_crosshair(tool) && response.hovered() && !panning {
        ui.set_cursor_icon(CursorIcon::Crosshair);
    }
    super::context_menu::show(&response, app);
    hover
}

/// Dragging an existing guide with the Select / Move tool; dropping it outside the canvas removes it.
fn move_guides(ui: &Ui, app: &mut FlashApp, response: &egui::Response, viewport: Rect, t: ViewTransform) {
    let page = app.projects[app.active_project].active_page_mut();
    let near = |p: Pos2| -> Option<GuideRef> {
        let v = page.vertical_guides.iter().position(|x| (t.to_screen(*x as f32, 0.0).x - p.x).abs() <= GUIDE_GRAB);
        let h = page.horizontal_guides.iter().position(|y| (t.to_screen(0.0, *y as f32).y - p.y).abs() <= GUIDE_GRAB);
        v.map(|index| GuideRef { vertical: true, index }).or(h.map(|index| GuideRef { vertical: false, index }))
    };
    if app.canvas.guide_drag.is_none()
        && let Some(guide) = response.hover_pos().and_then(near)
    {
        ui.set_cursor_icon(if guide.vertical { CursorIcon::ResizeHorizontal } else { CursorIcon::ResizeVertical });
        if response.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
            app.canvas.guide_drag = Some(guide); // taken on press, so the Move tool does not start a move too
        }
    }
    let Some(guide) = app.canvas.guide_drag else { return };
    let pointer = ui.input(|i| i.pointer.interact_pos());
    if let Some(p) = pointer {
        let c = t.to_canvas(p);
        let list = if guide.vertical { &mut page.vertical_guides } else { &mut page.horizontal_guides };
        if let Some(value) = list.get_mut(guide.index) {
            *value = (if guide.vertical { c.x } else { c.y } as f64).round();
        }
        ui.set_cursor_icon(if guide.vertical { CursorIcon::ResizeHorizontal } else { CursorIcon::ResizeVertical });
    }
    if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
        if pointer.is_some_and(|p| !viewport.contains(p)) {
            let list = if guide.vertical { &mut page.vertical_guides } else { &mut page.horizontal_guides };
            if guide.index < list.len() {
                list.remove(guide.index);
            }
        }
        app.canvas.guide_drag = None;
    }
}

/// Pulling a new guide out of a ruler: the top ruler makes horizontal guides, the left ruler vertical ones.
pub fn ruler_drags(ui: &mut Ui, app: &mut FlashApp, top: Rect, left: Rect, viewport: Rect, t: ViewTransform) {
    let top_response = ui.interact(top, Id::unique("ruler-top"), Sense::drag());
    let left_response = ui.interact(left, Id::unique("ruler-left"), Sense::drag());
    for (response, vertical) in [(&top_response, false), (&left_response, true)] {
        if response.drag_started() {
            app.canvas.ruler_drag = Some(vertical);
        }
        if app.canvas.ruler_drag != Some(vertical) {
            continue;
        }
        let pointer = ui.input(|i| i.pointer.interact_pos());
        if let Some(p) = pointer {
            let c = t.to_canvas(p);
            app.canvas.temp_guide = Some((if vertical { c.x } else { c.y } as f64).round());
        }
        if response.drag_stopped() {
            if let (Some(at), Some(p)) = (app.canvas.temp_guide, pointer)
                && viewport.contains(p)
            {
                let page = app.projects[app.active_project].active_page_mut();
                if vertical { &mut page.vertical_guides } else { &mut page.horizontal_guides }.push(at);
                app.aids.guides = true; // a new guide is always shown
            }
            app.canvas.ruler_drag = None;
            app.canvas.temp_guide = None;
        }
    }
}
