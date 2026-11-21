//! Image Size, Canvas Size, Rotate and Straighten (C# `ImageSizeDialog`, `CanvasSizeDialog`, `ParametersDialog`).

use egui::{Id, RichText, vec2};
use fe_core::effects::{Param, Values};
use fe_core::transform::ResampleMode;
use fe_ui_kit::theme::colors;

use super::effect::param_rows;

/// The most pixels a side may have.
const MAX_SIDE: u32 = fe_core::document::canvas_ops::MAX_SIDE as u32;

/// Image > Image Size.
pub struct ImageSizeForm {
    pub width: u32,
    pub height: u32,
    original: (u32, u32),
    pub keep_proportions: bool,
    pub mode: ResampleMode,
}

impl ImageSizeForm {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, original: (width, height), keep_proportions: true, mode: ResampleMode::Smooth }
    }
}

fn number(ui: &mut egui::Ui, v: &mut u32) -> bool {
    let mut f = f64::from(*v);
    let changed =
        ui.add(egui::DragValue::new(&mut f).range(1.0..=f64::from(MAX_SIDE)).speed(1.0).max_decimals(0)).changed();
    *v = f.round().max(1.0) as u32;
    changed
}

/// `Some(true)` = OK, `Some(false)` = Cancel, `None` = open.
pub fn show_image_size(ctx: &egui::Context, f: &mut ImageSizeForm) -> Option<bool> {
    let mut result = None;
    let modal = egui::Modal::new(Id::NULL.with("image-size")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.label(RichText::new("Image Size").heading());
        ui.add_space(6.0);
        let mp = f64::from(f.original.0) * f64::from(f.original.1) / 1e6;
        ui.label(RichText::new(format!("Current size: {} x {} px ({mp:.2} megapixels)", f.original.0, f.original.1)).color(colors::SUBTLE_TEXT));
        ui.add_space(8.0);
        let ratio = f64::from(f.original.0) / f64::from(f.original.1);
        egui::Grid::new("image-size-grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            ui.label("Width (px)");
            if number(ui, &mut f.width) && f.keep_proportions {
                f.height = ((f64::from(f.width) / ratio).round() as u32).clamp(1, MAX_SIDE);
            }
            ui.end_row();
            ui.label("Height (px)");
            if number(ui, &mut f.height) && f.keep_proportions {
                f.width = ((f64::from(f.height) * ratio).round() as u32).clamp(1, MAX_SIDE);
            }
            ui.end_row();
            ui.label("Percent");
            let mut pct = f64::from(f.width) / f64::from(f.original.0) * 100.0;
            if ui.add(egui::DragValue::new(&mut pct).range(1.0..=1600.0).speed(1.0).suffix(" %").max_decimals(1)).changed() {
                f.width = ((f64::from(f.original.0) * pct / 100.0).round() as u32).clamp(1, MAX_SIDE);
                f.height = ((f64::from(f.original.1) * pct / 100.0).round() as u32).clamp(1, MAX_SIDE);
            }
            ui.end_row();
        });
        ui.checkbox(&mut f.keep_proportions, "Keep proportions");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Method");
            egui::ComboBox::from_id_salt("resample").selected_text(f.mode.name()).show_ui(ui, |ui| {
                for m in ResampleMode::ALL {
                    ui.selectable_value(&mut f.mode, m, m.name());
                }
            });
        });
        ui.add_space(6.0);
        ui.add(
            egui::Label::new(
                RichText::new("Scales the whole picture (all layers). To only make the saved file smaller, use File > Export instead.")
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(10.0);
        buttons(ui, "Resize", &mut result);
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}

fn buttons(ui: &mut egui::Ui, ok: &str, result: &mut Option<bool>) {
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new(ok).fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
            *result = Some(true);
        }
        if ui.add(egui::Button::new("Cancel").min_size(vec2(90.0, 28.0))).clicked() {
            *result = Some(false);
        }
    });
}

