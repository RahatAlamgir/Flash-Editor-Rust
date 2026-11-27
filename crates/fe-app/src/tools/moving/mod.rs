//! The Select / Move tool (C# `MoveTool`, the pixel and layer parts): click anything to select its layer and
//! drag to move it; Shift+click adds or removes a layer; Ctrl+drag a box selects every layer it touches; dragging
//! inside a pixel selection moves the selected pixels; arrow keys nudge (Shift = 10 px). Resizing, rotating and
//! editing objects follow with the objects (Phase 8).

mod drag;
pub mod hit;
pub mod pan;
pub mod transform;

use egui::{Key, Modifiers};
use fe_core::document::LayerId;
use fe_core::imaging::{PixelRect, PointD};
use fe_core::objects::handles::Handle;

use super::Pointer;
use crate::app::FlashApp;
use crate::commands::checks;
use drag::{Drag, Remembered};

/// The Select / Move tool's state.
#[derive(Default)]
pub struct MoveTool {
    drag: Option<Drag>,
    /// Where the mouse went down.
    start: PointD,
    /// The selected piece kept between moves.
    remembered: Option<Remembered>,
    /// A Ctrl+drag rubber band: (start, Shift held = add to the selected layers).
    band: Option<(PointD, bool)>,
    band_end: PointD,
    /// A resize, turn or reshape of the selected layer.
    transform: Option<transform::TransformDrag>,
    /// Corner handles keep the width / height ratio of shapes too (Properties switch).
    pub keep_proportions: bool,
    /// Reshape is on: the corner points of the selected object are the handles.
    pub reshaping: bool,
    /// The part of a group being edited: (the group's layer, the part's index).
    pub part: Option<(LayerId, usize)>,
    /// A filled frame whose picture is being moved (double-click it, then drag; the wheel zooms).
    pub picture_layer: Option<LayerId>,
    pan: Option<pan::PanDrag>,
    /// Mouse wheel notches waiting to zoom the picture of a frame.
    pub wheel: f32,
    /// The mouse has moved far enough since it went down for this to count as a drag.
    armed: bool,
}

/// Screen pixels the mouse must travel before a press turns into a drag.
const DEAD_ZONE: f64 = 4.0;

impl MoveTool {
    pub fn is_moving(&self) -> bool {
        self.drag.is_some() || self.transform.is_some() || self.pan.is_some()
    }

    /// The rubber band rectangle being dragged (canvas pixels), if any.
    pub fn band_rect(&self) -> Option<PixelRect> {
        let (start, _) = self.band?;
        Some(PixelRect::from_bounds(
            start.x.min(self.band_end.x),
            start.y.min(self.band_end.y),
            start.x.max(self.band_end.x),
            start.y.max(self.band_end.y),
        ))
    }
}

/// The bottom painted layer when it covers the whole canvas (a plain page or a full-size photo).
fn is_page_background(doc: &fe_core::document::Document, id: LayerId) -> bool {
    doc.index_of(id) == Some(0)
        && doc.layer(id).is_some_and(|l| {
            l.object().is_none() && {
                let b = l.visible_bounds();
                b.width >= doc.width() as i32 && b.height >= doc.height() as i32
            }
        })
}

/// Nothing selected: no layer, no part of a group, no picture being moved in a frame, no picked pages.
pub fn clear_selection(app: &mut FlashApp) {
    crate::commands::parts::select_part(app, None);
    app.tools.mover.picture_layer = None;
    app.pages_ui.selected.clear();
    app.project_mut().active_page_mut().doc.set_active(None);
    app.status = "Nothing selected. Click a shape, text, image or painted area to select it.".into();
}

fn part_none(app: &FlashApp) -> bool {
    crate::commands::parts::active_part(app).is_none()
}

fn select_only(app: &mut FlashApp, id: LayerId) {
    app.project_mut().active_page_mut().doc.set_active(Some(id));
}

/// Shift+click: adds the layer to the selected layers, or takes it out (the last one stays).
fn toggle_selected(app: &mut FlashApp, id: LayerId) {
    let doc = &mut app.project_mut().active_page_mut().doc;
    let mut ids = doc.selected_ids();
    let active = if let Some(i) = ids.iter().position(|s| *s == id) {
        if ids.len() == 1 {
            return;
        }
        ids.remove(i);
        doc.active_id().filter(|a| *a != id).or(ids.last().copied())
    } else {
        ids.push(id);
        Some(id)
    };
    doc.set_selection(&ids, active);
}

