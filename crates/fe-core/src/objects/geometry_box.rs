//! `ObjectBox`: where an object is (C# `ObjectBox`): an upright box (top-left, width, height in canvas pixels)
//! turned by `rotation` degrees clockwise around its center.

use crate::imaging::{PixelRect, PointD};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObjectBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
}

impl ObjectBox {
    pub const fn new(x: f64, y: f64, width: f64, height: f64, rotation: f64) -> Self {
        Self { x, y, width, height, rotation }
    }

    pub fn center(&self) -> PointD {
        PointD::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn from_center(center: PointD, width: f64, height: f64, rotation: f64) -> Self {
        Self::new(center.x - width / 2.0, center.y - height / 2.0, width, height, rotation)
    }

    pub fn offset(&self, dx: f64, dy: f64) -> Self {
        Self { x: self.x + dx, y: self.y + dy, ..*self }
    }

    pub fn with_center(&self, center: PointD) -> Self {
        Self::from_center(center, self.width, self.height, self.rotation)
    }

    /// The canvas position of a point given relative to the center, before rotation.
    pub fn to_canvas(&self, lx: f64, ly: f64) -> PointD {
        let (s, c) = self.rotation.to_radians().sin_cos();
        let m = self.center();
        PointD::new(m.x + lx * c - ly * s, m.y + lx * s + ly * c)
    }

    /// A canvas point relative to the center, turned back upright.
    pub fn to_local(&self, p: PointD) -> PointD {
        let (s, c) = (-self.rotation).to_radians().sin_cos();
        let m = self.center();
        let (dx, dy) = (p.x - m.x, p.y - m.y);
        PointD::new(dx * c - dy * s, dx * s + dy * c)
    }

    /// True when the point lies inside the (turned) box grown by `margin`.
    pub fn contains(&self, p: PointD, margin: f64) -> bool {
        let l = self.to_local(p);
        l.x.abs() <= self.width / 2.0 + margin && l.y.abs() <= self.height / 2.0 + margin
    }

    /// Top-left, top-right, bottom-right, bottom-left on the canvas.
    pub fn corners(&self) -> [PointD; 4] {
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        [self.to_canvas(-hw, -hh), self.to_canvas(hw, -hh), self.to_canvas(hw, hh), self.to_canvas(-hw, hh)]
    }

    /// The upright rectangle around the turned box, grown by `margin`.
    pub fn bounds(&self, margin: f64) -> PixelRect {
        let c = self.corners();
        let (l, t) = c.iter().fold((f64::MAX, f64::MAX), |(l, t), p| (l.min(p.x), t.min(p.y)));
        let (r, b) = c.iter().fold((f64::MIN, f64::MIN), |(r, b), p| (r.max(p.x), b.max(p.y)));
        PixelRect::from_bounds(l - margin, t - margin, r + margin, b + margin)
    }

    /// True when the turned box and an upright rectangle overlap (rubber-band selection; exact for turned boxes).
    pub fn intersects_rect(&self, left: f64, top: f64, right: f64, bottom: f64) -> bool {
        let c = self.corners();
        let (minx, maxx) = c.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
        let (miny, maxy) = c.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
        if maxx < left || minx > right || maxy < top || miny > bottom {
            return false;
        }
        let local = [
            self.to_local(PointD::new(left, top)),
            self.to_local(PointD::new(right, top)),
            self.to_local(PointD::new(right, bottom)),
            self.to_local(PointD::new(left, bottom)),
        ];
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        let (lx0, lx1) = local.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
        let (ly0, ly1) = local.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
        !(lx1 < -hw || lx0 > hw || ly1 < -hh || ly0 > hh)
    }

    /// An angle in -180..180.
    pub fn normalize_angle(degrees: f64) -> f64 {
        let mut d = degrees % 360.0;
        if d > 180.0 {
            d -= 360.0;
        }
        if d <= -180.0 {
            d += 360.0;
        }
        d
    }
}
