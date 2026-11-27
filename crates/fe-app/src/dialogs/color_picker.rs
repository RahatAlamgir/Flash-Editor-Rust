//! The color picker (C# `ColorPickerDialog`): a saturation / brightness square, a hue strip, hex and RGB boxes,
//! new / current swatches, Brand Kit colors, recent colors, the user's own swatches and quick swatches.

use egui::{Color32, ColorImage, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions, Ui, pos2, vec2};
use fe_core::imaging::{ColorRgba, color_math};
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::app::settings::Settings;

/// Side of the saturation / brightness square.
const AREA: f32 = 256.0;
const HUE_W: f32 = 22.0;
/// The quick swatches under the lists (C# `SwatchHex`).
const QUICK: [&str; 16] = [
    "#000000", "#404040", "#808080", "#C0C0C0", "#FFFFFF", "#FF3B30", "#FF9500", "#FFCC00", "#34C759", "#00C7BE",
    "#30B0C7", "#007AFF", "#5856D6", "#AF52DE", "#FF2D55", "#A2845E",
];
const MAX_RECENT: usize = 14;
const MAX_SWATCHES: usize = 28;

/// Which color of an effect is picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectColor {
    Shadow,
    Glow,
    InnerShadow,
    Outline,
    ThreeD,
}

/// What the picked color is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTarget {
    Foreground,
    /// Edit > Fill Color: fills the selection (or the layer) once picked.
    Fill,
    /// The fill of the selected shape.
    ShapeFill,
    /// The border or line of the selected shape.
    ShapeStroke,
    /// The color of the selected text.
    TextColor,
    /// The outline color of the selected text.
    TextOutline,
    /// A color of an effect (shadow, glow, ...).
    Effect(EffectColor),
    /// One color stop of the gradient.
    GradientStop(usize),
    /// A color of a QR code, barcode or chart.
    Object(crate::commands::codes::ObjectColor),
    /// A color of the Brand Kit: the n-th one, or a new one.
    BrandColor(Option<usize>),
}

/// The open color picker.
pub struct ColorPicker {
    pub target: ColorTarget,
    title: &'static str,
    old: ColorRgba,
    hue: f64,
    saturation: f64,
    value: f64,
    hex: String,
    rgb: [String; 3],
    /// The square's picture and the hue it was drawn for.
    square: Option<(TextureHandle, f64)>,
    hue_strip: Option<TextureHandle>,
    /// Shown over another dialog (the New Project window) instead of alone.
    nested: bool,
    /// The Eyedropper button was clicked.
    wants_eyedropper: bool,
}

fn color32(c: ColorRgba) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

impl ColorPicker {
    pub fn new(target: ColorTarget, initial: ColorRgba) -> Self {
        let title = match target {
            ColorTarget::Foreground => "Paint Color",
            ColorTarget::Fill => "Fill with which color?",
            ColorTarget::ShapeFill => "Fill Color",
            ColorTarget::ShapeStroke => "Border Color",
            ColorTarget::TextColor => "Text Color",
            ColorTarget::TextOutline => "Outline Color",
            ColorTarget::Effect(_) => "Effect Color",
            ColorTarget::GradientStop(_) => "Gradient Color",
            ColorTarget::Object(_) => "Color",
            ColorTarget::BrandColor(None) => "Add a brand color",
            ColorTarget::BrandColor(Some(_)) => "Brand color",
        };
        let mut picker = Self {
            target,
            title,
            old: initial.with_alpha(255),
            hue: 0.0,
            saturation: 0.0,
            value: 0.0,
            hex: String::new(),
            rgb: Default::default(),
            square: None,
            hue_strip: None,
            nested: false,
            wants_eyedropper: false,
        };
        picker.set_color(initial);
        picker
    }

    /// A picker that returns its color to the window that opened it (see `show_nested`) instead of applying it.
    pub fn nested(title: &'static str, initial: ColorRgba) -> Self {
        let mut picker = Self::new(ColorTarget::Foreground, initial);
        (picker.title, picker.nested) = (title, true);
        picker
    }

    fn color(&self) -> ColorRgba {
        color_math::from_hsv(self.hue, self.saturation, self.value, 255)
    }

    fn set_color(&mut self, c: ColorRgba) {
        let (h, s, v) = color_math::rgb_to_hsv(c.r as f64 / 255.0, c.g as f64 / 255.0, c.b as f64 / 255.0);
        (self.hue, self.saturation, self.value) = (h, s, v);
        self.refresh_text();
    }

