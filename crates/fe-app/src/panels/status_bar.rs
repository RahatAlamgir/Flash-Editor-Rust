//! The status bar: a message on the left; cursor position, canvas size (click = Image Size) and zoom on the right.

use egui::{Align, FontId, Layout, RichText, Stroke, Ui};
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let rect = ui.max_rect();
    ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, colors::DIVIDER));
    let doc = &app.project().active_page().doc;
    let size = format!("{} x {} px", doc.width(), doc.height());
    let zoom = format!("{:.1}%", app.zoom * 100.0);
    let font = FontId::proportional(12.0);
    let mut size_clicked = false;
    let mut cancel_ai = false;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(10.0);
        if let Some(job) = &app.ai.job {
            // The AI is working: a progress bar and a Cancel button (the window stays usable).
            cancel_ai = ui.small_button("Cancel").clicked();
            let bar = egui::ProgressBar::new(job.fraction()).desired_width(170.0).text(format!(
                "{} {:.0}%",
                job.title,
                job.fraction() * 100.0
            ));
            ui.add(bar);
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
            ui.add_space(16.0);
        }
        ui.add_sized([60.0, 20.0], egui::Label::new(RichText::new(zoom).font(font.clone()).color(colors::TEXT)));
        ui.add_space(16.0);
        let link = egui::Label::new(RichText::new(size).font(font.clone()).color(colors::SUBTLE_TEXT).underline())
            .sense(egui::Sense::click());
        size_clicked = ui.add(link).on_hover_text("Click to change the image size / resolution").clicked();
        ui.add_space(24.0);
        ui.add_sized(
            [110.0, 20.0],
            egui::Label::new(RichText::new(&app.cursor_text).font(font.clone()).color(colors::SUBTLE_TEXT)),
        );
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(10.0);
            ui.add(egui::Label::new(RichText::new(&app.status).font(font).color(colors::TEXT)).truncate());
        });
    });
    if cancel_ai && let Some(job) = &app.ai.job {
        job.cancel();
    }
    if size_clicked {
        app.not_yet("Image Size");
    }
}
