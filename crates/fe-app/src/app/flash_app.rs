//! The main app object: everything the program knows. What happens each frame is in `frame.rs`, the window
//! layout in `layout.rs`.

use fe_core::document::{Document, Page, Project};
use fe_core::imaging::ColorRgba;

use super::autosave::AutoSave;
use super::jobs::Jobs;
use super::settings::Settings;
use super::thumbnails::Thumbnails;
use super::{StartupOptions, ViewAids, WindowPlacement};
use crate::canvas::bench::Bench;
use crate::canvas::{CanvasState, ViewState};
use crate::dialogs::Dialog;
use crate::dialogs::project_manager::ManagerState;
use crate::panels::pages::PagesUi;
use crate::panels::right::layers::LayersUi;
use crate::panels::{ribbon, view_aids_bar::AidPopup};
use crate::tools::ToolState;

/// Everything the program knows. Grows phase by phase.
pub struct FlashApp {
    pub ribbon: ribbon::RibbonState,
    pub aids: ViewAids,
    pub tools: ToolState,
    /// Open projects (tabs).
    pub projects: Vec<Project>,
    pub active_project: usize,
    pub foreground: ColorRgba,
    /// The eyedropper of the color picker.
    pub eyedropper: super::eyedropper::Eyedropper,
    /// The slideshow or transition preview on screen.
    pub slideshow: Option<crate::slideshow::Slideshow>,
    /// Text on the left of the status bar.
    pub status: String,
    /// The pixel under the mouse, for the status bar.
    pub cursor_text: String,
    pub history_expanded: bool,
    /// Zoom shown in the status bar (the canvas sets it).
    pub zoom: f32,
    pub canvas: CanvasState,
    /// The open settings card of the grid / guides / snap buttons.
    pub aid_popup: Option<AidPopup>,
    /// "wgpu" or "OpenGL".
    pub renderer_name: &'static str,
    pub thumbnails: Thumbnails,
    pub layers_ui: LayersUi,
    pub pages_ui: PagesUi,
    /// Share of the right column's height given to Properties (the rest goes to Layers).
    pub props_share: f32,
    /// Which tab of the right column is shown: 0 Properties, 1 Effects, 2 Adjust, 3 Animation.
    pub props_tab: usize,
    /// Start-up: only the Project Manager is shown (no editor behind it) until a project is opened or made.
    pub launcher: bool,
    /// Which tab of the Layers panel is shown: 0 Layers, 1 Animations (presentations).
    pub layers_tab: usize,
    /// The right column is folded away to a thin strip.
    pub right_collapsed: bool,
    /// The page strip under the canvas is folded away to a thin bar.
    pub pages_collapsed: bool,
    /// The Settings > Undo Steps window is open.
    pub undo_dialog: bool,
    /// The list under a ribbon "▾" button that is open: (tab, group, item, button rectangle).
    pub ribbon_menu: Option<(usize, usize, usize, egui::Rect)>,
    /// The File menu (under the File button) is open. The full File drawer comes in Phase 15.
    pub file_menu: bool,
    pub settings: Settings,
    /// The settings as last written to disk (to notice changes).
    pub(super) saved_settings: Settings,
    pub jobs: Jobs,
    pub dialog: Option<Dialog>,
    /// Small pictures of badges, graphics and diagrams for the pickers.
    pub previews: crate::dialogs::previews::Previews,
    pub manager: ManagerState,
    pub autosave: AutoSave,
    /// The window is closing after "Save all".
    pub closing_window: bool,
    /// Close without asking again.
    pub quitting: bool,
    pub(super) startup: StartupOptions,
    pub(super) started: bool,
    pub(super) bench: Option<Bench>,
    pub(super) placement: Option<WindowPlacement>,
    /// The window title last sent to Windows (only sent again when it changes).
    pub(super) title: String,
    /// Copied pictures and layers.
    pub clipboard: super::clipboard::Clipboard,
    /// The pictures of the Insert > Shapes gallery.
    pub shape_icons: crate::panels::shape_menu::ShapeIcons,
    /// The looks the user saved (read from `settings\styles.json` the first time they are needed).
    pub styles: Option<Vec<fe_core::objects::style::SavedStyle>>,
    /// The text being typed on the canvas, if any.
    pub text_edit: Option<crate::text_edit::TextEditor>,
    /// The AI: models, the running job.
    pub ai: crate::ai::AiState,
    /// A project tab whose name is being typed (double-click on the tab): which tab, and the text so far.
    pub tab_rename: Option<(usize, String)>,
}

