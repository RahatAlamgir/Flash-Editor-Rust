//! Object animations of a page (entrance, emphasis and exit effects on layers, like PowerPoint's): the effects, when each
//! one starts (on click, with the previous one, after the previous one), and how a layer looks at any moment of the
//! slideshow. The slideshow, the Animation pane and the PowerPoint export all use this.

use super::{LayerId, TransitionDirection};

/// What kind of effect it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectClass {
    /// The layer comes in (it is hidden before).
    Entrance,
    /// The layer stays and draws attention.
    Emphasis,
    /// The layer goes away.
    Exit,
}

impl EffectClass {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Entrance => "Entrance",
            Self::Emphasis => "Emphasis",
            Self::Exit => "Exit",
        }
    }
}

/// The effects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effect {
    Appear,
    #[default]
    FadeIn,
    FlyIn,
    FloatIn,
    ZoomIn,
    WipeIn,
    GrowTurn,
    Pulse,
    Spin,
    GrowShrink,
    Teeter,
    Disappear,
    FadeOut,
    FlyOut,
    FloatOut,
    ZoomOut,
    WipeOut,
}

impl Effect {
    pub const ALL: [Effect; 17] = [
        Self::Appear,
        Self::FadeIn,
        Self::FlyIn,
        Self::FloatIn,
        Self::ZoomIn,
        Self::WipeIn,
        Self::GrowTurn,
        Self::Pulse,
        Self::Spin,
        Self::GrowShrink,
        Self::Teeter,
        Self::Disappear,
        Self::FadeOut,
        Self::FlyOut,
        Self::FloatOut,
        Self::ZoomOut,
        Self::WipeOut,
    ];

    pub const fn class(self) -> EffectClass {
        match self {
            Self::Appear
            | Self::FadeIn
            | Self::FlyIn
            | Self::FloatIn
            | Self::ZoomIn
            | Self::WipeIn
            | Self::GrowTurn => EffectClass::Entrance,
            Self::Pulse | Self::Spin | Self::GrowShrink | Self::Teeter => EffectClass::Emphasis,
            Self::Disappear | Self::FadeOut | Self::FlyOut | Self::FloatOut | Self::ZoomOut | Self::WipeOut => {
                EffectClass::Exit
            }
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Appear => "Appear",
            Self::FadeIn => "Fade In",
            Self::FlyIn => "Fly In",
            Self::FloatIn => "Float In",
            Self::ZoomIn => "Zoom In",
            Self::WipeIn => "Wipe In",
            Self::GrowTurn => "Grow & Turn",
            Self::Pulse => "Pulse",
            Self::Spin => "Spin",
            Self::GrowShrink => "Grow / Shrink",
            Self::Teeter => "Teeter",
            Self::Disappear => "Disappear",
            Self::FadeOut => "Fade Out",
            Self::FlyOut => "Fly Out",
            Self::FloatOut => "Float Out",
            Self::ZoomOut => "Zoom Out",
            Self::WipeOut => "Wipe Out",
        }
    }

    /// True when the direction matters.
    pub const fn has_direction(self) -> bool {
        matches!(self, Self::FlyIn | Self::FlyOut | Self::WipeIn | Self::WipeOut)
    }

    /// True for the effects that happen at once (no duration).
    pub const fn instant(self) -> bool {
        matches!(self, Self::Appear | Self::Disappear)
    }

    /// The word saved in the file.
    pub fn key(self) -> &'static str {
        self.name()
    }

    pub fn from_key(key: &str) -> Self {
        Self::ALL.into_iter().find(|e| e.name().eq_ignore_ascii_case(key)).unwrap_or_default()
    }

    /// A good starting length.
    pub const fn default_duration(self) -> f32 {
        match self {
            Self::Appear | Self::Disappear => 0.0,
            Self::Spin => 1.0,
            _ => 0.5,
        }
    }
}

/// When an animation starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Start {
    /// With the next click or key.
    #[default]
    OnClick,
    /// Together with the animation before it.
    WithPrevious,
    /// When the animation before it has finished.
    AfterPrevious,
}

impl Start {
    pub const ALL: [Start; 3] = [Self::OnClick, Self::WithPrevious, Self::AfterPrevious];

