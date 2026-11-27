//! The settings of the active tool in the bar under the ribbon (C# `OptionsBar.xaml`). Tools without settings
//! show their hint there instead.

use egui::{RichText, Ui};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use super::painting::previews::BrushPreviews;
use super::{ToolKind, ToolState};
use fe_core::paint::brush::BrushType;

fn label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).color(colors::TEXT));
}

fn hint(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).color(colors::SUBTLE_TEXT)).truncate());
}

/// Draws the settings of the active tool. False when the tool has none (the caller shows its hint).
pub fn show(ui: &mut Ui, tools: &mut ToolState) -> bool {
    match tools.active() {
        ToolKind::MagicWand => {
            let o = &mut tools.select.wand;
            label(ui, "Tolerance");
            number_slider(ui, &mut o.tolerance, 0.0..=255.0, "", 110.0, 58.0);
            ui.add_space(16.0);
            ui.checkbox(&mut o.contiguous, "Connected only")
                .on_hover_text("On: only the touching area. Off: every similar color in the picture.");
            ui.add_space(12.0);
            ui.checkbox(&mut o.all_layers, "All layers")
                .on_hover_text("On: look at what you see. Off: only the selected layer.");
            hint(ui, "  Shift = add, Alt = subtract");
            true
        }
        ToolKind::QuickSelect => {
            let o = &mut tools.select.quick;
            label(ui, "Brush size");
            number_slider(ui, &mut o.size, 2.0..=500.0, "px", 110.0, 58.0);
            label(ui, "Tolerance");
            number_slider(ui, &mut o.tolerance, 1.0..=255.0, "", 110.0, 58.0);
            ui.add_space(12.0);
            ui.checkbox(&mut o.all_layers, "All layers");
            hint(ui, "  Drag to select. Alt+drag removes. [ ] = size. W switches to Magic Wand.");
            true
        }
        ToolKind::Brush | ToolKind::Eraser | ToolKind::MaskBrush => {
            brush_options(ui, tools);
            true
        }
        ToolKind::CloneStamp
        | ToolKind::HealingBrush
        | ToolKind::SpotHealing
        | ToolKind::DodgeBurn
        | ToolKind::BlurSharpen
        | ToolKind::Patch
        | ToolKind::Liquify => {
            retouch_options(ui, tools);
            true
        }
        ToolKind::RemoveBrush => {
            label(ui, "Brush size");
            number_slider(ui, &mut tools.ai.remove.size, 2.0..=500.0, "px", 110.0, 58.0);
            hint(ui, "  Paint over what you want gone, then let go: the AI fills it in on a new layer. [ ] = size.");
            true
        }
        ToolKind::Refine => {
            let o = &mut tools.ai.refine;
            label(ui, "Mode");
            ui.selectable_value(&mut o.restore, true, "Restore")
                .on_hover_text("Paint back what was removed (X switches)");
            ui.selectable_value(&mut o.restore, false, "Remove")
                .on_hover_text("Paint away background that was missed (X switches)");
            ui.add_space(12.0);
            label(ui, "Brush size");
            number_slider(ui, &mut o.size, 2.0..=500.0, "px", 110.0, 58.0);
            label(ui, "Hardness");
            number_slider(ui, &mut o.hardness, 0.0..=100.0, "%", 90.0, 58.0);
            hint(ui, "  Alt = the other mode for one stroke. [ ] = size.");
            true
        }
        ToolKind::ClickSelect => {
            hint(ui, "Click an object to select it, or drag a box around it. Shift = add, Alt = subtract.");
            true
        }
        ToolKind::Crop | ToolKind::PerspectiveCrop => {
            if tools.active() == ToolKind::Crop {
                label(ui, "Shape");
                egui::ComboBox::from_id_salt("crop-aspect")
                    .width(200.0)
                    .selected_text(super::crop::ASPECTS[tools.crop.aspect].0)
                    .show_ui(ui, |ui| {
                        for (i, (name, _)) in super::crop::ASPECTS.iter().enumerate() {
                            if ui.selectable_label(tools.crop.aspect == i, *name).clicked() {
                                tools.crop.aspect = i;
                                tools.crop.rect.rect =
                                    tools.crop.rect.rect.map(|r| super::crop::fit_aspect(r, super::crop::ASPECTS[i].1));
                            }
                        }
                    });
            }
            let text = tools.crop.hint(tools.active());
            hint(ui, &format!("  {text}"));
            true
        }
        ToolKind::Shape => {
            label(ui, "Shape");
            let kind = &mut tools.shape.kind;
            egui::ComboBox::from_id_salt("shape-tool-kind").width(170.0).selected_text(kind.name()).show_ui(ui, |ui| {
                for (group, kinds) in crate::panels::shape_menu::GROUPS {
                    ui.label(RichText::new(*group).strong().color(colors::SUBTLE_TEXT));
                    for k in kinds.iter() {
                        ui.selectable_value(kind, *k, k.name());
                    }
                }
            });
            hint(ui, &format!("  {}", ToolKind::Shape.hint()));
            true
        }
        ToolKind::Text => {
            let o = &mut tools.text;
            label(ui, "Font");
            font_picker(ui, "text-tool-font", &mut o.family, 190.0);
            label(ui, "Size");
            number_slider(ui, &mut o.size, 4.0..=300.0, "px", 110.0, 58.0);
            ui.checkbox(&mut o.bold, "Bold");
            ui.checkbox(&mut o.italic, "Italic");
            hint(ui, "  Click on the picture and type. Click existing text to change it.");
            true
        }
        _ => false,
    }
}

