//! Settings of the painting tools (C# `BrushOptions`, `BucketOptions`).

use fe_core::paint::brush::{BrushSettings, BrushType};

/// Brush, Eraser and Mask Brush settings (each tool has its own).
#[derive(Clone, Debug)]
pub struct BrushOptions {
    pub kind: BrushType,
    /// Diameter, 1..1000 px.
    pub size: f64,
    /// 0..100 %.
    pub hardness: f64,
    /// 1..100 %.
    pub opacity: f64,
}

impl Default for BrushOptions {
    fn default() -> Self {
        Self { kind: BrushType::Round, size: 20.0, hardness: 80.0, opacity: 100.0 }
    }
}

impl BrushOptions {
    pub fn settings(&self) -> BrushSettings {
        BrushSettings {
            size: self.size.round().clamp(1.0, 1000.0),
            hardness: self.hardness.clamp(0.0, 100.0) / 100.0,
            opacity: self.opacity.clamp(1.0, 100.0) / 100.0,
            kind: self.kind,
        }
    }

    /// "]": bigger in sensible steps.
    pub fn grow(&mut self) {
        let s = self.size;
        self.size = (s + if s < 10.0 {
            1.0
        } else if s < 50.0 {
            5.0
        } else if s < 200.0 {
            10.0
        } else {
            50.0
        })
        .clamp(1.0, 1000.0);
    }

    /// "[": smaller in sensible steps.
    pub fn shrink(&mut self) {
        let s = self.size;
        self.size = (s - if s <= 10.0 {
            1.0
        } else if s <= 50.0 {
            5.0
        } else if s <= 200.0 {
            10.0
        } else {
            50.0
        })
        .clamp(1.0, 1000.0);
    }
}

/// Paint Bucket settings.
#[derive(Clone, Debug)]
pub struct BucketOptions {
    /// 0 = exactly that color, 255 = everything.
    pub tolerance: f64,
    /// Only connected pixels (otherwise every similar pixel).
    pub contiguous: bool,
}

impl Default for BucketOptions {
    fn default() -> Self {
        Self { tolerance: 32.0, contiguous: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_change_the_size_in_steps_like_the_csharp_version() {
        let mut o = BrushOptions { size: 9.0, ..Default::default() };
        o.grow();
        assert_eq!(10.0, o.size);
        o.grow();
        assert_eq!(15.0, o.size);
        o.size = 200.0;
        o.shrink();
        assert_eq!(190.0, o.size);
        o.size = 1.0;
        o.shrink();
        assert_eq!(1.0, o.size);
    }
}
