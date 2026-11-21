//! Zoom and scroll position of one page's canvas, with the same rules as the C# `CanvasView`:
//! fit-to-window (40 px margin, at most 800 %) that follows the window size until the user zooms or pans,
//! zoom steps, mouse-wheel zoom around the pointer, and a resized window that scales the chosen zoom.

use egui::{Pos2, Vec2, vec2};

pub const MIN_ZOOM: f32 = 0.02;
pub const MAX_ZOOM: f32 = 64.0;
const FIT_MARGIN: f32 = 40.0;
const FIT_MAX: f32 = 8.0;
/// The zoom levels of Zoom In / Zoom Out (C# `ZoomSteps`).
pub const ZOOM_STEPS: [f32; 21] = [
    0.02, 0.05, 0.1, 0.167, 0.25, 0.333, 0.5, 0.667, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0,
    64.0,
];

/// The view of one page.
#[derive(Clone, Debug)]
pub struct ViewState {
    pub zoom: f32,
    /// Where canvas pixel (0, 0) is, relative to the top-left of the viewport (screen points).
    pub offset: Vec2,
    /// "Fit to window": the picture follows the window size until the user zooms or pans.
    pub fitted: bool,
    fit_pending: bool,
    /// The zoom the user chose and the viewport size at that moment: a resized window scales that zoom.
    ref_zoom: f32,
    ref_size: Vec2,
    /// Viewport size of the last frame.
    pub size: Vec2,
    doc_size: Vec2,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
            fitted: false,
            fit_pending: true,
            ref_zoom: 1.0,
            ref_size: Vec2::ZERO,
            size: Vec2::ZERO,
            doc_size: vec2(1.0, 1.0),
        }
    }
}

impl ViewState {
    /// Call every frame with the viewport and picture size; keeps the view right when either changes.
    pub fn update(&mut self, size: Vec2, doc_w: u32, doc_h: u32) {
        let doc_size = vec2(doc_w as f32, doc_h as f32);
        let doc_changed = doc_size != self.doc_size;
        self.doc_size = doc_size;
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        if self.fit_pending || (self.fitted && (size != self.size || doc_changed)) {
            self.size = size;
            self.fit();
        } else if size != self.size && self.size.x > 0.0 && self.size.y > 0.0 {
            if self.ref_size.x <= 0.0 {
                self.set_reference(self.size);
            }
            let scale = (size.x / self.ref_size.x).min(size.y / self.ref_size.y);
            let image_center = (self.size / 2.0 - self.offset) / self.zoom;
            self.zoom = (self.ref_zoom * scale).clamp(MIN_ZOOM, MAX_ZOOM);
            self.offset = size / 2.0 - image_center * self.zoom;
            self.size = size;
        }
    }

    /// The zoom "Fit on Screen" would use.
    pub fn fit_zoom(&self) -> f32 {
        let fit = ((self.size.x - FIT_MARGIN) / self.doc_size.x).min((self.size.y - FIT_MARGIN) / self.doc_size.y);
        fit.min(FIT_MAX).clamp(MIN_ZOOM, MAX_ZOOM)
    }

    /// Shows the whole picture, centered (Fit on Screen, Ctrl+0).
    pub fn fit(&mut self) {
        self.fit_pending = false;
        self.fitted = true;
        self.zoom = self.fit_zoom();
        self.offset = (self.size - self.doc_size * self.zoom) / 2.0;
        self.set_reference(self.size);
    }

    /// Zooms so the canvas point under `point` (viewport coordinates) stays under it.
    pub fn zoom_at(&mut self, point: Vec2, new_zoom: f32) {
        let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let image_point = (point - self.offset) / self.zoom;
        self.zoom = new_zoom;
        self.offset = point - image_point * self.zoom;
        self.fitted = false;
        self.fit_pending = false;
        self.set_reference(self.size);
    }

    /// Moves the picture by a screen distance.
    pub fn pan_by(&mut self, delta: Vec2) {
        self.offset += delta;
        self.fitted = false;
        self.fit_pending = false;
        self.set_reference(self.size);
    }

    pub fn zoom_in(&mut self) {
        let next = ZOOM_STEPS.iter().copied().find(|z| *z > self.zoom * 1.001).unwrap_or(MAX_ZOOM);
        self.zoom_at(self.size / 2.0, next);
    }

    pub fn zoom_out(&mut self) {
        let next = ZOOM_STEPS.iter().rev().copied().find(|z| *z < self.zoom * 0.999).unwrap_or(MIN_ZOOM);
        self.zoom_at(self.size / 2.0, next);
    }

    /// 100 % (Ctrl+1).
    pub fn actual_size(&mut self) {
        self.zoom_at(self.size / 2.0, 1.0);
    }

    /// Where canvas pixel (0, 0) is drawn, given the viewport's top-left corner. Rounded to whole pixels so the
    /// picture stays crisp, like the C# version.
    pub fn origin(&self, viewport_min: Pos2) -> Pos2 {
        viewport_min + vec2(self.offset.x.round(), self.offset.y.round())
    }

    fn set_reference(&mut self, size: Vec2) {
        self.ref_zoom = self.zoom;
        self.ref_size = size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(w: f32, h: f32, doc_w: u32, doc_h: u32) -> ViewState {
        let mut v = ViewState::default();
        v.update(vec2(w, h), doc_w, doc_h);
        v
    }

    #[test]
    fn a_new_view_fits_the_picture_with_a_margin() {
        let v = view(1040.0, 740.0, 2000, 1000);
        assert!(v.fitted);
        assert!((v.zoom - 0.5).abs() < 1e-6);
        assert_eq!(vec2(20.0, 120.0), v.offset);
    }

    #[test]
    fn small_pictures_are_enlarged_at_most_8_times() {
        assert_eq!(8.0, view(1000.0, 1000.0, 10, 10).zoom);
    }

    #[test]
    fn a_fitted_picture_follows_the_window_size() {
        let mut v = view(1040.0, 740.0, 2000, 1000);
        v.update(vec2(2040.0, 1040.0), 2000, 1000);
        assert!((v.zoom - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zooming_keeps_the_point_under_the_mouse() {
        let mut v = view(1040.0, 740.0, 2000, 1000);
        let p = vec2(300.0, 200.0);
        let before = (p - v.offset) / v.zoom;
        v.zoom_at(p, 2.0);
        assert!(((p - v.offset) / v.zoom - before).length() < 1e-3);
        assert!(!v.fitted);
    }

    #[test]
    fn zoom_steps_go_up_and_down() {
        let mut v = view(1000.0, 1000.0, 2000, 2000);
        v.actual_size();
        v.zoom_in();
        assert_eq!(1.5, v.zoom);
        v.zoom_out();
        v.zoom_out();
        assert_eq!(0.667, v.zoom);
    }

    #[test]
    fn after_zooming_a_resized_window_scales_the_zoom() {
        let mut v = view(1000.0, 1000.0, 2000, 2000);
        v.actual_size();
        v.update(vec2(500.0, 500.0), 2000, 2000);
        assert!((v.zoom - 0.5).abs() < 1e-6);
    }
}
