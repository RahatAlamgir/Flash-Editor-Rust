//! An axis-aligned rectangle in whole pixels.

use std::fmt;

/// A rectangle in pixels. Empty when width or height is 0 or less.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl PixelRect {
    pub const EMPTY: Self = Self::new(0, 0, 0, 0);

    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub const fn right(&self) -> i32 {
        self.x + self.width
    }

    pub const fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub const fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn area(&self) -> i64 {
        if self.is_empty() { 0 } else { self.width as i64 * self.height as i64 }
    }

    pub fn from_corners(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let (l, t) = (x1.min(x2), y1.min(y2));
        Self::new(l, t, x1.max(x2) - l, y1.max(y2) - t)
    }

    /// The smallest whole-pixel rectangle that covers the given edges.
    pub fn from_bounds(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        let (l, t) = (left.floor() as i32, top.floor() as i32);
        let (r, b) = (right.ceil() as i32, bottom.ceil() as i32);
        Self::new(l, t, (r - l).max(0), (b - t).max(0))
    }

    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let (l, t) = (self.x.max(other.x), self.y.max(other.y));
        let (r, b) = (self.right().min(other.right()), self.bottom().min(other.bottom()));
        if r <= l || b <= t { Self::EMPTY } else { Self::new(l, t, r - l, b - t) }
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        let (l, t) = (self.x.min(other.x), self.y.min(other.y));
        let (r, b) = (self.right().max(other.right()), self.bottom().max(other.bottom()));
        Self::new(l, t, r - l, b - t)
    }

    pub const fn inflate(&self, amount: i32) -> Self {
        Self::new(self.x - amount, self.y - amount, self.width + amount * 2, self.height + amount * 2)
    }

    pub const fn offset(&self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }
}

impl fmt::Display for PixelRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{} {}x{}", self.x, self.y, self.width, self.height)
    }
}