/// Size, hardness and strength of a retouching tool, with what it needs besides (aligned, dodge / burn, blur / sharpen,
/// liquify mode) and a hint.
fn retouch_options(ui: &mut Ui, tools: &mut ToolState) {
    let tool = tools.active();
    if tool == ToolKind::Liquify {
        let o = &mut tools.retouch.liquify;
        label(ui, "Mode");
        egui::ComboBox::from_id_salt("liquify-mode").width(100.0).selected_text(o.mode.name()).show_ui(ui, |ui| {
            for m in fe_core::retouch::LiquifyMode::ALL {
                ui.selectable_value(&mut o.mode, m, m.name());
            }
        });
        ui.add_space(10.0);
        label(ui, "Size");
        number_slider(ui, &mut o.size, 5.0..=600.0, "px", 110.0, 58.0);
        label(ui, "Strength");
        number_slider(ui, &mut o.strength, 1.0..=100.0, "%", 110.0, 58.0);
    } else if let Some(o) = tools.retouch.options_mut(tool) {
        label(ui, "Size");
        number_slider(ui, &mut o.size, 1.0..=1000.0, "px", 110.0, 58.0);
        label(ui, "Hardness");
        number_slider(ui, &mut o.hardness, 0.0..=100.0, "%", 110.0, 58.0);
        match tool {
            ToolKind::DodgeBurn => {
                label(ui, "Exposure");
                number_slider(ui, &mut o.strength, 1.0..=100.0, "%", 110.0, 58.0);
                ui.selectable_value(&mut o.burn, false, "Dodge");
                ui.selectable_value(&mut o.burn, true, "Burn");
                egui::ComboBox::from_id_salt("tone-range").width(100.0).selected_text(o.range.name()).show_ui(
                    ui,
                    |ui| {
                        for r in fe_core::retouch::ToneRange::ALL {
                            ui.selectable_value(&mut o.range, r, r.name());
                        }
                    },
                );
            }
            ToolKind::BlurSharpen => {
                label(ui, "Strength");
                number_slider(ui, &mut o.strength, 1.0..=100.0, "%", 110.0, 58.0);
                ui.selectable_value(&mut o.sharpen, false, "Blur");
                ui.selectable_value(&mut o.sharpen, true, "Sharpen");
            }
            ToolKind::CloneStamp | ToolKind::HealingBrush => {
                label(ui, "Opacity");
                number_slider(ui, &mut o.strength, 1.0..=100.0, "%", 110.0, 58.0);
                ui.checkbox(&mut o.aligned, "Aligned")
                    .on_hover_text("On: strokes keep the same distance to the copied place. Off: every stroke starts at the copied place again.");
            }
            _ => {}
        }
    }
    if let Some(text) = super::retouch::hint(tool) {
        hint(ui, &format!("  {text}"));
    }
}

/// A list of the installed fonts, each name shown in plain text. True when another font was picked.
pub fn font_picker(ui: &mut Ui, id: &str, family: &mut String, width: f32) -> bool {
    let mut changed = false;
    // A family can be a list ("Segoe Fluent Icons, Segoe MDL2 Assets" for icons): show the first name, cut to the box, so a long
    // name never makes the box (and the panel around it) wider.
    let shown = family.split(',').next().unwrap_or_default().trim().to_owned();
    egui::ComboBox::from_id_salt(id).width(width).selected_text(shown).wrap_mode(egui::TextWrapMode::Truncate).show_ui(
        ui,
        |ui| {
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for name in fe_core::text::fonts::families() {
                    if ui.selectable_label(name == family, name).clicked() && name != family {
                        *family = name.clone();
                        changed = true;
                    }
                }
            });
        },
    );
    changed
}

/// Type (with a picture of each), size, hardness and opacity of the Brush, Eraser or Mask Brush.
fn brush_options(ui: &mut Ui, tools: &mut ToolState) {
    let tool = tools.active();
    let paint = &mut tools.paint;
    let previews = &mut paint.previews;
    let Some(o) = (match tool {
        ToolKind::Brush => Some(&mut paint.brush),
        ToolKind::Eraser => Some(&mut paint.eraser),
        ToolKind::MaskBrush => Some(&mut paint.mask),
        _ => None,
    }) else {
        return;
    };
    label(ui, "Type");
    let ctx = ui.ctx().clone();
    let mut picked = None;
    egui::ComboBox::from_id_salt("brush-type")
        .width(200.0)
        .selected_text(o.kind.name())
        .show_ui(ui, |ui| {
            for kind in BrushType::ALL {
                let texture = previews.texture(&ctx, kind);
                let row = ui.horizontal(|ui| {
                    ui.add(egui::Image::new((texture, BrushPreviews::size())));
                    ui.selectable_label(o.kind == kind, kind.name())
                });
                let response = row.inner.union(row.response.interact(egui::Sense::click()));
                if response.on_hover_text(kind.description()).clicked() {
                    picked = Some(kind);
                }
            }
        })
        .response
        .on_hover_text("Brush type");
    if let Some(kind) = picked {
        o.kind = kind;
    }
    ui.add_space(10.0);
    label(ui, "Size");
    number_slider(ui, &mut o.size, 1.0..=500.0, "px", 110.0, 58.0);
    let uses = o.kind.uses_hardness();
    ui.add_enabled_ui(uses, |ui| {
        label(ui, "Hardness");
        number_slider(ui, &mut o.hardness, 0.0..=100.0, "%", 110.0, 58.0)
    })
    .response
    .on_disabled_hover_text("Has no effect for this brush type");
    label(ui, "Opacity");
    number_slider(ui, &mut o.opacity, 1.0..=100.0, "%", 110.0, 58.0);
    hint(ui, "  [ ] change size");
}
