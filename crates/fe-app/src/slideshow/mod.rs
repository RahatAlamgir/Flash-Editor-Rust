//! The slideshow: shows the pages of a presentation one after the other, playing each page's slide transition.
//! F5 plays it full screen; "Preview Transition" plays one transition inside the window. Click, Right, Space, Enter or
//! Page Down go forward; Left, Backspace or Page Up go back; Esc stops.

mod motion;

use egui::{
    Align2, Color32, ColorImage, Context, FontId, Key, Modifiers, Order, Rect, Sense, TextureHandle, TextureOptions,
    pos2, vec2,
};
use fe_core::document::{Transition, TransitionKind};
use fe_core::paint::compositor;

use crate::app::FlashApp;

/// What is on screen.
enum Phase {
    /// One page, waiting for a click (or its automatic time).
    Resting { page: usize, since: f64 },
    /// Moving from page `from` (black when none) to page `to`.
    Moving { from: Option<usize>, to: usize, start: f64 },
}

pub struct Slideshow {
    pictures: Vec<TextureHandle>,
    transitions: Vec<Transition>,
    phase: Phase,
    /// Full screen (F5) or inside the window (preview).
    fullscreen: bool,
    /// A preview ends by itself a moment after its transition.
    preview: bool,
}

fn texture_of(ctx: &Context, name: usize, doc: &fe_core::document::Document) -> TextureHandle {
    let flat = compositor::flatten(doc);
    let image = ColorImage::from_rgba_unmultiplied([flat.width() as usize, flat.height() as usize], flat.data());
    ctx.load_texture(format!("slideshow-{name}"), image, TextureOptions::LINEAR)
}

/// Starts the slideshow of the shown project. `from_current`: start at the shown page. `preview`: only the transition of the
/// shown page, inside the window.
pub fn start(app: &mut FlashApp, ctx: &Context, from_current: bool, preview: bool) {
    let project = app.project();
    let pages = project.pages();
    let current = project.active_index();
    let pictures: Vec<TextureHandle> = pages.iter().enumerate().map(|(i, p)| texture_of(ctx, i, &p.doc)).collect();
    let transitions = pages.iter().map(|p| p.transition).collect();
    let now = ctx.input(|i| i.time);
    // A preview moves from the page before (or from black) to the shown page; a slideshow starts on its first page.
    let phase = if preview {
        Phase::Moving { from: current.checked_sub(1), to: current, start: now }
    } else {
        Phase::Moving { from: None, to: if from_current { current } else { 0 }, start: now }
    };
    if !preview {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
    }
    app.slideshow = Some(Slideshow { pictures, transitions, phase, fullscreen: !preview, preview });
    ctx.request_repaint();
}

fn stop(app: &mut FlashApp, ctx: &Context) {
    if app.slideshow.as_ref().is_some_and(|s| s.fullscreen) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    }
    app.slideshow = None;
}

/// True while a slideshow or preview is on screen (the editor keys are then not used).
pub fn active(app: &FlashApp) -> bool {
    app.slideshow.is_some()
}

/// How long a page's movement lasts.
fn length(t: &Transition) -> f64 {
    if t.kind == TransitionKind::None { 0.0 } else { f64::from(t.duration) }
}

/// Draws the slideshow over the window and handles its keys and clicks. Call every frame.
pub fn show(ctx: &Context, app: &mut FlashApp) {
    let Some(mut show) = app.slideshow.take() else { return };
    let now = ctx.input(|i| i.time);
    let screen = ctx.content_rect();
    let (mut next, mut back, mut quit) = (false, false, false);
    ctx.input_mut(|i| {
        for key in [Key::ArrowRight, Key::Space, Key::Enter, Key::PageDown, Key::ArrowDown] {
            next |= i.consume_key(Modifiers::NONE, key);
        }
        for key in [Key::ArrowLeft, Key::Backspace, Key::PageUp, Key::ArrowUp] {
            back |= i.consume_key(Modifiers::NONE, key);
        }
        quit |= i.consume_key(Modifiers::NONE, Key::Escape);
    });
    let mut clicked = false;
    egui::Area::new(egui::Id::NULL.with("slideshow")).order(Order::Debug).fixed_pos(screen.min).show(ctx, |ui| {
        let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
        ui.painter().rect_filled(rect, 0, Color32::BLACK);
        clicked = response.clicked();
        let area = if show.preview {
            ui.painter().text(
                pos2(rect.center().x, rect.bottom() - 18.0),
                Align2::CENTER_CENTER,
                "Preview of the transition. Click or press Esc to close.",
                FontId::proportional(13.0),
                Color32::from_gray(170),
            );
            rect.shrink2(vec2(rect.width() * 0.12, rect.height() * 0.1))
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            rect
        };
        draw(ui, area, &show, now);
    });
    back |= ctx.input(|i| i.pointer.secondary_clicked());
    if clicked {
        if show.preview {
            quit = true;
        } else {
            next = true;
        }
    }

    match show.phase {
        Phase::Moving { to, start, .. } => {
            // A click or key skips the rest of the movement.
            if next || now - start >= length(&show.transitions[to]) {
                show.phase = Phase::Resting { page: to, since: now };
            }
            ctx.request_repaint();
        }
        Phase::Resting { page, since } => {
            let count = show.pictures.len();
            let automatic = show.transitions[page].after.is_some_and(|s| now - since >= f64::from(s));
            if show.preview {
                // The preview closes a moment after its transition ended.
                if now - since > 0.6 {
                    quit = true;
                }
                ctx.request_repaint();
            } else if next || automatic {
                if page + 1 < count {
                    show.phase = Phase::Moving { from: Some(page), to: page + 1, start: now };
                } else {
                    quit = true; // after the last page
                }
            } else if back && page > 0 {
                show.phase = Phase::Moving { from: Some(page), to: page - 1, start: now };
            }
            if let Some(seconds) = show.transitions[page].after.filter(|_| !show.preview) {
                let left = f64::from(seconds) - (now - since);
                ctx.request_repaint_after(std::time::Duration::from_secs_f64(left.max(0.02)));
            }
        }
    }
    app.slideshow = Some(show);
    if quit {
        stop(app, ctx);
    }
}

fn draw(ui: &egui::Ui, area: Rect, show: &Slideshow, now: f64) {
    let (from, to, progress, transition) = match show.phase {
        Phase::Resting { page, .. } => (None, page, 1.0, Transition::default()),
        Phase::Moving { from, to, start } => {
            let t = show.transitions[to];
            let seconds = length(&t);
            let p = if seconds <= 0.0 { 1.0 } else { (((now - start) / seconds) as f32).clamp(0.0, 1.0) };
            (from, to, p, t)
        }
    };
    let size = show.pictures[to].size_vec2();
    let scale = (area.width() / size.x).min(area.height() / size.y);
    let page = Rect::from_center_size(area.center(), size * scale);
    let old = from.map(|i| &show.pictures[i]);
    motion::paint(ui.painter(), page, old, &show.pictures[to], &transition, ease(progress));
}

/// Smooth start and end.
fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