    fn refresh_text(&mut self) {
        let c = self.color();
        self.hex = c.to_hex();
        self.rgb = [c.r.to_string(), c.g.to_string(), c.b.to_string()];
    }

    /// Reads the hex or RGB boxes after typing (bad input puts the current values back).
    fn commit_text(&mut self, from_hex: bool) {
        let typed = if from_hex {
            ColorRgba::parse_hex(self.hex.trim())
        } else {
            match (self.rgb[0].trim().parse(), self.rgb[1].trim().parse(), self.rgb[2].trim().parse()) {
                (Ok(r), Ok(g), Ok(b)) => Some(ColorRgba::rgb(r, g, b)),
                _ => None,
            }
        };
        match typed {
            Some(c) => self.set_color(c.with_alpha(255)),
            None => self.refresh_text(),
        }
    }

    fn square_texture(&mut self, ctx: &egui::Context) -> egui::TextureId {
        if self.square.as_ref().is_none_or(|(_, h)| *h != self.hue) {
            let n = AREA as usize;
            let mut pixels = Vec::with_capacity(n * n);
            for y in 0..n {
                let v = 1.0 - y as f64 / (n - 1) as f64;
                for x in 0..n {
                    pixels.push(color32(color_math::from_hsv(self.hue, x as f64 / (n - 1) as f64, v, 255)));
                }
            }
            let texture = ctx.load_texture("color-square", ColorImage::new([n, n], pixels), TextureOptions::LINEAR);
            self.square = Some((texture, self.hue));
        }
        self.square.as_ref().map(|(t, _)| t.id()).unwrap_or_default()
    }

    fn hue_texture(&mut self, ctx: &egui::Context) -> egui::TextureId {
        self.hue_strip
            .get_or_insert_with(|| {
                let pixels =
                    (0..256).map(|y| color32(color_math::from_hsv(y as f64 / 255.0 * 359.9, 1.0, 1.0, 255))).collect();
                ctx.load_texture("color-hue", ColorImage::new([1, 256], pixels), TextureOptions::LINEAR)
            })
            .id()
    }
}

/// A row of small color squares; returns the clicked one, and with `removable` the right-clicked one.
fn swatch_row(ui: &mut Ui, list: &[String], removable: bool) -> (Option<ColorRgba>, Option<String>) {
    let (mut picked, mut removed) = (None, None);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for hex in list {
            let Some(c) = ColorRgba::parse_hex(hex) else { continue };
            let (rect, response) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
            ui.painter().rect(rect, 3, color32(c), Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
            let tip = if removable { format!("{hex} (right-click removes it)") } else { hex.clone() };
            let response = response.on_hover_text(tip);
            if response.clicked() {
                picked = Some(c);
            }
            if removable && response.secondary_clicked() {
                removed = Some(hex.clone());
            }
        }
    });
    (picked, removed)
}

