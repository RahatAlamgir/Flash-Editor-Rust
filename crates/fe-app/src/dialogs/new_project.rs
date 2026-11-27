//! New project (C# `NewImageDialog` with a name): name, a ready-made size, width / height, background, and
//! "Start from" (Blank or one of the user's saved templates, marked with a star). The ready-made layouts of the
//! C# version (Title slide, Quote card...) need editable text and shapes; they arrive with Phase 10.

use egui::{ComboBox, DragValue, Id, RichText};
use fe_core::document::{Document, Page};
use fe_core::imaging::ColorRgba;
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::commands::file;

/// Sizes offered in the list (C# `NewImageDialog.Presets`).
pub const PRESETS: [(&str, u32, u32); 19] = [
    ("Custom", 0, 0),
    ("Full HD / presentation (1920 x 1080)", 1920, 1080),
    ("HD (1280 x 720)", 1280, 720),
    ("YouTube thumbnail (1280 x 720)", 1280, 720),
    ("Instagram post, square (1080 x 1080)", 1080, 1080),
    ("Instagram post, portrait (1080 x 1350)", 1080, 1350),
    ("Instagram / TikTok story (1080 x 1920)", 1080, 1920),
    ("Facebook post (1200 x 630)", 1200, 630),
    ("Facebook cover (820 x 312)", 820, 312),
    ("X / Twitter header (1500 x 500)", 1500, 500),
    ("LinkedIn banner (1584 x 396)", 1584, 396),
    ("Pinterest pin (1000 x 1500)", 1000, 1500),
    ("A4 at 300 dpi (2480 x 3508)", 2480, 3508),
    ("A5 at 300 dpi (1748 x 2480)", 1748, 2480),
    ("US Letter at 300 dpi (2550 x 3300)", 2550, 3300),
    ("Business card at 300 dpi (1050 x 600)", 1050, 600),
    ("Poster 18 x 24 in at 150 dpi (2700 x 3600)", 2700, 3600),
    ("Icon (256 x 256)", 256, 256),
    ("Small (800 x 600)", 800, 600),
];

/// Background of the new page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    White,
    Transparent,
    /// The background color of the toolbox.
    Color,
}

/// What the user typed so far.
pub struct NewProjectForm {
    pub name: String,
    pub preset: usize,
    pub width: u32,
    pub height: u32,
    pub background: Background,
    /// `None` = Blank; otherwise a user template name.
    pub template: Option<String>,
    /// A ready-made layout ("Blank", "Title slide"...) when no user template is chosen.
    pub layout: &'static str,
    templates: Vec<String>,
    background_color: ColorRgba,
}

impl NewProjectForm {
    pub fn new(background_color: ColorRgba) -> Self {
        Self {
            name: "Untitled project".into(),
            preset: 2,
            width: 1280,
            height: 720,
            background: Background::White,
            template: None,
            layout: fe_core::library::layouts::BLANK,
            templates: file::user_templates(),
            background_color,
        }
    }
}

/// Shows the window. `Some(true)` = Create, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, form: &mut NewProjectForm) -> Option<bool> {
    let mut result = None;
    let modal = egui::Modal::new(Id::unique("new-project")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.label(RichText::new("New Project").heading());
        ui.add_space(8.0);
        let label = |ui: &mut egui::Ui, t: &str| ui.label(RichText::new(t).color(colors::SUBTLE_TEXT));
        label(ui, "Project name");
        ui.add(egui::TextEdit::singleline(&mut form.name).char_limit(80).desired_width(f32::INFINITY));
        ui.add_space(8.0);
        label(ui, "Preset");
        let template_size = form.template.is_some();
        ui.add_enabled_ui(!template_size, |ui| {
            ComboBox::from_id_salt("new-preset").width(360.0).selected_text(PRESETS[form.preset].0).show_ui(ui, |ui| {
                for (i, (name, w, h)) in PRESETS.iter().enumerate() {
                    if ui.selectable_value(&mut form.preset, i, *name).clicked() && *w > 0 {
                        form.width = *w;
                        form.height = *h;
                    }
                }
            });
        });
        ui.add_space(8.0);
        label(ui, "Start from");
        let selected = form.template.as_ref().map_or_else(|| form.layout.to_owned(), |t| format!("★ {t}"));
        ComboBox::from_id_salt("new-template").width(360.0).selected_text(selected).show_ui(ui, |ui| {
            for layout in fe_core::library::layouts::NAMES {
                if ui.selectable_label(form.template.is_none() && form.layout == layout, layout).clicked() {
                    form.template = None;
                    form.layout = layout;
                }
            }
            ui.separator();
            for t in form.templates.clone() {
                ui.selectable_value(&mut form.template, Some(t.clone()), format!("★ {t}"));
            }
        });
        ui.label(
            RichText::new(
                "Blank = an empty picture. The ready-made layouts keep every word, shape and color editable. Your own templates (File > Save as Template) are marked with a star.",
            )
            .size(11.0)
            .color(colors::SUBTLE_TEXT),
        );
        ui.add_space(8.0);
        ui.add_enabled_ui(!template_size, |ui| {
            ui.horizontal(|ui| {
                label(ui, "Width");
                if ui.add(DragValue::new(&mut form.width).range(1..=16384).suffix(" px")).changed() {
                    form.preset = 0;
                }
                label(ui, "Height");
                if ui.add(DragValue::new(&mut form.height).range(1..=16384).suffix(" px")).changed() {
                    form.preset = 0;
                }
            });
            ui.add_space(8.0);
            label(ui, "Background");
            ui.radio_value(&mut form.background, Background::White, "White");
            ui.radio_value(&mut form.background, Background::Transparent, "Transparent");
            ui.radio_value(&mut form.background, Background::Color, "Background color");
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Create").fill(colors::ACCENT).min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(false);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}

/// Makes the project the form describes.
pub fn create(ctx: &egui::Context, app: &mut FlashApp, form: &NewProjectForm) {
    let pages = match &form.template {
        Some(name) => match file::template_pages(name) {
            Ok(pages) => pages,
            Err(err) => {
                app.status = format!("Could not open the template \"{name}\": {err}");
                return;
            }
        },
        None => {
            let background = match form.background {
                Background::White => ColorRgba::WHITE,
                Background::Transparent => ColorRgba::TRANSPARENT,
                Background::Color => form.background_color,
            };
            let mut doc = Document::blank(form.width, form.height, background);
            fe_core::library::layouts::apply(&mut doc, form.layout);
            vec![Page::new("Page 1", doc)]
        }
    };
    file::create_project(app, ctx, &form.name, pages);
}
