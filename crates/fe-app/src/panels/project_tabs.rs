//! The row above the canvas: the Projects button, one tab per open project, then (on the right) the New and
//! Open buttons and the view aids (C# `MainWindow.xaml`, "Projects").

use egui::{Align, Align2, Color32, FontId, Layout, Rect, Sense, Stroke, Ui, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::{colors, metrics};
use fe_ui_kit::widgets::icon_button;

use super::view_aids_bar;
use crate::app::FlashApp;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let bar = ui.max_rect();
    ui.painter().hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0, colors::DIVIDER));
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        view_aids_bar::show(ui, app);
        ui.add_space(4.0);
        if icon_button(ui, "Folder", "Open a picture or project (Ctrl+O)").clicked() {
            crate::commands::file::open_dialog(app, ui.ctx());
        }
        if icon_button(ui, "Add", "New project (Ctrl+N)").clicked() {
            app.dialog = Some(crate::dialogs::Dialog::NewProject(crate::dialogs::NewProjectForm::new(app.background)));
        }
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(6.0);
            let projects = ui
                .add(egui::Button::new(egui::RichText::new("\u{2302} Projects").size(12.0)).min_size(vec2(0.0, 24.0)))
                .on_hover_text("Project Manager: all your projects, and new project (File menu too)");
            if projects.clicked() {
                crate::dialogs::project_manager::open(app, ui.ctx());
            }
            ui.add_space(6.0);
            for index in 0..app.projects.len() {
                project_tab(ui, app, index);
            }
        });
    });
}

fn project_tab(ui: &mut Ui, app: &mut FlashApp, index: usize) {
    if app.tab_rename.as_ref().is_some_and(|(i, _)| *i == index) {
        rename_tab(ui, app, index);
        return;
    }
    let project = &app.projects[index];
    let title = if project.is_dirty() { format!("{} *", project.name) } else { project.name.clone() };
    let font = FontId::proportional(12.5);
    let text_w = ui.fonts_mut(|f| f.layout_no_wrap(title.clone(), font.clone(), Color32::WHITE).size().x).min(240.0);
    let (rect, response) =
        ui.allocate_exact_size(vec2(12.0 + text_w + 8.0 + 20.0 + 6.0, metrics::PROJECT_TAB), Sense::click());
    let selected = index == app.active_project;
    let painter = ui.painter();
    let fill = if selected {
        colors::CANVAS_AREA
    } else if response.hovered() {
        colors::HOVER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 0, fill);
    if selected {
        painter.rect_filled(Rect::from_min_size(rect.min, vec2(rect.width(), 2.0)), 0, colors::ACCENT);
    } else {
        painter.vline(rect.right() - 0.5, rect.y_range(), Stroke::new(1.0, colors::DIVIDER));
    }
    let color = if selected || response.hovered() { colors::TEXT } else { colors::SUBTLE_TEXT };
    painter.text(rect.left_center() + vec2(12.0, 0.0), Align2::LEFT_CENTER, &title, font, color);
    let close = Rect::from_center_size(rect.right_center() - vec2(6.0 + 10.0, 0.0), vec2(20.0, 20.0));
    let close_response = ui.interact(close, ui.scope_id().with(("close-project", index)), Sense::click());
    if close_response.hovered() {
        ui.painter().rect_filled(close, 4, colors::HOVER);
    }
    ui.painter().text(close.center(), Align2::CENTER_CENTER, glyphs::CLOSE, glyphs::font(9.0), color);
    if close_response.on_hover_text("Close this project (Ctrl+W)").clicked() || response.middle_clicked() {
        crate::commands::file::close_project(app, index);
    } else if response.double_clicked() {
        app.active_project = index;
        app.tab_rename = Some((index, app.projects[index].name.clone()));
    } else if response.clicked() {
        app.active_project = index;
    }
}

/// The tab as a text box: Enter keeps the new name, Esc (or clicking elsewhere) leaves the old one.
fn rename_tab(ui: &mut Ui, app: &mut FlashApp, index: usize) {
    let Some((_, text)) = app.tab_rename.as_mut() else { return };
    let (rect, _) = ui.allocate_exact_size(vec2(180.0, metrics::PROJECT_TAB), Sense::hover());
    let edit = ui.put(rect.shrink2(vec2(2.0, 3.0)), egui::TextEdit::singleline(text).desired_width(176.0));
    if !edit.has_focus() && !edit.lost_focus() {
        edit.request_focus();
    }
    let (enter, escape) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
    if escape {
        app.tab_rename = None;
    } else if (enter || edit.lost_focus())
        && let Some((_, name)) = app.tab_rename.take()
    {
        let name = crate::app::settings::clean_name(&name);
        if !name.is_empty() && name != app.projects[index].name {
            app.projects[index].name = name;
        }
    }
}
