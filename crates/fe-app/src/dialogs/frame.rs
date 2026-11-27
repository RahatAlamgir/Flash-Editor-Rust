//! The look every dialog shares: a rounded window with room around its content (not touching the border) that can be
//! dragged with the mouse by any empty part of it. `modal` also dims the window behind and blocks it; `floating` leaves
//! the picture visible (for the settings windows with a live preview).

use egui::{Align2, Color32, Context, Frame, Id, Key, Margin, Order, Sense, Ui, Vec2};

/// The room between the border of a dialog and what is in it.
pub const PADDING: i8 = 22;

/// What a dialog says about closing: Esc was pressed.
pub struct Shown {
    escape: bool,
}

impl Shown {
    pub fn should_close(&self) -> bool {
        self.escape
    }
}

fn window_frame(ctx: &Context) -> Frame {
    Frame::window(&ctx.global_style()).inner_margin(Margin::same(PADDING)).corner_radius(8)
}

fn escape(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::Escape))
}

/// A dialog in the middle of the window, over a dimmed backdrop. `name` keeps its place when it was moved.
pub fn modal(ctx: &Context, name: &str, add: impl FnOnce(&mut Ui)) -> Shown {
    modal_at(ctx, name, (Order::Middle, Order::Foreground), add)
}

/// Like `modal`, for a dialog opened from another dialog (above it, dimming it).
pub fn modal_over(ctx: &Context, name: &str, add: impl FnOnce(&mut Ui)) -> Shown {
    modal_at(ctx, name, (Order::Foreground, Order::Tooltip), add)
}

fn modal_at(ctx: &Context, name: &str, (dim, window): (Order, Order), add: impl FnOnce(&mut Ui)) -> Shown {
    let screen = ctx.content_rect();
    egui::Area::new(Id::NULL.with(("dialog-dim", name))).order(dim).fixed_pos(screen.min).show(ctx, |ui| {
        ui.allocate_rect(screen, Sense::click_and_drag()); // nothing behind the dialog gets the mouse
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(120));
    });
    egui::Window::new(name)
        .id(Id::NULL.with(("dialog", name)))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .order(window)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(screen.center())
        .frame(window_frame(ctx))
        .show(ctx, add);
    Shown { escape: escape(ctx) }
}

/// A dialog that leaves the picture visible: starts near the bottom right (`offset` from that corner).
pub fn floating(ctx: &Context, name: &str, offset: Vec2, add: impl FnOnce(&mut Ui)) -> Shown {
    let screen = ctx.content_rect();
    egui::Window::new(name)
        .id(Id::NULL.with(("dialog", name)))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .order(Order::Foreground)
        .pivot(Align2::RIGHT_BOTTOM)
        .default_pos(screen.right_bottom() + offset)
        .frame(window_frame(ctx))
        .show(ctx, add);
    Shown { escape: escape(ctx) }
}

/// The frame for the ordinary (non-modal) windows with a title bar, such as AI Models and the shortcut list.
pub fn titled_frame(ctx: &Context) -> Frame {
    window_frame(ctx)
}