fn press(app: &mut FlashApp, p: &Pointer) {
    app.tools.mover.start = p.pos;
    // Ctrl + drag: a rubber band that selects every layer it touches.
    if p.mods.command {
        app.tools.mover.band = Some((p.pos, p.mods.shift));
        app.tools.mover.band_end = p.pos;
        return;
    }
    let (x, y) = p.pixel();
    let in_selection = app.doc().selected_area().is_some_and(|s| s.is_selected(x, y));
    // 1) Inside a pixel selection: move the selected pixels.
    if !p.mods.shift && in_selection && app.doc().active_layer().is_some() {
        if let Some(layer) = checks::editable_layer(app) {
            app.tools.mover.drag = drag::begin_pixels(app, layer);
        }
        return;
    }
    // 2) A Reshape corner, or a handle of the selected layer's box.
    if start_handle_drag(app, p) {
        return;
    }
    // 3) Whatever is under the mouse: select it and start moving it.
    let margin = 3.0 / p.zoom;
    // The page itself (a painted layer at the bottom that covers the whole canvas) counts as empty canvas.
    let hit = hit::layer_at(app.doc(), p.pos, margin, false).filter(|id| !is_page_background(app.doc(), *id));
    let Some(hit) = hit else {
        let locked = hit::layer_at(app.doc(), p.pos, margin, true).and_then(|id| app.doc().layer(id));
        app.status = match locked {
            Some(l) => format!(
                "\"{}\" is locked, so it cannot be picked on the canvas. Select it in the Layers panel to unlock it.",
                l.name
            ),
            None => "Nothing here. Click a shape, text, image or painted area to select it.".into(),
        };
        // A click on empty canvas lets go of everything (Shift keeps the selection).
        if !p.mods.shift {
            clear_selection(app);
        }
        return;
    };
    if p.mods.shift {
        toggle_selected(app, hit);
        return;
    }
    // A filled frame whose picture is being moved: dragging moves the picture, not the frame.
    if app.tools.mover.picture_layer == Some(hit) && app.doc().active_id() == Some(hit) {
        app.tools.mover.pan = pan::begin(app, hit, p.pos);
        if app.tools.mover.pan.is_some() {
            return;
        }
    }
    app.tools.mover.picture_layer = None;
    // Dragging one of several selected layers moves them all.
    let selected = app.doc().selected_ids();
    if selected.len() > 1 && selected.contains(&hit) {
        let doc = &mut app.project_mut().active_page_mut().doc;
        doc.set_selection(&selected, Some(hit));
        app.tools.mover.drag = drag::begin_layers(app, &selected);
        return;
    }
    // Inside a group: a click picks the part under the mouse; dragging it moves just that part.
    if crate::commands::parts::active_part(app).is_some()
        && app.doc().active_id() == Some(hit)
        && let Some(group) = app.doc().layer(hit).and_then(|l| l.object()).filter(|o| o.kind() == "group")
        && let Some(index) = hit::part_at(group, p.pos, 3.0 / p.zoom)
    {
        crate::commands::parts::select_part(app, Some((hit, index)));
        app.tools.mover.transform = transform::begin(app, hit, Handle::Move, None, p.pos);
        return;
    }
    crate::commands::parts::select_part(app, None);
    select_only(app, hit);
    let Some(layer) = app.doc().layer(hit) else { return };
    if layer.locked {
        app.status = format!(
            "\"{}\" is locked, so it cannot be moved. Right-click > Unlock (or Ctrl+L) to change it.",
            layer.name
        );
        return;
    }
    // An object moves as a whole and stays editable; plain pixels move with the selection, if there is one.
    app.tools.mover.drag =
        if layer.object().is_some() { drag::begin_layers(app, &[hit]) } else { drag::begin_pixels(app, hit) };
}

/// Starts dragging a Reshape corner or a resize / rotate handle of the single selected layer. True when it did.
fn start_handle_drag(app: &mut FlashApp, p: &Pointer) -> bool {
    let doc = app.doc();
    let Some(active) = doc.active_layer().filter(|l| !l.locked) else { return false };
    let id = active.id();
    if doc.selected_ids().len() > 1 || p.mods.shift {
        return false;
    }
    let part = crate::commands::parts::active_part(app);
    if app.tools.mover.reshaping && part.is_none() {
        let near = transform::reshape_points(doc)
            .iter()
            .position(|c| c.distance_to(&p.pos) <= (fe_core::objects::handles::HANDLE_SIZE / 2.0 + 4.0) / p.zoom);
        if let Some(corner) = near {
            app.tools.mover.transform = transform::begin(app, id, Handle::Move, Some(corner), p.pos);
            return true;
        }
        return false;
    }
    let Some(frame) = transform::frame_of_part(doc, id, part) else { return false };
    let target = crate::commands::parts::target_object(app);
    match fe_core::objects::handles::hit_test(&frame, target.as_ref(), p.pos, p.zoom) {
        Some(h) if h != Handle::Move => {
            app.tools.mover.transform = transform::begin(app, id, h, None, p.pos);
            true
        }
        _ => false,
    }
}

