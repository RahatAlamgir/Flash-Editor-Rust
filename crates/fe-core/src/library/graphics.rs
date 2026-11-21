//! Ready-made graphics like the ones Canva offers: lines and dividers, arrows, blobs and shapes, decorations,
//! frames and borders, patterns. Each one is a group of shapes, so it can be ungrouped, recolored and changed
//! (C# `Graphics`).

use std::f64::consts::PI;

use super::maker::{Maker, c};
use crate::imaging::{ColorRgba, PointD};
use crate::objects::{CurveNode, ShapeKind::*, StoredObject};

pub struct GraphicInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

pub const CATEGORIES: [&str; 6] =
    ["Lines and dividers", "Arrows", "Blobs and shapes", "Decorations", "Frames and borders", "Patterns"];

const LINES: &str = "Lines and dividers";
const ARROWS: &str = "Arrows";
const BLOBS: &str = "Blobs and shapes";
const DECO: &str = "Decorations";
const FRAMES: &str = "Frames and borders";
const PATTERNS: &str = "Patterns";

const fn g(id: &'static str, name: &'static str, category: &'static str) -> GraphicInfo {
    GraphicInfo { id, name, category }
}

pub const ALL: [GraphicInfo; 50] = [
    g("line", "Line", LINES),
    g("dashed", "Dashed line", LINES),
    g("dotted", "Dotted line", LINES),
    g("thick", "Thick line", LINES),
    g("double", "Double line", LINES),
    g("arrowline", "Arrow line", LINES),
    g("wave", "Wavy line", LINES),
    g("zigzag", "Zigzag", LINES),
    g("dividerstar", "Star divider", LINES),
    g("dividerdots", "Dot divider", LINES),
    g("underline", "Brush underline", LINES),
    g("arrowbig", "Big arrow", ARROWS),
    g("arrowdouble", "Double arrow", ARROWS),
    g("chevrons", "Chevrons", ARROWS),
    g("curved", "Curved arrow", ARROWS),
    g("loop", "Loop arrow", ARROWS),
    g("swoosh", "Swoosh", ARROWS),
    g("arrowpair", "Two arrows", ARROWS),
    g("blob1", "Blob 1", BLOBS),
    g("blob2", "Blob 2", BLOBS),
    g("blob3", "Blob 3", BLOBS),
    g("blob4", "Blob 4", BLOBS),
    g("rings", "Rings", BLOBS),
    g("cluster", "Circle cluster", BLOBS),
    g("halfsun", "Half sun", BLOBS),
    g("triangles", "Triangles", BLOBS),
    g("pills", "Pills", BLOBS),
    g("sparkles", "Sparkles", DECO),
    g("confetti", "Confetti", DECO),
    g("starsrow", "Row of stars", DECO),
    g("hearts", "Hearts", DECO),
    g("starburst", "Starburst", DECO),
    g("sun", "Sun", DECO),
    g("flower", "Flower", DECO),
    g("moon", "Moon and stars", DECO),
    g("clouds", "Clouds", DECO),
    g("crown", "Crown", DECO),
    g("border", "Simple border", FRAMES),
    g("borderdouble", "Double border", FRAMES),
    g("borderround", "Rounded border", FRAMES),
    g("corners", "Corner marks", FRAMES),
    g("ticket", "Ticket", FRAMES),
    g("photo", "Photo frame", FRAMES),
    g("dots", "Dots", PATTERNS),
    g("stripes", "Stripes", PATTERNS),
    g("checker", "Checkerboard", PATTERNS),
    g("circlegrid", "Circle grid", PATTERNS),
    g("plus", "Plus signs", PATTERNS),
    g("zigzagrows", "Zigzag rows", PATTERNS),
    g("waverows", "Wave rows", PATTERNS),
];

/// The display name of a graphic.
pub fn name_of(id: &str) -> &'static str {
    ALL.iter().find(|i| i.id == id).map_or("Graphic", |i| i.name)
}

fn p(x: f64, y: f64) -> PointD {
    PointD::new(x, y)
}

fn node(point: PointD, handle_in: PointD, handle_out: PointD) -> CurveNode {
    CurveNode { point, handle_in, handle_out }
}

