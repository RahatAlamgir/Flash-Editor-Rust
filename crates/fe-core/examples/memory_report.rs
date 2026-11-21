//! Prints how much memory the layers of a project hold. Usage:
//! `cargo run -p fe-core --release --example memory_report -- file.flash`

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: memory_report <file.flash>");
        std::process::exit(2);
    });
    let content = fe_core::io::project::open(&path).unwrap_or_else(|e| {
        eprintln!("cannot open: {e}");
        std::process::exit(1);
    });
    let (mut layers, mut tiles, mut bytes, mut objects, mut object_png) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut used_pixels = 0u64;
    for page in &content.pages {
        for layer in page.doc.layers() {
            layers += 1;
            tiles += layer.pixels().tile_count();
            bytes += layer.pixels().memory_bytes() + layer.mask().map_or(0, |m| m.memory_bytes());
            let b = layer.visible_bounds();
            used_pixels += b.area().max(0) as u64;
            if let Some(o) = layer.object() {
                objects += 1;
                object_png += o.images.values().map(|v| v.len()).sum::<usize>();
            }
        }
    }
    println!("pages {}, layers {layers} ({objects} objects)", content.pages.len());
    println!(
        "tiles {tiles} = {:.1} MB; area actually painted {:.1} MB",
        bytes as f64 / 1e6,
        used_pixels as f64 * 4.0 / 1e6
    );
    println!("object pictures (PNG bytes kept for saving): {:.1} MB", object_png as f64 / 1e6);
}
