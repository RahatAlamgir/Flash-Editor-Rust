//! Insert > Icons, Badges and Graphics (C# `IconPickerDialog`): a grid of pictures with categories and a search box.
//! Click one to put it in the middle of the picture.

use egui::{Color32, Id, Rect, RichText, Sense, Stroke, vec2};
use fe_core::library::{badges, graphics, icons};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::colors;

use super::previews::Previews;

/// The page of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Icons,
    Badges,
    Graphics,
}

/// What was picked.
#[derive(Clone, Debug)]
pub enum Pick {
    Icon(usize),
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

/// The color of an icon group (like the C# `Icon.<color>` resources).
fn category_color(category: &str) -> Color32 {
    match category {
        "Arrows" => Color32::from_rgb(59, 130, 246),
        "Symbols" => Color32::from_rgb(245, 158, 11),
        "Communication" => Color32::from_rgb(34, 197, 94),
        "Media" => Color32::from_rgb(236, 72, 153),
        "Objects" => Color32::from_rgb(249, 115, 22),
        "Places and travel" => Color32::from_rgb(20, 184, 166),
        "Weather and nature" => Color32::from_rgb(56, 189, 248),
        _ => Color32::from_rgb(139, 92, 246),
    }
}

/// `Some(Some(pick))` = something was picked, `Some(None)` = closed, `None` = still open.
pub fn show(ctx: &egui::Context, picker: &mut IconPicker, previews: &mut Previews) -> Option<Option<Pick>> {
    let mut result = None;
    let modal = egui::Modal::new(Id::NULL.with("icon-picker")).show(ctx, |ui| {
        ui.set_width(640.0);
        ui.label(RichText::new("Icons, badges and graphics").heading());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (page, name) in [(Page::Icons, "Icons"), (Page::Badges, "Badges"), (Page::Graphics, "Graphics")] {
                if ui.selectable_label(picker.page == page, RichText::new(name).size(14.0)).clicked() {
                    picker.page = page;
                    picker.category = None;
                }
            }
        });
        ui.add_space(6.0);
        let names: &[&str] = match picker.page {
            Page::Icons => &icons::CATEGORIES,
            Page::Badges => &badges::CATEGORIES,
            Page::Graphics => &graphics::CATEGORIES,
        };
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut picker.search).hint_text("Search").desired_width(220.0));
            let all = match picker.page {
                Page::Icons => "All icons",
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
        egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                match picker.page {
                    Page::Icons => {
                        for (i, icon) in icons::ALL.iter().enumerate() {
                            if wanted.is_some_and(|w| w != icon.category) || !matches(&query, icon.name, icon.category) {
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
                                category_color(icon.category),
                            );
                            if response.on_hover_text(icon.name).clicked() {
                                result = Some(Some(Pick::Icon(i)));
                            }
                        }
                    }
                    Page::Badges => {
                        for info in &badges::ALL {
                            if wanted.is_some_and(|w| w != info.category) || !matches(&query, info.name, info.category) {
                                continue;
                            }
                            count += 1;
                            if preview_card(ui, ctx, previews.badge(ctx, info.id), info.name) {
                                result = Some(Some(Pick::Badge(info.id)));
                            }
                        }
                    }
                    Page::Graphics => {
                        for info in &graphics::ALL {
                            if wanted.is_some_and(|w| w != info.category) || !matches(&query, info.name, info.category) {
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
        ui.add_space(8.0);
        let hint = match (count, picker.page) {
            (0, _) => "Nothing matches. Try another word.",
            (_, Page::Icons) => "Click an icon to put it in the middle of your picture. It is text: change its color, size, outline and shadow in Properties.",
            (_, Page::Badges) => "Click a badge to put it in the middle of your picture. It is a group: ungroup it (Ctrl+Shift+G) to change the colors and words.",
            (_, Page::Graphics) => "Click a graphic to put it in the middle of your picture. It is a group of shapes: ungroup it (Ctrl+Shift+G) to recolor or change its parts.",
        };
        ui.add(egui::Label::new(RichText::new(hint).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
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
