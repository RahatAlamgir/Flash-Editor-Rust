//! Flash Editor: starts logging and opens the main window.

// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod app;
mod canvas;
mod commands;
mod dialogs;
mod panels;
mod text_edit;
mod tools;

fn main() {
    app::logging::init();
    let startup = app::StartupOptions::from_args(std::env::args());
    if let Err(err) = app::window::run(startup) {
        log::error!("Flash Editor could not start: {err}");
        std::process::exit(1);
    }
}

// The AI and GPU crates are wired in from Phase 3 / Phase 13 on; linking them now keeps the
// workspace building as one program.
use fe_ai as _;
use fe_core as _;
use fe_gpu as _;
