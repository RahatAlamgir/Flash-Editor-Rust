//! The retouching tools (C# `Tools/Retouch`): Clone Stamp, Healing Brush, Spot Healing, Dodge / Burn, Blur / Sharpen
//! (brush strokes with an operation, `brush.rs`), Patch (`patch.rs`) and Liquify (`liquify.rs`). One undo step per
//! stroke; strokes respect the selection (Liquify, like in C#, only works on the layer's pixels).

mod brush;
mod liquify;
mod patch;

use egui::Key;
use fe_core::imaging::PointD;
use fe_core::paint::brush::{BrushSettings, BrushType};
use fe_core::retouch::{LiquifyMode, ToneRange};

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// Settings of a brush-like retouching tool. Each tool shows only the parts it uses.
#[derive(Clone, Debug)]
pub struct RetouchOptions {
    pub size: f64,
    pub hardness: f64,
    pub strength: f64,
    /// Strokes keep the copy distance (Clone Stamp, Healing Brush).
    pub aligned: bool,
    pub burn: bool,
    pub sharpen: bool,
    pub range: ToneRange,
}

impl RetouchOptions {
    fn new(size: f64, strength: f64) -> Self {
        Self { size, hardness: 60.0, strength, aligned: true, burn: false, sharpen: false, range: ToneRange::Midtones }
    }

    pub fn settings(&self, opacity: f64) -> BrushSettings {
        BrushSettings {
            size: self.size.round().clamp(1.0, 1000.0),
            hardness: self.hardness.clamp(0.0, 100.0) / 100.0,
            opacity: opacity.clamp(0.01, 1.0),
            kind: BrushType::Round,
        }
    }

    fn step(size: f64) -> f64 {
        if size < 10.0 {
            1.0
        } else if size < 50.0 {
            5.0
        } else if size < 200.0 {
            10.0
        } else {
            50.0
        }
    }

    pub fn grow(&mut self) {
        self.size = (self.size + Self::step(self.size)).clamp(1.0, 1000.0);
    }

    pub fn shrink(&mut self) {
        let s = if self.size <= 10.0 { 1.0 } else { Self::step(self.size - 1.0) };
        self.size = (self.size - s).clamp(1.0, 1000.0);
    }
}

/// Liquify settings.
#[derive(Clone, Debug)]
pub struct LiquifyOptions {
    pub mode: LiquifyMode,
    pub size: f64,
    pub strength: f64,
}

impl Default for LiquifyOptions {
    fn default() -> Self {
        Self { mode: LiquifyMode::Push, size: 120.0, strength: 70.0 }
    }
}

/// What the canvas draws for the retouching tools.
pub struct RetouchOverlay {
    /// The place being copied (a small cross).
    pub cross: Option<PointD>,
    /// The outline of the Patch area, moved to where it is dragged.
    pub outline: Vec<PointD>,
}

/// The state of all retouching tools.
pub struct RetouchTools {
    pub clone: RetouchOptions,
    pub heal: RetouchOptions,
    pub spot: RetouchOptions,
    pub dodge: RetouchOptions,
    pub blur: RetouchOptions,
    pub liquify: LiquifyOptions,
    source: Option<PointD>,
    offset: Option<(i32, i32)>,
    cross: Option<PointD>,
    stroke: Option<brush::Active>,
    patch: patch::Patch,
    liq: Option<liquify::Active>,
}

impl Default for RetouchTools {
    fn default() -> Self {
        Self {
            clone: RetouchOptions::new(30.0, 100.0),
            heal: RetouchOptions::new(30.0, 100.0),
            spot: RetouchOptions::new(24.0, 100.0),
            dodge: RetouchOptions::new(30.0, 30.0),
            blur: RetouchOptions::new(30.0, 50.0),
            liquify: LiquifyOptions::default(),
            source: None,
            offset: None,
            cross: None,
            stroke: None,
            patch: patch::Patch::default(),
            liq: None,
        }
    }
}

impl RetouchTools {
    /// Drops anything half done (the tool was changed).
    pub fn cancel(&mut self) {
        self.patch.clear();
        self.cross = None;
    }

