//! What the ribbon contains: tabs, groups and buttons, copied from the C# `RibbonBar.xaml`
//! (same order, labels, icon glyphs and colorful-style colors).

/// The color of a button icon in the "Colorful" icon style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconColor {
    Blue,
    Sky,
    Teal,
    Green,
    Lime,
    Amber,
    Orange,
    Red,
    Pink,
    Purple,
    Slate,
    /// Toggle buttons keep the plain gray.
    Plain,
}

/// One big ribbon button.
#[derive(Clone, Copy, Debug)]
pub struct RibbonItem {
    /// Code point in the Windows icon font.
    pub glyph: char,
    pub label: &'static str,
    pub color: IconColor,
    /// On / off button (Rulers, Grid...).
    pub toggle: bool,
}

/// A group of buttons with a divider after it.
pub struct RibbonGroup {
    pub name: &'static str,
    pub items: &'static [RibbonItem],
}

/// One ribbon tab.
pub struct RibbonTab {
    pub name: &'static str,
    pub groups: &'static [RibbonGroup],
}

const fn b(glyph: char, label: &'static str, color: IconColor) -> RibbonItem {
    RibbonItem { glyph, label, color, toggle: false }
}

const fn t(glyph: char, label: &'static str) -> RibbonItem {
    RibbonItem { glyph, label, color: IconColor::Plain, toggle: true }
}

use IconColor::*;

