//! The built-in speed test (`flash-editor.exe --bench`): opens a 4000 x 3000 picture with 10 layers, zooms in
//! and out and pans for a few seconds, then writes frame times to `settings\bench.txt` and closes.

use std::fmt::Write as _;
use std::time::Instant;

use egui::{Ui, Vec2, vec2};
use fe_core::document::{AdjustmentStyle, Document, Layer};
use fe_core::imaging::{BlendMode, ColorRgba, PixelBuffer, PixelRect};

use crate::app::paths;

const WARM_UP: u32 = 60;
const ZOOM_FRAMES: u32 = 240;
const PAN_FRAMES: u32 = 240;

/// The test picture: a photo-like pattern, 8 half-see-through shapes with blend modes and an adjustment layer.
pub fn bench_document() -> Document {
    let (w, h) = (4000u32, 3000u32);
    let mut photo = PixelBuffer::new(w, h);
    for (i, p) in photo.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (i as u32 % w, i as u32 / w);
        *p = [(x / 8) as u8, (y / 8) as u8, ((x ^ y) / 4) as u8, 255];
    }
    let mut doc = Document::from_image(&photo, "Photo");
    let modes = [BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen, BlendMode::Overlay, BlendMode::SoftLight];
    for i in 0..8 {
        let mut layer = Layer::new(format!("Shape {}", i + 1));
        let rect = PixelRect::new(200 + i * 300, 150 + i * 250, 900, 700);
        layer.pixels_mut().fill_rect(rect, ColorRgba::new(40 * i as u8, 200, 90, 200));
        layer.blend = modes[i as usize % modes.len()];
        layer.set_opacity(0.8);
        doc.push_layer(layer);
    }
    doc.push_layer(Layer::adjustment(
        "Adjust",
        AdjustmentStyle { brightness: 10.0, saturation: 20.0, ..AdjustmentStyle::default() },
    ));
    doc
}

/// The running test.
pub struct Bench {
    frame: u32,
    last: Option<Instant>,
    intervals_ms: Vec<f32>,
    cpu_ms: Vec<f32>,
    render_ms_max: f32,
    textures_mb_max: f32,
    renderer_name: String,
}

/// What the test asks the view to do this frame.
pub enum BenchStep {
    /// Zoom to this factor of the fitted zoom, around the center.
    ZoomTo(f32),
    PanBy(Vec2),
    Wait,
    Done,
}

impl Bench {
    pub fn new(renderer_name: &str) -> Self {
        Self {
            frame: 0,
            last: None,
            intervals_ms: Vec::new(),
            cpu_ms: Vec::new(),
            render_ms_max: 0.0,
            textures_mb_max: 0.0,
            renderer_name: renderer_name.to_owned(),
        }
    }

    /// Records the last frame and says what to do next. `cpu_ms` = time egui spent on the last frame.
    pub fn step(&mut self, ui: &Ui, cpu_ms: f32, render_ms: f32, textures_mb: f32) -> BenchStep {
        let now = Instant::now();
        if self.frame > WARM_UP
            && let Some(last) = self.last
        {
            self.intervals_ms.push((now - last).as_secs_f32() * 1000.0);
            self.cpu_ms.push(cpu_ms);
            self.render_ms_max = self.render_ms_max.max(render_ms);
        }
        self.textures_mb_max = self.textures_mb_max.max(textures_mb);
        self.last = Some(now);
        self.frame += 1;
        ui.ctx().request_repaint();
        let f = self.frame;
        if f <= WARM_UP {
            BenchStep::Wait
        } else if f <= WARM_UP + ZOOM_FRAMES {
            let t = (f - WARM_UP) as f32 / ZOOM_FRAMES as f32;
            BenchStep::ZoomTo(1.0 + 7.0 * (t * std::f32::consts::PI * 2.0).sin().abs())
        } else if f <= WARM_UP + ZOOM_FRAMES + PAN_FRAMES {
            let a = (f as f32) * 0.05;
            BenchStep::PanBy(vec2(a.cos(), a.sin()) * 12.0)
        } else {
            BenchStep::Done
        }
    }

    /// Writes the results to `settings\bench.txt` (and the log) and returns the text.
    pub fn report(&self) -> String {
        let stats = |v: &[f32]| -> (f32, f32, f32) {
            let mut s = v.to_vec();
            s.sort_by(f32::total_cmp);
            let avg = s.iter().sum::<f32>() / s.len().max(1) as f32;
            let p95 = s.get(s.len() * 95 / 100).copied().unwrap_or(0.0);
            (avg, p95, s.last().copied().unwrap_or(0.0))
        };
        let (avg, p95, max) = stats(&self.intervals_ms);
        let (cpu_avg, cpu_p95, cpu_max) = stats(&self.cpu_ms);
        let mut text = String::new();
        let _ = writeln!(text, "Flash Editor canvas benchmark ({} renderer)", self.renderer_name);
        let _ =
            writeln!(text, "Picture 4000 x 3000, 10 layers; {} frames of zooming and panning", self.intervals_ms.len());
        let _ = writeln!(text, "Frame interval: avg {avg:.1} ms, 95% {p95:.1} ms, max {max:.1} ms (16.7 ms = 60 fps)");
        let _ = writeln!(text, "CPU per frame:  avg {cpu_avg:.1} ms, 95% {cpu_p95:.1} ms, max {cpu_max:.1} ms");
        let _ = writeln!(text, "Slowest tile update in a frame: {:.1} ms", self.render_ms_max);
        let _ = writeln!(text, "Largest texture cache: {:.1} MB", self.textures_mb_max);
        log::info!("{text}");
        let _ = std::fs::create_dir_all(paths::settings_dir());
        let _ = std::fs::write(paths::settings_dir().join("bench.txt"), &text);
        text
    }
}
