//! How a layer mixes with the layers below it.

/// Layer blend modes. The order and names match the C# `BlendMode` (they are saved in `.flash` files).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    Darken,
    Lighten,
    Difference,
}

impl BlendMode {
    pub const ALL: [Self; 9] = [
        Self::Normal,
        Self::Multiply,
        Self::Screen,
        Self::Overlay,
        Self::SoftLight,
        Self::HardLight,
        Self::Darken,
        Self::Lighten,
        Self::Difference,
    ];

    /// The name saved in project files (`"SoftLight"`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Multiply => "Multiply",
            Self::Screen => "Screen",
            Self::Overlay => "Overlay",
            Self::SoftLight => "SoftLight",
            Self::HardLight => "HardLight",
            Self::Darken => "Darken",
            Self::Lighten => "Lighten",
            Self::Difference => "Difference",
        }
    }

    /// Reads a saved name; unknown names give `None`.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.key().eq_ignore_ascii_case(key))
    }

    /// The name shown in the Blend list.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::SoftLight => "Soft light",
            Self::HardLight => "Hard light",
            other => other.key(),
        }
    }

    /// One-line explanation for tooltips.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Multiply => "Darkens: the layer's colors multiply with what is below (good for shadows)",
            Self::Screen => "Lightens: like projecting light on top (good for glows)",
            Self::Overlay => "More contrast: darks get darker and lights lighter, based on what is below",
            Self::SoftLight => "A gentle version of Overlay",
            Self::HardLight => "A strong version of Overlay, based on the layer itself",
            Self::Darken => "Keeps the darker color of the two",
            Self::Lighten => "Keeps the lighter color of the two",
            Self::Difference => "Shows how different the two colors are (inverts where the layer is white)",
            Self::Normal => "The layer simply covers what is below",
        }
    }
}
