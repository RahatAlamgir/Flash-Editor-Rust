//! Diagrams like Office's SmartArt: processes, lists, cycles, hierarchies, relationships, matrices and pyramids.
//! You give the words, the program draws the shapes. The result is a group of shapes and text, so it can be
//! ungrouped and changed (C# `SmartArt`).

mod cycles;
mod flows;
mod relations;

use super::maker::{Maker, c};
use crate::imaging::ColorRgba;
use crate::objects::StoredObject;

pub struct LayoutInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub min: usize,
    pub max: usize,
    pub sample: &'static [&'static str],
}

pub const CATEGORIES: [&str; 7] = ["List", "Process", "Cycle", "Hierarchy", "Relationship", "Matrix", "Pyramid"];

const FOUR: &[&str] = &["Plan", "Design", "Build", "Launch"];
const THREE: &[&str] = &["Idea", "Work", "Result"];

const fn l(
    id: &'static str,
    name: &'static str,
    category: &'static str,
    min: usize,
    max: usize,
    sample: &'static [&'static str],
) -> LayoutInfo {
    LayoutInfo { id, name, category, min, max, sample }
}

pub const LAYOUTS: [LayoutInfo; 16] = [
    l("list", "Basic block list", "List", 2, 8, FOUR),
    l("vlist", "Vertical bullet list", "List", 2, 8, FOUR),
    l("process", "Basic process", "Process", 2, 6, FOUR),
    l("vprocess", "Vertical process", "Process", 2, 6, FOUR),
    l("chevron", "Chevron process", "Process", 2, 6, FOUR),
    l("steps", "Step up process", "Process", 2, 6, FOUR),
    l("timeline", "Timeline", "Process", 2, 6, FOUR),
    l("cycle", "Basic cycle", "Cycle", 3, 8, FOUR),
    l("hub", "Hub and spokes", "Cycle", 3, 8, &["Goal", "Time", "People", "Money", "Tools"]),
    l("org", "Organization chart", "Hierarchy", 2, 6, &["Manager", "Design", "Sales", "Support"]),
    l("tree", "Horizontal tree", "Hierarchy", 2, 7, &["Product", "Idea", "Design", "Test"]),
    l("venn", "Basic Venn", "Relationship", 2, 4, THREE),
    l("target", "Target", "Relationship", 2, 5, THREE),
    l("funnel", "Funnel", "Relationship", 2, 5, FOUR),
    l("matrix", "Basic matrix", "Matrix", 4, 4, FOUR),
    l("pyramid", "Basic pyramid", "Pyramid", 2, 6, THREE),
];

pub fn layout(id: &str) -> Option<&'static LayoutInfo> {
    LAYOUTS.iter().find(|l| l.id == id)
}

/// The color sets to choose from.
pub fn schemes() -> Vec<(&'static str, Vec<ColorRgba>)> {
    vec![
        (
            "Colorful",
            vec![
                c(59, 130, 246),
                c(249, 115, 22),
                c(34, 197, 94),
                c(168, 85, 247),
                c(236, 72, 153),
                c(234, 179, 8),
                c(14, 165, 233),
                c(239, 68, 68),
            ],
        ),
        (
            "Blue",
            vec![
                c(30, 64, 175),
                c(37, 99, 235),
                c(59, 130, 246),
                c(96, 165, 250),
                c(14, 116, 144),
                c(14, 165, 233),
                c(56, 189, 248),
                c(29, 78, 216),
            ],
        ),
        (
            "Green",
            vec![
                c(21, 128, 61),
                c(34, 197, 94),
                c(22, 163, 74),
                c(101, 163, 13),
                c(5, 150, 105),
                c(132, 204, 22),
                c(13, 148, 136),
                c(74, 222, 128),
            ],
        ),
        (
            "Warm",
            vec![
                c(220, 38, 38),
                c(249, 115, 22),
                c(234, 179, 8),
                c(236, 72, 153),
                c(190, 18, 60),
                c(251, 146, 60),
                c(245, 158, 11),
                c(244, 63, 94),
            ],
        ),
        (
            "Purple",
            vec![
                c(88, 28, 135),
                c(124, 58, 237),
                c(168, 85, 247),
                c(192, 132, 252),
                c(190, 24, 93),
                c(217, 70, 239),
                c(99, 102, 241),
                c(139, 92, 246),
            ],
        ),
        (
            "Gray",
            vec![
                c(30, 41, 59),
                c(51, 65, 85),
                c(71, 85, 105),
                c(100, 116, 139),
                c(30, 41, 59),
                c(51, 65, 85),
                c(71, 85, 105),
                c(100, 116, 139),
            ],
        ),
    ]
}

pub(super) const WHITE: ColorRgba = ColorRgba::rgb(255, 255, 255);
pub(super) const DARK: ColorRgba = ColorRgba::rgb(30, 41, 59);
pub(super) const SOFT: ColorRgba = ColorRgba::rgb(226, 232, 240);

/// Where the diagram goes and the words and colors it is made of.
pub(super) struct Job<'a> {
    pub canvas_w: f64,
    pub canvas_h: f64,
    pub width: f64,
    pub words: &'a [String],
    pub colors: &'a [ColorRgba],
}

impl Job<'_> {
    pub fn maker(&self, design_height: f64) -> Maker {
        Maker::new(self.canvas_w, self.canvas_h, self.width, design_height)
    }

    pub fn color(&self, i: usize) -> ColorRgba {
        self.colors[i % self.colors.len()]
    }
}

/// The diagram for these words, centered on a canvas and about `width` pixels wide. `None` for an unknown layout.
pub fn build(
    id: &str,
    items: &[String],
    scheme: &str,
    canvas_w: f64,
    canvas_h: f64,
    width: f64,
) -> Option<StoredObject> {
    let info = layout(id)?;
    let mut words: Vec<String> =
        items.iter().map(|i| i.trim().to_owned()).filter(|i| !i.is_empty()).take(info.max).collect();
    while words.len() < info.min {
        words.push(format!("Item {}", words.len() + 1));
    }
    let schemes = schemes();
    let colors = schemes.iter().find(|s| s.0 == scheme).map_or_else(|| schemes[0].1.clone(), |s| s.1.clone());
    let job = Job { canvas_w, canvas_h, width, words: &words, colors: &colors };
    let maker = match id {
        "process" => flows::process(&job),
        "vprocess" => flows::vertical_process(&job),
        "chevron" => flows::chevrons(&job),
        "steps" => flows::steps(&job),
        "timeline" => flows::timeline(&job),
        "list" => flows::list(&job),
        "vlist" => flows::vertical_list(&job),
        "cycle" => cycles::cycle(&job),
        "hub" => cycles::hub(&job),
        "org" => cycles::org_chart(&job),
        "tree" => cycles::tree(&job),
        "venn" => relations::venn(&job),
        "target" => relations::target(&job),
        "funnel" => relations::funnel(&job),
        "matrix" => relations::matrix(&job),
        "pyramid" => relations::pyramid(&job),
        _ => return None,
    };
    maker.done()
}
