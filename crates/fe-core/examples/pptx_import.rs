//! Opens a .pptx and saves a picture of every slide: `cargo run --release -p fe-core --example pptx_import -- in.pptx out_dir`.
use fe_core::io::{encode, pptx};
use fe_core::paint::compositor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let imported = pptx::open_pptx(&args[1]).expect("opened");
    std::fs::create_dir_all(&args[2]).expect("folder");
    println!("{} slides, {} skipped", imported.pages.len(), imported.skipped);
    for (i, page) in imported.pages.iter().enumerate() {
        let flat = compositor::flatten(&page.doc);
        let png = encode::encode_png(&flat).expect("png");
        std::fs::write(format!("{}/slide{}.png", args[2], i + 1), png).expect("write");
        println!("slide {}: {} layers", i + 1, page.doc.layers().len());
    }
}