    pub const fn name(self) -> &'static str {
        match self {
            Self::OnClick => "On click",
            Self::WithPrevious => "With previous",
            Self::AfterPrevious => "After previous",
        }
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL.into_iter().find(|s| s.name().eq_ignore_ascii_case(name)).unwrap_or_default()
    }
}

/// One animation of a layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animation {
    pub layer: LayerId,
    pub effect: Effect,
    pub start: Start,
    /// For Fly and Wipe: the edge it comes from (or goes to).
    pub direction: TransitionDirection,
    pub duration: f32,
    pub delay: f32,
}

impl Animation {
    pub fn new(layer: LayerId, effect: Effect) -> Self {
        Self {
            layer,
            effect,
            start: Start::OnClick,
            direction: TransitionDirection::Left,
            duration: effect.default_duration(),
            delay: 0.0,
        }
    }

    /// Seconds the movement takes (nothing for the instant effects).
    pub fn length(&self) -> f32 {
        if self.effect.instant() { 0.0 } else { self.duration.max(0.05) }
    }
}

/// When one animation runs: in which click step, and at which seconds after that step began.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timed {
    /// Index into the page's animation list.
    pub index: usize,
    /// 0 = starts by itself when the slide appears; 1, 2... = after that many clicks.
    pub step: usize,
    /// Seconds after the step began when the delay starts.
    pub begin: f32,
    /// Seconds after the step began when it is over.
    pub end: f32,
}

/// The animations of a page put in time order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Timeline {
    pub items: Vec<Timed>,
    /// How many clicks the animations of the page need.
    pub clicks: usize,
}

impl Timeline {
    /// The time the step is over (the end of its last animation).
    pub fn step_length(&self, step: usize) -> f32 {
        self.items.iter().filter(|t| t.step == step).map(|t| t.end).fold(0.0, f32::max)
    }
}

/// Works out when every animation of the list runs.
pub fn timeline(animations: &[Animation]) -> Timeline {
    let mut items: Vec<Timed> = Vec::with_capacity(animations.len());
    let mut step = 0usize;
    for (index, a) in animations.iter().enumerate() {
        let previous = items.last().copied();
        let begin = match (a.start, previous) {
            (Start::OnClick, _) => {
                step += 1;
                0.0
            }
            (Start::WithPrevious, Some(p)) => p.begin,
            (Start::AfterPrevious, Some(p)) => p.end,
            (_, None) => 0.0,
        };
        items.push(Timed { index, step, begin, end: begin + a.delay.max(0.0) + a.length() });
    }
    Timeline { items, clicks: step }
}

/// How a layer looks at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub visible: bool,
    pub alpha: f32,
    /// Shift, as a fraction of the page's width and height.
    pub dx: f32,
    pub dy: f32,
    pub scale: f32,
    /// Turn, in radians.
    pub angle: f32,
    /// Only this part is shown: (the edge the reveal starts from, how much 0..1).
    pub reveal: Option<(TransitionDirection, f32)>,
}

impl Look {
    pub const SHOWN: Look = Look { visible: true, alpha: 1.0, dx: 0.0, dy: 0.0, scale: 1.0, angle: 0.0, reveal: None };

