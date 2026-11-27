//! Opens the main window: uses the OpenGL renderer (glow) and falls back to wgpu (DirectX 12 / Vulkan) on PCs
//! where OpenGL cannot start. OpenGL is the default because it measured as smooth as wgpu and needs the least
//! memory (Phase 3). Restores the last window placement.

use super::{FlashApp, StartupOptions, WindowPlacement};

const APP_ICON_PNG: &[u8] = include_bytes!("../../../../assets/icons/app/FlashEditor.png");

/// Runs the app until the window is closed.
pub fn run(startup: StartupOptions) -> Result<(), eframe::Error> {
    if startup.wgpu {
        return run_with(eframe::Renderer::Wgpu, startup);
    }
    match run_with(eframe::Renderer::Glow, startup.clone()) {
        Ok(()) => Ok(()),
        Err(err) => {
            log::warn!("OpenGL renderer failed: {err}. Trying wgpu.");
            run_with(eframe::Renderer::Wgpu, startup)
        }
    }
}

fn run_with(renderer: eframe::Renderer, startup: StartupOptions) -> Result<(), eframe::Error> {
    // Without a file to open the program starts as just the small Projects window; the editor window comes when a project is
    // chosen (like the C# version).
    let launcher = startup.files.is_empty() && !startup.bench;
    let mut viewport = if launcher {
        egui::ViewportBuilder::default()
            .with_title("Flash Editor - Projects")
            .with_inner_size([1000.0, 680.0])
            .with_min_inner_size([720.0, 480.0])
    } else {
        placed(
            egui::ViewportBuilder::default()
                .with_title("Flash Editor")
                .with_inner_size([1440.0, 900.0])
                .with_min_inner_size([900.0, 560.0]),
        )
    };
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, renderer, centered: launcher, ..Default::default() };
    log::info!("Starting with renderer {renderer:?}");
    eframe::run_native("Flash Editor", options, Box::new(move |cc| Ok(Box::new(FlashApp::new(cc, startup)))))
}

/// The editor window where it was last time (maximized on the first start, like the C# version).
fn placed(viewport: egui::ViewportBuilder) -> egui::ViewportBuilder {
    match WindowPlacement::load() {
        Some(p) => {
            viewport.with_position([p.left, p.top]).with_inner_size([p.width, p.height]).with_maximized(p.maximized)
        }
        None => viewport.with_maximized(true), // first start: full screen, like the C# version
    }
}

fn load_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(APP_ICON_PNG).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData { rgba: image.into_raw(), width, height })
}