/// Image > Canvas Size.
pub struct CanvasSizeForm {
    pub width: u32,
    pub height: u32,
    original: (u32, u32),
    /// 0..8, row by row (4 = center).
    pub anchor: usize,
}

impl CanvasSizeForm {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, original: (width, height), anchor: 4 }
    }
}

pub fn show_canvas_size(ctx: &egui::Context, f: &mut CanvasSizeForm) -> Option<bool> {
    let mut result = None;
    let modal = egui::Modal::new(Id::NULL.with("canvas-size")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.label(RichText::new("Canvas Size").heading());
        ui.add_space(6.0);
        ui.label(RichText::new(format!("Current canvas: {} x {} px", f.original.0, f.original.1)).color(colors::SUBTLE_TEXT));
        ui.add_space(8.0);
        egui::Grid::new("canvas-size-grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            ui.label("Width (px)");
            number(ui, &mut f.width);
            ui.end_row();
            ui.label("Height (px)");
            number(ui, &mut f.height);
            ui.end_row();
        });
        ui.add_space(6.0);
        ui.label("Keep the picture at");
        const ARROWS: [&str; 9] = ["↖", "↑", "↗", "←", "●", "→", "↙", "↓", "↘"];
        egui::Grid::new("anchor-grid").spacing([4.0, 4.0]).show(ui, |ui| {
            for (i, arrow) in ARROWS.iter().enumerate() {
                if ui.add_sized([34.0, 30.0], egui::Button::new(*arrow).selected(f.anchor == i)).clicked() {
                    f.anchor = i;
                }
                if i % 3 == 2 {
                    ui.end_row();
                }
            }
        });
        ui.add_space(6.0);
        ui.add(
            egui::Label::new(
                RichText::new("Adds or removes space around the picture without scaling it. New space is transparent. Parts outside a smaller canvas are kept on their layers.")
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(10.0);
        buttons(ui, "OK", &mut result);
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}

/// Which image command a settings window is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOp {
    Rotate,
    Straighten,
}

/// Rotate Image / Straighten: a few settings and a sentence of help.
pub struct ImageOpForm {
    pub op: ImageOp,
    pub values: Values,
}

const ROTATE: [Param; 1] = [Param {
    key: "angle",
    label: "Angle (clockwise)",
    min: -180.0,
    max: 180.0,
    default: 15.0,
    step: 0.5,
    unit: "°",
    toggle: false,
}];
const STRAIGHTEN: [Param; 2] = [
    Param {
        key: "angle",
        label: "Angle (clockwise)",
        min: -45.0,
        max: 45.0,
        default: 0.0,
        step: 0.1,
        unit: "°",
        toggle: false,
    },
    Param {
        key: "crop",
        label: "Cut off the empty corners",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        step: 1.0,
        unit: "",
        toggle: true,
    },
];

impl ImageOpForm {
    pub fn new(op: ImageOp) -> Self {
        let params: &[Param] = if op == ImageOp::Rotate { &ROTATE } else { &STRAIGHTEN };
        Self { op, values: Values::defaults(params) }
    }
}

pub fn show_image_op(ctx: &egui::Context, f: &mut ImageOpForm) -> Option<bool> {
    let (title, params, help): (&str, &[Param], &str) = match f.op {
        ImageOp::Rotate => {
            ("Rotate Image", &ROTATE, "The canvas grows so no corner is cut off. New areas are transparent.")
        }
        ImageOp::Straighten => (
            "Straighten",
            &STRAIGHTEN,
            "For a tilted horizon or a crooked photo: turn the picture until it is level. Positive turns clockwise.",
        ),
    };
    let mut result = None;
    let modal = egui::Modal::new(Id::NULL.with("image-op")).show(ctx, |ui| {
        ui.set_width(320.0);
        ui.label(RichText::new(title).heading());
        ui.add_space(8.0);
        param_rows(ui, params, &mut f.values);
        ui.add(egui::Label::new(RichText::new(help).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(10.0);
        buttons(ui, "OK", &mut result);
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}
