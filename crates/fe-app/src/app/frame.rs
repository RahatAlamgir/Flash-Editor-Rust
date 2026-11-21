//! What happens every frame: start-up work, finished background jobs, dropped files, the speed test, keyboard,
//! layout, dialogs, autosave, settings, title and closing the window.

use egui::Ui;
use fe_core::document::{Page, Project};

use super::jobs::JobDone;
use super::{FlashApp, WindowPlacement, autosave, keyboard, layout, settings_sync};
use crate::canvas::bench::{Bench, BenchStep, bench_document};
use crate::commands::file;
use crate::dialogs::{self, project_manager};
use crate::panels;

impl eframe::App for FlashApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.started {
            first_frame(self, &ctx);
        }
        self.ai.ctx = Some(ctx.clone());
        self.thumbnails.begin_frame(&ctx);
        finished_jobs(self, &ctx);
        dropped_files(self, ui);
        drive_bench(self, ui, frame);
        keyboard::handle(ui, self);
        layout::show(self, ui);
        panels::popups::show(ui, self);
        project_manager::show(&ctx, self);
        dialogs::show(&ctx, self);
        autosave::tick(self, &ctx);
        settings_sync::save_if_changed(self);
        update_title(self, ui);
        track_placement(self, ui);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(fe_ui_kit::theme::colors::WINDOW).to_array()
    }
}

/// Start-up: crash recovery, files from the command line, the speed test, or the Project Manager.
fn first_frame(app: &mut FlashApp, ctx: &egui::Context) {
    app.started = true;
    if app.startup.bench {
        file::add_project(app, Project::new("Benchmark", Page::new("Page 1", bench_document())));
        app.bench = Some(Bench::new(app.renderer_name));
        return;
    }
    if autosave::recovery_file().exists() {
        autosave::offer_recovery(app);
    }
    let files = std::mem::take(&mut app.startup.files);
    crate::commands::tutorial::first_start(app);
    if files.is_empty() {
        project_manager::open(app, ctx);
    }
    for path in files {
        file::open_path(app, ctx, &path, false);
    }
}

fn finished_jobs(app: &mut FlashApp, ctx: &egui::Context) {
    for done in app.jobs.finished() {
        match done {
            JobDone::Opened { path, result, as_layers } => file::on_opened(app, path, result, as_layers),
            JobDone::Saved { project, path, position, result, then } => {
                file::on_saved(app, ctx, project, path, position, result, then);
            }
            JobDone::AutoSaved(result) => autosave::on_saved(app, result),
            JobDone::ProjectList(entries) => project_manager::on_list(app, entries),
            JobDone::Ai(done) => crate::ai::outcome::on_done(app, done),
            JobDone::EffectPreview { generation, tiles } => {
                crate::commands::effects::on_preview(app, ctx, generation, tiles);
            }
        }
    }
}

/// Files dropped onto the window open as projects; with Ctrl held, pictures are added as layers.
fn dropped_files(app: &mut FlashApp, ui: &Ui) {
    let (paths, ctrl): (Vec<std::path::PathBuf>, bool) = ui.input(|i| {
        let paths =
            i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect();
        (paths, i.modifiers.command)
    });
    for path in paths {
        let as_layers = ctrl && !fe_core::io::project::is_project(&path);
        file::open_path(app, ui.ctx(), &path, as_layers);
    }
}

/// Runs one step of the speed test.
fn drive_bench(app: &mut FlashApp, ui: &Ui, frame: &eframe::Frame) {
    let Some(bench) = app.bench.as_mut() else { return };
    let cpu_ms = frame.info().cpu_usage.unwrap_or(0.0) * 1000.0;
    let stats = app.canvas.stats;
    match bench.step(ui, cpu_ms, stats.render_ms, stats.texture_bytes as f32 / 1e6) {
        BenchStep::Wait => {}
        BenchStep::ZoomTo(factor) => {
            let view = app.current_view_mut();
            let fit = view.fit_zoom();
            view.zoom_at(view.size / 2.0, fit * factor);
        }
        BenchStep::PanBy(delta) => app.current_view_mut().pan_by(delta),
        BenchStep::Done => {
            if let Some(bench) = app.bench.take() {
                app.status = bench.report().lines().nth(2).unwrap_or_default().to_owned();
            }
            app.quitting = true;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

/// "Name - Flash Editor", like the C# window title.
fn update_title(app: &mut FlashApp, ui: &Ui) {
    let title = format!("{} - Flash Editor", app.project().name);
    if title != app.title {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
        app.title = title;
    }
}

/// Remembers the window placement; on close asks about unsaved work, then saves the placement and removes
/// the crash backup.
fn track_placement(app: &mut FlashApp, ui: &Ui) {
    let (info, closing) = ui.input(|i| (i.viewport().clone(), i.viewport().close_requested()));
    if let Some(p) = WindowPlacement::from_viewport(&info) {
        // While maximized keep the last normal size, so "restore" goes back to it next time.
        app.placement = Some(match (p.maximized, app.placement) {
            (true, Some(old)) => WindowPlacement { maximized: true, ..old },
            _ => p,
        });
    }
    if !closing {
        return;
    }
    if file::window_may_close(app) {
        if let Some(p) = app.placement {
            p.save();
        }
        settings_sync::save_if_changed(app);
        autosave::delete_recovery();
    } else {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::CancelClose);
    }
}
