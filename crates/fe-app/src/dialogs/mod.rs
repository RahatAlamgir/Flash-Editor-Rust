//! Dialog windows. One file per dialog. Only one ordinary dialog is open at a time (`FlashApp::dialog`); the
//! Project Manager has its own state because it lists many files.

mod ask;
mod batch;
mod brand_kit;
mod chart_picker;
pub mod color_picker;
pub mod effect;
pub mod export;
mod export_crop;
mod export_for;
pub mod frame;
mod icon_picker;
mod magic_resize;
mod new_project;
mod pages_export;
mod photo_grid;
pub mod previews;
pub mod project_manager;
pub mod remove_bg;
mod save_style;
mod save_template;
mod shortcuts;
mod size;
mod smartart;
mod split_sheet;
pub mod upscale;

pub use ask::{Answer, Ask};
pub use batch::BatchForm;
pub use brand_kit::BrandKit;
pub use color_picker::{ColorPicker, ColorTarget, EffectColor};
pub use export_for::ExportForForm;
pub use icon_picker::{IconPicker, Page as IconPage, Pick};
pub use magic_resize::MagicResizeForm;
pub use new_project::NewProjectForm;
pub use pages_export::PagesForm;
pub use photo_grid::{PhotoGridForm, PhotoGridSettings};
pub use shortcuts::ShortcutsForm;
pub use size::{CanvasSizeForm, ImageOp, ImageOpForm, ImageSizeForm};
pub use smartart::SmartArtForm;
pub use split_sheet::SplitSheetForm;

use crate::app::FlashApp;
use crate::commands::{file, pages};

/// The open dialog.
pub enum Dialog {
    Ask(Ask),
    NewProject(NewProjectForm),
    SaveTemplate(String),
    Color(ColorPicker),
    SaveStyle(String),
    PhotoGrid(PhotoGridForm),
    Icons(IconPicker),
    BrandKit(BrandKit),
    Effect(Box<crate::commands::effects::EffectSession>),
    ImageSize(ImageSizeForm),
    CanvasSize(CanvasSizeForm),
    ImageOp(ImageOpForm),
    SmartArt(SmartArtForm),
    /// Insert > Chart: the picture list of chart kinds.
    Chart,
    RemoveBg(Box<crate::ai::remove_bg::RemoveBgSession>),
    Upscale(crate::ai::upscale::UpscaleForm),
    AiModels,
    Shortcuts(ShortcutsForm),
    Export(Box<export::ExportForm>),
    ExportFor(Box<ExportForForm>),
    Pages(PagesForm),
    Batch(Box<BatchForm>),
    MagicResize(MagicResizeForm),
    SplitSheet(Box<SplitSheetForm>),
}

