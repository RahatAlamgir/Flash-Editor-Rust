//! Insert > Icons, Badges and Graphics (C# `IconPickerDialog`): a grid of pictures with categories and a search box.
//! Click one to put it in the middle of the picture.

use egui::{Color32, Rect, RichText, Sense, Stroke, vec2};
use fe_core::library::{badges, graphics, icons, material};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::colors;

use super::previews::Previews;

/// The page of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Icons,
    /// Google's Material icons (a few thousand, with search).
    Material,
    Badges,
    Graphics,
}

/// What was picked.
#[derive(Clone, Debug)]
pub enum Pick {
    Icon(usize),
    /// A Google icon, by its code in the font.
    Material(u32),
    Badge(&'static str),
    Graphic(&'static str),
}

/// The open dialog.
pub struct IconPicker {
    page: Page,
    search: String,
    category: Option<usize>,
}

impl IconPicker {
    pub fn new(page: Page) -> Self {
        Self { page, search: String::new(), category: None }
    }
}

/// Every word of the search must be in the name or the category.
fn matches(query: &str, name: &str, category: &str) -> bool {
    let (name, category) = (name.to_lowercase(), category.to_lowercase());
    query.split_whitespace().all(|w| name.contains(&w.to_lowercase()) || category.contains(&w.to_lowercase()))
}

/// `Some(Some(pick))` = something was picked, `Some(None)` = closed, `None` = still open.
pub fn show(ctx: &egui::Context, picker: &mut IconPicker, previews: &mut Previews) -> Option<Option<Pick>> {
    let mut result = None;
    let modal = super::frame::modal(ctx, "icon-picker", |ui| {
        ui.set_width(640.0);
        ui.label(RichText::new("Icons, badges and graphics").heading());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (page, name) in [
                (Page::Icons, "Icons"),
                (Page::Material, "Google icons"),
                (Page::Badges, "Badges"),
                (Page::Graphics, "Graphics"),
            ] {
                if ui.selectable_label(picker.page == page, RichText::new(name).size(14.0)).clicked() {
                    picker.page = page;
                    // The Google icons start with the popular ones: the whole list is a few thousand.
                    picker.category = if page == Page::Material { Some(0) } else { None };
                }
            }
        });
        ui.add_space(6.0);
        let names: &[&str] = match picker.page {
            Page::Icons => &icons::CATEGORIES,
            Page::Material => &["Popular"],
            Page::Badges => &badges::CATEGORIES,
            Page::Graphics => &graphics::CATEGORIES,
        };
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut picker.search).hint_text("Search").desired_width(220.0));
            let all = match picker.page {
                Page::Icons | Page::Material => "All icons",
                Page::Badges => "All badges",
                Page::Graphics => "All graphics",
            };
            egui::ComboBox::from_id_salt("icon-category")
                .width(180.0)
                .selected_text(picker.category.map_or(all, |i| names[i]))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut picker.category, None, all);
                    for (i, n) in names.iter().enumerate() {
                        ui.selectable_value(&mut picker.category, Some(i), *n);
                    }
                });
        });
        ui.add_space(8.0);
        let wanted = picker.category.map(|i| names[i]);
        let query = picker.search.trim().to_owned();
        let mut count = 0;
        if picker.page == Page::Material {
            count = material_grid(ui, &query, picker.category.is_some(), &mut result);
        } else {
            egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    match picker.page {
                        Page::Material => {}
                        Page::Icons => {
                            for (i, icon) in icons::ALL.iter().enumerate() {
                                if wanted.is_some_and(|w| w != icon.category)
                                    || !matches(&query, icon.name, icon.category)
                                {
                                    continue;
                                }
                                count += 1;
                                let (rect, response) = ui.allocate_exact_size(vec2(62.0, 62.0), Sense::click());
                                card(ui, rect, response.hovered());
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    icon.glyph(),
                                    glyphs::font(28.0),
                                    colors::TEXT,
                                );
                                if response.on_hover_text(icon.name).clicked() {
                                    result = Some(Some(Pick::Icon(i)));
                                }
                            }
                        }
                        Page::Badges => {
                            for info in &badges::ALL {
                                if wanted.is_some_and(|w| w != info.category)
                                    || !matches(&query, info.name, info.category)
                                {
                                    continue;
                                }
                                count += 1;
                                if preview_card(ui, ctx, previews.badge(ctx, info.id), info.name) {
                                    result = Some(Some(Pick::Badge(info.id)));
                                }
                            }
                        }
                        Page::Graphics => {
                            for info in graphics::all() {
                                if wanted.is_some_and(|w| w != info.category)
                                    || !matches(&query, info.name, info.category)
                                {
                                    continue;
                                }
                                count += 1;
                                if preview_card(ui, ctx, previews.graphic(ctx, info.id), info.name) {
                                    result = Some(Some(Pick::Graphic(info.id)));
                                }
                            }
                        }
                    }
                });
            });
        }
        ui.add_space(8.0);
        if count == 0 {
            ui.label(RichText::new("Nothing matches. Try another word.").size(11.5).color(colors::SUBTLE_TEXT));
        }
        ui.add_space(10.0);
        if ui.add(egui::Button::new("Close").min_size(vec2(90.0, 28.0))).clicked() {
            result = Some(None);
        }
    });
    if result.is_none() && modal.should_close() {
        result = Some(None);
    }
    result
}

