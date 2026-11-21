//! Every tool with its name, icon, shortcut key and options-bar hint, and how the toolbox groups them.
//! Copied from the C# `ToolsViewModel` and the tool classes.

/// All tools, in the order of the C# `ToolsViewModel.Tools` list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolKind {
    Move,
    RectSelect,
    EllipseSelect,
    Lasso,
    PolygonLasso,
    Pen,
    QuickSelect,
    MagicWand,
    Crop,
    Brush,
    Eraser,
    Refine,
    CloneStamp,
    SpotHealing,
    HealingBrush,
    Patch,
    DodgeBurn,
    BlurSharpen,
    Bucket,
    Eyedropper,
    Text,
    Hand,
    Zoom,
    MaskBrush,
    PerspectiveCrop,
    Liquify,
    RemoveBrush,
    ClickSelect,
    Curve,
}

use ToolKind::*;

/// The toolbox buttons from top to bottom; tools in one group share a button (C# layout order).
pub const GROUPS: &[&[ToolKind]] = &[
    &[Move],
    &[RectSelect, EllipseSelect],
    &[Lasso, PolygonLasso],
    &[Pen, Curve],
    &[QuickSelect, MagicWand, ClickSelect],
    &[Crop, PerspectiveCrop],
    &[Brush, Eraser, Refine, MaskBrush],
    &[SpotHealing, HealingBrush, Patch, RemoveBrush],
    &[CloneStamp],
    &[DodgeBurn],
    &[BlurSharpen, Liquify],
    &[Bucket],
    &[Eyedropper],
    &[Text],
    &[Hand],
    &[Zoom],
];

impl ToolKind {
    /// The name shown in the hint bar and tooltips.
    pub const fn name(self) -> &'static str {
        match self {
            Move => "Select / Move",
            RectSelect => "Rectangle Select",
            EllipseSelect => "Ellipse Select",
            Lasso => "Lasso",
            PolygonLasso => "Polygon Lasso",
            Pen => "Pen (curve selection)",
            QuickSelect => "Quick Select (brush)",
            MagicWand => "Magic Wand",
            Crop => "Crop",
            Brush => "Brush",
            Eraser => "Eraser",
            Refine => "Refine (restore / remove)",
            CloneStamp => "Clone Stamp",
            SpotHealing => "Spot Healing",
            HealingBrush => "Healing Brush",
            Patch => "Patch",
            DodgeBurn => "Dodge / Burn",
            BlurSharpen => "Blur / Sharpen",
            Bucket => "Paint Bucket",
            Eyedropper => "Eyedropper",
            Text => "Text",
            Hand => "Hand",
            Zoom => "Zoom",
            MaskBrush => "Mask Brush",
            PerspectiveCrop => "Perspective Crop",
            Liquify => "Liquify",
            RemoveBrush => "Remove Brush (AI)",
            ClickSelect => "Click Select (AI)",
            Curve => "Curve (Bezier)",
        }
    }

    /// The line icon (a name from the C# `Icons.xaml`).
    pub const fn icon(self) -> &'static str {
        match self {
            Move => "Move",
            RectSelect => "RectSelect",
            EllipseSelect => "EllipseSelect",
            Lasso => "Lasso",
            PolygonLasso => "PolygonLasso",
            Pen => "Pen",
            QuickSelect => "QuickSelect",
            MagicWand => "MagicWand",
            Crop => "Crop",
            Brush => "Brush",
            Eraser => "Eraser",
            Refine => "Refine",
            CloneStamp => "Clone",
            SpotHealing => "SpotHeal",
            HealingBrush => "Heal",
            Patch => "Patch",
            DodgeBurn => "Tone",
            BlurSharpen => "BlurSharpen",
            Bucket => "Bucket",
            Eyedropper => "Eyedropper",
            Text => "Text",
            Hand => "Hand",
            Zoom => "Zoom",
            MaskBrush => "MaskBrush",
            PerspectiveCrop => "PerspectiveCrop",
            Liquify => "Liquify",
            RemoveBrush => "RemoveBrush",
            ClickSelect => "ClickSelect",
            Curve => "Curve",
        }
    }

    /// The single key that selects this tool (pressing it again cycles through tools with the same key).
    pub const fn shortcut(self) -> char {
        match self {
            Move => 'V',
            RectSelect | EllipseSelect => 'M',
            Lasso | PolygonLasso => 'L',
            Pen | Curve => 'P',
            QuickSelect | MagicWand | ClickSelect => 'W',
            Crop | PerspectiveCrop => 'C',
            Brush => 'B',
            Eraser => 'E',
            Refine => 'R',
            CloneStamp => 'S',
            SpotHealing | HealingBrush | Patch | RemoveBrush => 'J',
            DodgeBurn => 'O',
            BlurSharpen | Liquify => 'Q',
            Bucket => 'G',
            Eyedropper => 'I',
            Text => 'T',
            Hand => 'H',
            Zoom => 'Z',
            MaskBrush => 'K',
        }
    }

    /// "Brush (B)".
    pub fn tooltip(self) -> String {
        format!("{} ({})", self.name(), self.shortcut())
    }

    /// The hint text in the options bar while the tool has no settings to show there yet.
    pub const fn hint(self) -> &'static str {
        match self {
            Move => {
                "Click to select, Shift+click or Ctrl+drag a box to select several (Ctrl+G groups them). Drag to move, handles resize / rotate. Double-click a group to edit a part of it (Esc = back to the group), double-click text to edit it. Right-click for more."
            }
            RectSelect => {
                "Drag to select a rectangle (Shift after starting = square). Hold Shift before dragging to add, Alt to subtract. Click to deselect. Press M to switch rectangle / ellipse."
            }
            EllipseSelect => {
                "Drag to select an ellipse (Shift after starting = circle). Hold Shift before dragging to add, Alt to subtract. Click to deselect. Press M to switch rectangle / ellipse."
            }
            Lasso => {
                "Hold the mouse button and draw around an area. Release to close the selection. Shift = add, Alt = subtract. Press L to switch to the Polygon Lasso."
            }
            PolygonLasso => {
                "Click to add corners. Double-click, Enter, or click the first point to finish. Backspace = undo last point, Esc = cancel. Press L to switch lasso types."
            }
            Pen => {
                "Click = corner, click and drag = smooth curve. Click the first point, double-click or press Enter to select the shape. Backspace = undo last point, Esc = cancel. Shift = add, Alt = subtract."
            }
            Curve => {
                "Click = corner, click and drag = smooth curve. Click the first point to close the shape, double-click or Enter to end a line. Select a curve to edit it: drag points and handles, double-click a point to make it smooth / sharp, double-click the line to add a point, Delete removes a point."
            }
            Hand => "Drag to move the view. Hold Space with any tool to do the same.",
            Zoom => "Click to zoom in, Alt+click to zoom out. The mouse wheel zooms too.",
            Eyedropper => "Click to pick the foreground color. Alt+click or right-click picks the background color.",
            _ => "The settings of this tool arrive in a later step of the Rust version.",
        }
    }

    /// All tools in catalog order.
    pub fn all() -> impl Iterator<Item = ToolKind> {
        GROUPS.iter().flat_map(|g| g.iter().copied())
    }
}
