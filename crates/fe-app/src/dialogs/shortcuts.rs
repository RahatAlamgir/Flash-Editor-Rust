//! The keyboard shortcut list (C# `ShortcutsDialog`, F1): every shortcut in a list. Click one, press the new keys.

use egui::{Align, Event, Layout, RichText, vec2};
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::app::shortcuts::{self, Entry};

const DEFAULT_HINT: &str = "Click a shortcut, then press the new keys. Backspace removes it, Esc cancels.";

/// The window's state.
pub struct ShortcutsForm {
    entries: Vec<Entry>,
    /// The entry waiting for its new keys.
    capturing: Option<usize>,
    hint: String,
}

impl Default for ShortcutsForm {
    fn default() -> Self {
        Self { entries: shortcuts::entries(), capturing: None, hint: DEFAULT_HINT.into() }
    }
}

/// Shows the window; false when it was closed.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, f: &mut ShortcutsForm) -> bool {
    let mut open = true;
    // The keys pressed this frame while an entry waits for them.
    let pressed: Vec<(egui::Key, egui::Modifiers)> = if f.capturing.is_some() {
        ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| {
                    if let Event::Key { key, pressed: true, repeat: false, modifiers, .. } = e {
                        Some((*key, *modifiers))
                    } else {
                        None
                    }
                })
                .collect()
        })
    } else {
        Vec::new()
    };
    for (key, mods) in pressed {
        let Some(index) = f.capturing else { break };
        match key {
            egui::Key::Escape => {
                f.capturing = None;
                f.hint = DEFAULT_HINT.into();
            }
            egui::Key::Backspace => assign(app, f, index, ""),
            _ => match shortcuts::from_keys(&f.entries[index], key, mods) {
                Ok(text) => assign(app, f, index, &text),
                Err(problem) => f.hint = problem,
            },
        }
    }
    let mut start = None;
    let mut reset = None;
    egui::Window::new("Keyboard Shortcuts")
        .open(&mut open)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .frame(super::frame::titled_frame(ctx))
        .collapsible(false)
        .resizable(false)
        .default_width(560.0)
        .show(ctx, |ui| {
            ui.add(egui::Label::new(RichText::new(&f.hint).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height(520.0).show(ui, |ui| {
                let mut section = "";
                for (i, entry) in f.entries.iter().enumerate() {
                    if entry.section != section {
                        section = entry.section;
                        ui.add_space(10.0);
                        ui.label(RichText::new(section.to_uppercase()).size(11.0).strong().color(colors::SUBTLE_TEXT));
                    }
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(vec2(280.0, 22.0), Layout::left_to_right(Align::Center), |ui| {
                            ui.label(entry.title);
                        });
                        let waiting = f.capturing == Some(i);
                        let text = shortcuts::pretty(&shortcuts::current(&app.settings, entry));
                        let label = if waiting {
                            RichText::new("Press the new keys...").color(colors::ACCENT)
                        } else if text.is_empty() {
                            RichText::new("(none)").color(colors::SUBTLE_TEXT)
                        } else {
                            RichText::new(text).color(colors::TEXT)
                        };
                        if ui
                            .add(egui::Button::new(label).min_size(vec2(150.0, 22.0)))
                            .on_hover_text("Click, then press the new keys")
                            .clicked()
                        {
                            start = Some(i);
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if shortcuts::is_changed(&app.settings, entry)
                                && ui.small_button("\u{21BA}").on_hover_text("Back to the normal shortcut").clicked()
                            {
                                reset = Some(i);
                            }
                        });
                    });
                }
            });
            ui.add_space(8.0);
            if ui.button("Reset all").on_hover_text("Every shortcut back to the normal one").clicked() {
                app.settings.shortcuts.clear();
                crate::app::settings_sync::apply_tool_keys(app);
                f.capturing = None;
                f.hint = DEFAULT_HINT.into();
            }
        });
    if let Some(i) = start {
        f.capturing = Some(i);
        f.hint = if f.entries[i].tool.is_some() {
            "Press one key for this tool. Backspace removes the shortcut, Esc cancels.".into()
        } else {
            "Press Ctrl or Alt with a key (or an F key). Backspace removes the shortcut, Esc cancels.".into()
        };
    }
    if let Some(i) = reset {
        shortcuts::reset(&mut app.settings, &f.entries[i]);
        crate::app::settings_sync::apply_tool_keys(app);
        f.capturing = None;
    }
    open
}

fn assign(app: &mut FlashApp, f: &mut ShortcutsForm, index: usize, text: &str) {
    let taken = shortcuts::set(&mut app.settings, &f.entries[index], text);
    crate::app::settings_sync::apply_tool_keys(app);
    f.capturing = None;
    f.hint = taken.map_or_else(|| DEFAULT_HINT.to_owned(), |t| format!("\"{t}\" had this shortcut and has none now."));
}
