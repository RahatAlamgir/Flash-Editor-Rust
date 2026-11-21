//! On / off settings for the helpers drawn over the canvas (C# `ViewAids`). Saved with the settings in Phase 5.

/// The grid spacings offered in the grid settings (C# `ViewAids.GridSizes`).
pub const GRID_SIZES: [f64; 8] = [5.0, 10.0, 20.0, 25.0, 40.0, 50.0, 100.0, 200.0];

/// Rulers, grid, guides and snapping.
#[derive(Clone, Debug)]
pub struct ViewAids {
    pub rulers: bool,
    pub grid: bool,
    pub guides: bool,
    /// The master switch: off = nothing snaps.
    pub snapping: bool,
    /// Distance between grid lines in pixels.
    pub grid_size: f64,
    pub snap_to_objects: bool,
    pub snap_to_canvas: bool,
    pub snap_to_spacing: bool,
    pub snap_to_grid: bool,
    pub snap_to_guides: bool,
}

impl Default for ViewAids {
    /// Same defaults as the C# version: guides and snapping on, rulers and grid off, grid every 50 px.
    fn default() -> Self {
        Self {
            rulers: false,
            grid: false,
            guides: true,
            snapping: true,
            grid_size: 50.0,
            snap_to_objects: true,
            snap_to_canvas: true,
            snap_to_spacing: true,
            snap_to_grid: true,
            snap_to_guides: true,
        }
    }
}
