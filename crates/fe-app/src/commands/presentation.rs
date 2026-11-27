//! Presentations: the Animation tab (slide transitions) and the slideshow.

use fe_core::document::{Transition, TransitionKind};

use crate::app::FlashApp;
use crate::panels::ribbon::animation_kind_of;

/// Copies what the Animation tab shows (is this a presentation, the transition of the shown page) into the ribbon.
pub fn prepare_ribbon(app: &mut FlashApp) {
    let project = app.project();
    let (presentation, transition) =
        (project.presentation, project.pages().get(project.active_index()).map(|p| p.transition).unwrap_or_default());
    app.ribbon.presentation = presentation;
    app.ribbon.transition = transition;
    app.ribbon.transition_changed = false;
}

/// The boxes of the Animation tab were edited: the shown page gets the new transition.
pub fn apply_ribbon_edit(app: &mut FlashApp) {
    if std::mem::take(&mut app.ribbon.transition_changed) {
        let transition = app.ribbon.transition;
        set_transition(app, transition);
    }
}

fn set_transition(app: &mut FlashApp, transition: Transition) {
    app.project_mut().edit_pages("Slide Transition", |pages, active| {
        if let Some(page) = pages.get_mut(*active) {
            page.transition = transition;
        }
    });
}

/// A click on a button of the Animation tab. False when the label is not one of them.
pub fn ribbon_button(app: &mut FlashApp, ctx: &egui::Context, label: &str) -> bool {
    if let Some(kind) = animation_kind_of(label) {
        let mut transition = app.ribbon.transition;
        transition.kind = kind;
        set_transition(app, transition);
        // The movement is shown at once, so the choice can be judged.
        if kind != TransitionKind::None {
            crate::slideshow::start(app, ctx, true, true);
        }
        return true;
    }
    match label {
        "Apply to All" => {
            let transition = app.ribbon.transition;
            app.project_mut().edit_pages("Apply Transition to All", |pages, _| {
                pages.iter_mut().for_each(|p| p.transition = transition);
            });
            app.status = "Every slide now has this transition.".into();
        }
        "Preview Transition" => crate::slideshow::start(app, ctx, true, true),
        "From Beginning" => slideshow(app, ctx, false),
        "From Current Slide" => slideshow(app, ctx, true),
        _ => return false,
    }
    true
}

/// F5 / Ctrl+F5: plays the slideshow.
pub fn slideshow(app: &mut FlashApp, ctx: &egui::Context, from_current: bool) {
    crate::slideshow::start(app, ctx, from_current, false);
}
