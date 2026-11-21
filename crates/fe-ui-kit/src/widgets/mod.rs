//! Reusable widgets that copy the look of the C# controls.

mod checker;
mod icon_button;
mod number_slider;
mod panel_title;
mod popup;

pub use popup::popup_below;
mod ribbon_button;
mod slider;
mod tool_button;

pub use slider::{slider, slider_size};

pub use checker::paint_checkerboard;
pub use icon_button::{aid_toggle, icon_button, icon_button_enabled};
pub use number_slider::number_slider;
pub use panel_title::{panel_title_bar, section_title};
pub use ribbon_button::{RibbonButton, ribbon_button};
pub use tool_button::{ToolButtonResponse, tool_button};