/// Draws the picker. `Some(Some(color))` = OK, `Some(None)` = Cancel, `None` = still open.
fn show_window(ctx: &egui::Context, picker: &mut ColorPicker, settings: &mut Settings) -> Option<Option<ColorRgba>> {
    let mut result = None;
    let square = picker.square_texture(ctx);
    let hue = picker.hue_texture(ctx);
    let nested = picker.nested;
    let body = |ui: &mut Ui| {
        ui.label(RichText::new(picker.title).heading());
        ui.add_space(8.0);
        ui.horizontal_top(|ui| {
            // Saturation (left to right) and brightness (top to bottom).
            let (rect, response) = ui.allocate_exact_size(vec2(AREA, AREA), Sense::click_and_drag());
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(square, rect, uv, Color32::WHITE);
            if let Some(p) =
                response.interact_pointer_pos().filter(|_| response.is_pointer_button_down_on() || response.clicked())
            {
                picker.saturation = ((p.x - rect.left()) / (AREA - 1.0)).clamp(0.0, 1.0) as f64;
                picker.value = (1.0 - (p.y - rect.top()) / (AREA - 1.0)).clamp(0.0, 1.0) as f64;
                picker.refresh_text();
            }
            let marker = pos2(
                rect.left() + picker.saturation as f32 * (AREA - 1.0),
                rect.top() + (1.0 - picker.value as f32) * (AREA - 1.0),
            );
            let ring = if picker.value > 0.6 && picker.saturation < 0.4 { Color32::BLACK } else { Color32::WHITE };
            ui.painter().circle_stroke(marker, 6.0, Stroke::new(2.0, ring));

            // Hue strip.
            ui.add_space(12.0);
            let (strip, response) = ui.allocate_exact_size(vec2(HUE_W, AREA), Sense::click_and_drag());
            ui.painter().image(hue, strip, uv, Color32::WHITE);
            if let Some(p) =
                response.interact_pointer_pos().filter(|_| response.is_pointer_button_down_on() || response.clicked())
            {
                picker.hue = ((p.y - strip.top()) / (AREA - 1.0)).clamp(0.0, 1.0) as f64 * 359.9;
                picker.refresh_text();
            }
            let y = strip.top() + picker.hue as f32 / 360.0 * (AREA - 1.0);
            let bar =
                Rect::from_min_max(Pos2::new(strip.left() - 2.0, y - 2.0), Pos2::new(strip.right() + 2.0, y + 2.0));
            ui.painter().rect_stroke(bar, 0, Stroke::new(1.5, Color32::WHITE), egui::StrokeKind::Middle);

            // New / current and the number boxes.
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_width(150.0);
                ui.label(RichText::new("New / Current").color(colors::SUBTLE_TEXT));
                let (both, _) = ui.allocate_exact_size(vec2(150.0, 56.0), Sense::hover());
                let (top, bottom) = both.split_top_bottom_at_fraction(0.5);
                ui.painter().rect_filled(top, 0, color32(picker.color()));
                ui.painter().rect_filled(bottom, 0, color32(picker.old));
                ui.add_space(8.0);
                let text_box = |ui: &mut Ui, label: &str, text: &mut String| -> bool {
                    ui.horizontal(|ui| {
                        ui.add_sized([40.0, 22.0], egui::Label::new(label));
                        let edit = ui.add(egui::TextEdit::singleline(text).desired_width(100.0));
                        edit.lost_focus()
                    })
                    .inner
                };
                if text_box(ui, "Hex", &mut picker.hex) {
                    picker.commit_text(true);
                }
                ui.add_space(4.0);
                let mut rgb_changed = false;
                for (i, label) in ["R", "G", "B"].iter().enumerate() {
                    rgb_changed |= text_box(ui, label, &mut picker.rgb[i]);
                }
                if rgb_changed {
                    picker.commit_text(false);
                }
                ui.add_space(8.0);
                let eyedropper = ui
                    .add(egui::Button::new("Eyedropper").min_size(vec2(150.0, 26.0)))
                    .on_hover_text("Click a color anywhere in the window to use it (Esc cancels)");
                if eyedropper.clicked() {
                    picker.wants_eyedropper = true;
                }
            });
        });

        // Color lists.
        ui.add_space(12.0);
        let mut picked = None;
        if !settings.brand_colors.is_empty() {
            ui.label(RichText::new("Brand colors").size(12.0).color(colors::SUBTLE_TEXT));
            picked = picked.or(swatch_row(ui, &settings.brand_colors, false).0);
        }
        ui.label(RichText::new("Recent").size(12.0).color(colors::SUBTLE_TEXT));
        picked = picked.or(swatch_row(ui, &settings.recent_colors, false).0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("My swatches").size(12.0).color(colors::SUBTLE_TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("+ Save this color").clicked() {
                    let hex = picker.color().to_hex();
                    if settings.swatches.len() < MAX_SWATCHES
                        && !settings.swatches.iter().any(|h| h.eq_ignore_ascii_case(&hex))
                    {
                        settings.swatches.push(hex);
                    }
                }
            });
        });
        let (mine, removed) = swatch_row(ui, &settings.swatches, true);
        picked = picked.or(mine);
        if let Some(hex) = removed {
            settings.swatches.retain(|h| !h.eq_ignore_ascii_case(&hex));
        }
        if settings.swatches.is_empty() {
            ui.label(
                RichText::new("Click + Save this color to keep colors you like here.")
                    .size(11.0)
                    .color(colors::SUBTLE_TEXT),
            );
        }
        ui.add_space(6.0);
        let quick: Vec<String> = QUICK.iter().map(|s| (*s).to_owned()).collect();
        picked = picked.or(swatch_row(ui, &quick, false).0);
        if let Some(c) = picked {
            picker.set_color(c);
        }

        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new("Cancel").min_size(vec2(90.0, 28.0))).clicked() {
                result = Some(None);
            }
            let ok = ui.add(egui::Button::new("OK").fill(colors::ACCENT).min_size(vec2(90.0, 28.0)));
            if ok.clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter) && !ctx.text_edit_focused()) {
                result = Some(Some(picker.color()));
            }
        });
    };
    let modal = if nested {
        super::frame::modal_over(ctx, "color-picker-nested", body)
    } else {
        super::frame::modal(ctx, "color-picker", body)
    };
    if result.is_none() && modal.should_close() {
        result = Some(None);
    }
    result
}

