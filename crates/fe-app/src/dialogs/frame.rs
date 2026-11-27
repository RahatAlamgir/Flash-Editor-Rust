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
    modal_at(ctx, name, (Order::Middle, Order::Foreground), 120, add)
}

/// The space around the content of `fullscreen`.
pub const LAUNCHER_MARGIN: (f32, f32) = (24.0, 20.0);

/// Content that fills the whole window (the start-up Project Manager, which has its own small window).
pub fn fullscreen(ctx: &Context, name: &str, add: impl FnOnce(&mut Ui)) -> Shown {
    let screen = ctx.content_rect();
    egui::Area::new(Id::NULL.with(("fullscreen", name))).order(Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
        let margin = Margin::symmetric(LAUNCHER_MARGIN.0 as i8, LAUNCHER_MARGIN.1 as i8);
        Frame::new().fill(ctx.global_style().visuals.window_fill).inner_margin(margin).show(ui, |ui| {
            ui.set_min_size(screen.size() - Vec2::new(2.0 * LAUNCHER_MARGIN.0, 2.0 * LAUNCHER_MARGIN.1));
            add(ui);
        });
    });
    Shown { escape: escape(ctx) }
}

/// Like `modal`, for a dialog opened from another dialog (above it, dimming it).
pub fn modal_over(ctx: &Context, name: &str, add: impl FnOnce(&mut Ui)) -> Shown {
    modal_at(ctx, name, (Order::Foreground, Order::Tooltip), 120, add)
}

fn modal_at(ctx: &Context, name: &str, (dim, window): (Order, Order), alpha: u8, add: impl FnOnce(&mut Ui)) -> Shown {
    let screen = ctx.content_rect();
    egui::Area::new(Id::NULL.with(("dialog-dim", name))).order(dim).fixed_pos(screen.min).show(ctx, |ui| {
        ui.allocate_rect(screen, Sense::click_and_drag()); // nothing behind the dialog gets the mouse
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(alpha));
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
