//! Insert > SmartArt (C# `SmartArtDialog`): pick a diagram from the pictures, type your words (one per line), choose
//! colors.

use egui::{Color32, Rect, RichText, Sense, Stroke, vec2};
use fe_core::library::smartart::{self, CATEGORIES, LAYOUTS};
use fe_ui_kit::theme::colors;

use super::previews::Previews;

/// What the dialog gave back.
pub struct SmartArtChoice {
    pub layout: &'static str,
    pub words: Vec<String>,
    pub scheme: String,
}

/// The open dialog.
pub struct SmartArtForm {
    selected: usize,
    scheme: usize,
    category: Option<usize>,
    words: String,
    /// The sample words of the last layout: new samples replace the words only while they were not changed.
    last_sample: String,
}

impl SmartArtForm {
    pub fn new() -> Self {
        let sample = LAYOUTS[0].sample.join("\n");
        Self { selected: 0, scheme: 0, category: None, words: sample.clone(), last_sample: sample }
    }

    fn select(&mut self, index: usize) {
        let untouched = self.words.trim() == self.last_sample.trim() || self.words.trim().is_empty();
        self.selected = index;
        if untouched {
            self.last_sample = LAYOUTS[index].sample.join("\n");
            self.words = self.last_sample.clone();
        }
    }
}

impl Default for SmartArtForm {
    fn default() -> Self {
        Self::new()
    }
}

/// `Some(Some(choice))` = Insert, `Some(None)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, form: &mut SmartArtForm, previews: &mut Previews) -> Option<Option<SmartArtChoice>> {
    let mut result = None;
    let schemes = smartart::schemes();
    let modal = super::frame::modal(ctx, "smartart", |ui| {
        ui.set_width(660.0);
        ui.label(RichText::new("SmartArt").heading());
        ui.add_space(6.0);
        ui.label(RichText::new("Pick a diagram, type your words, choose colors.").color(colors::SUBTLE_TEXT));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Colors");
            egui::ComboBox::from_id_salt("smartart-scheme").width(120.0).selected_text(schemes[form.scheme].0).show_ui(
                ui,
                |ui| {
                    for (i, (name, _)) in schemes.iter().enumerate() {
                        ui.selectable_value(&mut form.scheme, i, *name);
                    }
                },
            );
            ui.add_space(12.0);
            ui.label("Show");
            egui::ComboBox::from_id_salt("smartart-category")
                .width(150.0)
                .selected_text(form.category.map_or("All diagrams", |i| CATEGORIES[i]))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut form.category, None, "All diagrams");
                    for (i, n) in CATEGORIES.iter().enumerate() {
                        ui.selectable_value(&mut form.category, Some(i), *n);
                    }
                });
        });
        ui.add_space(8.0);
        let scheme = schemes[form.scheme].0;
        let mut clicked = None;
        egui::ScrollArea::vertical().max_height(300.0).auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for (i, layout) in LAYOUTS.iter().enumerate() {
                    if form.category.is_some_and(|c| CATEGORIES[c] != layout.category) {
                        continue;
                    }
                    let (rect, response) = ui.allocate_exact_size(vec2(138.0, 152.0), Sense::click());
                    let chosen = form.selected == i;
                    let stroke =
                        if chosen { Stroke::new(2.0, colors::ACCENT) } else { Stroke::new(1.0, colors::DIVIDER) };
                    let fill = if response.hovered() { colors::HOVER } else { colors::PANEL_HEADER };
                    ui.painter().rect(rect, 6, fill, stroke, egui::StrokeKind::Inside);
                    let picture = Rect::from_min_size(rect.min + vec2(13.0, 6.0), vec2(112.0, 112.0));
                    ui.painter().rect_filled(picture, 4, Color32::from_rgb(238, 238, 242));
                    if let Some(t) = previews.smartart(ctx, layout.id, scheme) {
                        ui.painter().image(
                            t.id(),
                            picture,
                            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    ui.painter().text(
                        rect.center_bottom() - vec2(0.0, 16.0),
                        egui::Align2::CENTER_CENTER,
                        layout.name,
                        egui::FontId::proportional(11.5),
                        colors::TEXT,
                    );
                    let response = response.on_hover_text(layout.name);
                    if response.clicked() {
                        clicked = Some(i);
                    }
                    if response.double_clicked() {
                        result = Some(Some(()));
                    }
                }
            });
        });
        if let Some(i) = clicked {
            form.select(i);
        }
        let layout = &LAYOUTS[form.selected];
        ui.add_space(8.0);
        let title = if layout.min == layout.max {
            format!("Your words (one per line, {} needed)", layout.max)
        } else {
            format!("Your words (one per line, {} to {})", layout.min, layout.max)
        };
        ui.label(title);
        ui.add(egui::TextEdit::multiline(&mut form.words).desired_rows(4).desired_width(f32::INFINITY));
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{}: the text and shapes become a group. Ungroup it (Ctrl+Shift+G) to move or recolor single parts.",
                    layout.name
                ))
                .size(11.5)
                .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Insert").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
                result = Some(Some(()));
            }
            if ui.add(egui::Button::new("Cancel").min_size(vec2(90.0, 28.0))).clicked() {
                result = Some(None);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(None);
    }
    result.map(|r| {
        r.map(|()| {
            let layout = &LAYOUTS[form.selected];
            let mut words: Vec<String> =
                form.words.lines().map(|l| l.trim().to_owned()).filter(|l| !l.is_empty()).collect();
            if words.is_empty() {
                words = layout.sample.iter().map(|s| (*s).to_owned()).collect();
            }
            SmartArtChoice { layout: layout.id, words, scheme: schemes[form.scheme].0.to_owned() }
        })
    })
}
