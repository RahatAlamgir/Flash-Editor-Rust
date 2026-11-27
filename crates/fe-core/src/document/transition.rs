//! The slide transition of a page (how the slideshow moves from the previous page to this one), like PowerPoint's.

/// The kind of movement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionKind {
    #[default]
    None,
    Fade,
    /// The new page pushes the old one out.
    Push,
    /// The new page is uncovered along an edge.
    Wipe,
    /// The new page slides in over the old one.
    Cover,
    /// The old page slides away and shows the new one underneath.
    Uncover,
    /// The new page grows from the middle.
    Zoom,
    /// The new page opens from the middle outward.
    Split,
}

/// Where the movement starts (the edge the new page comes from).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionDirection {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

impl TransitionKind {
    pub const ALL: [TransitionKind; 8] =
        [Self::None, Self::Fade, Self::Push, Self::Wipe, Self::Cover, Self::Uncover, Self::Zoom, Self::Split];

    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Fade => "Fade",
            Self::Push => "Push",
            Self::Wipe => "Wipe",
            Self::Cover => "Cover",
            Self::Uncover => "Uncover",
            Self::Zoom => "Zoom",
            Self::Split => "Split",
        }
    }

    /// True when the direction matters.
    pub const fn has_direction(self) -> bool {
        matches!(self, Self::Push | Self::Wipe | Self::Cover | Self::Uncover)
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL.into_iter().find(|k| k.name().eq_ignore_ascii_case(name)).unwrap_or_default()
    }
}

impl TransitionDirection {
    pub const ALL: [TransitionDirection; 4] = [Self::Left, Self::Right, Self::Up, Self::Down];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "From left",
            Self::Right => "From right",
            Self::Up => "From top",
            Self::Down => "From bottom",
        }
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL.into_iter().find(|d| d.key().eq_ignore_ascii_case(name)).unwrap_or_default()
    }

    /// The word saved in the file.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Up => "Up",
            Self::Down => "Down",
        }
    }

    /// The unit step the new page travels along (from where it starts toward its place).
    pub const fn travel(self) -> (f32, f32) {
        match self {
            Self::Left => (1.0, 0.0),
            Self::Right => (-1.0, 0.0),
            Self::Up => (0.0, 1.0),
            Self::Down => (0.0, -1.0),
        }
    }
}

/// A page's transition and when the slideshow moves on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub kind: TransitionKind,
    pub direction: TransitionDirection,
    /// Seconds the movement takes.
    pub duration: f32,
    /// Moves on with a click or key (default).
    pub on_click: bool,
    /// Also moves on by itself after this many seconds on the page.
    pub after: Option<f32>,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            kind: TransitionKind::None,
            direction: TransitionDirection::Left,
            duration: 0.7,
            on_click: true,
            after: None,
        }
    }
}

impl Transition {
    /// True for the plain "no transition, advance on click" that needs no entry in the file.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The same movement for a bigger or smaller `duration` (kept within 0.1 to 6 seconds).
    pub fn with_duration(mut self, seconds: f32) -> Self {
        self.duration = seconds.clamp(0.1, 6.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for kind in TransitionKind::ALL {
            assert_eq!(kind, TransitionKind::from_name(kind.name()));
        }
        for direction in TransitionDirection::ALL {
            assert_eq!(direction, TransitionDirection::from_name(direction.key()));
        }
        assert!(Transition::default().is_default());
    }
}
