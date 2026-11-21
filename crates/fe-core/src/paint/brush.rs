//! Brush tips and settings (C# `BrushType`, `BrushSettings`).

/// The shape / texture of the mark a brush leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BrushType {
    /// Round, with a hardness setting (the classic brush).
    #[default]
    Round,
    Square,
    /// A flat, slanted tip: thick strokes one way, thin the other.
    Calligraphy,
    /// Hard-edged and slightly grainy, like graphite.
    Pencil,
    /// Soft, rough and speckled, like chalk or charcoal.
    Chalk,
    /// A fine mist of tiny dots (airbrush).
    Spray,
    /// Random blobs of different sizes around the path.
    Splatter,
}

impl BrushType {
    pub const ALL: [BrushType; 7] = [
        BrushType::Round,
        BrushType::Square,
        BrushType::Calligraphy,
        BrushType::Pencil,
        BrushType::Chalk,
        BrushType::Spray,
        BrushType::Splatter,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            BrushType::Round => "Round",
            BrushType::Square => "Square",
            BrushType::Calligraphy => "Calligraphy",
            BrushType::Pencil => "Pencil",
            BrushType::Chalk => "Chalk",
            BrushType::Spray => "Spray",
            BrushType::Splatter => "Splatter",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            BrushType::Round => "Round brush with adjustable hardness",
            BrushType::Square => "Square tip",
            BrushType::Calligraphy => "Flat slanted tip: thick one way, thin the other",
            BrushType::Pencil => "Hard-edged, slightly grainy graphite",
            BrushType::Chalk => "Soft, rough, speckled chalk",
            BrushType::Spray => "Fine mist of dots (airbrush)",
            BrushType::Splatter => "Random blobs around the path",
        }
    }

    /// Whether the Hardness setting changes how this type looks.
    pub const fn uses_hardness(self) -> bool {
        matches!(self, BrushType::Round | BrushType::Square | BrushType::Calligraphy | BrushType::Chalk)
    }
}

/// Brush shape and strength.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSettings {
    /// Diameter in pixels (1..1000).
    pub size: f64,
    /// 0 = very soft edge, 1 = crisp edge.
    pub hardness: f64,
    /// 0..1, the most one stroke can cover.
    pub opacity: f64,
    pub kind: BrushType,
}

impl BrushSettings {
    pub fn radius(&self) -> f64 {
        (self.size / 2.0).max(0.5)
    }
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self { size: 20.0, hardness: 0.8, opacity: 1.0, kind: BrushType::Round }
    }
}
