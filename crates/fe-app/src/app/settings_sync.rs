//! Keeps the settings file in step with the app: applies it at start-up, and writes it whenever a remembered
//! choice changes (undo steps, icon style, rulers / grid / guides / snapping, projects folder, recent list).

use super::FlashApp;

/// Applies the loaded settings to the app.
pub fn apply(app: &mut FlashApp) {
    let s = &app.settings;
    app.aids.rulers = s.show_rulers;
    app.aids.grid = s.show_grid;
    app.aids.guides = s.show_guides;
    app.aids.snapping = s.snap_enabled;
    app.aids.grid_size = s.grid_size;
    app.aids.snap_to_objects = s.snap_to_objects;
    app.aids.snap_to_canvas = s.snap_to_canvas;
    app.aids.snap_to_spacing = s.snap_to_spacing;
    app.aids.snap_to_grid = s.snap_to_grid;
    app.aids.snap_to_guides = s.snap_to_guides;
    app.ribbon.colorful = s.icon_style == "Colorful";
    let steps = s.undo_steps;
    for project in &mut app.projects {
        project.history_mut().set_max_steps(steps);
    }
}

/// Copies the app's choices into the settings and writes the file when something changed.
pub fn save_if_changed(app: &mut FlashApp) {
    let a = &app.aids;
    let s = &mut app.settings;
    s.show_rulers = a.rulers;
    s.show_grid = a.grid;
    s.show_guides = a.guides;
    s.snap_enabled = a.snapping;
    s.grid_size = a.grid_size;
    s.snap_to_objects = a.snap_to_objects;
    s.snap_to_canvas = a.snap_to_canvas;
    s.snap_to_spacing = a.snap_to_spacing;
    s.snap_to_grid = a.snap_to_grid;
    s.snap_to_guides = a.snap_to_guides;
    s.icon_style = if app.ribbon.colorful { "Colorful" } else { "Plain" }.into();
    s.undo_steps = app.projects.first().map_or(s.undo_steps, |p| p.history().max_steps());
    if app.settings != app.saved_settings {
        app.settings.save();
        app.saved_settings = app.settings.clone();
    }
}