/// Ends a Ctrl+drag rubber band: selects the layers it touches.
fn end_band(app: &mut FlashApp, zoom: f64) {
    let Some(rect) = app.tools.mover.band_rect() else { return };
    let adds = app.tools.mover.band.is_some_and(|(_, adds)| adds);
    app.tools.mover.band = None;
    if (rect.width as f64) < 3.0 / zoom && (rect.height as f64) < 3.0 / zoom {
        return; // a click without dragging selects nothing
    }
    let mut found = hit::layers_touching(app.doc(), rect);
    if found.is_empty() {
        app.status = "No object in that area. Locked and hidden layers are skipped; the background layer is never \
                      picked this way."
            .into();
        return;
    }
    if adds {
        found.extend(app.doc().selected_ids());
    }
    // Keep the layers in stack order (bottom to top); the top-most one becomes the active layer.
    let ordered: Vec<LayerId> = app.doc().layers().iter().map(|l| l.id()).filter(|id| found.contains(id)).collect();
    let active = ordered.last().copied();
    app.project_mut().active_page_mut().doc.set_selection(&ordered, active);
    app.status = if ordered.len() == 1 {
        let name = app.doc().active_layer().map(|l| l.name.clone()).unwrap_or_default();
        format!("Selected \"{name}\".")
    } else {
        format!("{} selected. Drag to move them together, Ctrl+D duplicates them.", ordered.len())
    };
}

/// A double-click: a group opens (the part under the mouse is edited), a curve is edited with the Curve tool.
fn double_click(app: &mut FlashApp, p: &Pointer) {
    let Some(hit) = hit::layer_at(app.doc(), p.pos, 3.0 / p.zoom, false) else { return };
    let Some(object) = app.doc().layer(hit).and_then(|l| l.object()).cloned() else { return };
    if object.kind() == "group" {
        if let Some(index) = hit::part_at(&object, p.pos, 3.0 / p.zoom) {
            app.project_mut().active_page_mut().doc.set_active(Some(hit));
            crate::commands::parts::select_part(app, Some((hit, index)));
            let name = object.part(index).map(|m| m.name).unwrap_or_default();
            app.status = format!("Editing \"{name}\" inside the group. Esc goes back to the whole group.");
            if object.part(index).is_some_and(|m| m.object.kind() == "text") {
                crate::text_edit::begin(app, hit, Some(index), crate::text_edit::Start::At(p.pos, true));
            }
        }
    } else if object.picture_fill().is_some() {
        app.project_mut().active_page_mut().doc.set_active(Some(hit));
        app.tools.mover.picture_layer = Some(hit);
        app.status =
            "Drag to move the picture inside the frame, mouse wheel to zoom it. Esc or a click elsewhere when done."
                .into();
    } else if object.kind() == "text" {
        crate::text_edit::begin(app, hit, None, crate::text_edit::Start::At(p.pos, true));
    } else if object.shape().is_some_and(|s| s.kind == fe_core::objects::ShapeKind::Path) {
        app.tools.activate(crate::tools::ToolKind::Curve);
    }
}

/// Handles the mouse for the Select / Move tool.
pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    pointer_inner(app, p);
    if p.double_click && !app.tools.mover.is_moving() && !p.mods.command && !p.mods.shift {
        double_click(app, p);
    }
}

