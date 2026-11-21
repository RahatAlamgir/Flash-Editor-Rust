//! Properties > Text > Designs: 16 ready-made text designs, each shown as a small picture made by the real
//! renderer (C# `TextPresetPreviews`).

use egui::{ColorImage, Sense, TextureHandle, TextureOptions, Ui, vec2};
use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::objects::{ObjectBox, StoredObject};
use fe_core::text::{TEXT_PRESETS, TextAlign, TextData};

use crate::app::FlashApp;
use crate::commands::objects as cmd;

const SIZE: i32 = 72;

/// A small picture of one design: "Aa" on its preview background.
fn preview(index: usize) -> ColorImage {
    let preset = &TEXT_PRESETS[index];
    let bg = preset.preview_background.unwrap_or(ColorRgba::new(238, 238, 242, 255));
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for _ in 0..SIZE * SIZE {
        data.extend_from_slice(&[bg.r, bg.g, bg.b, 255]);
    }
    let mut sample = TextData::new("Aa", "Segoe UI", 34.0, ColorRgba::BLACK);
    sample.align = TextAlign::Center;
    let mut object =
        StoredObject::new_text(ObjectBox::new(0.0, f64::from(SIZE) * 0.5 - 24.0, f64::from(SIZE), 48.0, 0.0), &sample);
    preset.apply(&mut object);
    // The box of a design keeps the sample's middle: put the (re-measured) text in the middle of the picture.
    if let Some(b) = object.box_() {
        object.set_box(ObjectBox::from_center(
            fe_core::imaging::PointD::new(f64::from(SIZE) / 2.0, f64::from(SIZE) / 2.0),
            b.width,
            b.height,
            0.0,
        ));
    }
    let canvas = PixelRect::new(0, 0, SIZE, SIZE);
    let tiles = fe_core::render::render_object(&object, canvas).unwrap_or_default();
    let top = tiles.to_buffer(canvas);
    for (o, p) in data.as_chunks_mut::<4>().0.iter_mut().zip(top.data().as_chunks::<4>().0.iter()) {
        let a = u32::from(p[3]);
        for c in 0..3 {
            o[c] = ((u32::from(p[c]) * a + u32::from(o[c]) * (255 - a)) / 255) as u8;
        }
    }
    ColorImage::from_rgba_unmultiplied([SIZE as usize, SIZE as usize], &data)
}

fn textures(ctx: &egui::Context) -> Vec<TextureHandle> {
    let id = egui::Id::NULL.with("text-design-previews");
    if let Some(list) = ctx.data(|d| d.get_temp::<Vec<TextureHandle>>(id)) {
        return list;
    }
    let list: Vec<TextureHandle> = (0..TEXT_PRESETS.len())
        .map(|i| ctx.load_texture(format!("text-design-{i}"), preview(i), TextureOptions::LINEAR))
        .collect();
    ctx.data_mut(|d| d.insert_temp(id, list.clone()));
    list
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let list = textures(ui.ctx());
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (i, preset) in TEXT_PRESETS.iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(vec2(SIZE as f32 * 0.8, SIZE as f32 * 0.8), Sense::click());
            ui.painter().image(
                list[i].id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            if response.hovered() {
                ui.painter().rect_stroke(
                    rect,
                    2,
                    egui::Stroke::new(2.0, fe_ui_kit::theme::colors::ACCENT),
                    egui::StrokeKind::Inside,
                );
            }
            if response.on_hover_text(format!("{}: {}", preset.name, preset.description)).clicked() {
                cmd::change_object(app, "Text Design", |o| preset.apply(o));
            }
        }
    });
}
