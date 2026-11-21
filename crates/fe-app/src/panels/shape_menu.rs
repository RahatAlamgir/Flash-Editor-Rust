//! Insert > Shapes ▾ (C# `MainWindow.ShapeMenu`): every shape in groups, each as a small picture of itself (drawn
//! by the real shape renderer, tinted in the group's icon color).

use egui::{ColorImage, RichText, TextureHandle, TextureOptions, Ui, vec2};
use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject};
use fe_ui_kit::theme::colors;

use ShapeKind::*;

/// The groups of the gallery, like the C# menu.
pub const GROUPS: &[(&str, &[ShapeKind])] = &[
    (
        "Basic",
        &[
            Rectangle,
            Ellipse,
            Triangle,
            RightTriangle,
            Diamond,
            Pentagon,
            Hexagon,
            Octagon,
            Trapezoid,
            Parallelogram,
            Polygon,
        ],
    ),
    (
        "Stars and symbols",
        &[Star, Sparkle, Burst, Heart, Cross, Check, XMark, Lightning, Cloud, Teardrop, Crescent, Shield, Home, Tag],
    ),
    ("Arrows and speech bubbles", &[Arrow, DoubleArrow, Chevron, SpeechBubble, OvalBubble]),
    ("Rings, slices and frames", &[Ring, HalfRing, HalfCircle, QuarterCircle, Pie, Frame]),
    ("Line", &[Line]),
];

const GROUP_COLORS: [egui::Color32; 5] =
    [colors::icon::BLUE, colors::icon::AMBER, colors::icon::GREEN, colors::icon::PURPLE, colors::icon::SKY];

/// The pictures of the shapes, made once.
#[derive(Default)]
pub struct ShapeIcons {
    textures: Vec<(ShapeKind, TextureHandle)>,
}

fn icon_image(kind: ShapeKind) -> ColorImage {
    let size = 36;
    let line = kind == Line;
    let b = if line { ObjectBox::new(2.0, 14.0, 32.0, 8.0, 0.0) } else { ObjectBox::new(2.0, 2.0, 32.0, 32.0, 0.0) };
    let mut shape = ShapeData::new(kind, (!line).then_some(ColorRgba::WHITE), line.then_some(ColorRgba::WHITE), 3.0);
    if kind == Pie {
        shape.inner = 0.0;
    }
    let canvas = PixelRect::new(0, 0, size, size);
    let tiles = fe_core::render::render_object(&StoredObject::new_shape(b, &shape), canvas).unwrap_or_default();
    let picture = tiles.to_buffer(canvas);
    ColorImage::from_rgba_unmultiplied([size as usize, size as usize], picture.data())
}

impl ShapeIcons {
    pub fn texture(&mut self, ctx: &egui::Context, kind: ShapeKind) -> egui::TextureId {
        if self.textures.is_empty() {
            self.textures = ShapeKind::ALL
                .iter()
                .filter(|k| **k != Path)
                .map(|k| (*k, ctx.load_texture(format!("shape-{}", k.name()), icon_image(*k), TextureOptions::LINEAR)))
                .collect();
        }
        self.textures.iter().find(|(k, _)| *k == kind).map(|(_, t)| t.id()).unwrap_or_default()
    }
}

/// Draws the gallery; returns the picked shape.
pub fn show(ui: &mut Ui, icons: &mut ShapeIcons) -> Option<ShapeKind> {
    let mut picked = None;
    ui.set_width(292.0);
    let ctx = ui.ctx().clone();
    for (g, (title, kinds)) in GROUPS.iter().enumerate() {
        ui.add_space(6.0);
        ui.label(RichText::new(*title).size(11.0).strong().color(colors::SUBTLE_TEXT));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for kind in *kinds {
                let image = egui::Image::new((icons.texture(&ctx, *kind), vec2(18.0, 18.0))).tint(GROUP_COLORS[g % 5]);
                let button = egui::Button::image(image).min_size(vec2(34.0, 34.0)).frame(false);
                if ui.add(button).on_hover_text(kind.name()).clicked() {
                    picked = Some(*kind);
                }
            }
        });
    }
    picked
}