fn pointer_inner(app: &mut FlashApp, p: &Pointer) {
    let wheel = std::mem::take(&mut app.tools.mover.wheel);
    if wheel != 0.0 && app.tools.mover.picture_layer.is_some() {
        pan::wheel(app, wheel);
    }
    if p.pressed {
        app.tools.mover.armed = false;
        press(app, p);
    }
    // A click must never nudge anything: nothing moves until the mouse has travelled a few screen pixels.
    if !app.tools.mover.armed && app.tools.mover.is_moving() {
        let start = app.tools.mover.start;
        let travelled = (p.pos.x - start.x).hypot(p.pos.y - start.y) * p.zoom;
        if travelled >= DEAD_ZONE {
            app.tools.mover.armed = true;
        } else if p.released || !p.down {
            // Let go without dragging: nothing changed, nothing to undo.
            app.tools.mover.transform = None;
            app.tools.mover.pan = None;
            if let Some(drag) = app.tools.mover.drag.take() {
                drag::end(app, drag);
            }
            return;
        } else {
            return;
        }
    }
    if let Some(d) = app.tools.mover.transform.take() {
        app.tools.mover.transform = transform::pointer(app, d, p);
        return;
    }
    if let Some(d) = app.tools.mover.pan.take() {
        app.tools.mover.pan = pan::follow(app, d, p);
        return;
    }
    if app.tools.mover.band.is_some() {
        app.tools.mover.band_end = p.pos;
        if p.released || !p.down {
            end_band(app, p.zoom);
        }
        return;
    }
    let Some(mut drag) = app.tools.mover.drag.take() else { return };
    let start = app.tools.mover.start;
    let (mut dx, mut dy) = ((p.pos.x - start.x).round() as i32, (p.pos.y - start.y).round() as i32);
    let (lock_x, lock_y) = match p.mods.shift {
        true if dx.abs() > dy.abs() => (false, true),
        true => (true, false),
        false => (false, false),
    };
    let (sx, sy) = drag.snapped(app, dx, dy, p.zoom, p.mods.alt);
    (dx, dy) = (if lock_x { 0 } else { sx }, if lock_y { 0 } else { sy });
    drag::move_to(app, &mut drag, dx, dy);
    if p.released || !p.down {
        drag::end(app, drag);
    } else {
        app.tools.mover.drag = Some(drag);
    }
}

/// Arrow keys nudge the selected pixels or layers by 1 px (Shift = 10 px). True when the key was used.
pub fn key(app: &mut FlashApp, key: Key, mods: Modifiers) -> bool {
    if key == Key::Enter
        && let Some(id) = app.doc().active_id()
        && crate::commands::parts::target_object(app).is_some_and(|o| o.kind() == "text")
    {
        let part = crate::commands::parts::active_part(app);
        crate::text_edit::begin(app, id, part, crate::text_edit::Start::End);
        return true;
    }
    if key == Key::Escape && app.tools.mover.picture_layer.take().is_some() {
        return true;
    }
    if key == Key::Escape && crate::commands::parts::active_part(app).is_some() {
        crate::commands::parts::select_part(app, None);
        app.status = "Back to the whole group.".into();
        return true;
    }
    if key == Key::Escape && app.tools.mover.reshaping {
        app.tools.mover.reshaping = false;
        app.status = "Reshape finished.".into();
        return true;
    }
    let step = if mods.shift { 10 } else { 1 };
    let (dx, dy) = match key {
        Key::ArrowLeft => (-step, 0),
        Key::ArrowRight => (step, 0),
        Key::ArrowUp => (0, -step),
        Key::ArrowDown => (0, step),
        _ => return false,
    };
    if app.tools.mover.is_moving() {
        return true;
    }
    let selected = app.doc().selected_ids();
    let drag = if selected.len() > 1 && app.doc().selected_area().is_none() {
        drag::begin_layers(app, &selected)
    } else {
        match checks::editable_layer(app) {
            Some(layer)
                if app.doc().layer(layer).is_some_and(|l| l.object().is_some())
                    && app.doc().selected_area().is_none() =>
            {
                drag::begin_layers(app, &[layer])
            }
            Some(layer) => drag::begin_pixels(app, layer),
            None => None,
        }
    };
    if let Some(mut drag) = drag {
        drag::move_to(app, &mut drag, dx, dy);
        drag::end(app, drag);
    }
    true
}

/// The mouse cursor over the selected layer's handles (C# `TransformHandles.CursorFor`).
pub fn hover_cursor(app: &FlashApp, pos: PointD, zoom: f64) -> Option<egui::CursorIcon> {
    let doc = app.doc();
    let layer = doc.active_layer().filter(|l| !l.locked && doc.selected_ids().len() == 1)?;
    if app.tools.mover.reshaping && part_none(app) {
        let near = transform::reshape_points(doc).iter().any(|c| c.distance_to(&pos) <= 9.0 / zoom);
        return near.then_some(egui::CursorIcon::Crosshair);
    }
    let part = crate::commands::parts::active_part(app);
    let frame = transform::frame_of_part(doc, layer.id(), part)?;
    let target = crate::commands::parts::target_object(app);
    let h = fe_core::objects::handles::hit_test(&frame, target.as_ref(), pos, zoom)?;
    Some(match h {
        Handle::Move => egui::CursorIcon::Move,
        Handle::Rotate => egui::CursorIcon::Grab,
        _ => match fe_core::objects::handles::cursor_octant(h, frame.rotation) {
            0 => egui::CursorIcon::ResizeHorizontal,
            1 => egui::CursorIcon::ResizeNwSe,
            2 => egui::CursorIcon::ResizeVertical,
            _ => egui::CursorIcon::ResizeNeSw,
        },
    })
}