/// All ribbon tabs, left to right.
pub const TABS: &[RibbonTab] = &[
    RibbonTab {
        name: "Edit",
        groups: &[
            RibbonGroup { name: "Undo", items: &[b('\u{E7A7}', "Undo", Blue), b('\u{E7A6}', "Redo", Blue)] },
            RibbonGroup {
                name: "Clipboard",
                items: &[
                    b('\u{E77F}', "Paste", Sky),
                    b('\u{E8C6}', "Cut", Sky),
                    b('\u{E8C8}', "Copy", Sky),
                    b('\u{E74D}', "Delete Pixels", Red),
                    b('\u{E790}', "Fill Color", Pink),
                ],
            },
            RibbonGroup {
                name: "Layer",
                items: &[
                    b('\u{E8C8}', "Duplicate", Sky),
                    b('\u{E74D}', "Delete", Red),
                    b('\u{E72E}', "Lock / Unlock", Amber),
                    b('\u{E8A9}', "Group", Sky),
                    b('\u{E8B4}', "Ungroup", Sky),
                    b('\u{E898}', "Order ▾", Sky),
                ],
            },
        ],
    },
    RibbonTab {
        name: "Select",
        groups: &[
            RibbonGroup {
                name: "Selection",
                items: &[b('\u{E8FF}', "Deselect", Teal), b('\u{E8B4}', "Invert Selection", Teal)],
            },
            RibbonGroup {
                name: "Smart",
                items: &[
                    b('\u{E794}', "Subject (AI)", Purple),
                    b('\u{E8BF}', "Layer Content", Teal),
                    b('\u{E9A3}', "Fill (AI)", Purple),
                ],
            },
            RibbonGroup {
                name: "Add mask",
                items: &[
                    b('\u{E8BF}', "Mask: Show Selection", Teal),
                    b('\u{E8BF}', "Mask: Hide Selection", Teal),
                    b('\u{E8BF}', "Mask: Show All", Teal),
                    b('\u{E8BF}', "Mask: Hide All", Teal),
                ],
            },
            RibbonGroup {
                name: "Mask",
                items: &[b('\u{E8EB}', "Invert Mask", Teal), b('\u{E74D}', "Delete Mask", Red)],
            },
        ],
    },
    RibbonTab {
        name: "Image",
        groups: &[
            RibbonGroup {
                name: "Size",
                items: &[
                    b('\u{E740}', "Image Size", Green),
                    b('\u{E8B2}', "Canvas Size", Green),
                    b('\u{E924}', "Trim Edges", Red),
                    b('\u{E7B3}', "Reveal All", Green),
                    b('\u{E7AD}', "Straighten", Green),
                    b('\u{E7AD}', "Rotate ▾", Green),
                ],
            },
            RibbonGroup {
                name: "Upscale and resize",
                items: &[
                    b('\u{E9E3}', "Upscale 2x", Green),
                    b('\u{E794}', "AI Upscale", Purple),
                    b('\u{E8AD}', "Magic Resize", Green),
                ],
            },
            RibbonGroup {
                name: "Background",
                items: &[b('\u{E75C}', "Remove BG (AI)", Red), b('\u{E790}', "Remove by Color", Red)],
            },
            RibbonGroup {
                name: "Split",
                items: &[b('\u{E8C6}', "Split Selection", Green), b('\u{E8A9}', "Split Asset Sheet", Green)],
            },
        ],
    },
    RibbonTab {
        name: "Layer",
        groups: &[
            RibbonGroup {
                name: "Layers",
                items: &[
                    b('\u{E710}', "New Layer", Green),
                    b('\u{E8C8}', "Duplicate", Sky),
                    b('\u{E74D}', "Delete", Red),
                    b('\u{E8AC}', "Rename", Purple),
                    b('\u{E72E}', "Lock / Unlock", Amber),
                    b('\u{E8A9}', "Group / Ungroup", Purple),
                    b('\u{E8B9}', "Flatten Image", Purple),
                ],
            },
            RibbonGroup {
                name: "Mask and adjust",
                items: &[
                    b('\u{E7C4}', "Clip Below", Purple),
                    b('\u{E8BF}', "Layer Mask ▾", Purple),
                    b('\u{E793}', "Adjustment ▾", Purple),
                    b('\u{E7A8}', "Crop Layer ▾", Purple),
                ],
            },
        ],
    },
    RibbonTab {
        name: "Insert",
        groups: &[
            RibbonGroup {
                name: "New layer",
                items: &[
                    b('\u{E710}', "New Layer", Green),
                    b('\u{E8F4}', "New Page", Green),
                    b('\u{E790}', "Gradient Background", Pink),
                ],
            },
            RibbonGroup {
                name: "Image",
                items: &[
                    b('\u{E91B}', "Place Image", Green),
                    b('\u{E8B2}', "Frame ▾", Orange),
                    b('\u{E80A}', "Photo Grid", Green),
                    b('\u{E8A8}', "Fill Frames", Orange),
                ],
            },
            RibbonGroup {
                name: "Text",
                items: &[
                    b('\u{E8D2}', "Heading", Green),
                    b('\u{E97E}', "Subheading", Green),
                    b('\u{E97F}', "Body Text", Green),
                ],
            },
            RibbonGroup { name: "Shape", items: &[b('\u{E739}', "Shapes ▾", Orange)] },
            RibbonGroup {
                name: "Icons",
                items: &[
                    b('\u{E899}', "Icons", Orange),
                    b('\u{E735}', "Badges", Orange),
                    b('\u{E81E}', "Graphics", Orange),
                    b('\u{E8B3}', "SmartArt", Orange),
                ],
            },
            RibbonGroup { name: "Chart", items: &[b('\u{E9D2}', "Chart", Orange)] },
            RibbonGroup {
                name: "Codes",
                items: &[b('\u{E8A9}', "QR Code", Orange), b('\u{E8EC}', "Barcode ▾", Orange)],
            },
        ],
    },
    RibbonTab {
        name: "Adjust",
        groups: &[
            RibbonGroup {
                name: "Tone",
                items: &[
                    b('\u{E9A9}', "Auto Levels", Amber),
                    b('\u{E793}', "Brightness / Contrast", Amber),
                    b('\u{E706}', "Exposure / Gamma", Amber),
                ],
            },
            RibbonGroup {
                name: "Color",
                items: &[
                    b('\u{E790}', "Recolor", Pink),
                    b('\u{E7A1}', "Hue / Saturation", Amber),
                    b('\u{E734}', "Vibrance", Amber),
                    b('\u{E9CA}', "Temperature / Tint", Amber),
                ],
            },
            RibbonGroup {
                name: "Adjustment layers",
                items: &[
                    b('\u{E9E9}', "Levels Layer", Amber),
                    b('\u{E9D2}', "Curves Layer", Amber),
                    b('\u{E793}', "Adjustment Layer", Amber),
                ],
            },
            RibbonGroup {
                name: "Looks",
                items: &[
                    b('\u{E7A1}', "Grayscale", Amber),
                    b('\u{E8EB}', "Invert Colors", Pink),
                    b('\u{E81D}', "Sepia", Pink),
                    b('\u{E8B7}', "Posterize", Amber),
                    b('\u{E73D}', "Threshold", Amber),
                ],
            },
        ],
    },
    RibbonTab {
        name: "Filter",
        groups: &[
            RibbonGroup {
                name: "Clean up",
                items: &[
                    b('\u{E794}', "AI Upscale", Purple),
                    b('\u{E9A3}', "Enhance", Purple),
                    b('\u{E9E3}', "Upscale 2x", Pink),
                ],
            },
            RibbonGroup {
                name: "Blur and sharpen",
                items: &[
                    b('\u{E7B3}', "Blur", Pink),
                    b('\u{E768}', "Motion Blur", Pink),
                    b('\u{E7C6}', "Sharpen", Pink),
                ],
            },
            RibbonGroup {
                name: "Noise",
                items: &[b('\u{E8FB}', "Reduce Noise", Pink), b('\u{E75F}', "Add Noise", Green)],
            },
            RibbonGroup {
                name: "Stylize",
                items: &[
                    b('\u{E80A}', "Pixelate", Pink),
                    b('\u{E81D}', "Vignette", Pink),
                    b('\u{E7C8}', "Emboss", Pink),
                    b('\u{E70F}', "Find Edges", Pink),
                ],
            },
        ],
    },
    RibbonTab {
        name: "Transitions",
        groups: &[
            RibbonGroup {
                name: "Transition to this slide",
                items: &[
                    b('\u{E711}', "No Transition", Slate),
                    b('\u{E706}', "Fade", Sky),
                    b('\u{E72A}', "Push", Blue),
                    b('\u{E8AB}', "Wipe", Teal),
                    b('\u{E8A7}', "Cover", Green),
                    b('\u{E7C3}', "Uncover", Lime),
                    b('\u{E8A3}', "Zoom", Amber),
                    b('\u{E7B5}', "Split", Orange),
                ],
            },
            // Drawn by hand (`animation.rs`): the direction and duration boxes.
            RibbonGroup { name: "Effect options", items: &[] },
            RibbonGroup { name: "Advance slide", items: &[] },
            RibbonGroup {
                name: "Apply",
                items: &[b('\u{E8F4}', "Apply to All", Purple), b('\u{E768}', "Preview Transition", Green)],
            },
            RibbonGroup {
                name: "Slideshow",
                items: &[b('\u{E7F4}', "From Beginning", Red), b('\u{E768}', "From Current Slide", Red)],
            },
        ],
    },
    RibbonTab {
        name: "Animations",
        groups: &[
            // Drawn by hand (`animation.rs`): the list of effects to add.
            RibbonGroup { name: "Add animation", items: &[] },
            // Drawn by hand: start, duration, delay, direction of the chosen animation.
            RibbonGroup { name: "Animation timing", items: &[] },
            RibbonGroup {
                name: "Order",
                items: &[
                    b('\u{E70E}', "Move Earlier", Blue),
                    b('\u{E70D}', "Move Later", Blue),
                    b('\u{E74D}', "Remove Animation", Red),
                ],
            },
            RibbonGroup { name: "Preview", items: &[b('\u{E768}', "Preview Animations", Green)] },
            RibbonGroup {
                name: "Slideshow",
                items: &[b('\u{E7F4}', "From Beginning", Red), b('\u{E768}', "From Current Slide", Red)],
            },
        ],
    },
    RibbonTab {
        name: "View",
        groups: &[
            RibbonGroup {
                name: "Zoom",
                items: &[
                    b('\u{E8A3}', "Zoom In", Lime),
                    b('\u{E71F}', "Zoom Out", Lime),
                    b('\u{E740}', "Fit on Screen", Lime),
                    b('\u{E9A6}', "100%", Lime),
                ],
            },
            RibbonGroup {
                name: "Show",
                items: &[
                    t('\u{E70B}', "Rulers"),
                    t('\u{E80A}', "Grid"),
                    t('\u{E8B2}', "Guides"),
                    t('\u{E718}', "Snapping"),
                    b('\u{E894}', "Clear Guides", Red),
                ],
            },
            RibbonGroup {
                name: "Pictures",
                items: &[
                    b('\u{E72A}', "Next Picture", Lime),
                    b('\u{E80F}', "Projects", Lime),
                    b('\u{E711}', "Close Page", Red),
                ],
            },
            RibbonGroup {
                name: "Help",
                items: &[
                    b('\u{E82D}', "Tutorial", Lime),
                    b('\u{E768}', "Quick Tour", Lime),
                    b('\u{E765}', "Shortcuts", Sky),
                    b('\u{E946}', "About", Lime),
                ],
            },
        ],
    },
    RibbonTab {
        name: "Settings",
        groups: &[
            RibbonGroup {
                name: "Settings",
                items: &[
                    b('\u{E81C}', "Undo Steps", Blue),
                    b('\u{E790}', "Brand Kit", Slate),
                    b('\u{E8D4}', "AI Models", Purple),
                    b('\u{E790}', "Icon Style ▾", Slate),
                ],
            },
            RibbonGroup { name: "Keys", items: &[b('\u{E765}', "Shortcuts", Sky)] },
        ],
    },
];
