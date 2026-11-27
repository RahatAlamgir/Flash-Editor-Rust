//! Which tool is active, which tool each toolbox group shows, and which group list is open.

use super::catalog::{GROUPS, ToolKind};
use super::moving::MoveTool;
use super::painting::PaintTools;
use super::selecting::SelectTools;

/// The toolbox state.
pub struct ToolState {
    active: ToolKind,
    /// The tool each group button shows (the last one picked from that group).
    current: Vec<ToolKind>,
    /// The group whose list of tools is open, if any.
    pub open_group: Option<usize>,
    /// The selection tools' state and settings.
    pub select: SelectTools,
    /// The Select / Move tool's state.
    pub mover: MoveTool,
    /// Brush, Eraser and Mask Brush state and settings.
    pub paint: PaintTools,
    /// The Curve tool's state.
    pub curve: super::curve::CurveTool,
    /// The Crop and Perspective Crop tools' state.
    pub crop: super::crop::CropTools,
    /// The retouching tools' state.
    pub retouch: super::retouch::RetouchTools,
    /// Remove Brush, Click Select and Refine.
    pub ai: super::ai_tools::AiTools,
    /// Tool keys the user changed (None = no key).
    pub key_overrides: std::collections::HashMap<ToolKind, Option<char>>,
    /// The Text tool's settings.
    pub text: super::text::TextTool,
    /// The Shape tool's state.
    pub shape: super::shape::ShapeTool,
}

impl Default for ToolState {
    /// The program starts on Select / Move, like the C# version.
    fn default() -> Self {
        // Objects keep their proportions when resized by a corner (hold Shift for a free resize).
        let mut mover = MoveTool::default();
        mover.keep_proportions = true;
        Self {
            active: ToolKind::Move,
            current: GROUPS.iter().map(|g| g[0]).collect(),
            open_group: None,
            select: SelectTools::default(),
            mover,
            paint: PaintTools::default(),
            curve: Default::default(),
            crop: Default::default(),
            retouch: Default::default(),
            ai: Default::default(),
            key_overrides: Default::default(),
            text: Default::default(),
            shape: Default::default(),
        }
    }
}

impl ToolState {
    pub fn active(&self) -> ToolKind {
        self.active
    }

    /// The tool group `index` shows on its button.
    pub fn current_of(&self, index: usize) -> ToolKind {
        self.current[index]
    }

    /// True when the active tool is in group `index`.
    pub fn is_group_active(&self, index: usize) -> bool {
        GROUPS[index].contains(&self.active)
    }

    /// Drops what the active tool has half done (a selection being drawn, a curve being placed).
    pub fn reset_active(&mut self) {
        self.select.cancel();
        self.curve.cancel();
        self.crop.cancel();
        self.retouch.cancel();
        self.ai.cancel();
    }

    /// Makes a tool active; its group button now shows it.
    pub fn activate(&mut self, tool: ToolKind) {
        if tool != self.active {
            self.select.cancel(); // drop a half-drawn selection, like C# `Deactivate`
            self.curve.cancel();
            self.crop.cancel();
            self.retouch.cancel();
            self.ai.cancel();
        }
        self.active = tool;
        if let Some(group) = GROUPS.iter().position(|g| g.contains(&tool)) {
            self.current[group] = tool;
        }
        self.open_group = None;
    }

    /// The key that selects a tool now (the user may have changed it).
    pub fn key_of(&self, tool: ToolKind) -> Option<char> {
        self.key_overrides.get(&tool).copied().unwrap_or_else(|| Some(tool.shortcut()))
    }

    /// Handles a tool key (like "B"): picks the next tool with that key. Returns false when no tool has it.
    pub fn select_by_shortcut(&mut self, key: char) -> bool {
        let key = key.to_ascii_uppercase();
        let matches: Vec<ToolKind> = ToolKind::all().filter(|t| self.key_of(*t) == Some(key)).collect();
        if matches.is_empty() {
            return false;
        }
        let next = match matches.iter().position(|t| *t == self.active) {
            Some(i) => matches[(i + 1) % matches.len()],
            None => matches[0],
        };
        self.activate(next);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_move_and_cycles_tools_that_share_a_key() {
        let mut tools = ToolState::default();
        assert_eq!(ToolKind::Move, tools.active());
        assert!(tools.select_by_shortcut('j'));
        assert_eq!(ToolKind::SpotHealing, tools.active());
        assert!(tools.select_by_shortcut('J'));
        assert_eq!(ToolKind::HealingBrush, tools.active());
        assert!(!tools.select_by_shortcut('9'));
    }

    #[test]
    fn a_group_button_shows_the_last_tool_picked_from_it() {
        let mut tools = ToolState::default();
        tools.activate(ToolKind::Eraser);
        let brush_group = GROUPS.iter().position(|g| g.contains(&ToolKind::Brush)).unwrap_or(0);
        assert_eq!(ToolKind::Eraser, tools.current_of(brush_group));
        assert!(tools.is_group_active(brush_group));
        tools.activate(ToolKind::Move);
        assert_eq!(ToolKind::Eraser, tools.current_of(brush_group));
        assert!(!tools.is_group_active(brush_group));
    }

    #[test]
    fn every_tool_icon_exists() {
        for tool in ToolKind::all() {
            assert!(fe_ui_kit::icons::path_data(tool.icon()).is_some(), "missing icon {}", tool.icon());
        }
    }
}
