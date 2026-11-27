//! The eyedropper of the color picker: takes a picture of the program window (without any dialog), shows it full size
//! with a magnifier at the mouse, and the clicked pixel becomes the picked color. Esc or a right click cancels.
//!
//! Steps: `start` (the dialogs stop being drawn) -> `tick` asks for the picture -> it arrives -> `paint` shows it and waits
//! for the click -> `result` holds the color for the picker that asked.

use std::sync::{Arc, Mutex};

use egui::{
    Align2, Color32, ColorImage, Context, CursorIcon, FontId, Key, Modifiers, Order, Rect, Sense, Stroke,
    TextureHandle, pos2, vec2,
};
use fe_core::imaging::ColorRgba;
use fe_ui_kit::theme::colors;

/// Pixels shown around the mouse in the magnifier (odd).
const LOUPE_PIXELS: i32 = 11;
/// Size of one magnified pixel.
const LOUPE_CELL: f32 = 12.0;

type Slot = Arc<Mutex<Option<Arc<ColorImage>>>>;

enum Phase {
    Idle,
    /// The dialogs were hidden this frame; the picture is asked for in the next one.
    Hidden,
    /// The picture was asked for.
    Waiting(Slot),
    /// The picture is on screen; the user is choosing a pixel.
    Picking(Arc<ColorImage>, TextureHandle),
}

pub struct Eyedropper {
    phase: Phase,
    /// The color that was picked, for the color picker to take.
    pub result: Option<ColorRgba>,
}

impl Default for Eyedropper {
    fn default() -> Self {
        Self { phase: Phase::Idle, result: None }
    }
}

impl Eyedropper {
    /// Begins picking (from the color picker's button).
    pub fn start(&mut self) {
        self.phase = Phase::Hidden;
        self.result = None;
    }

    /// True while picking: the dialogs are not drawn, so they are not in the picture.
    pub fn active(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    /// Runs the steps that wait for the picture; call once per frame before anything is drawn.
    pub fn tick(&mut self, ctx: &Context) {
        match &self.phase {
            Phase::Idle => {}
            Phase::Hidden => {
                let slot: Slot = Arc::new(Mutex::new(None));
                let target = slot.clone();
                ctx.request_screenshot(move |image| {
                    if let Ok(mut place) = target.lock() {
                        *place = Some(image);
                    }
                });
                ctx.request_repaint();
                self.phase = Phase::Waiting(slot);
            }
            Phase::Waiting(slot) => {
                let arrived = slot.lock().ok().and_then(|mut place| place.take());
                match arrived {
                    Some(image) => {
                        let texture = ctx.load_texture("eyedropper", (*image).clone(), egui::TextureOptions::NEAREST);
                        self.phase = Phase::Picking(image, texture);
                        ctx.request_repaint();
                    }
                    None => ctx.request_repaint_after(std::time::Duration::from_millis(16)),
                }
            }
            Phase::Picking(..) => {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                    self.phase = Phase::Idle;
                }
            }
        }
    }

    /// Draws the picture over the whole window and handles the mouse.
    pub fn paint(&mut self, ctx: &Context) {
        let Phase::Picking(image, texture) = &self.phase else { return };
        let screen = ctx.content_rect();
        let (w, h) = (image.size[0] as i32, image.size[1] as i32);
        let mut picked = None;
        let mut cancel = false;
        egui::Area::new(egui::Id::NULL.with("eyedropper")).order(Order::Tooltip).fixed_pos(screen.min).show(
            ctx,
            |ui| {
                let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter().image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                let Some(pointer) = response.hover_pos() else { return };
                let px = (((pointer.x - rect.left()) / rect.width()) * w as f32) as i32;
                let py = (((pointer.y - rect.top()) / rect.height()) * h as f32) as i32;
                let at = |x: i32, y: i32| -> Option<Color32> {
                    (x >= 0 && y >= 0 && x < w && y < h).then(|| image.pixels[(y * w + x) as usize])
                };
                let Some(under) = at(px, py) else { return };
                // The magnifier: a block of pixels around the mouse, with the middle one boxed, and its hex value below.
                let side = LOUPE_PIXELS as f32 * LOUPE_CELL;
                let mut corner = pointer + vec2(22.0, 22.0);
                if corner.x + side > screen.right() - 4.0 {
                    corner.x = pointer.x - 22.0 - side;
                }
                if corner.y + side + 24.0 > screen.bottom() - 4.0 {
                    corner.y = pointer.y - 22.0 - side - 24.0;
                }
                let painter = ui.painter();
                let frame = Rect::from_min_size(corner, vec2(side, side));
                painter.rect_filled(frame.expand(3.0), 4, Color32::BLACK);
                for dy in 0..LOUPE_PIXELS {
                    for dx in 0..LOUPE_PIXELS {
                        let color =
                            at(px + dx - LOUPE_PIXELS / 2, py + dy - LOUPE_PIXELS / 2).unwrap_or(Color32::BLACK);
                        let cell = Rect::from_min_size(
                            corner + vec2(dx as f32 * LOUPE_CELL, dy as f32 * LOUPE_CELL),
                            vec2(LOUPE_CELL, LOUPE_CELL),
                        );
                        painter.rect_filled(cell, 0, color);
                    }
                }
                let middle = Rect::from_min_size(
                    corner + vec2((LOUPE_PIXELS / 2) as f32 * LOUPE_CELL, (LOUPE_PIXELS / 2) as f32 * LOUPE_CELL),
                    vec2(LOUPE_CELL, LOUPE_CELL),
                );
                painter.rect_stroke(middle, 0, Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Outside);
                painter.rect_stroke(middle.expand(2.0), 0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Outside);
                let color = ColorRgba::rgb(under.r(), under.g(), under.b());
                let label = Rect::from_min_size(corner + vec2(0.0, side + 6.0), vec2(side, 20.0));
                painter.rect_filled(label, 3, Color32::BLACK);
                painter.text(
                    label.center(),
                    Align2::CENTER_CENTER,
                    color.to_hex(),
                    FontId::monospace(12.0),
                    colors::TEXT,
                );
                if response.clicked() {
                    picked = Some(color);
                }
                if response.secondary_clicked() {
                    cancel = true;
                }
            },
        );
        if picked.is_some() || cancel {
            self.result = picked;
            self.phase = Phase::Idle;
        }
        ctx.request_repaint(); // the magnifier follows the mouse
    }
}
