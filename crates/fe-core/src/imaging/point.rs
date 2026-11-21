//! A point with fractional coordinates (canvas pixels).

/// A point in canvas pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointD {
    pub x: f64,
    pub y: f64,
}

impl PointD {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }

    pub fn offset(&self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}
