//! Prints the text objects of a project with their box and the size the Rust layout measures (for comparing).
fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_text <file.flash>");
    let content = fe_core::io::project::open(&path).expect("cannot open");
    for (pi, page) in content.pages.iter().enumerate() {
        for layer in page.doc.layers() {
            let Some(obj) = layer.object() else { continue };
            let Some(t) = obj.text() else { continue };
            let b = obj.box_().unwrap_or_default();
            let (w, h) = fe_core::render::measure_text(&t);
            println!(
                "p{pi} {:?} font={} size={} bold={} box=({:.1},{:.1} {:.1}x{:.1}) measured={w}x{h} spacing={}/{}/{} arc={}",
                t.text.lines().next().unwrap_or(""),
                t.family,
                t.size,
                t.bold,
                b.x,
                b.y,
                b.width,
                b.height,
                t.line_spacing,
                t.letter_spacing,
                t.word_spacing,
                t.arc
            );
        }
    }
}