/// The background of a grid card.
fn card(ui: &egui::Ui, rect: Rect, hovered: bool) {
    let fill = if hovered { colors::HOVER } else { colors::PANEL_HEADER };
    let stroke = if hovered { Stroke::new(1.5, colors::ACCENT) } else { Stroke::new(1.0, colors::DIVIDER) };
    ui.painter().rect(rect, 6, fill, stroke, egui::StrokeKind::Inside);
}

/// A card with a picture and a name. True when it was clicked.
fn preview_card(ui: &mut egui::Ui, _ctx: &egui::Context, texture: Option<&egui::TextureHandle>, name: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(112.0, 124.0), Sense::click());
    card(ui, rect, response.hovered());
    let picture = Rect::from_min_size(rect.min + vec2(11.0, 6.0), vec2(90.0, 90.0));
    ui.painter().rect_filled(picture, 4, Color32::from_rgb(238, 238, 242));
    if let Some(t) = texture {
        ui.painter().image(
            t.id(),
            picture,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    ui.painter().text(
        rect.center_bottom() - vec2(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(11.0),
        colors::SUBTLE_TEXT,
    );
    response.on_hover_text(name).clicked()
}

/// The Google icons: only the rows on screen are drawn (the whole list is thousands). Returns how many icons match.
fn material_grid(ui: &mut egui::Ui, query: &str, popular: bool, result: &mut Option<Option<Pick>>) -> usize {
    const COLUMNS: usize = 9;
    const CARD: f32 = 62.0;
    let list: Vec<&material::MaterialIcon> = if !query.is_empty() {
        material::all().iter().filter(|i| material::matches(query, i)).collect()
    } else if popular {
        material::popular()
    } else {
        material::all().iter().collect()
    };
    let rows = list.len().div_ceil(COLUMNS);
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
    egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show_rows(ui, CARD, rows, |ui, range| {
        for row in range {
            ui.horizontal(|ui| {
                for icon in list.iter().skip(row * COLUMNS).take(COLUMNS) {
                    let (rect, response) = ui.allocate_exact_size(vec2(CARD, CARD), Sense::click());
                    card(ui, rect, response.hovered());
                    let font =
                        egui::FontId::new(30.0, egui::FontFamily::Name(fe_ui_kit::theme::fonts::MATERIAL.into()));
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, icon.glyph(), font, colors::TEXT);
                    if response.on_hover_text(&icon.name).clicked() {
                        *result = Some(Some(Pick::Material(icon.code)));
                    }
                }
            });
        }
    });
    list.len()
}