/// A smooth wave: `half_waves` crests and troughs from the left to the right of the unit box.
fn wave(half_waves: usize) -> Vec<CurveNode> {
    let handle = 0.36 / half_waves as f64;
    (0..=half_waves)
        .map(|i| {
            let (x, y) = (i as f64 / half_waves as f64, if i % 2 == 0 { 1.0 } else { 0.0 });
            node(p(x, y), p(if i == 0 { x } else { x - handle }, y), p(if i == half_waves { x } else { x + handle }, y))
        })
        .collect()
}

fn zigzag(points: usize) -> Vec<CurveNode> {
    (0..=points).map(|i| CurveNode::corner(p(i as f64 / points as f64, if i % 2 == 0 { 1.0 } else { 0.0 }))).collect()
}

/// An organic closed shape: points around the center at the given distances (0..0.5), joined smoothly.
fn blob(radii: &[f64]) -> Vec<CurveNode> {
    let n = radii.len();
    radii
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let angle = i as f64 * PI * 2.0 / n as f64 - PI / 2.0;
            let point = p(0.5 + angle.cos() * r, 0.5 + angle.sin() * r);
            let handle = r * (4.0 / 3.0) * (PI / (2.0 * n as f64)).tan();
            let tangent = p(-angle.sin(), angle.cos());
            node(
                point,
                p(point.x - tangent.x * handle, point.y - tangent.y * handle),
                p(point.x + tangent.x * handle, point.y + tangent.y * handle),
            )
        })
        .collect()
}

fn swoosh() -> Vec<CurveNode> {
    vec![node(p(0.0, 0.85), p(0.0, 0.85), p(0.25, 0.1)), node(p(1.0, 0.25), p(0.55, 0.0), p(1.0, 0.25))]
}

fn arrow_loop() -> Vec<CurveNode> {
    vec![
        node(p(0.0, 0.9), p(0.0, 0.9), p(0.25, 0.9)),
        node(p(0.62, 0.55), p(0.5, 0.95), p(0.75, 0.15)),
        node(p(0.42, 0.18), p(0.5, 0.05), p(0.3, 0.3)),
        node(p(1.0, 0.7), p(0.7, 0.4), p(1.0, 0.7)),
    ]
}

fn crown() -> Vec<CurveNode> {
    [(0.0, 1.0), (0.0, 0.25), (0.25, 0.55), (0.5, 0.0), (0.75, 0.55), (1.0, 0.25), (1.0, 1.0)]
        .iter()
        .map(|(x, y)| CurveNode::corner(p(*x, *y)))
        .collect()
}

/// The same "random" numbers every time (the C# version uses `new Random(7)`).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f64) / f64::from(1u32 << 31)
    }
}

