//! The File drawer (C# `FileDrawer`): slides in from the left over the window and lists everything about files (new, open,
//! save, export, pages...). Esc, a click on the dimmed area or a command closes it.

use egui::{Align2, Color32, Id, Order, Rect, RichText, Sense, pos2, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::colors::{self, icon};

use crate::app::FlashApp;
use crate::app::shortcuts;
use crate::commands::export::{self, PagesKind};
use crate::commands::{file, pages};

const WIDTH: f32 = 320.0;

#[derive(Clone, Copy)]
enum Item {
    /// A command of the shortcut list (its shortcut is shown).
    Command(&'static str),
    Special(&'static str),
}

struct Row {
    label: &'static str,
    glyph: char,
    color: Color32,
    item: Item,
    tip: &'static str,
}

fn row(label: &'static str, glyph: char, color: Color32, item: Item, tip: &'static str) -> Row {
    Row { label, glyph, color, item, tip }
}

fn sections() -> Vec<(&'static str, Vec<Row>)> {
    use Item::{Command, Special};
    let (blue, green, red, sky) = (icon::BLUE, icon::GREEN, icon::RED, icon::SKY);
    vec![
        (
            "Project",
            vec![
                row("Project Manager", '\u{E80F}', blue, Special("manager"), "All your projects, and create a new one"),
                row("New...", '\u{E7C3}', green, Command("New"), "New picture"),
                row("Open (picture or project)", '\u{E838}', blue, Command("Open"), "Open a picture or a project"),
                row("Place Image as Layer...", '\u{E91B}', green, Special("place"), ""),
                row("Close Project", '\u{E8BB}', red, Command("Close project"), ""),
            ],
        ),
        (
            "Save",
            vec![
                row(
                    "Save Project",
                    '\u{E74E}',
                    blue,
                    Command("Save project"),
                    "Saves a .fe file that keeps every layer, shape and text editable",
                ),
                row("Save Project As", '\u{E792}', blue, Command("Save project as"), ""),
                row(
                    "Save as Template",
                    '\u{E8A1}',
                    blue,
                    Special("template"),
                    "Keep this picture as a starting point for new pictures (Project Manager > Templates > My templates)",
                ),
            ],
        ),
        (
            "Pages",
            vec![
                row(
                    "Add Page",
                    '\u{E8F4}',
                    green,
                    Command("Add page"),
                    "A new empty page after this one. All pages are saved together in one .fe file.",
                ),
                row("Duplicate Page", '\u{E8C8}', sky, Special("duplicate-page"), ""),
                row(
                    "Presentation (on / off)",
                    '\u{E7F4}',
                    sky,
                    Special("presentation"),
                    "Makes this project a presentation: the Animation tab (slide transitions, slideshow) appears. Click again to turn it off.",
                ),
                row("Close Page", '\u{E711}', red, Special("close-page"), ""),
                row("Next Picture", '\u{E72A}', blue, Command("Next picture"), ""),
            ],
        ),
        (
            "Export",
            vec![
                row(
                    "As Picture (PNG, JPG, WEBP...)",
                    '\u{E78B}',
                    blue,
                    Command("Export as picture"),
                    "Makes a normal picture file; choose format, quality and size",
                ),
                row(
                    "For Social Media, Video, Print...",
                    '\u{E72D}',
                    blue,
                    Special("export-for"),
                    "A ready-made size for social media, video, print or the web; also for all pages at once",
                ),
                row(
                    "All Pages as Pictures",
                    '\u{E8F1}',
                    blue,
                    Special("export-all"),
                    "One picture file for every page",
                ),
                row(
                    "As PDF (all pages)",
                    '\u{E8A5}',
                    blue,
                    Special("pdf"),
                    "One PDF page for every page of the project",
                ),
                row(
                    "As PowerPoint (.pptx)",
                    '\u{E7BE}',
                    blue,
                    Special("pptx"),
                    "One slide for every page of the project",
                ),
                row(
                    "As Animated GIF (all pages)",
                    '\u{E768}',
                    blue,
                    Special("gif"),
                    "Every page becomes one frame of an animated GIF",
                ),
                row(
                    "As SVG (vector)",
                    '\u{E8A9}',
                    blue,
                    Special("svg"),
                    "Shapes and text stay sharp vectors; effects and pictures are embedded",
                ),
                row(
                    "As Photoshop File (.psd, with layers)",
                    '\u{E81E}',
                    blue,
                    Special("psd"),
                    "Saves every layer so Photoshop and other editors can open it",
                ),
                row(
                    "Batch Convert Folder",
                    '\u{E895}',
                    blue,
                    Special("batch"),
                    "Convert every picture in a folder to another format and / or size",
                ),
            ],
        ),
        ("App", vec![row("Exit", '\u{E7E8}', red, Special("exit"), "")]),
    ]
}

fn run_special(app: &mut FlashApp, ctx: &egui::Context, id: &str) {
    match id {
        "manager" => crate::dialogs::project_manager::open(app, ctx),
        "place" => file::place_image(app, ctx),
        "template" => app.dialog = Some(crate::dialogs::Dialog::SaveTemplate(app.project().name.clone())),
        "duplicate-page" => pages::duplicate(app, &[app.project().active_index()]),
        "presentation" => {
            let project = app.project_mut();
            project.presentation = !project.presentation;
            project.history_mut().mark_unsaved();
            let on = project.presentation;
            app.status = if on {
                "This project is now a presentation: see the Animation tab.".into()
            } else {
                "This project is no longer a presentation.".into()
            };
        }
        "close-page" => pages::ask_delete(app, vec![app.project().active_index()]),
        "export-for" => export::open_export_for(app),
        "export-all" => export::open_export_all(app),
        "pdf" => export::open_pages_export(app, PagesKind::Pdf),
        "pptx" => export::open_pages_export(app, PagesKind::Pptx),
        "gif" => export::open_pages_export(app, PagesKind::Gif),
        "svg" => export::export_document(app, true),
        "psd" => export::export_document(app, false),
        "batch" => export::open_batch(app),
        "exit" => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        _ => {}
    }
}

/// Every drawer item in order (for the command search): name and icon.
pub fn flat_items() -> Vec<(&'static str, char, &'static str, &'static str)> {
    sections()
        .into_iter()
        .flat_map(|(heading, rows)| rows.into_iter().map(move |r| (r.label, r.glyph, r.tip, heading)))
        .collect()
}

/// Runs the drawer item at this position of [`flat_items`].
pub fn run_flat(app: &mut FlashApp, ctx: &egui::Context, index: usize) {
    let Some(r) = sections().into_iter().flat_map(|(_, rows)| rows).nth(index) else { return };
    match r.item {
        Item::Command(title) => {
            shortcuts::run(app, ctx, title);
        }
        Item::Special(id) => run_special(app, ctx, id),
    }
}

/// Draws the drawer while it is open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp) {
    // Slide in (and out) over a fraction of a second.
    let t = ctx.animate_bool_with_time(Id::NULL.with("file-drawer"), app.file_menu, 0.16);
    if t <= 0.0 {
        return;
    }
    let screen = ctx.content_rect();
    let mut chosen = None;
    let mut close = false;
    egui::Area::new(Id::NULL.with("file-drawer-area")).order(Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
        // The dimmed area: a click closes the drawer.
        let scrim = ui.allocate_rect(screen, Sense::click());
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha((120.0 * t) as u8));
        if scrim.clicked() {
            close = true;
        }
        let x = -WIDTH * (1.0 - t);
        let panel = Rect::from_min_size(pos2(x, screen.top()), vec2(WIDTH, screen.height()));
        ui.painter().rect_filled(panel, 0.0, colors::PANEL);
        ui.painter().vline(panel.right(), panel.y_range(), egui::Stroke::new(1.0, colors::DIVIDER));
        ui.scope_builder(egui::UiBuilder::new().max_rect(panel.shrink2(vec2(10.0, 0.0))), |ui| {
            ui.add_space(18.0);
            ui.label(RichText::new("File").size(22.0).strong().color(colors::TEXT));
            ui.add_space(6.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for (heading, rows) in sections() {
                    ui.add_space(10.0);
                    ui.label(RichText::new(heading.to_uppercase()).size(11.0).strong().color(colors::SUBTLE_TEXT));
                    for r in rows {
                        let (rect, response) = ui.allocate_exact_size(vec2(WIDTH - 20.0, 30.0), Sense::click());
                        if response.hovered() {
                            ui.painter().rect_filled(rect, 4.0, colors::HOVER);
                        }
                        let color = if app.ribbon.colorful { r.color } else { colors::ICON };
                        ui.painter().text(
                            pos2(rect.left() + 16.0, rect.center().y),
                            Align2::CENTER_CENTER,
                            r.glyph,
                            glyphs::font(16.0),
                            color,
                        );
                        ui.painter().text(
                            pos2(rect.left() + 38.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            r.label,
                            egui::FontId::proportional(13.0),
                            colors::TEXT,
                        );
                        if let Item::Command(title) = r.item {
                            let keys = shortcuts::text_for(&app.settings, title);
                            ui.painter().text(
                                pos2(rect.right() - 8.0, rect.center().y),
                                Align2::RIGHT_CENTER,
                                keys,
                                egui::FontId::proportional(11.0),
                                colors::SUBTLE_TEXT,
                            );
                        }
                        let response = if r.tip.is_empty() { response } else { response.on_hover_text(r.tip) };
                        if response.clicked() {
                            chosen = Some(r.item);
                        }
                    }
                }
            });
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
    }
    if let Some(item) = chosen {
        close = true;
        match item {
            Item::Command(title) => {
                shortcuts::run(app, ctx, title);
            }
            Item::Special(id) => run_special(app, ctx, id),
        }
    }
    if close {
        app.file_menu = false;
    }
}