/// Shows the open dialog (if any) and runs what the user chose.
pub fn show(ctx: &egui::Context, app: &mut FlashApp) {
    let Some(dialog) = app.dialog.take() else { return };
    match dialog {
        Dialog::Ask(ask) => match ask::show(ctx, &ask) {
            Some(answer) => run(ctx, app, answer),
            None => app.dialog = Some(Dialog::Ask(ask)),
        },
        Dialog::NewProject(mut form) => match new_project::show(ctx, app, &mut form) {
            Some(true) => new_project::create(ctx, app, &form),
            Some(false) => {
                if let Some(kind) = form.browse.take() {
                    project_manager::open_templates(app, ctx, kind);
                }
            }
            None => app.dialog = Some(Dialog::NewProject(form)),
        },
        Dialog::SaveTemplate(mut name) => match save_template::show(ctx, &mut name) {
            Some(true) => file::save_as_template(app, &name),
            Some(false) => {}
            None => app.dialog = Some(Dialog::SaveTemplate(name)),
        },
        Dialog::PhotoGrid(mut form) => match photo_grid::show(ctx, &mut form) {
            Some(true) => crate::commands::frames::photo_grid(app, &form.settings),
            Some(false) => {}
            None => app.dialog = Some(Dialog::PhotoGrid(form)),
        },
        Dialog::Icons(mut picker) => match icon_picker::show(ctx, &mut picker, &mut app.previews) {
            Some(Some(pick)) => match pick {
                Pick::Icon(i) => crate::commands::insert::add_icon(app, &fe_core::library::icons::ALL[i]),
                Pick::Material(code) => crate::commands::insert::add_material_icon(app, code),
                Pick::Badge(id) => crate::commands::insert::add_badge(app, id),
                Pick::Graphic(id) => crate::commands::insert::add_graphic(app, id),
            },
            Some(None) => {}
            None => app.dialog = Some(Dialog::Icons(picker)),
        },
        Dialog::Chart => match chart_picker::show(ctx, &mut app.previews) {
            Some(Some(kind)) => crate::commands::insert::add_chart(app, kind),
            Some(None) => {}
            None => app.dialog = Some(Dialog::Chart),
        },
        Dialog::SmartArt(mut form) => match smartart::show(ctx, &mut form, &mut app.previews) {
            Some(Some(choice)) => {
                crate::commands::insert::add_smartart(app, choice.layout, &choice.words, &choice.scheme);
            }
            Some(None) => {}
            None => app.dialog = Some(Dialog::SmartArt(form)),
        },
        Dialog::BrandKit(kit) => {
            if brand_kit::show(ctx, app) {
                app.dialog = Some(Dialog::BrandKit(kit));
            }
        }
        Dialog::Effect(mut session) => match effect::show(ctx, app, &mut session) {
            Some(true) => crate::commands::effects::commit(app, *session),
            Some(false) => crate::commands::effects::cancel(app, *session),
            None => app.dialog = Some(Dialog::Effect(session)),
        },
        Dialog::ImageSize(mut form) => match size::show_image_size(ctx, &mut form) {
            Some(true) => crate::commands::image::image_size(app, form.width, form.height, form.mode),
            Some(false) => {}
            None => app.dialog = Some(Dialog::ImageSize(form)),
        },
        Dialog::CanvasSize(mut form) => match size::show_canvas_size(ctx, &mut form) {
            Some(true) => crate::commands::image::canvas_size(app, form.width, form.height, form.anchor),
            Some(false) => {}
            None => app.dialog = Some(Dialog::CanvasSize(form)),
        },
        Dialog::ImageOp(mut form) => match size::show_image_op(ctx, &mut form) {
            Some(true) => {
                let angle = form.values.get("angle");
                match form.op {
                    ImageOp::Rotate => crate::commands::image::rotate_custom(app, angle),
                    ImageOp::Straighten => crate::commands::image::straighten(app, angle, form.values.on("crop")),
                }
            }
            Some(false) => {}
            None => app.dialog = Some(Dialog::ImageOp(form)),
        },
        Dialog::SaveStyle(mut name) => match save_style::show(ctx, &mut name) {
            Some(true) => crate::commands::styles::add(app, &name),
            Some(false) => {}
            None => app.dialog = Some(Dialog::SaveStyle(name)),
        },
        Dialog::RemoveBg(mut session) => match remove_bg::show(ctx, app, &mut session) {
            Some(true) => crate::ai::remove_bg::commit(app, *session),
            Some(false) => crate::ai::remove_bg::cancel(app, *session),
            None => app.dialog = Some(Dialog::RemoveBg(session)),
        },
        Dialog::Upscale(mut form) => match upscale::show(ctx, &mut form) {
            Some(true) => crate::ai::upscale::run(app, form.scale, form.strength),
            Some(false) => {}
            None => app.dialog = Some(Dialog::Upscale(form)),
        },
        Dialog::Export(mut form) => match export::show(ctx, app, &mut form) {
            Some(true) => crate::commands::export::finish(app, *form),
            Some(false) => {}
            None => app.dialog = Some(Dialog::Export(form)),
        },
        Dialog::ExportFor(mut form) => match export_for::show(ctx, &mut form) {
            Some(true) => crate::commands::export::finish_for(app, *form),
            Some(false) => {}
            None => app.dialog = Some(Dialog::ExportFor(form)),
        },
        Dialog::Pages(mut form) => match pages_export::show(ctx, &mut form) {
            Some(true) => crate::commands::export::finish_pages(app, form.kind, form.values),
            Some(false) => {}
            None => app.dialog = Some(Dialog::Pages(form)),
        },
        Dialog::Batch(mut form) => match batch::show(ctx, &mut form) {
            Some(true) => {
                if let Some(options) = form.options() {
                    crate::commands::export::finish_batch(app, options);
                }
            }
            Some(false) => {}
            None => app.dialog = Some(Dialog::Batch(form)),
        },
        Dialog::MagicResize(mut form) => match magic_resize::show(ctx, &mut form) {
            Some(true) => crate::commands::magic_resize::run(app, form.width, form.height, form.all_pages),
            Some(false) => {}
            None => app.dialog = Some(Dialog::MagicResize(form)),
        },
        Dialog::SplitSheet(mut form) => match split_sheet::show(ctx, app, &mut form) {
            Some(true) => crate::commands::split_sheet::finish(app, *form),
            Some(false) => {}
            None => app.dialog = Some(Dialog::SplitSheet(form)),
        },
        Dialog::Shortcuts(mut form) => {
            if shortcuts::show(ctx, app, &mut form) && app.dialog.is_none() {
                app.dialog = Some(Dialog::Shortcuts(form));
            }
        }
        Dialog::AiModels => {
            if crate::ai::models_dialog::show(ctx, app) {
                // A question opened from inside (a failed download...) takes over.
                if app.dialog.is_none() {
                    app.dialog = Some(Dialog::AiModels);
                }
            }
        }
        Dialog::Color(mut picker) => {
            if color_picker::show(ctx, app, &mut picker) {
                app.dialog = Some(Dialog::Color(picker));
            }
        }
    }
}

fn run(ctx: &egui::Context, app: &mut FlashApp, answer: Answer) {
    let index_of = |app: &FlashApp, uid: u64| app.projects.iter().position(|p| p.uid() == uid);
    match answer {
        Answer::Nothing => {}
        Answer::SaveAndCloseProject(uid) => {
            if let Some(index) = index_of(app, uid) {
                match app.projects[index].path.clone() {
                    Some(path) => file::save_to(app, ctx, index, path, crate::app::jobs::AfterSave::CloseProject),
                    None => {
                        file::save_as(app, ctx, index, crate::app::jobs::AfterSave::CloseProject);
                    }
                }
            }
        }
        Answer::CloseProject(uid) => {
            if let Some(index) = index_of(app, uid) {
                file::remove_project(app, index);
            }
        }
        Answer::SaveAllAndQuit => file::continue_closing_window(app, ctx),
        Answer::Quit => file::quit(app, ctx),
        Answer::RecoverWork => crate::app::autosave::recover(app),
        Answer::DiscardRecovery => crate::app::autosave::delete_recovery(),
        Answer::DeletePages(indices) => pages::delete_now(app, &indices),
        Answer::DeleteProjectFile(path) => project_manager::delete_file(app, &path),
        Answer::DownloadModels(pending) => crate::ai::pending::download(app, pending),
    }
}
