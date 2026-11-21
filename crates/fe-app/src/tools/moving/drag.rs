//! Moving pixels with the Select / Move tool (C# `PixelMover` and `MultiMover`): the selected pixels of a layer,
//! or whole layers. While dragging only the page is changed; the end of the drag becomes one undo step.

use std::sync::Arc;

use fe_core::document::{Document, LayerId};
use fe_core::imaging::TileMap;
use fe_core::objects::StoredObject;
use fe_core::paint::pixel_move::FloatingPiece;
use fe_core::selection::Selection;

use crate::app::FlashApp;

/// A selected piece kept between moves, so moving it off the canvas and back brings it back whole.
pub struct Remembered {
    piece: FloatingPiece,
    /// Which project and page it belongs to.
    page: (u64, usize),
    /// Total offset from where the piece started.
    dx: i32,
    dy: i32,
    /// The layer's pixels and the selection right after the last move: if either changed since, the piece is
    /// lifted again.
    pixels: TileMap,
    selection: Option<Arc<Selection>>,
}

/// What a drag moves.
enum What {
    /// The selected pixels of one layer (the piece is in `MoveTool::remembered`).
    Piece { base_dx: i32, base_dy: i32 },
    /// Whole layers: each one's pixels and object when the drag began.
    Layers(Vec<(LayerId, TileMap, Option<StoredObject>)>),
}

/// A move in progress.
pub struct Drag {
    before: Document,
    what: What,
    dx: i32,
    dy: i32,
    /// Whole-layer moves snap (selected pixels do not, like C#): the targets and the area being moved.
    snap: Option<(fe_core::layout::snapping::SnapSession, fe_core::imaging::PixelRect)>,
}

impl Drag {
    /// The move after snapping the moved area to other objects, the canvas, the grid and the guides.
    pub fn snapped(&self, app: &mut FlashApp, dx: i32, dy: i32, zoom: f64, alt: bool) -> (i32, i32) {
        let Some((session, area)) = &self.snap else { return (dx, dy) };
        let (x, y) = crate::tools::snap::snap_move(app, Some(session), *area, dx as f64, dy as f64, zoom, alt);
        (x.round() as i32, y.round() as i32)
    }
}

fn page_key(app: &FlashApp) -> (u64, usize) {
    (app.project().uid(), app.project().active_index())
}

/// Starts moving the selected pixels of `layer` (or the whole layer when nothing is selected).
pub fn begin_pixels(app: &mut FlashApp, layer: LayerId) -> Option<Drag> {
    let before = app.doc().clone();
    let Some(selection) = before.selected_area_arc().cloned() else {
        return begin_layers(app, &[layer]);
    };
    let page = page_key(app);
    let l = before.layer(layer)?;
    let remembered = app.tools.mover.remembered.as_ref();
    let can_continue = remembered.is_some_and(|r| {
        r.page == page
            && r.piece.layer == layer
            && l.pixels().shares_all_tiles_with(&r.pixels)
            && r.selection.as_ref().is_some_and(|s| Arc::ptr_eq(s, &selection))
    });
    if !can_continue {
        app.tools.mover.remembered = Some(Remembered {
            piece: FloatingPiece::lift(l, &selection),
            page,
            dx: 0,
            dy: 0,
            pixels: TileMap::empty(),
            selection: None,
        });
    }
    let r = app.tools.mover.remembered.as_ref()?;
    Some(Drag { before, what: What::Piece { base_dx: r.dx, base_dy: r.dy }, dx: 0, dy: 0, snap: None })
}

/// Starts moving whole layers (locked and hidden ones stay where they are).
pub fn begin_layers(app: &mut FlashApp, ids: &[LayerId]) -> Option<Drag> {
    let before = app.doc().clone();
    let layers: Vec<_> = ids
        .iter()
        .filter_map(|id| before.layer(*id))
        .filter(|l| !l.locked && l.visible && !l.is_adjustment())
        .map(|l| (l.id(), l.pixels().clone(), l.object().cloned()))
        .collect();
    if layers.is_empty() {
        return None;
    }
    let moving: Vec<LayerId> = layers.iter().map(|l| l.0).collect();
    let area = moving
        .iter()
        .filter_map(|id| before.layer(*id))
        .fold(fe_core::imaging::PixelRect::EMPTY, |a, l| a.union(&fe_core::layout::snapping::layer_area(l)));
    let snap = crate::tools::snap::session(app, &moving).map(|s| (s, area));
    Some(Drag { before, what: What::Layers(layers), dx: 0, dy: 0, snap })
}

/// Shows the pixels moved by (dx, dy) from where the drag began.
pub fn move_to(app: &mut FlashApp, drag: &mut Drag, dx: i32, dy: i32) {
    if (dx, dy) == (drag.dx, drag.dy) {
        return;
    }
    (drag.dx, drag.dy) = (dx, dy);
    match &drag.what {
        What::Piece { base_dx, base_dy } => {
            let Some(r) = app.tools.mover.remembered.as_ref() else { return };
            let pixels = r.piece.pixels_at(base_dx + dx, base_dy + dy);
            let layer = r.piece.layer;
            if let Some(l) = app.project_mut().active_page_mut().doc.layer_mut(layer) {
                l.replace_pixels(pixels);
            }
            app.canvas.selection_offset = (dx, dy);
        }
        What::Layers(layers) => {
            let doc = &mut app.projects[app.active_project].active_page_mut().doc;
            for (id, pixels, object) in layers {
                if let Some(l) = doc.layer_mut(*id) {
                    let moved = pixels.offset(dx, dy);
                    match object {
                        Some(o) => l.set_pixels_with_object(moved, Some(o.offset(dx as f64, dy as f64))),
                        None => l.replace_pixels(moved),
                    }
                }
            }
        }
    }
    app.status = format!("Move: {dx:+}, {dy:+} px");
}

/// Finishes a move: one undo step (nothing when it did not move).
pub fn end(app: &mut FlashApp, drag: Drag) {
    app.canvas.selection_offset = (0, 0);
    crate::tools::snap::end(app);
    if (drag.dx, drag.dy) == (0, 0) {
        app.project_mut().active_page_mut().doc = drag.before; // undo any preview
        return;
    }
    let name = match &drag.what {
        What::Layers(layers) if layers.len() > 1 => "Move Layers",
        _ => "Move",
    };
    if let What::Piece { base_dx, base_dy } = drag.what {
        let total = (base_dx + drag.dx, base_dy + drag.dy);
        let moved = app.tools.mover.remembered.as_ref().and_then(|r| r.piece.selection_at(total.0, total.1));
        app.project_mut().active_page_mut().doc.set_selected_area(moved);
        let selection = app.doc().selected_area_arc().cloned();
        let layer = app.tools.mover.remembered.as_ref().map(|r| r.piece.layer);
        let pixels = layer.and_then(|id| app.doc().layer(id)).map(|l| l.pixels().clone()).unwrap_or_default();
        if let Some(r) = app.tools.mover.remembered.as_mut() {
            (r.dx, r.dy) = total;
            r.pixels = pixels;
            r.selection = selection;
        }
    }
    app.project_mut().record(name, drag.before);
    let canvas = app.doc().bounds();
    let hidden = app.doc().active_layer().is_some_and(|l| l.has_hidden_pixels(canvas));
    app.status = if hidden {
        "Moved. Parts outside the canvas are kept (Image > Reveal All shows them).".into()
    } else {
        "Moved.".into()
    };
}
