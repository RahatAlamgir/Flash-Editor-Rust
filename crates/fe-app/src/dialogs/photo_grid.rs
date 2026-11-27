//! Insert > Photo Grid (C# `PhotoGridDialog`): pick a collage layout; it is added as empty frames to fill with
//! pictures. A small preview shows the frames on a canvas of the same shape.

use egui::{Color32, Rect, RichText, Sense, Stroke, pos2, vec2};
use fe_core::objects::COLLAGE_LAYOUTS;
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

/// What the dialog gave back.
#[derive(Clone, Copy, Debug)]
pub struct PhotoGridSettings {
    pub layout: usize,
    pub gap: f64,
    pub padding: f64,
    pub corner_radius: f64,
}

/// The open dialog.
pub struct PhotoGridForm {
    pub settings: PhotoGridSettings,
    canvas: (f64, f64),
}

impl PhotoGridForm {
    pub fn new(canvas_w: u32, canvas_h: u32) -> Self {
        Self {
            settings: PhotoGridSettings { layout: 4, gap: 12.0, padding: 12.0, corner_radius: 8.0 },
            canvas: (f64::from(canvas_w), f64::from(canvas_h)),
        }
    }
}

/// `Some(true)` = Add, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, form: &mut PhotoGridForm) -> Option<bool> {
    let mut result = None;
    let modal = super::frame::modal(ctx, "photo-grid", |ui| {
        ui.set_width(440.0);
        ui.label(RichText::new("Photo Grid").heading());
        ui.add_space(6.0);
        ui.label(
            RichText::new("Pick a layout. It adds empty frames to fill with your pictures.").color(colors::SUBTLE_TEXT),
        );
        ui.add_space(8.0);
        let s = &mut form.settings;
        ui.horizontal(|ui| {
            ui.label("Layout");
            egui::ComboBox::from_id_salt("collage-layout")
                .width(250.0)
                .selected_text(COLLAGE_LAYOUTS[s.layout].name)
                .show_ui(ui, |ui| {
                    for (i, layout) in COLLAGE_LAYOUTS.iter().enumerate() {
                        ui.selectable_value(&mut s.layout, i, layout.name);
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Gap      ");
            number_slider(ui, &mut s.gap, 0.0..=100.0, "px", 160.0, 52.0);
        });
        ui.horizontal(|ui| {
            ui.label("Border  ");
            number_slider(ui, &mut s.padding, 0.0..=100.0, "px", 160.0, 52.0);
        });
        ui.horizontal(|ui| {
            ui.label("Corners");
            number_slider(ui, &mut s.corner_radius, 0.0..=100.0, "px", 160.0, 52.0);
        });
        ui.add_space(8.0);
        preview(ui, form);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Add frames").fill(colors::ACCENT).min_size(vec2(110.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(vec2(90.0, 28.0))).clicked() {
                result = Some(false);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}

/// A small picture of the frames on a canvas of the same shape.
fn preview(ui: &mut egui::Ui, form: &PhotoGridForm) {
    let (area, _) = ui.allocate_exact_size(vec2(400.0, 190.0), Sense::hover());
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 2, colors::PANEL_HEADER);
    let (cw, ch) = form.canvas;
    let scale = (f64::from(area.width()) / cw).min(f64::from(area.height()) / ch) as f32 * 0.96;
    let size = vec2(cw as f32 * scale, ch as f32 * scale);
    let paper = Rect::from_center_size(area.center(), size);
    painter.rect_filled(paper, 0, Color32::WHITE);
    let s = form.settings;
    for (x, y, w, h) in COLLAGE_LAYOUTS[s.layout].place(cw, ch, s.padding, s.gap) {
        let r = Rect::from_min_size(
            pos2(paper.left() + x as f32 * scale, paper.top() + y as f32 * scale),
            vec2((w as f32 * scale).max(1.0), (h as f32 * scale).max(1.0)),
        );
        painter.rect(
            r,
            (s.corner_radius as f32 * scale).min(r.height() / 2.0),
            Color32::from_rgb(203, 213, 225),
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );
    }
}
