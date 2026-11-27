//! The slideshow: shows the pages of a presentation one after the other, playing each page's slide transition and its object
//! animations. F5 plays it full screen; "Preview Transition" / "Preview Animations" play one slide inside the window.
//! Click, Right, Space, Enter or Page Down go forward (the next animation, then the next page); Left, Backspace or Page Up
//! go back; Esc stops.

mod motion;
mod pages;
mod parts;

use egui::{Align2, Color32, Context, FontId, Key, Modifiers, Order, Rect, Sense, pos2, vec2};
use fe_core::document::{Transition, TransitionKind};

use crate::app::FlashApp;
use pages::PageShow;

/// What is on screen.
enum Phase {
    /// One page; `step` click-steps of its animations have started, the last one at `step_start`.
    Resting { page: usize, since: f64, step: usize, step_start: f64 },
    /// Moving from page `from` (black when none) to page `to`.
    Moving { from: Option<usize>, to: usize, start: f64 },
}

/// Why the slideshow is on screen.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// F5: full screen, everything.
    Slideshow,
    /// A transition played once inside the window.
    TransitionPreview,
    /// The animations of one slide played once inside the window.
    AnimationPreview,
}

pub struct Slideshow {
    pages: Vec<PageShow>,
    transitions: Vec<Transition>,
    phase: Phase,
    kind: Kind,
}

impl Slideshow {
    fn windowed(&self) -> bool {
        self.kind != Kind::Slideshow
    }
}

fn begin(app: &mut FlashApp, ctx: &Context, kind: Kind, from_current: bool) {
    let project = app.project();
    let current = project.active_index();
    let pages: Vec<PageShow> = project.pages().iter().enumerate().map(|(i, p)| pages::build(ctx, i, p)).collect();
    let transitions = project.pages().iter().map(|p| p.transition).collect();
    let now = ctx.input(|i| i.time);
    let phase = match kind {
        // A transition preview moves from the page before (or from black) to the shown page.
        Kind::TransitionPreview => Phase::Moving { from: current.checked_sub(1), to: current, start: now },
        Kind::AnimationPreview => Phase::Resting { page: current, since: now, step: 0, step_start: now },
        Kind::Slideshow => Phase::Moving { from: None, to: if from_current { current } else { 0 }, start: now },
    };
    if kind == Kind::Slideshow {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
    }
    app.slideshow = Some(Slideshow { pages, transitions, phase, kind });
    ctx.request_repaint();
}

/// Starts the slideshow of the shown project. `from_current`: start at the shown page. `preview`: only the transition of the
/// shown page, inside the window.
pub fn start(app: &mut FlashApp, ctx: &Context, from_current: bool, preview: bool) {
    begin(app, ctx, if preview { Kind::TransitionPreview } else { Kind::Slideshow }, from_current);
}

/// Plays the animations of the shown slide once, inside the window.
pub fn start_animation_preview(app: &mut FlashApp, ctx: &Context) {
    begin(app, ctx, Kind::AnimationPreview, true);
}

fn stop(app: &mut FlashApp, ctx: &Context) {
    if app.slideshow.as_ref().is_some_and(|s| s.kind == Kind::Slideshow) {
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
        let area = if show.windowed() {
            ui.painter().text(
                pos2(rect.center().x, rect.bottom() - 18.0),
                Align2::CENTER_CENTER,
                "Preview. Click or press Esc to close.",
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
        if show.windowed() {
            quit = true;
        } else {
            next = true;
        }
    }

    match show.phase {
        Phase::Moving { to, start, .. } => {
            // A click or key skips the rest of the movement.
            if next || now - start >= length(&show.transitions[to]) {
                show.phase = Phase::Resting { page: to, since: now, step: 0, step_start: now };
            }
            ctx.request_repaint();
        }
        Phase::Resting { page, since, step, step_start } => {
            let count = show.pages.len();
            let clicks = show.pages[page].timeline.clicks;
            let automatic = show.transitions[page].after.is_some_and(|s| now - since >= f64::from(s));
            ctx.request_repaint();
            match show.kind {
                Kind::TransitionPreview => {
                    // Closes a moment after the transition ended.
                    quit |= now - since > 0.6;
                }
                Kind::AnimationPreview => {
                    // Plays every step by itself, a short pause after each one.
                    let length = f64::from(show.pages[page].timeline.step_length(step));
                    if now - step_start >= length + 0.5 {
                        if step < clicks {
                            show.phase = Phase::Resting { page, since, step: step + 1, step_start: now };
                        } else {
                            quit = true;
                        }
                    }
                }
                Kind::Slideshow => {
                    if next && step < clicks {
                        show.phase = Phase::Resting { page, since, step: step + 1, step_start: now };
                    } else if next || automatic {
                        if page + 1 < count {
                            show.phase = Phase::Moving { from: Some(page), to: page + 1, start: now };
                        } else {
                            quit = true; // after the last page
                        }
                    } else if back && page > 0 {
                        // Going back shows the earlier page as it ended, without its movement.
                        let before = page - 1;
                        let done = show.pages[before].timeline.clicks;
                        show.phase =
                            Phase::Resting { page: before, since: now, step: done + 1, step_start: now - 3600.0 };
                    }
                }
            }
        }
    }
    app.slideshow = Some(show);
    if quit {
        stop(app, ctx);
    }
}

fn draw(ui: &egui::Ui, area: Rect, show: &Slideshow, now: f64) {
    match show.phase {
        Phase::Moving { from, to, start } => {
            let transition = show.transitions[to];
            let seconds = length(&transition);
            let progress = if seconds <= 0.0 { 1.0 } else { (((now - start) / seconds) as f32).clamp(0.0, 1.0) };
            let new = &show.pages[to].initial;
            let page = fit(area, new.size_vec2());
            let old = from.map(|i| show.pages[i].finished.as_ref().unwrap_or(&show.pages[i].initial));
            motion::paint(ui.painter(), page, old, new, &transition, ease(progress));
        }
        Phase::Resting { page, step, step_start, .. } => {
            let shown = &show.pages[page];
            let rect = fit(area, shown.initial.size_vec2());
            parts::paint(ui.painter(), rect, shown, step, (now - step_start) as f32);
        }
    }
}

fn fit(area: Rect, size: egui::Vec2) -> Rect {
    let scale = (area.width() / size.x).min(area.height() / size.y);
    Rect::from_center_size(area.center(), size * scale)
}

/// Smooth start and end.
fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
