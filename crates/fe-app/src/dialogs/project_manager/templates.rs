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
/// Space between cards.
const SPACING: f32 = 12.0;
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
    /// `category` was chosen from outside (the New Project window) and must survive the catalog arriving.
    pub wanted_category: bool,
    /// "My templates" (saved with File > Save as Template) is shown instead of the online ones.
    pub mine: bool,
    pub selected: Option<String>,
    /// The preview window of a template with several pages.
    pub preview: Option<super::template_preview::Preview>,
    textures: HashMap<String, TextureHandle>,
    requested: HashSet<String>,
    inflight: usize,
    /// "Getting ..." while a template is downloaded.
    pub busy: Option<String>,
}

/// Most card pictures kept in memory; the ones scrolled far away are dropped (they come back from the disk cache).
const KEEP_TEXTURES: usize = 120;

impl TemplatesState {
    /// Drops pictures that are not in `keep` once there are more than `KEEP_TEXTURES`.
    fn evict_except(&mut self, keep: &HashSet<String>) {
        if self.textures.len() <= KEEP_TEXTURES {
            return;
        }
        let gone: Vec<String> = self.textures.keys().filter(|k| !keep.contains(*k)).cloned().collect();
        for key in gone {
            self.textures.remove(&key);
            self.requested.remove(&key);
        }
    }

    pub fn texture(&self, key: &str) -> Option<&TextureHandle> {
        self.textures.get(key)
    }
}

/// Starts loading one picture of the catalog (card picture or a page) once; `key` names the texture.
pub fn request_picture(app: &mut FlashApp, ctx: &egui::Context, key: String, path: String) {
    if !app.manager.templates.requested.insert(key.clone()) {
        return;
    }
    app.manager.templates.inflight += 1;
    app.jobs.spawn(ctx, move || {
        let image =
            default_source().cached_or_fetch(&path).ok().and_then(|bytes| image::load_from_memory(&bytes).ok()).map(
                |img| {
                    let rgba = img.into_rgba8();
                    ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw())
                },
            );
        JobDone::TemplateThumb { id: key, image }
    });
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
            if !std::mem::take(&mut t.wanted_category) {
                t.category = None;
            }
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
            let path = fe_core::io::batch::unique_path(&folder, &stem, ".fe");
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
    let mine = file::user_templates();
    ui.horizontal_wrapped(|ui| {
        for c in catalog.categories() {
            let on = !app.manager.templates.mine && app.manager.templates.category == c.id;
            if ui.selectable_label(on, format!("{} ({})", c.name, c.count)).clicked() {
                app.manager.templates.category = c.id;
                app.manager.templates.mine = false;
            }
        }
        if ui.selectable_label(app.manager.templates.mine, format!("My templates ({})", mine.len())).clicked() {
            app.manager.templates.mine = true;
        }
    });
    ui.separator();
    if app.manager.templates.mine {
        show_mine(ui, ctx, app, &mine);
        return;
    }
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
    let mut opened = None;
    let mut visible: HashSet<String> = HashSet::new();
    egui::ScrollArea::vertical().max_height(height).auto_shrink(false).show_viewport(ui, |ui, viewport| {
        // Only the cards on screen (and one row around them) are drawn and their pictures loaded, so a catalog of
        // thousands of templates opens at once.
        let step = CARD + vec2(SPACING, SPACING);
        let columns = (((ui.available_width() + SPACING) / step.x) as usize).max(1);
        let rows = shown.len().div_ceil(columns);
        let origin = ui.min_rect().min;
        ui.set_height((rows as f32 * step.y).max(1.0));
        let first = ((viewport.min.y / step.y).floor() as usize).saturating_sub(1);
        let last = (((viewport.max.y / step.y).ceil() as usize) + 1).min(rows);
        for row in first..last {
            for column in 0..columns {
                let Some(template) = shown.get(row * columns + column) else { break };
                let key = format!("{}/{}", template.kind, template.id);
                if app.manager.templates.inflight < PARALLEL {
                    request_picture(app, ctx, key.clone(), template.thumbnail_path());
                }
                let rect = Rect::from_min_size(origin + vec2(column as f32 * step.x, row as f32 * step.y), CARD);
                let selected = app.manager.templates.selected.as_deref() == Some(&template.id);
                if card(ui, rect, template, app.manager.templates.textures.get(&key), selected) {
                    opened = Some(template.clone());
                }
                visible.insert(key);
            }
        }
    });
    app.manager.templates.evict_except(&visible);
    if let Some(template) = opened {
        app.manager.templates.selected = Some(template.id.clone());
        app.manager.templates.preview = Some(super::template_preview::Preview { template, page: 0 });
    }
    ui.horizontal(|ui| {
        let pick = app.manager.templates.selected.clone().and_then(|id| shown.iter().find(|t| t.id == id).cloned());
        let enter = ctx.input(|i| i.key_pressed(Key::Enter))
            && !ctx.text_edit_focused()
            && app.manager.templates.preview.is_none();
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
    if let Some(template) = super::template_preview::show(ctx, app) {
        start = Some(template);
    }
    if let Some(template) = start {
        choose(app, ctx, &template);
    }
}

/// One card: the picture, the name, and what the template is. Returns true when it was clicked.
fn card(ui: &mut egui::Ui, rect: Rect, t: &OnlineTemplate, texture: Option<&TextureHandle>, selected: bool) -> bool {
    let response = ui.interact(rect, egui::Id::NULL.with(("template-card", &t.kind, &t.id)), Sense::click());
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
    response.clicked()
}

/// "My templates": the projects saved with File > Save as Template, as a list.
fn show_mine(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut FlashApp, names: &[String]) {
    if names.is_empty() {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No templates of your own yet").heading());
            ui.label(RichText::new("Open a project and choose File > Save as Template.").color(colors::SUBTLE_TEXT));
        });
        return;
    }
    let mut chosen = None;
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        for name in names {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("★ {name}")).color(colors::TEXT));
                if ui.button("Use template").clicked() {
                    chosen = Some(name.clone());
                }
            });
        }
    });
    if let Some(name) = chosen {
        match file::template_pages(&name) {
            Ok(pages) => {
                app.manager.open = false;
                file::create_project(app, ctx, &name, pages);
            }
            Err(err) => crate::ai::outcome::tell(app, "Templates", format!("Could not open the template: {err}")),
        }
    }
}