    fn hidden() -> Look {
        Look { visible: false, ..Self::SHOWN }
    }
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The look of the layer while `a` is running, `t` = how far (0..1).
fn running(a: &Animation, t: f32) -> Look {
    let p = smooth(t);
    let (tx, ty) = a.direction.travel();
    let mut look = Look::SHOWN;
    match a.effect {
        Effect::Appear | Effect::Disappear => {}
        Effect::FadeIn => look.alpha = p,
        Effect::FadeOut => look.alpha = 1.0 - p,
        Effect::FlyIn => (look.dx, look.dy) = (-tx * (1.0 - p), -ty * (1.0 - p)),
        Effect::FlyOut => (look.dx, look.dy) = (-tx * p, -ty * p),
        Effect::FloatIn => (look.dy, look.alpha) = (0.06 * (1.0 - p), p),
        Effect::FloatOut => (look.dy, look.alpha) = (-0.06 * p, 1.0 - p),
        Effect::ZoomIn => (look.scale, look.alpha) = (p.max(0.01), p),
        Effect::ZoomOut => (look.scale, look.alpha) = ((1.0 - p).max(0.01), 1.0 - p),
        Effect::WipeIn => look.reveal = Some((a.direction, p)),
        Effect::WipeOut => look.reveal = Some((a.direction, 1.0 - p)),
        Effect::GrowTurn => {
            (look.scale, look.alpha, look.angle) = (p.max(0.01), p, -(1.0 - p) * std::f32::consts::FRAC_PI_2)
        }
        Effect::Pulse => look.scale = 1.0 + 0.15 * (std::f32::consts::PI * t).sin(),
        Effect::Spin => look.angle = std::f32::consts::TAU * p,
        Effect::GrowShrink => look.scale = 1.0 + 0.5 * (std::f32::consts::PI * t).sin(),
        Effect::Teeter => look.angle = 0.14 * (std::f32::consts::TAU * 2.0 * t).sin() * (1.0 - t),
    }
    look
}

/// How `layer` looks when `step` clicks have happened and `time` seconds have passed since that step began.
/// (`step` 0 = the slide just appeared.) A layer whose first animation is an entrance is hidden before it.
pub fn look_of(animations: &[Animation], timeline: &Timeline, layer: LayerId, step: usize, time: f32) -> Look {
    let own: Vec<&Timed> = timeline.items.iter().filter(|t| animations[t.index].layer == layer).collect();
    let Some(first) = own.first() else { return Look::SHOWN };
    let mut look =
        if animations[first.index].effect.class() == EffectClass::Entrance { Look::hidden() } else { Look::SHOWN };
    for timed in own {
        let a = &animations[timed.index];
        let started = timed.step < step || (timed.step == step && time >= timed.begin + a.delay.max(0.0));
        if !started {
            continue;
        }
        let begin = timed.begin + a.delay.max(0.0);
        let done = timed.step < step || time >= timed.end;
        look = if done {
            match a.effect.class() {
                EffectClass::Exit => Look::hidden(),
                _ => Look::SHOWN,
            }
        } else {
            running(a, (time - begin) / a.length())
        };
    }
    look
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::imaging::ColorRgba;

    fn id() -> LayerId {
        Document::blank(10, 10, ColorRgba::WHITE).layers()[0].id()
    }

    #[test]
    fn click_with_and_after_previous_make_a_timeline() {
        let l = id();
        let mut a = Animation::new(l, Effect::FadeIn);
        a.duration = 1.0;
        let mut b = Animation::new(l, Effect::FlyIn);
        b.start = Start::WithPrevious;
        b.duration = 0.5;
        b.delay = 0.25;
        let mut c = Animation::new(l, Effect::Pulse);
        c.start = Start::AfterPrevious;
        let mut d = Animation::new(l, Effect::FadeOut);
        d.start = Start::OnClick;
        let t = timeline(&[a, b, c, d]);
        assert_eq!(2, t.clicks);
        assert_eq!((1, 0.0, 1.0), (t.items[0].step, t.items[0].begin, t.items[0].end));
        assert_eq!((1, 0.0, 0.75), (t.items[1].step, t.items[1].begin, t.items[1].end)); // with the first: same start
        assert_eq!((1, 0.75, 1.25), (t.items[2].step, t.items[2].begin, t.items[2].end)); // after the one before it
        assert_eq!(2, t.items[3].step);
    }

    #[test]
    fn an_entrance_hides_the_layer_until_its_click() {
        let l = id();
        let anims = [Animation::new(l, Effect::FadeIn)];
        let t = timeline(&anims);
        assert!(!look_of(&anims, &t, l, 0, 5.0).visible);
        let half = look_of(&anims, &t, l, 1, 0.25);
        assert!(half.visible && half.alpha > 0.0 && half.alpha < 1.0);
        assert_eq!(Look::SHOWN, look_of(&anims, &t, l, 1, 9.0));
        assert_eq!(Look::SHOWN, look_of(&anims, &t, l, 2, 0.0));
    }

    #[test]
    fn an_exit_hides_the_layer_after_it_and_emphasis_returns_to_normal() {
        let l = id();
        let anims = [Animation::new(l, Effect::Pulse), Animation::new(l, Effect::Disappear)];
        let t = timeline(&anims);
        assert!(look_of(&anims, &t, l, 0, 0.0).visible); // not an entrance: shown from the start
        assert!(look_of(&anims, &t, l, 1, 0.25).scale > 1.0);
        assert_eq!(Look::SHOWN, look_of(&anims, &t, l, 1, 5.0));
        assert!(!look_of(&anims, &t, l, 2, 0.0).visible);
    }
}
