//! Sizes in points, taken from the C# XAML (MainWindow, Ribbon, Styles, panels). One place to tune the layout.

/// Ribbon: height of the row with the tab names.
pub const RIBBON_TAB_ROW: f32 = 27.0;
/// Ribbon: left padding of the tab row (room for the File button).
pub const RIBBON_TABS_LEFT: f32 = 74.0;
/// Ribbon: height of the button area under the tabs.
pub const RIBBON_BODY: f32 = 62.0;
/// Ribbon: smallest width of a big button.
pub const RIBBON_BUTTON_MIN_W: f32 = 46.0;
/// Ribbon: widest label before it wraps.
pub const RIBBON_LABEL_MAX_W: f32 = 74.0;
/// Ribbon: size of the icon glyph in a big button.
pub const RIBBON_GLYPH: f32 = 20.0;
/// Ribbon: search box width.
pub const SEARCH_W: f32 = 240.0;

/// The bar under the ribbon with the tool name and its hint.
pub const HINT_BAR: f32 = 34.0;
/// Status bar height.
pub const STATUS_BAR: f32 = 24.0;
/// Toolbox width.
pub const TOOLBOX_W: f32 = 52.0;
/// One tool button.
pub const TOOL_BUTTON: [f32; 2] = [36.0, 34.0];
/// Right column width.
pub const RIGHT_COLUMN_W: f32 = 290.0;
/// Panel title bar height (PROPERTIES, LAYERS, HISTORY).
pub const PANEL_TITLE: f32 = 28.0;
/// Project tab height.
pub const PROJECT_TAB: f32 = 34.0;
/// Page strip height under the canvas.
pub const PAGE_STRIP: f32 = 128.0;
/// Page card width and picture height.
pub const PAGE_CARD: [f32; 2] = [140.0, 84.0];
/// One row in the layer list.
pub const LAYER_ROW: f32 = 36.0;
/// Layer thumbnail.
pub const LAYER_THUMB: [f32; 2] = [42.0, 28.0];
/// Line icons are drawn on a 20 x 20 grid.
pub const ICON: f32 = 20.0;
/// Small icon button (padding 4 around a 20 px icon).
pub const ICON_BUTTON: f32 = 28.0;
/// Corner radius of buttons.
pub const RADIUS: u8 = 4;
/// Labels in the Properties / Layers panels.
pub const INSPECTOR_LABEL_W: f32 = 64.0;
