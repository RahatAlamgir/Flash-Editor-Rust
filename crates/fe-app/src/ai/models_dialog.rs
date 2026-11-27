//! The AI Models window (C# `AiModelsWindow`): every model with its size, whether it is on this PC, and buttons to
//! download it, delete it or open the folder. It is an ordinary window (not modal), so the progress bar and Cancel in the
//! status bar stay usable while a model downloads.

use egui::{RichText, vec2};
use fe_ai::models::{self, ModelInfo};
use fe_ui_kit::theme::colors;

use super::pending;
use crate::app::FlashApp;

enum Click {
    Download(&'static ModelInfo),
    Delete(&'static ModelInfo),
    ShowFolder,
}

fn megabytes(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
}

/// Shows the window; false when it was closed.
pub fn show(ctx: &egui::Context, app: &mut FlashApp) -> bool {
    let mut open = true;
    let mut click = None;
    let busy = app.ai.job.is_some();
    let folder = app.ai.store.folder().display().to_string();
    egui::Window::new("AI Models").pivot(egui::Align2::CENTER_CENTER).default_pos(ctx.content_rect().center()).frame(crate::dialogs::frame::titled_frame(ctx)).open(&mut open).collapsible(false).resizable(false).default_width(520.0).show(ctx, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(
                    "AI features need these models. Each is downloaded once, after asking, and works without internet afterwards.",
                )
                .size(11.5)
                .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(6.0);
        for model in models::all() {
            let info = models::describe(model);
            let installed = app.ai.store.is_installed(model);
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(350.0);
                    ui.label(RichText::new(info.title).strong().color(colors::TEXT));
                    ui.label(RichText::new(info.used_for).size(11.5).color(colors::SUBTLE_TEXT));
                    let state = if installed {
                        format!("Installed ({} on disk)", megabytes(app.ai.store.bytes_on_disk(model)))
                    } else {
                        format!("Not installed (download: {})", model.size_text())
                    };
                    ui.label(RichText::new(state).size(11.5).color(if installed { colors::TEXT } else { colors::SUBTLE_TEXT }));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if installed {
                        if ui.add_enabled(!busy, egui::Button::new("Delete").min_size(vec2(84.0, 26.0))).clicked() {
                            click = Some(Click::Delete(model));
                        }
                    } else if ui.add_enabled(!busy, egui::Button::new("Download").fill(colors::ACCENT).min_size(vec2(84.0, 26.0))).clicked() {
                        click = Some(Click::Download(model));
                    }
                });
            });
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Folder: {folder}")).size(11.5).color(colors::SUBTLE_TEXT));
            if ui.button("Show folder").clicked() {
                click = Some(Click::ShowFolder);
            }
        });
    });
    match click {
        Some(Click::Download(model)) => {
            pending::download_list(app, &format!("Downloading the {}", model.display_name), vec![model], None);
        }
        Some(Click::Delete(model)) => {
            app.ai.services.release_all(); // the file must not be in use
            app.status = match app.ai.store.delete(model) {
                Ok(()) => format!("Deleted the {}.", model.display_name),
                Err(e) => format!("Could not delete the {}: {e}", model.display_name),
            };
        }
        Some(Click::ShowFolder) => {
            let path = app.ai.store.folder().to_path_buf();
            let _ = std::fs::create_dir_all(&path);
            let _ = std::process::Command::new("explorer").arg(&path).spawn();
        }
        None => {}
    }
    open
}