/// The graphic, centered on a canvas, about `width` pixels wide. `None` for an unknown id.
pub fn build(id: &str, canvas_w: f64, canvas_h: f64, width: f64) -> Option<StoredObject> {
    let (blue, pink, amber, green, purple) =
        (c(59, 130, 246), c(244, 114, 182), c(250, 204, 21), c(34, 197, 94), c(139, 92, 246));
    let (dark, orange, red, white, sky, gray) =
        (c(30, 41, 59), c(249, 115, 22), c(239, 68, 68), c(255, 255, 255), c(56, 189, 248), c(148, 163, 184));
    let rainbow: [ColorRgba; 8] = [blue, pink, amber, green, purple, orange, sky, red];
    let mut rng = Rng(7);
    let mk = |h: f64| Maker::new(canvas_w, canvas_h, width, h);
    let mut m;
    match id {
        // lines and dividers
        "line" => {
            m = mk(4.0);
            m.line(50, 2, 100, 0, dark, 1.6);
        }
        "dashed" => {
            m = mk(4.0);
            m.line(50, 2, 100, 0, dark, 1.6).dashed(5, 3.5);
        }
        "dotted" => {
            m = mk(4.0);
            m.line(50, 2, 100, 0, pink, 2.2).dotted(3.2);
        }
        "thick" => {
            m = mk(8.0);
            m.line(50, 4, 100, 0, blue, 5);
        }
        "double" => {
            m = mk(8.0);
            m.line(50, 1.5, 100, 0, dark, 1.2);
            m.line(50, 6.5, 100, 0, dark, 1.2);
        }
        "arrowline" => {
            m = mk(10.0);
            m.line(48, 5, 92, 0, dark, 2).arrow();
        }
        "wave" => {
            m = mk(14.0);
            m.path("Wave", 0, 2, 100, 10, wave(10), false).stroke(blue, 2.4);
        }
        "zigzag" => {
            m = mk(14.0);
            m.path("Zigzag", 0, 2, 100, 10, zigzag(14), false).stroke(orange, 2.4);
        }
        "dividerstar" => {
            m = mk(12.0);
            m.line(22, 6, 44, 0, dark, 1.2);
            m.line(78, 6, 44, 0, dark, 1.2);
            m.shape("Star", Star, 44, 0, 12, 12).fill(amber);
        }
        "dividerdots" => {
            m = mk(8.0);
            m.line(20, 4, 36, 0, gray, 1);
            m.line(80, 4, 36, 0, gray, 1);
            for i in 0..3 {
                m.shape("Dot", Ellipse, 44.0 + f64::from(i) * 6.0, 1.8, 4.4, 4.4).fill(if i == 1 {
                    pink
                } else {
                    gray
                });
            }
        }
        "underline" => {
            m = mk(12.0);
            let nodes = vec![node(p(0.0, 0.7), p(0.0, 0.7), p(0.3, 0.2)), node(p(1.0, 0.3), p(0.65, 0.9), p(1.0, 0.3))];
            m.path("Brush", 0, 2, 100, 8, nodes, false).stroke(amber, 4);
        }

        // arrows
        "arrowbig" => {
            m = mk(50.0);
            m.shape("Arrow", Arrow, 0, 0, 100, 50).fill(blue);
        }
        "arrowdouble" => {
            m = mk(40.0);
            m.shape("Arrow", DoubleArrow, 0, 0, 100, 40).fill(orange);
        }
        "chevrons" => {
            m = mk(40.0);
            for i in 0..3 {
                m.shape("Chevron", Chevron, f64::from(i) * 31.0, 0, 28, 40).fill([sky, blue, purple][i as usize]);
            }
        }
        "curved" => {
            m = mk(40.0);
            m.line(50, 22, 96, 0, dark, 2.2).arrow().bend(-0.55);
        }
        "loop" => {
            m = mk(50.0);
            m.path("Loop", 0, 0, 100, 50, arrow_loop(), false).stroke(dark, 2.2);
            m.shape("Head", Triangle, 92, 29, 10, 12).fill(dark).rotate(100);
        }
        "swoosh" => {
            m = mk(40.0);
            m.path("Swoosh", 0, 0, 100, 40, swoosh(), false).stroke(pink, 3.5);
            m.shape("Head", Triangle, 92, 3, 11, 13).fill(pink).rotate(80);
        }
        "arrowpair" => {
            m = mk(22.0);
            m.line(50, 4, 90, 0, dark, 2).arrow();
            m.line(50, 18, 90, 180, dark, 2).arrow();
        }

        // blobs and shapes
        "blob1" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.46, 0.38, 0.47, 0.4, 0.45, 0.37, 0.48, 0.41]), true).fill(pink);
        }
        "blob2" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.42, 0.48, 0.36, 0.47, 0.4, 0.45, 0.37]), true).fill(blue);
        }
        "blob3" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.48, 0.4, 0.44, 0.49, 0.38, 0.46, 0.43, 0.36, 0.47]), true)
                .fill(amber);
        }
        "blob4" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.4, 0.47, 0.5, 0.41, 0.36, 0.45]), true).fill(green);
        }
        "rings" => {
            m = mk(100.0);
            for i in 0..4 {
                let k = f64::from(i);
                m.shape("Ring", Ellipse, 5.0 + k * 12.0, 5.0 + k * 12.0, 90.0 - k * 24.0, 90.0 - k * 24.0)
                    .stroke(rainbow[i as usize], 5);
            }
        }
        "cluster" => {
            m = mk(80.0);
            m.shape("Circle", Ellipse, 0, 20, 46, 46).fill(blue);
            m.shape("Circle", Ellipse, 34, 0, 34, 34).fill(pink);
            m.shape("Circle", Ellipse, 48, 38, 52, 42).fill(amber);
            m.shape("Circle", Ellipse, 22, 58, 20, 20).fill(purple);
        }
        "halfsun" => {
            m = mk(50.0);
            m.shape("Half", HalfCircle, 0, 0, 100, 50).fill(orange);
            m.shape("Half", HalfCircle, 16, 16, 68, 34).fill(amber);
            m.shape("Half", HalfCircle, 32, 32, 36, 18).fill(red);
        }
        "triangles" => {
            m = mk(60.0);
            m.shape("Triangle", Triangle, 0, 22, 44, 38).fill(blue);
            m.shape("Triangle", Triangle, 30, 0, 44, 38).fill(pink);
            m.shape("Triangle", Triangle, 56, 22, 44, 38).fill(amber);
        }
        "pills" => {
            m = mk(60.0);
            m.shape("Pill", Rectangle, 0, 0, 100, 16).fill(blue).radius(8);
            m.shape("Pill", Rectangle, 14, 22, 86, 16).fill(pink).radius(8);
            m.shape("Pill", Rectangle, 0, 44, 64, 16).fill(amber).radius(8);
        }

        // decorations
        "sparkles" => {
            m = mk(90.0);
            m.shape("Sparkle", Sparkle, 20, 24, 60, 60).fill(amber);
            m.shape("Sparkle", Sparkle, 0, 0, 30, 30).fill(pink);
            m.shape("Sparkle", Sparkle, 70, 0, 24, 24).fill(sky);
        }
        "confetti" => {
            m = mk(70.0);
            for i in 0..34 {
                let color = rainbow[(rng.next() * 8.0) as usize % 8];
                let (x, y) = (rng.next() * 94.0, rng.next() * 64.0);
                let (size, turn) = (3.0 + rng.next() * 4.0, rng.next() * 180.0);
                let kind = i % 3;
                let w = if kind == 0 { size * 1.8 } else { size };
                m.shape("Piece", [Rectangle, Ellipse, Triangle][kind], x, y, w, size).fill(color).rotate(turn);
            }
        }
        "starsrow" => {
            m = mk(20.0);
            for i in 0..5 {
                m.shape("Star", Star, f64::from(i) * 20.0, 0, 20, 20).fill(amber);
            }
        }
        "hearts" => {
            m = mk(70.0);
            m.shape("Heart", Heart, 30, 18, 50, 46).fill(red);
            m.shape("Heart", Heart, 0, 0, 28, 26).fill(pink);
            m.shape("Heart", Heart, 70, 4, 26, 24).fill(c(251, 113, 133));
        }
        "starburst" => {
            m = mk(100.0);
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(amber);
            m.shape("Burst", Burst, 18, 18, 64, 64).fill(orange);
        }
        "sun" => {
            m = mk(100.0);
            for i in 0..8 {
                let a = f64::from(i) * PI / 4.0;
                m.line(50.0 + a.cos() * 38.0, 50.0 + a.sin() * 38.0, 18, f64::from(i) * 45.0, amber, 4);
            }
            m.shape("Sun", Ellipse, 26, 26, 48, 48).fill(amber);
        }
        "flower" => {
            m = mk(100.0);
            // Each petal is turned around its own middle, so it is placed on the circle around the flower's middle.
            for i in 0..6 {
                let a = f64::from(i) * PI / 3.0;
                let (px, py) = (50.0 + a.sin() * 24.0, 50.0 - a.cos() * 24.0);
                m.shape("Petal", Ellipse, px - 14.0, py - 22.0, 28, 44)
                    .fill(if i % 2 == 0 { pink } else { c(251, 113, 133) })
                    .rotate(f64::from(i) * 60.0);
            }
            m.shape("Center", Ellipse, 36, 36, 28, 28).fill(amber);
        }
        "moon" => {
            m = mk(90.0);
            m.shape("Moon", Crescent, 0, 0, 70, 90).fill(amber);
            m.shape("Star", Star, 68, 10, 20, 20).fill(amber);
            m.shape("Star", Star, 78, 50, 14, 14).fill(amber);
        }
        "clouds" => {
            m = mk(60.0);
            m.shape("Cloud", Cloud, 20, 0, 80, 42).fill(sky);
            m.shape("Cloud", Cloud, 0, 24, 60, 34).fill(c(186, 230, 253));
        }
        "crown" => {
            m = mk(70.0);
            m.path("Crown", 0, 0, 100, 70, crown(), true).fill(amber);
            m.shape("Gem", Ellipse, 42, 40, 16, 16).fill(red);
        }

        // frames and borders
        "border" => {
            m = mk(70.0);
            m.shape("Border", Rectangle, 0, 0, 100, 70).stroke(dark, 1.4);
        }
        "borderdouble" => {
            m = mk(70.0);
            m.shape("Border", Rectangle, 0, 0, 100, 70).stroke(dark, 1.8);
            m.shape("Inner", Rectangle, 4, 4, 92, 62).stroke(dark, 0.8);
        }
        "borderround" => {
            m = mk(70.0);
            m.shape("Border", Rectangle, 0, 0, 100, 70).stroke(blue, 2.6).radius(14);
        }
        "corners" => {
            m = mk(70.0);
            for (cx, cy, sx, sy) in
                [(0.0, 0.0, 1.0, 1.0), (100.0, 0.0, -1.0, 1.0), (0.0, 70.0, 1.0, -1.0), (100.0, 70.0, -1.0, -1.0)]
            {
                m.line(cx + sx * 9.0, cy, 18, 0, dark, 1.8);
                m.line(cx, cy + sy * 9.0, 18, 90, dark, 1.8);
            }
        }
        "ticket" => {
            m = mk(50.0);
            m.shape("Ticket", Rectangle, 0, 0, 100, 50).fill(amber).radius(4);
            m.shape("Notch", Ellipse, -6, 19, 12, 12).fill(white);
            m.shape("Notch", Ellipse, 94, 19, 12, 12).fill(white);
            m.line(70, 25, 40, 90, c(161, 98, 7), 1).dashed(3, 2);
        }
        "photo" => {
            m = mk(90.0);
            m.shape("Frame", Rectangle, 0, 0, 100, 90).fill(white).stroke(c(203, 213, 225), 1).radius(2);
            m.shape("Picture", Rectangle, 6, 6, 88, 66).fill(c(226, 232, 240));
        }

        // patterns
        "dots" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Dot", Ellipse, x * 10.0 + 2.0, y * 10.0 + 2.0, 5, 5).fill(blue);
            });
        }
        "stripes" => {
            m = mk(60.0);
            for i in 0..8 {
                m.shape("Stripe", Rectangle, 0, f64::from(i) * 8.0, 100, 4).fill(if i % 2 == 0 {
                    pink
                } else {
                    c(251, 207, 232)
                });
            }
        }
        "checker" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                if (x as i32 + y as i32) % 2 == 0 {
                    m.shape("Square", Rectangle, x * 10.0, y * 10.0, 10, 10).fill(dark);
                }
            });
        }
        "circlegrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Circle", Ellipse, x * 10.0 + 1.0, y * 10.0 + 1.0, 8, 8).stroke(purple, 1.2);
            });
        }
        "plus" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Plus", Cross, x * 10.0 + 1.5, y * 10.0 + 1.5, 7, 7).fill(green);
            });
        }
        "zigzagrows" => {
            m = mk(50.0);
            for r in 0..4 {
                m.path("Zigzag", 0, f64::from(r) * 14.0, 100, 8, zigzag(16), false).stroke(orange, 1.6);
            }
        }
        "waverows" => {
            m = mk(50.0);
            for r in 0..4 {
                m.path("Wave", 0, f64::from(r) * 14.0, 100, 8, wave(12), false).stroke(sky, 1.6);
            }
        }
        _ => return None,
    }
    m.done()
}

/// A 10 x 5 grid of pattern pieces: the closure gets the column and the row.
fn grid(m: &mut Maker, mut piece: impl FnMut(&mut Maker, f64, f64)) {
    for r in 0..5 {
        for col in 0..10 {
            piece(m, f64::from(col), f64::from(r));
        }
    }
}
