//! The template browser of the Project Manager (C# `TemplatesView`): search box, category buttons (the project types) and a
//! card for every template. Pictures are loaded a few at a time in the background, so the cards appear at once. Choosing a
//! template downloads it (checked), saves it as a new project in the projects folder and opens it.

use std::collections::{HashMap, HashSet};

use egui::{Align2, Color32, ColorImage, FontId, Key, Rect, RichText, Sense, Stroke, TextureHandle, pos2, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::paint_checkerboard;

use crate::app::FlashApp;
use crate::app::jobs::JobDone;
use crate::commands::file;
use crate::templates::{Catalog, OnlineTemplate, default_source, download};

const CARD: egui::Vec2 = egui::Vec2::new(214.0, 214.0);
/// How many pictures are loaded at the same time.
const PARALLEL: usize = 4;

#[derive(Default)]
pub struct TemplatesState {
    /// The browser is shown instead of the project list.
    pub view: bool,
    pub catalog: Option<Catalog>,
    pub loading: bool,
    pub error: Option<String>,
    pub search: String,
    pub category: Option<String>,
    pub selected: Option<String>,
    textures: HashMap<String, TextureHandle>,
    requested: HashSet<String>,
    inflight: usize,
    /// "Getting ..." while a template is downloaded.
    pub busy: Option<String>,
}

/// Starts reading the catalog the first time the browser is shown (or again after "Try again").
pub fn ensure_loaded(app: &mut FlashApp, ctx: &egui::Context) {
    let t = &mut app.manager.templates;
    if t.catalog.is_some() || t.loading {
        return;
    }
    t.loading = true;
    t.error = None;
    app.jobs.spawn(ctx, move || JobDone::TemplateCatalog(Catalog::load(&default_source())));
}

/// The catalog arrived.
pub fn on_catalog(app: &mut FlashApp, result: Result<Catalog, String>) {
    let t = &mut app.manager.templates;
    t.loading = false;
    match result {
        Ok(catalog) => {
            t.catalog = Some(catalog);
            t.category = None;
        }
        Err(_) => t.error = Some(
            "Check your internet connection and try again. Templates you opened before still work without internet."
                .into(),
        ),
    }
}

/// A small picture arrived (or could not be had).
pub fn on_thumb(app: &mut FlashApp, ctx: &egui::Context, id: String, image: Option<ColorImage>) {
    let t = &mut app.manager.templates;
    t.inflight = t.inflight.saturating_sub(1);
    if let Some(image) = image {
        t.textures.insert(id.clone(), ctx.load_texture(format!("template-{id}"), image, Default::default()));
    }
}

/// The chosen template was downloaded: it becomes a new project in the projects folder, and is opened.
pub fn on_file(app: &mut FlashApp, ctx: &egui::Context, name: &str, result: Result<Vec<u8>, String>) {
    app.manager.templates.busy = None;
    match result {
        Ok(data) => {
            let folder = app.settings.projects_folder();
            let stem = crate::app::settings::clean_name(name);
            let path = fe_core::io::batch::unique_path(&folder, &stem, ".flash");
            let written = std::fs::create_dir_all(&folder).and_then(|()| std::fs::write(&path, data));
            match written {
                Ok(()) => {
                    app.settings.add_recent(&path);
                    app.manager.open = false;
                    file::open_path(app, ctx, &path, false);
                }
                Err(e) => crate::ai::outcome::tell(app, "Templates", format!("The template could not be saved: {e}")),
            }
        }
        Err(message) => crate::ai::outcome::tell(app, "Templates", message),
    }
}

/// Starts the download of a template.
fn choose(app: &mut FlashApp, ctx: &egui::Context, template: &OnlineTemplate) {
    if app.manager.templates.busy.is_some() {
        return;
    }
    app.manager.templates.busy = Some(format!("Getting \"{}\"...", template.name));
    let t = template.clone();
    app.jobs.spawn(ctx, move || {
        let result = download(&default_source(), &t);
        JobDone::TemplateFile { name: t.name, result }
    });
}

/// Starts loading the pictures of the shown cards, a few at a time.
fn pump_thumbs(app: &mut FlashApp, ctx: &egui::Context, shown: &[OnlineTemplate]) {
    for template in shown {
        if app.manager.templates.inflight >= PARALLEL {
            break;
        }
        let key = format!("{}/{}", template.kind, template.id);
        if !app.manager.templates.requested.insert(key.clone()) {
            continue;
        }
        app.manager.templates.inflight += 1;
        let path = template.thumbnail_path();
        app.jobs.spawn(ctx, move || {
            let image = default_source()
                .cached_or_fetch(&path)
                .ok()
                .and_then(|bytes| image::load_from_memory(&bytes).ok())
                .map(|img| {
                    let rgba = img.into_rgba8();
                    ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw())
                });
            JobDone::TemplateThumb { id: key, image }
        });
    }
}

