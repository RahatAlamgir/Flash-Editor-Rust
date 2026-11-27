//! A question window with buttons ("Save changes?", "Delete these pages?", "Bring back your work?"), like the
//! C# `MessageDialog`. The answer runs an [`Answer`].

use std::path::PathBuf;

use egui::RichText;
use fe_ui_kit::theme::colors;

/// What a button does.
#[derive(Clone, Debug, PartialEq)]
pub enum Answer {
    /// Just close the question.
    Nothing,
    SaveAndCloseProject(u64),
    CloseProject(u64),
    /// Save every unsaved project, then close the window.
    SaveAllAndQuit,
    /// Close the window without saving.
    Quit,
    RecoverWork,
    DiscardRecovery,
    DeletePages(Vec<usize>),
    DeleteProjectFile(PathBuf),
    /// Download the models an AI feature needs, then run the feature.
    DownloadModels(crate::ai::Pending),
}

/// A question with its buttons (first = the main button).
#[derive(Clone, Debug)]
pub struct Ask {
    pub title: String,
    pub text: String,
    pub buttons: Vec<(String, Answer)>,
}

impl Ask {
    pub fn new(title: &str, text: impl Into<String>, buttons: &[(&str, Answer)]) -> Self {
        Self {
            title: title.to_owned(),
            text: text.into(),
            buttons: buttons.iter().map(|(t, a)| ((*t).to_owned(), a.clone())).collect(),
        }
    }
}

/// Shows the question; returns the answer when a button was clicked (Esc = the last button).
pub fn show(ctx: &egui::Context, ask: &Ask) -> Option<Answer> {
    let mut answer = None;
    let modal = super::frame::modal(ctx, "ask", |ui| {
        ui.set_max_width(420.0);
        ui.label(RichText::new(&ask.title).heading().color(colors::TEXT));
        ui.add_space(8.0);
        ui.label(RichText::new(&ask.text).color(colors::TEXT));
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            for (index, (text, value)) in ask.buttons.iter().enumerate() {
                let button = egui::Button::new(RichText::new(text).size(12.5));
                let button = if index == 0 { button.fill(colors::ACCENT) } else { button };
                if ui.add(button.min_size(egui::vec2(90.0, 28.0))).clicked() {
                    answer = Some(value.clone());
                }
            }
        });
    });
    if answer.is_none() && modal.should_close() {
        answer = ask.buttons.last().map(|(_, a)| a.clone());
    }
    answer
}
