//! Opens a `.flash` project and saves it again with the Rust writer (for checking compatibility with the C#
//! app). Usage: `cargo run -p fe-core --release --example resave -- in.flash out.flash`

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: resave <in.flash> <out.flash>");
        std::process::exit(2);
    }
    let start = std::time::Instant::now();
    let content = fe_core::io::project::open(&args[1]).unwrap_or_else(|e| {
        eprintln!("cannot open: {e}");
        std::process::exit(1);
    });
    let opened = start.elapsed();
    fe_core::io::project::save(&content.pages, content.active_page, &args[2]).unwrap_or_else(|e| {
        eprintln!("cannot save: {e}");
        std::process::exit(1);
    });
    println!("opened in {opened:?}, saved in {:?}", start.elapsed() - opened);
}
