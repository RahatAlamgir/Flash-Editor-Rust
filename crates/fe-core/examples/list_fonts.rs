//! Lists installed font families that contain a word (to check how fonts are found). Usage: `list_fonts segoe`.
fn main() {
    let word = std::env::args().nth(1).unwrap_or_default().to_lowercase();
    for name in fe_core::text::fonts::families() {
        if name.to_lowercase().contains(&word) {
            println!("{name}");
        }
    }
}
