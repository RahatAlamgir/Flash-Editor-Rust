//! The Quick Tour (shown once on the first start, and from the Settings tab) and About (C# `ViewCommands`).

use crate::ai::outcome::tell;
use crate::app::FlashApp;

pub const TOUR_TEXT: &str = "A few things worth knowing:\n\n\
\u{2022}  Everything is a layer. Click an object with the Select / Move tool (V), then change it in Properties: size, rotation, colors, text designs, shadows and more.\n\n\
\u{2022}  Right-click a layer on the canvas or in the Layers panel for quick options. Drag layers up and down to reorder them.\n\n\
\u{2022}  Crop (C) and Perspective Crop work like a selection: drag a box, adjust it, press Enter. The original layer is kept, hidden.\n\n\
\u{2022}  Ctrl+Z undoes anything, in every page of a project. The History panel lists the steps.\n\n\
\u{2022}  AI tools (Remove Background, Click Select, Remove / Fill...) ask once before downloading their model. Settings > AI Models shows them.\n\n\
\u{2022}  Settings > Brand Kit keeps your colors and fonts. File > Save as Template keeps a design for new pictures.\n\n\
Press F1 any time for the keyboard shortcuts. The search box at the top right finds any command by name. This tour is on the Settings tab.";

/// Settings > Quick Tour.
pub fn show_tour(app: &mut FlashApp) {
    tell(app, "Welcome to Flash Editor", TOUR_TEXT);
    app.settings.tour_seen = true;
}

/// The very first start: shows the tour once.
pub fn on_first_run(app: &mut FlashApp) {
    if !app.settings.tour_seen && app.dialog.is_none() {
        show_tour(app);
    }
}

/// Settings > About.
pub fn show_about(app: &mut FlashApp) {
    let text = format!(
        "Flash Editor {}\nA small, fast layer-based image editor with offline AI tools, built with Rust and egui.\n\nProjects are .flash files that the C# version of Flash Editor opens too.",
        env!("CARGO_PKG_VERSION")
    );
    tell(app, "About Flash Editor", text);
}
