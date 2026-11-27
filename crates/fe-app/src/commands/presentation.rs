//! Presentations: the Transitions and Animations tabs (slide transitions, object animations) and the slideshow.

use fe_core::document::animation::{Animation, Effect};
use fe_core::document::{Transition, TransitionKind};

use crate::app::FlashApp;
use crate::panels::ribbon::animation_kind_of;

/// Copies what the tabs show (is this a presentation, the transition of the shown page, the chosen animation) into the ribbon.
pub fn prepare_ribbon(app: &mut FlashApp) {
    let project = app.project();
    let page = project.pages().get(project.active_index());
    let presentation = project.presentation;
    let transition = page.map(|p| p.transition).unwrap_or_default();
    let selected = app.ribbon.animation_selected;
    let animation = page.and_then(|p| selected.and_then(|i| p.animations.get(i).copied()));
    let animation = animation.filter(|a| page.is_some_and(|p| p.doc.index_of(a.layer).is_some()));
    app.ribbon.presentation = presentation;
    app.ribbon.transition = transition;
    app.ribbon.transition_changed = false;
    app.ribbon.animation = animation;
    app.ribbon.animation_changed = false;
    app.ribbon.add_effect = None;
}

/// The boxes of the tabs were edited: the shown page gets the change.
pub fn apply_ribbon_edit(app: &mut FlashApp) {
    if std::mem::take(&mut app.ribbon.transition_changed) {
        let transition = app.ribbon.transition;
        set_transition(app, transition);
    }
    if std::mem::take(&mut app.ribbon.animation_changed)
        && let (Some(a), Some(i)) = (app.ribbon.animation, app.ribbon.animation_selected)
    {
        edit_animations(app, "Animation Timing", |list| {
            if let Some(slot) = list.get_mut(i) {
                *slot = a;
            }
        });
    }
    if let Some(effect) = app.ribbon.add_effect.take() {
        add_animation(app, effect);
    }
}

fn set_transition(app: &mut FlashApp, transition: Transition) {
    app.project_mut().edit_pages("Slide Transition", |pages, active| {
        if let Some(page) = pages.get_mut(*active) {
            page.transition = transition;
        }
    });
}

/// Changes the animation list of the shown page as one undo step.
pub fn edit_animations(app: &mut FlashApp, name: &str, change: impl FnOnce(&mut Vec<Animation>)) {
    app.project_mut().edit_pages(name, |pages, active| {
        if let Some(page) = pages.get_mut(*active) {
            change(&mut page.animations);
        }
    });
}

/// "Add Animation": the effect goes on the selected layer.
fn add_animation(app: &mut FlashApp, effect: Effect) {
    let Some(layer) =
        app.project().pages().get(app.project().active_index()).and_then(|p| p.doc.active_layer()).map(|l| l.id())
    else {
        app.status = "Select a layer first, then add an animation to it.".into();
        return;
    };
    let mut position = 0;
    edit_animations(app, "Add Animation", |list| {
        list.push(Animation::new(layer, effect));
        position = list.len() - 1;
    });
    app.ribbon.animation_selected = Some(position);
    app.ribbon.animation_pane = true;
    app.status = format!("Added {} to the selected layer.", effect.name());
}

/// Removes, or moves one place, the chosen animation.
fn move_selected(app: &mut FlashApp, by: isize) {
    let Some(i) = app.ribbon.animation_selected else { return };
    let mut now = i;
    edit_animations(app, "Reorder Animations", |list| {
        let to = i as isize + by;
        if i < list.len() && to >= 0 && (to as usize) < list.len() {
            list.swap(i, to as usize);
            now = to as usize;
        }
    });
    app.ribbon.animation_selected = Some(now);
}

pub fn remove(app: &mut FlashApp, index: usize) {
    edit_animations(app, "Remove Animation", |list| {
        if index < list.len() {
            list.remove(index);
        }
    });
    app.ribbon.animation_selected = None;
}

/// A click on a button of the tabs. False when the label is not one of them.
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
        "Preview Animations" => crate::slideshow::start_animation_preview(app, ctx),
        "From Beginning" => slideshow(app, ctx, false),
        "From Current Slide" => slideshow(app, ctx, true),
        "Move Earlier" => move_selected(app, -1),
        "Move Later" => move_selected(app, 1),
        "Remove Animation" => {
            if let Some(i) = app.ribbon.animation_selected {
                remove(app, i);
            }
        }
        "Animation Pane" => app.ribbon.animation_pane = !app.ribbon.animation_pane,
        _ => return false,
    }
    true
}

/// F5 / Ctrl+F5: plays the slideshow.
pub fn slideshow(app: &mut FlashApp, ctx: &egui::Context, from_current: bool) {
    crate::slideshow::start(app, ctx, from_current, false);
}
