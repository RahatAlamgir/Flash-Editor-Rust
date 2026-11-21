//! Perspective Crop: drag a box over something photographed at an angle, drag its four corners onto the corners of the
//! thing, press Enter: it is cut out straightened into a flat rectangle.

use fe_core::document::layer_crop;
use fe_core::imaging::PointD;
use fe_core::transform::suggested_size;

use super::insert_piece;
use crate::app::FlashApp;
use crate::tools::Pointer;

#[derive(Default)]
pub struct QuadCrop {
    /// Top-left, top-right, bottom-right, bottom-left (empty = nothing drawn yet).
    pub points: Vec<PointD>,
    dragging: Option<usize>,
    drawing: bool,
    moving: bool,
    start: PointD,
    start_points: Vec<PointD>,
}

impl QuadCrop {
    pub fn clear(&mut self) {
        self.points.clear();
        self.dragging = None;
        self.drawing = false;
        self.moving = false;
    }

    pub fn hint(&self) -> String {
        if self.points.len() == 4 {
            let (w, h) = suggested_size(&self.points);
            format!("The result will be {w} x {h} px. Drag the corners onto the corners of the area, then press Enter.")
        } else {
            "Drag a box over the area you want to straighten.".into()
        }
    }

    fn inside(&self, p: PointD) -> bool {
        let pts = &self.points;
        let mut inside = false;
        let mut j = pts.len() - 1;
        for i in 0..pts.len() {
            if (pts[i].y > p.y) != (pts[j].y > p.y)
                && p.x < (pts[j].x - pts[i].x) * (p.y - pts[i].y) / (pts[j].y - pts[i].y) + pts[i].x
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    let (w, h) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    let q = &mut app.tools.crop.quad;
    if p.pressed {
        let reach = 12.0 / p.zoom.max(0.05);
        q.start = p.pos;
        if let Some(i) = q.points.iter().position(|pt| (pt.x - p.pos.x).abs() < reach && (pt.y - p.pos.y).abs() < reach)
        {
            q.dragging = Some(i);
            return;
        }
        if q.points.len() == 4 && q.inside(p.pos) {
            q.moving = true;
            q.start_points = q.points.clone();
            return;
        }
        q.clear(); // outside: draw a new box
        q.drawing = true;
        return;
    }
    if !p.down {
        if q.drawing
            && q.points.len() == 4
            && (q.points[2].x - q.points[0].x < 4.0 || q.points[2].y - q.points[0].y < 4.0)
        {
            q.clear(); // a plain click draws nothing
        }
        q.dragging = None;
        q.drawing = false;
        q.moving = false;
        return;
    }
    let (cx, cy) = (p.pos.x.clamp(0.0, w), p.pos.y.clamp(0.0, h));
    if q.drawing {
        let (x1, y1) = (q.start.x.clamp(0.0, w), q.start.y.clamp(0.0, h));
        let (l, r, t, b) = (x1.min(cx), x1.max(cx), y1.min(cy), y1.max(cy));
        q.points = vec![PointD::new(l, t), PointD::new(r, t), PointD::new(r, b), PointD::new(l, b)];
    } else if q.moving {
        let (dx, dy) = (p.pos.x - q.start.x, p.pos.y - q.start.y);
        q.points = q.start_points.iter().map(|s| PointD::new(s.x + dx, s.y + dy)).collect();
    } else if let Some(i) = q.dragging.filter(|i| *i < q.points.len()) {
        q.points[i] = PointD::new(cx, cy);
    }
}

pub fn apply(app: &mut FlashApp) {
    let quad = app.tools.crop.quad.points.clone();
    if quad.len() != 4 {
        return;
    }
    let Some(layer) = app.doc().active_layer().filter(|l| l.visible && !l.is_adjustment()) else {
        app.status = "Select a visible layer first (the box stays, so you can pick one and press Enter).".into();
        return;
    };
    let (id, name) = (layer.id(), layer.name.clone());
    let Some(done) = layer_crop::perspective_layer(layer, &quad, app.doc().bounds()) else {
        app.status = "Those four points do not make a four-sided shape. Move the corners and try again.".into();
        return;
    };
    app.tools.crop.quad.clear();
    insert_piece(app, "Perspective Crop", id, done.layer);
    app.status = format!(
        "Straightened to {} x {} px on a new layer. \"{name}\" is hidden, not changed. The canvas keeps its size. Ctrl+Z undoes it.",
        done.size.0, done.size.1
    );
}
