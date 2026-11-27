//! Lays out and draws the canvas area: rulers (when on), the page with its checkerboard and tiles, a thin
//! border, grid and guide lines.

use egui::{Color32, Rect, Sense, Stroke, Ui, pos2, vec2};
use fe_gpu::{ViewTransform, paint_checkerboard};
use fe_ui_kit::theme::colors;

use super::rulers::{self, Ruler};
use super::selection_overlay::{self, OverlayInput};
use super::{aids_overlay, debug_overlay, input};
use crate::app::FlashApp;

/// Draws the canvas into all remaining space of `ui`.
pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let area = ui.available_rect_before_wrap();
    ui.allocate_rect(area, Sense::hover());
    ui.painter().rect_filled(area, 0, colors::CANVAS_AREA);
    let ruler_w = if app.aids.rulers { rulers::THICKNESS } else { 0.0 };
    let viewport = Rect::from_min_max(area.min + vec2(ruler_w, ruler_w), area.max);

    let key = (app.active_project, app.projects[app.active_project].active_index());
    let (doc_w, doc_h) = {
        let doc = &app.projects[app.active_project].active_page().doc;
        (doc.width(), doc.height())
    };
    app.canvas.views.entry(key).or_default().update(viewport.size(), doc_w, doc_h);

    let hover = input::handle(ui, app, viewport, key);

    let view = &app.canvas.views[&key];
    let transform = ViewTransform { zoom: view.zoom, origin: view.origin(viewport.min) };
    app.zoom = view.zoom;
    let painter = ui.painter_at(viewport);
    let page = &app.projects[app.active_project].active_page();
    let page_rect = transform.rect_to_screen(page.doc.bounds());
    let visible_page = page_rect.intersect(viewport);
    if visible_page.is_positive() {
        paint_checkerboard(ui.ctx(), &painter, visible_page, page_rect.min);
    }
    app.canvas.stats = app.canvas.renderer.paint(ui.ctx(), &painter, &page.doc, transform, viewport);
    painter.rect_stroke(page_rect, 0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Outside);

    if app.aids.grid {
        aids_overlay::paint_grid(&painter, transform, doc_w as f64, doc_h as f64, app.aids.grid_size);
    }
    if app.aids.guides {
        aids_overlay::paint_guides(&painter, transform, viewport, &page.vertical_guides, &page.horizontal_guides);
    }
    if let (Some(vertical), Some(at)) = (app.canvas.ruler_drag, app.canvas.temp_guide) {
        let (v, h) = if vertical { (vec![at], vec![]) } else { (vec![], vec![at]) };
        aids_overlay::paint_guides(&painter, transform, viewport, &v, &h);
    }
    if let Some(snap) = app.canvas.snap.as_ref().filter(|s| crate::tools::snap::shows(s)) {
        aids_overlay::paint_snap(&painter, transform, viewport, snap);
    }
    let tool = app.tools.active();
    let overlay = OverlayInput {
        doc: &page.doc,
        tool,
        preview: app.tools.select.preview(tool, page.doc.bounds()),
        pen: if tool == crate::tools::ToolKind::Pen { app.tools.select.pen_anchors() } else { &[] },
        hover,
        brush: app
            .tools
            .paint
            .brush_size(tool)
            .or_else(|| (tool == crate::tools::ToolKind::QuickSelect).then_some(app.tools.select.quick.size))
            .or_else(|| app.tools.retouch.brush_size(tool))
            .or_else(|| app.tools.ai.brush_size(tool)),
        offset: app.canvas.selection_offset,
        band: app.tools.mover.band_rect(),
        moving: app.tools.mover.is_moving(),
        part: crate::commands::parts::active_part(app),
        editing: crate::text_edit::active(app),
        curve: crate::tools::curve::overlay(app),
        crop: crate::tools::crop::overlay(app),
        retouch: crate::tools::retouch::overlay(app),
        reshape: if app.tools.mover.reshaping {
            crate::tools::moving::transform::reshape_points(&page.doc)
        } else {
            Vec::new()
        },
    };
    selection_overlay::paint(ui.ctx(), &painter, &mut app.canvas.overlay, transform, &overlay);
    crate::text_edit::paint::paint(&painter, transform, app, ui.input(|i| i.time));
    if super::distance_guides::wanted(ui, app) {
        super::distance_guides::paint(&painter, transform, app);
    }
    if super::animation_badges::wanted(app) {
        super::animation_badges::paint(&painter, transform, app, viewport);
    }

    if app.aids.rulers {
        let origin = transform.origin;
        let top = Rect::from_min_max(pos2(viewport.left(), area.top()), pos2(viewport.right(), viewport.top()));
        let left = Rect::from_min_max(pos2(area.left(), viewport.top()), pos2(viewport.left(), viewport.bottom()));
        let painter = ui.painter();
        Ruler {
            rect: top,
            vertical: false,
            zoom: transform.zoom as f64,
            offset: (origin.x - viewport.left()) as f64,
            page_length: doc_w as f64,
            marker: hover.map(|p| p.x - viewport.left()),
        }
        .paint(painter);
        Ruler {
            rect: left,
            vertical: true,
            zoom: transform.zoom as f64,
            offset: (origin.y - viewport.top()) as f64,
            page_length: doc_h as f64,
            marker: hover.map(|p| p.y - viewport.top()),
        }
        .paint(painter);
        rulers::paint_corner(painter, Rect::from_min_max(area.min, viewport.min));
        input::ruler_drags(ui, app, top, left, viewport, transform);
    }
    if app.canvas.show_debug {
        debug_overlay::paint(ui, viewport, &app.canvas.stats, transform.zoom);
    }
}
