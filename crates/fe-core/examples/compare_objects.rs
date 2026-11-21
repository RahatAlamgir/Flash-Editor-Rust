//! Draws every object of a project with the Rust renderer and compares it with the drawing the C# app saved.
//! Usage: `cargo run -p fe-core --release --example compare_objects -- file.flash [folder-for-pictures]`

use fe_core::imaging::PixelRect;
use fe_core::render;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: compare_objects <file.flash> [out-folder]");
    let out = args.next();
    let content = fe_core::io::project::open(&path).expect("cannot open");
    let mut results = Vec::new();
    for (pi, page) in content.pages.iter().enumerate() {
        let canvas = page.doc.bounds();
        for layer in page.doc.layers() {
            let Some(obj) = layer.object() else { continue };
            let what = format!("p{pi} {} ({})", layer.name.lines().next().unwrap_or(""), obj.kind());
            if !obj.can_draw() || obj.has_looks() {
                continue;
            }
            let Some(mine) = render::render_object(obj, canvas) else { continue };
            let area = layer.pixels().visible_bounds().union(&mine.visible_bounds()).intersect(&canvas);
            if area.is_empty() {
                continue;
            }
            let (a, b) = (layer.pixels().read_rect(area), mine.read_rect(area));
            // Difference of premultiplied colors, so see-through pixels count little.
            let (mut sum, mut big) = (0.0f64, 0usize);
            for (p, q) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0) {
                let d = (0..4)
                    .map(|c| {
                        let pa = if c == 3 { p[3] as f64 } else { p[c] as f64 * p[3] as f64 / 255.0 };
                        let qa = if c == 3 { q[3] as f64 } else { q[c] as f64 * q[3] as f64 / 255.0 };
                        (pa - qa).abs()
                    })
                    .fold(0.0, f64::max);
                sum += d;
                if d > 64.0 {
                    big += 1;
                }
            }
            let n = (area.width * area.height) as f64;
            results.push((sum / n, big as f64 / n * 100.0, what.clone()));
            if let Some(dir) = &out {
                let save = |tiles: &fe_core::imaging::TileMap, tag: &str| {
                    let buf = tiles.to_buffer(area);
                    let name = format!("{dir}/{}-{tag}.png", what.replace(['/', '\\', ':', '"', '?', '*'], "_"));
                    let _ = fe_core::io::encode::save(&buf, &name, fe_core::io::ImageFileFormat::Png, 90, None);
                };
                save(layer.pixels(), "csharp");
                save(&mine, "rust");
            }
        }
    }
    results.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (mean, big, what) in &results {
        println!("{mean:6.2} avg  {big:5.2}% >64   {what}");
    }
    let _ = PixelRect::EMPTY;
}
