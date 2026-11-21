//! What the mouse did over the canvas this frame, in canvas pixels (C# `ToolMouseArgs`). The canvas builds it;
//! the tools read it.

use egui::{Modifiers, PointerButton, Pos2, Response, Ui};
use fe_core::imaging::PointD;
use fe_gpu::ViewTransform;

/// The mouse over the canvas.
#[derive(Clone, Copy, Debug)]
pub struct Pointer {
    /// Where the mouse is, in canvas pixels (fractional).
    pub pos: PointD,
    /// The left button went down this frame over the canvas.
    pub pressed: bool,
    /// The left button went up this frame.
    pub released: bool,
    /// The left button is held.
    pub down: bool,
    /// The right button went down this frame over the canvas.
    pub secondary_pressed: bool,
    /// The second click of a double-click ended this frame.
    pub double_click: bool,
    pub mods: Modifiers,
    /// Screen points per canvas pixel.
    pub zoom: f64,
}

impl Pointer {
    /// Reads the mouse for this frame. `None` when the mouse position is unknown.
    pub fn read(ui: &Ui, response: &Response, transform: ViewTransform) -> Option<Self> {
        let hovered = response.hovered();
        ui.input(|i| {
            let screen: Pos2 = i.pointer.latest_pos()?;
            let c = transform.to_canvas(screen);
            Some(Self {
                pos: PointD::new(c.x as f64, c.y as f64),
                pressed: hovered && i.pointer.primary_pressed(),
                released: i.pointer.primary_released(),
                down: i.pointer.primary_down(),
                secondary_pressed: hovered && i.pointer.secondary_pressed(),
                double_click: hovered && i.pointer.button_double_clicked(PointerButton::Primary),
                mods: i.modifiers,
                zoom: transform.zoom as f64,
            })
        })
    }

    /// The whole pixel under the mouse.
    pub fn pixel(&self) -> (i32, i32) {
        (self.pos.x.floor() as i32, self.pos.y.floor() as i32)
    }
}
