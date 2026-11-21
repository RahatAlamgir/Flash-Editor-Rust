//! Speed and memory check for a big document. Ignored in normal test runs; run it with:
//! `cargo test -p fe-core --release --test speed_compositor -- --ignored --nocapture`

use std::time::Instant;

use fe_core::document::{AdjustmentStyle, Document, Layer};
use fe_core::imaging::{BlendMode, ColorRgba, PixelBuffer, PixelRect};
use fe_core::paint::compositor;

#[test]
#[ignore = "speed check, run on demand"]
fn flatten_a_4000_x_3000_document_with_10_layers() {
    let (w, h) = (4000, 3000);
    let mut photo = PixelBuffer::new(w, h);
    for (i, p) in photo.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = ((i as u32 % w) as u8, (i as u32 / w) as u8);
        *p = [x, y, x ^ y, 255];
    }
    let mut doc = Document::from_image(&photo, "Photo");
    let modes = [BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen, BlendMode::Overlay, BlendMode::SoftLight];
    for i in 0..8 {
        let mut layer = Layer::new(format!("Shape {i}"));
        let rect = PixelRect::new(200 + i * 300, 150 + i * 250, 900, 700);
        layer.pixels_mut().fill_rect(rect, ColorRgba::new(40 * i as u8, 200, 90, 200));
        layer.blend = modes[i as usize % modes.len()];
        layer.set_opacity(0.8);
        doc.push_layer(layer);
    }
    let adjust = AdjustmentStyle { brightness: 10.0, saturation: 20.0, ..AdjustmentStyle::default() };
    doc.push_layer(Layer::adjustment("Adjust", adjust));

    let start = Instant::now();
    let flat = compositor::flatten(&doc);
    let full = start.elapsed();
    let start = Instant::now();
    let mut target = flat.clone();
    compositor::flatten_region(&doc, PixelRect::new(1000, 1000, 256, 256), &mut target);
    let region = start.elapsed();

    let dense = 10 * (w as usize * h as usize * 4);
    println!("flatten 4000x3000, 10 layers: {full:?}; one 256 px region: {region:?}");
    println!(
        "layer memory: {:.1} MB (canvas-sized buffers like the C# version would need {:.1} MB)",
        doc.memory_bytes() as f64 / 1e6,
        dense as f64 / 1e6
    );
}

#[test]
#[ignore = "speed check, run on demand"]
fn paint_a_long_stroke_on_a_4000_x_3000_picture() {
    use fe_core::imaging::{PointD, TileMap};
    use fe_core::paint::brush::{BrushSettings, BrushType};
    use fe_core::paint::stroke::{Effect, PaintStroke};
    let canvas = PixelRect::new(0, 0, 4000, 3000);
    let mut pixels = TileMap::empty();
    pixels.fill_rect(canvas, ColorRgba::WHITE);
    for (size, kind) in
        [(30.0, BrushType::Round), (150.0, BrushType::Round), (150.0, BrushType::Chalk), (150.0, BrushType::Spray)]
    {
        let brush = BrushSettings { size, hardness: 0.5, opacity: 0.8, kind };
        let mut stroke = PaintStroke::new(&pixels, brush, Effect::Paint(ColorRgba::rgb(200, 30, 30)), None, canvas);
        // A mouse move every 16 ms, 25 px apart (a quick drag across the picture).
        let start = Instant::now();
        let mut worst = 0.0f64;
        let mut steps = 0;
        for i in 0..150 {
            let t = Instant::now();
            stroke
                .add_point(&mut pixels, PointD::new(100.0 + i as f64 * 25.0, 1500.0 + (i as f64 / 10.0).sin() * 800.0));
            worst = worst.max(t.elapsed().as_secs_f64() * 1000.0);
            steps += 1;
        }
        let total = start.elapsed().as_secs_f64() * 1000.0;
        println!("{kind:?} {size} px: {:.2} ms per mouse move on average, worst {worst:.2} ms", total / steps as f64);
    }
}
