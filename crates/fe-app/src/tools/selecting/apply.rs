//! The rule all selection tools share (C# `SelectionCombiner`): Shift adds to the selection, Alt subtracts.

use egui::Modifiers;
use fe_core::selection::{CombineMode, Selection};

use crate::app::FlashApp;

/// The combine mode for the keys held when the selection was started.
pub fn mode_for(mods: Modifiers) -> CombineMode {
    if mods.shift {
        CombineMode::Add
    } else if mods.alt {
        CombineMode::Subtract
    } else {
        CombineMode::Replace
    }
}

/// Puts a newly drawn selection on the page (combined with the old one) and says so in the status bar.
/// Selecting is not an undo step, like in C#.
pub fn apply(app: &mut FlashApp, drawn: Option<Selection>, mods: Modifiers) {
    let mode = mode_for(mods);
    let doc = &mut app.project_mut().active_page_mut().doc;
    let combined = Selection::combine(doc.selected_area(), drawn.as_ref(), mode);
    doc.set_selected_area(combined);
    app.status = match app.doc().selected_area() {
        None => "Selection cleared".into(),
        Some(s) => {
            let verb = match mode {
                CombineMode::Add => "Added to selection",
                CombineMode::Subtract => "Removed from selection",
                CombineMode::Replace => "Selected",
            };
            let b = s.bounds();
            format!("{verb} ({} x {} px). Shift = add, Alt = subtract.", b.width, b.height)
        }
    };
}

/// Clears the selection (no undo step).
pub fn deselect(app: &mut FlashApp) {
    app.project_mut().active_page_mut().doc.set_selected_area(None);
}
