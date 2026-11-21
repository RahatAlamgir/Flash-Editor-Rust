//! Ribbon, toolbox, options / hint bar, project tabs, right panels, page strip, status bar and pop-ups.

pub mod hint_bar;
pub mod pages;
pub mod popups;
pub mod project_tabs;
pub mod ribbon;
pub mod right;
pub mod shape_menu;
pub mod status_bar;
pub mod toolbox;
pub mod view_aids_bar;

use crate::app::FlashApp;
use crate::commands::{edit, insert as ins, layers as cmd, mask, objects as obj, pages as page_cmd, selection as sel};
use ribbon::{RibbonAction, model::TABS};

/// Whether a ribbon button can be used right now (dimmed when not). Buttons of later phases stay usable so
/// they can say "coming later".
pub fn ribbon_enabled(app: &FlashApp, label: &str) -> bool {
    let doc = app.doc();
    let history = app.project().history();
    match label {
        "Undo" => history.can_undo(),
        "Redo" => history.can_redo(),
        "Delete" => doc.layers().len() > 1,
        "Flatten Image" => doc.layers().len() > 1,
        "Close Page" => app.project().pages().len() > 1,
        "Deselect" | "Delete Pixels" | "Split Selection" | "Mask: Show Selection" | "Mask: Hide Selection" => {
            doc.selected_area().is_some()
        }
        "Invert Mask" | "Delete Mask" => doc.active_layer().is_some_and(|l| l.mask().is_some()),
        "Mask: Show All" | "Mask: Hide All" => {
            doc.active_layer().is_some_and(|l| l.mask().is_none() && !l.is_adjustment())
        }
        "Clip Below" => doc.active_index().is_some_and(|i| i > 0),
        _ => true,
    }
}

/// The adjustment or filter a ribbon button runs.
fn effect_for(label: &str) -> Option<&'static fe_core::effects::Effect> {
    use fe_core::effects as e;
    Some(match label {
        "Auto Levels" => &e::AUTO_LEVELS,
        "Brightness / Contrast" => &e::BRIGHTNESS_CONTRAST,
        "Exposure / Gamma" => &e::EXPOSURE_GAMMA,
        "Recolor" => &e::RECOLOR,
        "Hue / Saturation" => &e::HUE_SATURATION,
        "Vibrance" => &e::VIBRANCE,
        "Temperature / Tint" => &e::TEMPERATURE_TINT,
        "Grayscale" => &e::GRAYSCALE,
        "Invert Colors" => &e::INVERT,
        "Sepia" => &e::SEPIA,
        "Posterize" => &e::POSTERIZE,
        "Threshold" => &e::THRESHOLD,
        "Enhance" => &e::ENHANCE,
        "Blur" => &e::GAUSSIAN_BLUR,
        "Motion Blur" => &e::MOTION_BLUR,
        "Sharpen" => &e::SHARPEN,
        "Reduce Noise" => &e::REDUCE_NOISE,
        "Add Noise" => &e::ADD_NOISE,
        "Pixelate" => &e::PIXELATE,
        "Vignette" => &e::VIGNETTE,
        "Emboss" => &e::EMBOSS,
        "Find Edges" => &e::FIND_EDGES,
        _ => return None,
    })
}

