//! The preview window of a template with several pages (like a presentation): one big page, arrows and a strip of all
//! the pages to jump to, and "Use template". Pages are small pictures from the online catalog, loaded when it opens.

use egui::{Color32, Key, Modifiers, Rect, RichText, Sense, Stroke, TextureHandle, pos2, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::paint_checkerboard;

use super::templates::{TemplatesState, request_picture};
use crate::app::FlashApp;
use crate::templates::OnlineTemplate;

const BIG: egui::Vec2 = egui::Vec2::new(760.0, 440.0);
const STRIP_HEIGHT: f32 = 78.0;

/// Which template is open and which page is shown.
pub struct Preview {
    pub template: OnlineTemplate,
    pub page: usize,
}

/// The texture key of page `number` (1 is the first) of a template.
pub fn page_key(t: &OnlineTemplate, number: usize) -> String {
    format!("{}/{}/{number}", t.kind, t.id)
}

fn fit(area: Rect, texture: &TextureHandle) -> Rect {
    let size = texture.size_vec2();
    let scale = (area.width() / size.x).min(area.height() / size.y);
    Rect::from_center_size(area.center(), size * scale)
}

fn draw_page(ui: &egui::Ui, area: Rect, texture: Option<&TextureHandle>, grid: f32) {
    ui.painter().rect_filled(area, 4, colors::CANVAS_AREA);
    match texture {
        Some(texture) => {
            let shown = fit(area, texture);
            paint_checkerboard(ui.painter(), shown, grid);
            ui.painter().image(texture.id(), shown, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            ui.painter().text(
                area.center(),
                egui::Align2::CENTER_CENTER,
                "Loading...",
                egui::FontId::proportional(13.0),
                colors::SUBTLE_TEXT,
            );
        }
    }
}

/// Shows the window when a preview is open. Returns the template when "Use template" was chosen.
pub fn show(ctx: &egui::Context, app: &mut FlashApp) -> Option<OnlineTemplate> {
    let (template, mut page) = {
        let preview = app.manager.templates.preview.as_ref()?;
        (preview.template.clone(), preview.page)
    };
    let pages = template.pages.max(1) as usize;
    for number in 1..=pages {
        request_picture(app, ctx, page_key(&template, number), template.page_path(number));
    }
    let state: &TemplatesState = &app.manager.templates;
    let (mut use_it, mut close) = (false, false);
    super::super::frame::modal_over(ctx, "template-preview", |ui| {
        ui.set_width(BIG.x);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&template.name).heading());
            ui.label(
                RichText::new(format!(
                    "{} \u{00B7} {pages} pages \u{00B7} {} x {}",
                    template.type_name, template.width, template.height
                ))
                .size(11.5)
                .color(colors::SUBTLE_TEXT),
            );
        });
        ui.add_space(6.0);
        let (area, _) = ui.allocate_exact_size(BIG, Sense::hover());
        draw_page(ui, area, state.texture(&page_key(&template, page + 1)), 12.0);
        // Arrows over the left and right edge of the big picture.
        for (left, label, enabled) in [(true, "\u{25C0}", page > 0), (false, "\u{25B6}", page + 1 < pages)] {
            let size = vec2(34.0, 60.0);
            let x = if left { area.left() + 6.0 } else { area.right() - 6.0 - size.x };
            let rect = Rect::from_min_size(pos2(x, area.center().y - size.y / 2.0), size);
            let response = ui.interact(
                rect,
                egui::Id::NULL.with(("page-arrow", left)),
                if enabled { Sense::click() } else { Sense::hover() },
            );
            let fill = match (enabled, response.hovered()) {
                (false, _) => Color32::from_black_alpha(60),
                (true, true) => Color32::from_black_alpha(220),
                (true, false) => Color32::from_black_alpha(150),
            };
            ui.painter().rect_filled(rect, 6, fill);
            let ink = if enabled { Color32::WHITE } else { Color32::from_gray(110) };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(16.0), ink);
            if response.clicked() {
                page = if left { page - 1 } else { page + 1 };
            }
        }
        ui.add_space(8.0);
        ui.label(RichText::new(format!("Page {} of {pages}", page + 1)).size(11.5).color(colors::SUBTLE_TEXT));
        egui::ScrollArea::horizontal().id_salt("template-pages").max_height(STRIP_HEIGHT + 14.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                for number in 0..pages {
                    let aspect = template.width as f32 / template.height.max(1) as f32;
                    let size = vec2((STRIP_HEIGHT * aspect).clamp(40.0, 140.0), STRIP_HEIGHT);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                    draw_page(ui, rect, state.texture(&page_key(&template, number + 1)), 4.0);
                    let stroke = if number == page {
                        Stroke::new(2.0, colors::ACCENT)
                    } else {
                        Stroke::new(1.0, colors::DIVIDER)
                    };
                    ui.painter().rect_stroke(rect, 4, stroke, egui::StrokeKind::Outside);
                    if response.clicked() {
                        page = number;
                    }
                }
            });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Use template").fill(colors::ACCENT).min_size(vec2(130.0, 30.0))).clicked() {
                use_it = true;
            }
            if ui.add(egui::Button::new("Close").min_size(vec2(90.0, 30.0))).clicked() {
                close = true;
            }
        });
    });
    // Keys: arrows turn pages, Esc closes only this window (not the Project Manager under it).
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        close = true;
    }
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) && page > 0 {
        page -= 1;
    }
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) && page + 1 < pages {
        page += 1;
    }
    if close || use_it {
        app.manager.templates.preview = None;
    } else if let Some(preview) = app.manager.templates.preview.as_mut() {
        preview.page = page;
    }
    use_it.then_some(template)
}
