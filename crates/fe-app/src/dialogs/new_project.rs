//! New project: name, a project type (Presentation, Card, Thumbnail... or Custom) that sets the size, width / height
//! and background. Ready-made designs are in the online Templates (Project Manager), not here.

use egui::{DragValue, RichText};
use fe_core::document::{Document, Page};
use fe_core::imaging::ColorRgba;
use fe_ui_kit::theme::colors;

use super::ColorPicker;
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

/// Project types shown in the dropdown: (online template type id, name, width, height). The first is Custom.
pub const TYPES: [(Option<&str>, &str, u32, u32); 10] = [
    (None, "Custom", 0, 0),
    (Some("presentation"), "Presentation", 1920, 1080),
    (Some("poster"), "Poster", 2700, 3600),
    (Some("card"), "Card", 1050, 600),
    (Some("youtube-thumbnail"), "Thumbnail", 1280, 720),
    (Some("instagram-post"), "Instagram post", 1080, 1080),
    (Some("story"), "Story", 1080, 1920),
    (Some("flyer"), "Flyer", 2480, 3508),
    (Some("resume"), "Resume", 2480, 3508),
    (None, "Banner", 1584, 396),
];

/// What the user typed so far.
pub struct NewProjectForm {
    pub name: String,
    /// Index in `TYPES` (0 = Custom).
    pub kind: usize,
    pub width: u32,
    pub height: u32,
    /// The color of the new page (white to start with).
    pub color: [u8; 4],
    /// No color at all: the page starts see-through.
    pub transparent: bool,
    /// The color picker, while it is open.
    picker: Option<ColorPicker>,
    /// "Browse templates" was clicked: show the online templates of this type (`None` = all) instead of creating.
    pub browse: Option<Option<String>>,
}

impl Default for NewProjectForm {
    fn default() -> Self {
        Self::new()
    }
}

impl NewProjectForm {
    pub fn new() -> Self {
        let (_, _, width, height) = TYPES[1];
        Self {
            name: "Untitled project".into(),
            kind: 1,
            width,
            height,
            color: [255, 255, 255, 255],
            transparent: false,
            picker: None,
            browse: None,
        }
    }
}

/// Shows the window. `Some(true)` = Create, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, form: &mut NewProjectForm) -> Option<bool> {
    let mut result = None;
    // While the color picker is open it takes the Esc key; this window must not close too.
    let picker_was_open = form.picker.is_some();
    let modal = super::frame::modal(ctx, "new-project", |ui| {
        ui.set_width(440.0);
        ui.label(RichText::new("New Project").heading());
        ui.add_space(8.0);
        let label = |ui: &mut egui::Ui, t: &str| ui.label(RichText::new(t).color(colors::SUBTLE_TEXT));
        label(ui, "Project name");
        ui.add(egui::TextEdit::singleline(&mut form.name).char_limit(80).desired_width(f32::INFINITY));
        ui.add_space(10.0);
        label(ui, "Project type");
        let selected = match TYPES[form.kind] {
            (_, name, w, h) if w > 0 => format!("{name} ({w} x {h})"),
            (_, name, _, _) => name.to_owned(),
        };
        egui::ComboBox::from_id_salt("new-project-type").width(420.0).selected_text(selected).show_ui(ui, |ui| {
            for (i, (_, name, w, h)) in TYPES.iter().enumerate() {
                let text = if *w > 0 { format!("{name} ({w} x {h})") } else { (*name).to_owned() };
                if ui.selectable_label(form.kind == i, text).clicked() {
                    form.kind = i;
                    if *w > 0 {
                        (form.width, form.height) = (*w, *h);
                    }
                }
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            label(ui, "Width");
            let wide = ui.add(DragValue::new(&mut form.width).range(1..=16384).suffix(" px")).changed();
            label(ui, "Height");
            let tall = ui.add(DragValue::new(&mut form.height).range(1..=16384).suffix(" px")).changed();
            if wide || tall {
                // A size that is not the type's own size is a custom size.
                let (_, _, w, h) = TYPES[form.kind];
                if (form.width, form.height) != (w, h) {
                    form.kind = 0;
                }
            }
        });
        ui.add_space(10.0);
        label(ui, "Background");
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!form.transparent, |ui| {
                let (rect, response) = ui.allocate_exact_size(egui::vec2(56.0, 26.0), egui::Sense::click());
                let [r, g, b, _] = form.color;
                ui.painter().rect(
                    rect,
                    4,
                    egui::Color32::from_rgb(r, g, b),
                    egui::Stroke::new(1.0, colors::DIVIDER),
                    egui::StrokeKind::Inside,
                );
                if response.on_hover_text("Click to choose the color").clicked() {
                    let [r, g, b, a] = form.color;
                    form.picker = Some(ColorPicker::nested("Background color", ColorRgba::new(r, g, b, a)));
                }
            });
            ui.checkbox(&mut form.transparent, "Transparent");
        });
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Create").fill(colors::ACCENT).min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(false);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (id, name, _, _) = TYPES[form.kind];
                let text =
                    if id.is_some() { format!("Browse {name} templates") } else { "Browse templates".to_owned() };
                if ui.button(text).clicked() {
                    form.browse = Some(id.map(str::to_owned));
                    result = Some(false);
                }
            });
        });
    });
    if let Some(mut picker) = form.picker.take() {
        match super::color_picker::show_nested(ctx, app, &mut picker) {
            Some(Some(color)) => form.color = [color.r, color.g, color.b, 255],
            Some(None) => {}
            None => form.picker = Some(picker),
        }
    }
    if result.is_none() && modal.should_close() && !picker_was_open {
        result = Some(false);
    }
    result
}

/// Makes the project the form describes.
pub fn create(ctx: &egui::Context, app: &mut FlashApp, form: &NewProjectForm) {
    let [r, g, b, a] = form.color;
    let background = if form.transparent { ColorRgba::TRANSPARENT } else { ColorRgba::new(r, g, b, a) };
    let doc = Document::blank(form.width, form.height, background);
    file::create_project(app, ctx, &form.name, vec![Page::new("Page 1", doc)]);
}
