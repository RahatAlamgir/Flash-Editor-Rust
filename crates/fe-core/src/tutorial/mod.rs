//! The tutorial project (C# `DemoProject`): 14 pages that each teach a few tools with real objects to click and
//! change. Everything is made from the program's own parts, so it always matches the program.

mod page;
mod pages_a;
mod pages_b;
mod pictures;

use crate::document::Page;

/// The file name of the tutorial.
pub const FILE_NAME: &str = "Flash Editor Tutorial.fe";

/// The 14 pages (1280 x 720).
pub fn build_pages() -> Vec<Page> {
    vec![
        pages_a::welcome(),
        pages_a::cut_out(),
        pages_a::photo_look(),
        pages_a::selecting(),
        pages_a::text_page(),
        pages_b::shapes_page(),
        pages_b::graphics_page(),
        pages_b::badges_page(),
        pages_b::charts_page(),
        pages_b::smartart_page(),
        pages_b::photos_page(),
        pages_b::effects_page(),
        pages_b::codes_page(),
        pages_b::wrap_up(),
    ]
}