/// Draws the browser into the Project Manager window.
pub fn show(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut FlashApp) {
    ensure_loaded(app, ctx);
    if let Some(busy) = app.manager.templates.busy.clone() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.label(RichText::new(busy).color(colors::TEXT));
        });
        return;
    }
    let mut retry = false;
    if app.manager.templates.loading {
        ui.label(RichText::new("Loading templates...").color(colors::SUBTLE_TEXT));
        return;
    }
    if let Some(error) = &app.manager.templates.error {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Templates could not be loaded").heading());
            ui.label(RichText::new(error).color(colors::SUBTLE_TEXT));
            retry = ui.button("Try again").clicked();
        });
        if retry {
            app.manager.templates.error = None;
            ensure_loaded(app, ctx);
        }
        return;
    }
    let Some(catalog) = app.manager.templates.catalog.clone() else { return };
    let mut start = None;
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.manager.templates.search)
                .hint_text("Search: name, type or keyword (school, business, sale...)")
                .desired_width(340.0),
        );
        let note = if catalog.offline {
            "Offline: showing the templates saved earlier.".to_owned()
        } else {
            format!("{} templates", catalog.templates.len())
        };
        ui.label(RichText::new(note).size(11.5).color(colors::SUBTLE_TEXT));
    });
    ui.horizontal_wrapped(|ui| {
        for c in catalog.categories() {
            let on = app.manager.templates.category == c.id;
            if ui.selectable_label(on, format!("{} ({})", c.name, c.count)).clicked() {
                app.manager.templates.category = c.id;
            }
        }
    });
    ui.separator();
    let shown: Vec<OnlineTemplate> = catalog
        .find(app.manager.templates.category.as_deref(), &app.manager.templates.search)
        .into_iter()
        .cloned()
        .collect();
    // Keep a valid pick: the first shown template when the old one is gone.
    if shown.is_empty() {
        app.manager.templates.selected = None;
    } else if !shown.iter().any(|t| Some(&t.id) == app.manager.templates.selected.as_ref()) {
        app.manager.templates.selected = Some(shown[0].id.clone());
    }
    pump_thumbs(app, ctx, &shown);
    if shown.is_empty() {
        let (title, text) = if app.manager.templates.search.trim().is_empty() {
            ("No templates here yet", "More are added all the time. Pick another category for now.")
        } else {
            ("No template matches", "Try another word, such as school, business, sale or party.")
        };
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(title).heading());
            ui.label(RichText::new(text).color(colors::SUBTLE_TEXT));
        });
    }
    let height = (ui.available_height() - 40.0).max(120.0);
    egui::ScrollArea::vertical().max_height(height).auto_shrink(false).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
            for template in &shown {
                let key = format!("{}/{}", template.kind, template.id);
                let selected = app.manager.templates.selected.as_deref() == Some(&template.id);
                let (clicked, double) = card(ui, template, app.manager.templates.textures.get(&key), selected);
                if clicked {
                    app.manager.templates.selected = Some(template.id.clone());
                }
                if double {
                    start = Some(template.clone());
                }
            }
        });
    });
    ui.horizontal(|ui| {
        let pick = app.manager.templates.selected.clone().and_then(|id| shown.iter().find(|t| t.id == id).cloned());
        let enter = ctx.input(|i| i.key_pressed(Key::Enter)) && !ctx.text_edit_focused();
        let clicked = ui
            .add_enabled(
                pick.is_some(),
                egui::Button::new("Use template").fill(colors::ACCENT).min_size(vec2(120.0, 28.0)),
            )
            .clicked();
        if clicked || enter {
            start = start.take().or(pick);
        }
    });
    if let Some(template) = start {
        choose(app, ctx, &template);
    }
}

/// One card: the picture, the name, and what the template is. Returns (clicked, double-clicked).
fn card(ui: &mut egui::Ui, t: &OnlineTemplate, texture: Option<&TextureHandle>, selected: bool) -> (bool, bool) {
    let (rect, response) = ui.allocate_exact_size(CARD, Sense::click());
    let painter = ui.painter().clone();
    let fill = if response.hovered() { colors::HOVER } else { colors::PANEL_HEADER };
    let stroke = if selected { Stroke::new(2.0, colors::ACCENT) } else { Stroke::new(1.0, colors::DIVIDER) };
    painter.rect(rect, 6, fill, stroke, egui::StrokeKind::Inside);
    let picture = Rect::from_min_size(rect.min + vec2(7.0, 7.0), vec2(CARD.x - 14.0, 140.0));
    painter.rect_filled(picture, 4, colors::CANVAS_AREA);
    if let Some(texture) = texture {
        let size = texture.size_vec2();
        let scale = (picture.width() / size.x).min(picture.height() / size.y);
        let shown = Rect::from_center_size(picture.center(), size * scale);
        paint_checkerboard(&painter, shown, 4.0);
        painter.image(texture.id(), shown, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    let name_font = FontId::proportional(13.0);
    painter.text(pos2(rect.left() + 10.0, rect.top() + 156.0), Align2::LEFT_TOP, &t.name, name_font, colors::TEXT);
    let pages = if t.pages == 1 { "1 page".to_owned() } else { format!("{} pages", t.pages) };
    painter.text(
        pos2(rect.left() + 10.0, rect.top() + 177.0),
        Align2::LEFT_TOP,
        format!("{} \u{00B7} {pages} \u{00B7} {} x {}", t.type_name, t.width, t.height),
        FontId::proportional(11.0),
        colors::SUBTLE_TEXT,
    );
    (response.clicked(), response.double_clicked())
}