/// Draws the picker once, with the eyedropper hooked in. `Some(Some(color))` = OK, `Some(None)` = Cancel.
fn draw(ctx: &egui::Context, app: &mut FlashApp, picker: &mut ColorPicker) -> Option<Option<ColorRgba>> {
    if let Some(color) = app.eyedropper.result.take() {
        picker.set_color(color);
    }
    let answer = show_window(ctx, picker, &mut app.settings);
    if std::mem::take(&mut picker.wants_eyedropper) {
        app.eyedropper.start();
    }
    if let Some(Some(color)) = answer {
        let hex = color.to_hex();
        app.settings.recent_colors.retain(|h| !h.eq_ignore_ascii_case(&hex));
        app.settings.recent_colors.insert(0, hex);
        app.settings.recent_colors.truncate(MAX_RECENT);
    }
    answer
}

/// A picker opened by another window (New Project): draws it and returns the answer; nothing is applied.
pub fn show_nested(ctx: &egui::Context, app: &mut FlashApp, picker: &mut ColorPicker) -> Option<Option<ColorRgba>> {
    draw(ctx, app, picker)
}

/// Shows the picker; when it is closed with OK, uses the color and remembers it as a recent color.
/// Returns false once the picker is closed.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, picker: &mut ColorPicker) -> bool {
    let Some(answer) = draw(ctx, app, picker) else { return true };
    if let Some(color) = answer {
        match picker.target {
            ColorTarget::Foreground => app.foreground = color,
            ColorTarget::Fill => crate::commands::selection::fill(app, color),
            ColorTarget::TextColor => {
                crate::commands::text::change_text(app, "Text Color", |t| t.color = color);
            }
            ColorTarget::TextOutline => {
                crate::commands::text::change_text(app, "Outline Color", |t| t.outline_color = Some(color));
            }
            ColorTarget::Effect(which) => {
                // The strength (alpha) of an effect has its own slider: only the color changes here.
                crate::commands::objects::change_object(app, "Effect Color", |o| {
                    let mut l = o.looks();
                    let keep = |old: fe_core::imaging::ColorRgba| fe_core::imaging::ColorRgba { a: old.a, ..color };
                    match which {
                        EffectColor::Shadow => l.shadow.iter_mut().for_each(|s| s.color = keep(s.color)),
                        EffectColor::Glow => l.glow.iter_mut().for_each(|g| g.color = keep(g.color)),
                        EffectColor::InnerShadow => l.inner_shadow.iter_mut().for_each(|i| i.color = keep(i.color)),
                        EffectColor::Outline => l.outline.iter_mut().for_each(|x| x.color = keep(x.color)),
                        EffectColor::ThreeD => l.three_d.iter_mut().for_each(|t| t.color = Some(color)),
                    }
                    o.set_looks(&l);
                });
            }
            ColorTarget::GradientStop(i) => {
                crate::commands::objects::change_object(app, "Gradient Color", |o| {
                    if let Some(mut g) = o.gradient() {
                        if let Some(stop) = g.stops.get_mut(i) {
                            stop.0 = fe_core::imaging::ColorRgba { a: stop.0.a, ..color };
                        }
                        o.set_gradient(Some(&g));
                    }
                });
            }
            ColorTarget::Object(which) => crate::commands::codes::apply_color(app, which, color),
            ColorTarget::BrandColor(index) => {
                let hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
                match index {
                    Some(i) if i < app.settings.brand_colors.len() => app.settings.brand_colors[i] = hex,
                    _ => app.settings.brand_colors.push(hex),
                }
            }
            ColorTarget::ShapeFill | ColorTarget::ShapeStroke => {
                let fill = picker.target == ColorTarget::ShapeFill;
                crate::commands::objects::change_object(app, "Color", |o| {
                    if let Some(mut s) = o.shape() {
                        if fill {
                            s.fill = Some(color);
                        } else {
                            s.stroke = Some(color);
                        }
                        o.set_shape(&s);
                    }
                });
            }
        }
    }
    // The Brand Kit window comes back after its color was picked (or the picker was cancelled).
    if matches!(picker.target, ColorTarget::BrandColor(_)) {
        app.dialog = Some(super::Dialog::BrandKit(super::BrandKit));
    }
    false
}