/// Runs what a ribbon click asks for. Commands of later phases say so in the status bar.
pub fn handle_ribbon_action(app: &mut FlashApp, ctx: &egui::Context, action: RibbonAction) {
    match action {
        RibbonAction::File => app.file_menu = !app.file_menu,
        RibbonAction::Button { tab, group, item, rect } => {
            let entry = &TABS[tab].groups[group].items[item];
            match entry.label {
                "Rulers" | "Grid" | "Guides" | "Snapping" => {} // toggled by the ribbon itself
                "Undo" => edit::undo(app),
                "Redo" => edit::redo(app),
                "Paste" => sel::paste(app),
                "Cut" => sel::copy(app, true),
                "Copy" => sel::copy(app, false),
                "Delete Pixels" => {
                    sel::delete_pixels(app);
                }
                "Fill Color" => {
                    let picker = crate::dialogs::ColorPicker::new(crate::dialogs::ColorTarget::Fill, app.foreground);
                    app.dialog = Some(crate::dialogs::Dialog::Color(picker));
                }
                "Deselect" => sel::deselect(app),
                "Invert Selection" => sel::invert(app),
                "Layer Content" => sel::layer_content(app),
                "Split Selection" => sel::split(app),
                "Reveal All" => sel::reveal_all(app),
                "Mask: Show Selection" => mask::from_selection(app, true),
                "Mask: Hide Selection" => mask::from_selection(app, false),
                "Mask: Show All" => mask::add(app, true),
                "Mask: Hide All" => mask::add(app, false),
                "Invert Mask" => mask::invert(app),
                "Delete Mask" => mask::delete(app),
                "Clip Below" => mask::clip_to_below(app),
                "Gradient Background" => obj::gradient_background(app),
                "Photo Grid" => {
                    let doc = app.doc();
                    let form = crate::dialogs::PhotoGridForm::new(doc.width(), doc.height());
                    app.dialog = Some(crate::dialogs::Dialog::PhotoGrid(form));
                }
                "Fill Frames" => crate::commands::frames::fill_frames(app),
                "Layer Mask ▾" | "Shapes ▾" | "Frame ▾" | "Chart ▾" | "Barcode ▾" => {
                    app.ribbon_menu = Some((tab, group, item, rect));
                }
                "QR Code" => ins::add_qr(app),
                "Heading" => ins::add_text(app, ins::TextKind::Heading),
                "Subheading" => ins::add_text(app, ins::TextKind::Subheading),
                "Body Text" => ins::add_text(app, ins::TextKind::Body),
                "Icons" | "Badges" | "Graphics" => {
                    let page = match entry.label {
                        "Badges" => crate::dialogs::IconPage::Badges,
                        "Graphics" => crate::dialogs::IconPage::Graphics,
                        _ => crate::dialogs::IconPage::Icons,
                    };
                    app.dialog = Some(crate::dialogs::Dialog::Icons(crate::dialogs::IconPicker::new(page)));
                }
                "Crop Layer ▾" => app.ribbon_menu = Some((tab, group, item, rect)),
                "Levels Layer" => crate::commands::adjustments::add_levels(app),
                "Curves Layer" => crate::commands::adjustments::add_curves(app),
                "Adjustment Layer" => crate::commands::adjustments::add_basic(app),
                "Image Size" => {
                    let d = app.doc();
                    app.dialog = Some(crate::dialogs::Dialog::ImageSize(crate::dialogs::ImageSizeForm::new(
                        d.width(),
                        d.height(),
                    )));
                }
                "Canvas Size" => {
                    let d = app.doc();
                    app.dialog = Some(crate::dialogs::Dialog::CanvasSize(crate::dialogs::CanvasSizeForm::new(
                        d.width(),
                        d.height(),
                    )));
                }
                "Remove BG (AI)" => crate::ai::remove_bg::start(app),
                "Subject (AI)" => crate::ai::subject::start(app),
                "Fill (AI)" => crate::ai::fill::start(app),
                "AI Upscale" => crate::ai::upscale::start(app),
                "AI Models" => app.dialog = Some(crate::dialogs::Dialog::AiModels),
                "Trim Edges" => crate::commands::image::trim(app),
                "Upscale 2x" => crate::commands::image::upscale_2x(app),
                "Straighten" => {
                    app.dialog = Some(crate::dialogs::Dialog::ImageOp(crate::dialogs::ImageOpForm::new(
                        crate::dialogs::ImageOp::Straighten,
                    )));
                }
                "Rotate ▾" => app.ribbon_menu = Some((tab, group, item, rect)),
                label if effect_for(label).is_some() => {
                    if let Some(e) = effect_for(label) {
                        crate::commands::effects::run(app, e);
                    }
                }
                "Tutorial" => crate::commands::tutorial::open(app, ctx),
                "Brand Kit" => app.dialog = Some(crate::dialogs::Dialog::BrandKit(crate::dialogs::BrandKit)),
                "SmartArt" => app.dialog = Some(crate::dialogs::Dialog::SmartArt(crate::dialogs::SmartArtForm::new())),
                "Group" => obj::group(app),
                "Ungroup" => obj::ungroup(app),
                "Group / Ungroup" => obj::toggle_group(app),
                "New Layer" => cmd::new_layer(app),
                "Duplicate" => cmd::duplicate(app),
                "Delete" => cmd::delete(app),
                "Rename" => right::layers::start_rename_active(app),
                "Lock / Unlock" => cmd::toggle_lock(app),
                "Flatten Image" => cmd::flatten(app),
                "Order ▾" => app.ribbon_menu = Some((tab, group, item, rect)),
                "Undo Steps" => app.undo_dialog = true,
                "Zoom In" => app.current_view_mut().zoom_in(),
                "Zoom Out" => app.current_view_mut().zoom_out(),
                "Fit on Screen" => app.current_view_mut().fit(),
                "100%" => app.current_view_mut().actual_size(),
                "Clear Guides" => {
                    let page = app.project_mut().active_page_mut();
                    page.vertical_guides.clear();
                    page.horizontal_guides.clear();
                }
                "Next Picture" => app.active_project = (app.active_project + 1) % app.projects.len(),
                "Projects" => crate::dialogs::project_manager::open(app, ctx),
                "New Page" => page_cmd::add_after_current(app),
                "Close Page" => page_cmd::ask_delete(app, vec![app.project().active_index()]),
                "Icon Style ▾" => {
                    app.ribbon.colorful = !app.ribbon.colorful;
                    let style = if app.ribbon.colorful { "Colorful" } else { "Plain" };
                    app.status = format!("Icon style: {style}.");
                }
                label => app.not_yet(label.trim_end_matches(" ▾")),
            }
        }
    }
}