impl FlashApp {
    /// Called once when the window opens.
    pub fn new(cc: &eframe::CreationContext<'_>, startup: StartupOptions) -> Self {
        fe_ui_kit::install(&cc.egui_ctx);
        fe_ui_kit::theme::fonts::add_material(&cc.egui_ctx, fe_core::library::material::FONT);
        let settings = Settings::load();
        let mut app = Self {
            ribbon: ribbon::RibbonState::default(),
            aids: ViewAids::default(),
            tools: ToolState::default(),
            projects: vec![Self::blank_project()],
            active_project: 0,
            foreground: ColorRgba::BLACK,
            eyedropper: Default::default(),
            slideshow: None,
            status: "Nothing selected. Click a shape, text, image or painted area to select it.".into(),
            cursor_text: String::new(),
            history_expanded: false,
            zoom: 1.0,
            canvas: CanvasState::default(),
            aid_popup: None,
            renderer_name: if cc.wgpu_render_state.is_some() { "wgpu" } else { "OpenGL" },
            thumbnails: Thumbnails::default(),
            layers_ui: LayersUi::default(),
            pages_ui: PagesUi::default(),
            props_share: 0.6,
            props_tab: 0,
            launcher: false,
            layers_tab: 0,
            right_collapsed: false,
            pages_collapsed: false,
            undo_dialog: false,
            ribbon_menu: None,
            file_menu: false,
            saved_settings: settings.clone(),
            settings,
            jobs: Jobs::default(),
            dialog: None,
            manager: ManagerState::default(),
            autosave: AutoSave::default(),
            closing_window: false,
            quitting: false,
            startup,
            started: false,
            bench: None,
            placement: None,
            title: String::new(),
            clipboard: Default::default(),
            shape_icons: Default::default(),
            previews: Default::default(),
            styles: None,
            text_edit: None,
            ai: Default::default(),
            tab_rename: None,
        };
        super::settings_sync::apply(&mut app);
        app
    }

    /// The empty project shown when nothing else is open.
    pub fn blank_project() -> Project {
        Project::new("Untitled", Page::new("Page 1", Document::blank(1280, 720, ColorRgba::WHITE)))
    }

    /// The Projects window at start-up turns into the editor window: its saved size and place (maximized on the first start).
    pub fn leave_launcher(&mut self) {
        if !self.launcher {
            return;
        }
        self.launcher = false;
        self.title.clear(); // the title is set again
        if let Some(ctx) = &self.ai.ctx {
            use egui::ViewportCommand as Cmd;
            ctx.send_viewport_cmd(Cmd::MinInnerSize(egui::vec2(900.0, 560.0)));
            match super::WindowPlacement::load() {
                Some(p) => {
                    ctx.send_viewport_cmd(Cmd::InnerSize(egui::vec2(p.width, p.height)));
                    ctx.send_viewport_cmd(Cmd::OuterPosition(egui::pos2(p.left, p.top)));
                    ctx.send_viewport_cmd(Cmd::Maximized(p.maximized));
                }
                None => ctx.send_viewport_cmd(Cmd::Maximized(true)),
            }
        }
    }

    pub fn project(&self) -> &Project {
        &self.projects[self.active_project]
    }

    pub fn project_mut(&mut self) -> &mut Project {
        &mut self.projects[self.active_project]
    }

    /// The document of the page that is shown.
    pub fn doc(&self) -> &Document {
        &self.project().active_page().doc
    }

    /// The view (zoom, scroll) of the page that is shown.
    pub fn current_view_mut(&mut self) -> &mut ViewState {
        let key = (self.active_project, self.projects[self.active_project].active_index());
        self.canvas.views.entry(key).or_default()
    }

    /// True while the start-up "Untitled" project has not been used.
    pub fn start_project_untouched(&self) -> bool {
        let p = &self.projects[0];
        p.name == "Untitled" && p.path.is_none() && !p.is_dirty() && p.pages().len() == 1
    }

    /// Tells the user that a command is not there yet (until its phase is built).
    pub fn not_yet(&mut self, what: &str) {
        self.status = format!("{what}: coming in a later step of the Rust version.");
    }
}