    pub fn options_mut(&mut self, tool: ToolKind) -> Option<&mut RetouchOptions> {
        match tool {
            ToolKind::CloneStamp => Some(&mut self.clone),
            ToolKind::HealingBrush => Some(&mut self.heal),
            ToolKind::SpotHealing => Some(&mut self.spot),
            ToolKind::DodgeBurn => Some(&mut self.dodge),
            ToolKind::BlurSharpen => Some(&mut self.blur),
            _ => None,
        }
    }

    /// The brush size of a retouching tool (for the brush circle on the canvas).
    pub fn brush_size(&self, tool: ToolKind) -> Option<f64> {
        match tool {
            ToolKind::CloneStamp => Some(self.clone.size),
            ToolKind::HealingBrush => Some(self.heal.size),
            ToolKind::SpotHealing => Some(self.spot.size),
            ToolKind::DodgeBurn => Some(self.dodge.size),
            ToolKind::BlurSharpen => Some(self.blur.size),
            ToolKind::Liquify => Some(self.liquify.size),
            _ => None,
        }
    }
}

pub fn overlay(app: &FlashApp) -> Option<RetouchOverlay> {
    let t = &app.tools.retouch;
    match app.tools.active() {
        ToolKind::CloneStamp | ToolKind::HealingBrush => {
            t.cross.or(t.source).map(|c| RetouchOverlay { cross: Some(c), outline: Vec::new() })
        }
        ToolKind::Patch => {
            let outline = t.patch.outline();
            (!outline.is_empty()).then_some(RetouchOverlay { cross: None, outline })
        }
        _ => None,
    }
}

/// Handles the mouse for a retouching tool. False when `tool` is not one.
pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) -> bool {
    match tool {
        ToolKind::CloneStamp
        | ToolKind::HealingBrush
        | ToolKind::SpotHealing
        | ToolKind::DodgeBurn
        | ToolKind::BlurSharpen => {
            brush::pointer(app, tool, p);
        }
        ToolKind::Patch => patch::pointer(app, p),
        ToolKind::Liquify => liquify::pointer(app, p),
        _ => return false,
    }
    true
}

/// [ ] change the brush size, Esc cancels a patch. True when the key was used.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    let tool = app.tools.active();
    if tool == ToolKind::Patch && key == Key::Escape {
        return patch::cancel(app);
    }
    let grow = match key {
        Key::CloseBracket => true,
        Key::OpenBracket => false,
        _ => return false,
    };
    if tool == ToolKind::Liquify {
        let l = &mut app.tools.retouch.liquify;
        l.size = (l.size + if grow { 10.0 } else { -10.0 }).clamp(5.0, 600.0);
        return true;
    }
    let Some(o) = app.tools.retouch.options_mut(tool) else { return false };
    if grow {
        o.grow();
    } else {
        o.shrink();
    }
    true
}

/// The status line each tool starts with.
pub fn hint(tool: ToolKind) -> Option<&'static str> {
    Some(match tool {
        ToolKind::CloneStamp => {
            "Clone Stamp: Alt+click the place to copy from, then paint where it should be copied to."
        }
        ToolKind::HealingBrush => {
            "Healing Brush: Alt+click the place to copy from, then paint over the blemish. The light and color of the surroundings are kept."
        }
        ToolKind::SpotHealing => {
            "Spot Healing: paint over a blemish (a spot, a speck, a wire); it is healed from a clean place nearby when you let go."
        }
        ToolKind::Patch => {
            "Patch: draw around the blemish, then drag the outlined area onto a clean part of the picture. Esc = cancel."
        }
        ToolKind::DodgeBurn => {
            "Dodge / Burn: paint light onto or shadow into the picture. Alt = the other one while painting."
        }
        ToolKind::BlurSharpen => {
            "Blur / Sharpen: soften or crisp up what you paint over. Alt = the other one while painting."
        }
        ToolKind::Liquify => {
            "Liquify: drag to push the pixels along. Choose Bloat or Pucker in the options bar to swell or pinch what is under the brush."
        }
        _ => return None,
    })
}
