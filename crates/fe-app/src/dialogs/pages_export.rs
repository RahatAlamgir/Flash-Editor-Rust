//! The small settings windows of File > Export as PDF and Export as Animated GIF (C# `ShowParameters`).

use egui::{RichText, vec2};
use fe_core::effects::{Param, Values};
use fe_ui_kit::theme::colors;

use super::effect::param_rows;
use crate::commands::export::PagesKind;

const PDF: [Param; 2] = [
    Param {
        key: "quality",
        label: "Picture quality",
        min: 40.0,
        max: 100.0,
        default: 90.0,
        step: 1.0,
        unit: "%",
        toggle: false,
    },
    Param {
        key: "dpi",
        label: "Resolution",
        min: 72.0,
        max: 600.0,
        default: 96.0,
        step: 1.0,
        unit: " dpi",
        toggle: false,
    },
];
const GIF: [Param; 2] = [
    Param {
        key: "delay",
        label: "Time per page",
        min: 100.0,
        max: 10000.0,
        default: 1000.0,
        step: 50.0,
        unit: " ms",
        toggle: false,
    },
    Param { key: "loop", label: "Repeat forever", min: 0.0, max: 1.0, default: 1.0, step: 1.0, unit: "", toggle: true },
];

const PPTX: [Param; 3] = [
    Param {
        key: "editable",
        label: "Editable (text and shapes)",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        step: 1.0,
        unit: "",
        toggle: true,
    },
    Param {
        key: "quality",
        label: "Picture quality",
        min: 40.0,
        max: 100.0,
        default: 90.0,
        step: 1.0,
        unit: "%",
        toggle: false,
    },
    Param {
        key: "lossless",
        label: "Lossless pictures (PNG), when not editable",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        step: 1.0,
        unit: "",
        toggle: true,
    },
];

pub struct PagesForm {
    pub kind: PagesKind,
    pub values: Values,
}

impl PagesForm {
    pub fn new(kind: PagesKind) -> Self {
        Self {
            kind,
            values: Values::defaults(match kind {
                PagesKind::Pdf => &PDF[..],
                PagesKind::Gif => &GIF[..],
                PagesKind::Pptx => &PPTX[..],
            }),
        }
    }
}

pub fn show(ctx: &egui::Context, f: &mut PagesForm) -> Option<bool> {
    let (title, params, help): (&str, &[Param], &str) = match f.kind {
        PagesKind::Pdf => (
            "Export as PDF",
            &PDF,
            "Every page of this project becomes a PDF page. 96 dpi keeps the on-screen size; use 300 dpi for pictures made for print (the page is then smaller on paper but sharper).",
        ),
        PagesKind::Pptx => (
            "Export as PowerPoint",
            &PPTX,
            "Every page of this project becomes one slide, as big as the page. Editable: text becomes text boxes and simple shapes become PowerPoint shapes you can change; pictures stay pictures, and effects such as shadows, charts and QR codes are placed as pictures. Not editable: each slide is one picture of the page.",
        ),
        PagesKind::Gif => (
            "Export as animated GIF",
            &GIF,
            "Every page of this project becomes one frame. GIF has only 256 colors per frame and no soft transparency, so photos look best as PDF or video.",
        ),
    };
    let mut result = None;
    let modal = super::frame::modal(ctx, "pages-export", |ui| {
        ui.set_width(380.0);
        ui.label(RichText::new(title).heading());
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(help).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(8.0);
        param_rows(ui, params, &mut f.values);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Save...").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
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
