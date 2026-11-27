//! More ready-made graphics (the first 50 are in `graphics.rs`): more lines, arrows, blobs, frames and patterns, and the new
//! groups Celebration, Nature and Business. Built the same way: a group of shapes in a design space 100 wide.

use std::f64::consts::PI;

use super::graphics::{GraphicInfo, Rng, blob, node, p, wave, zigzag};
use super::maker::{Maker, c};
use crate::imaging::ColorRgba;
use crate::objects::{CurveNode, ShapeKind::*, StoredObject};

const LINES: &str = "Lines and dividers";
const ARROWS: &str = "Arrows";
const BLOBS: &str = "Blobs and shapes";
const FRAMES: &str = "Frames and borders";
const PATTERNS: &str = "Patterns";
const PARTY: &str = "Celebration";
const NATURE: &str = "Nature";
const BUSINESS: &str = "Business";

const fn g(id: &'static str, name: &'static str, category: &'static str) -> GraphicInfo {
    GraphicInfo { id, name, category }
}

pub const MORE: [GraphicInfo; 65] = [
    g("diamonddiv", "Diamond divider", LINES),
    g("heartdiv", "Heart divider", LINES),
    g("stitched", "Stitched line", LINES),
    g("doublewave", "Double wave", LINES),
    g("scribble", "Scribble", LINES),
    g("ornament", "Ornament divider", LINES),
    g("arrowup", "Arrow up", ARROWS),
    g("arrowdown", "Arrow down", ARROWS),
    g("arrowthree", "Three arrows", ARROWS),
    g("arrowcurvedown", "Curved arrow down", ARROWS),
    g("uturn", "U-turn arrow", ARROWS),
    g("dasharrow", "Dashed arrow", ARROWS),
    g("cycle", "Cycle arrows", ARROWS),
    g("blob5", "Blob 5", BLOBS),
    g("blob6", "Blob 6", BLOBS),
    g("diamonds", "Diamonds", BLOBS),
    g("hexagons", "Hexagons", BLOBS),
    g("bubbles", "Bubbles", BLOBS),
    g("squares", "Tilted squares", BLOBS),
    g("orbit", "Orbit", BLOBS),
    g("balloons", "Balloons", PARTY),
    g("present", "Present", PARTY),
    g("firework", "Firework", PARTY),
    g("banner", "Ribbon banner", PARTY),
    g("partyhat", "Party hat", PARTY),
    g("streamers", "Streamers", PARTY),
    g("starscatter", "Scattered stars", PARTY),
    g("tree", "Tree", NATURE),
    g("mountains", "Mountains", NATURE),
    g("sunrise", "Sunrise", NATURE),
    g("leaf", "Leaf", NATURE),
    g("leaves", "Two leaves", NATURE),
    g("cloudrain", "Rain cloud", NATURE),
    g("bolt", "Lightning", NATURE),
    g("house", "House", NATURE),
    g("waves", "Waves", NATURE),
    g("daisy", "Daisy", NATURE),
    g("snowflake", "Snowflake", NATURE),
    g("rainbow", "Rainbow", NATURE),
    g("bars", "Bar chart", BUSINESS),
    g("target", "Target", BUSINESS),
    g("lightbulb", "Light bulb", BUSINESS),
    g("speech", "Speech bubbles", BUSINESS),
    g("checklist", "Checklist", BUSINESS),
    g("gear", "Gear", BUSINESS),
    g("medal", "Medal", BUSINESS),
    g("calendar", "Calendar", BUSINESS),
    g("envelope", "Envelope", BUSINESS),
    g("circleframe", "Circle frame", FRAMES),
    g("ovalframe", "Oval frame", FRAMES),
    g("heartframe", "Heart frame", FRAMES),
    g("archframe", "Arch frame", FRAMES),
    g("stampframe", "Stamp border", FRAMES),
    g("polaroid", "Polaroid", FRAMES),
    g("tape", "Tape strips", FRAMES),
    g("slashes", "Slashes", PATTERNS),
    g("trianglesgrid", "Triangle pattern", PATTERNS),
    g("heartsgrid", "Heart pattern", PATTERNS),
    g("starsgrid", "Star pattern", PATTERNS),
    g("hexgrid", "Hexagon pattern", PATTERNS),
    g("diamondsgrid", "Diamond pattern", PATTERNS),
    g("bubblesgrid", "Bubble pattern", PATTERNS),
    g("gridlines", "Grid lines", PATTERNS),
    g("squaresgrid", "Square pattern", PATTERNS),
    g("rainbowbands", "Rainbow bands", PATTERNS),
];

/// A leaf: two curves from the bottom left to the top right of the unit box.
fn leaf() -> Vec<CurveNode> {
    vec![node(p(0.0, 1.0), p(0.65, 1.0), p(0.0, 0.35)), node(p(1.0, 0.0), p(0.35, 0.0), p(1.0, 0.65))]
}

/// The graphic, centered on a canvas, about `width` pixels wide. `None` for an id that is not in this list.
pub fn build(id: &str, canvas_w: f64, canvas_h: f64, width: f64) -> Option<StoredObject> {
    let (blue, pink, amber, green, purple) =
        (c(59, 130, 246), c(244, 114, 182), c(250, 204, 21), c(34, 197, 94), c(139, 92, 246));
    let (dark, orange, red, white, sky, gray) =
        (c(30, 41, 59), c(249, 115, 22), c(239, 68, 68), c(255, 255, 255), c(56, 189, 248), c(148, 163, 184));
    let rainbow: [ColorRgba; 8] = [blue, pink, amber, green, purple, orange, sky, red];
    let brown = c(146, 92, 46);
    let mk = |h: f64| Maker::new(canvas_w, canvas_h, width, h);
    let mut m;
    match id {
        // lines and dividers
        "diamonddiv" => {
            m = mk(12.0);
            m.line(22, 6, 44, 0, dark, 1.2);
            m.line(78, 6, 44, 0, dark, 1.2);
            m.shape("Diamond", Diamond, 44, 0, 12, 12).fill(pink);
        }
        "heartdiv" => {
            m = mk(12.0);
            m.line(22, 6, 44, 0, dark, 1.2);
            m.line(78, 6, 44, 0, dark, 1.2);
            m.shape("Heart", Heart, 44, 0, 12, 11).fill(red);
        }
        "stitched" => {
            m = mk(4.0);
            m.line(50, 2, 100, 0, pink, 2.4).dashed(4, 3);
        }
        "doublewave" => {
            m = mk(22.0);
            m.path("Wave", 0, 1, 100, 8, wave(10), false).stroke(blue, 2);
            m.path("Wave", 0, 13, 100, 8, wave(10), false).stroke(pink, 2);
        }
        "scribble" => {
            m = mk(14.0);
            m.path("Scribble", 0, 2, 100, 10, zigzag(9), false).stroke(pink, 3.5);
        }
        "ornament" => {
            m = mk(12.0);
            m.line(18, 6, 36, 0, dark, 1.2);
            m.line(82, 6, 36, 0, dark, 1.2);
            m.shape("Diamond", Diamond, 36, 3, 6, 6).fill(gray);
            m.shape("Diamond", Diamond, 44, 0, 12, 12).fill(amber);
            m.shape("Diamond", Diamond, 58, 3, 6, 6).fill(gray);
        }

        // arrows
        "arrowup" => {
            m = mk(100.0);
            m.shape("Arrow", Arrow, 0, 25, 100, 50).fill(blue).rotate(270);
        }
        "arrowdown" => {
            m = mk(100.0);
            m.shape("Arrow", Arrow, 0, 25, 100, 50).fill(green).rotate(90);
        }
        "arrowthree" => {
            m = mk(60.0);
            m.shape("Arrow", Arrow, 0, 0, 100, 18).fill(sky);
            m.shape("Arrow", Arrow, 0, 21, 74, 18).fill(blue);
            m.shape("Arrow", Arrow, 0, 42, 48, 18).fill(purple);
        }
        "arrowcurvedown" => {
            m = mk(40.0);
            m.line(50, 18, 96, 0, dark, 2.2).arrow().bend(0.55);
        }
        "uturn" => {
            m = mk(60.0);
            let nodes = vec![
                node(p(0.0, 1.0), p(0.0, 1.0), p(0.0, 0.35)),
                node(p(0.5, 0.0), p(0.12, 0.0), p(0.88, 0.0)),
                node(p(1.0, 0.6), p(1.0, 0.2), p(1.0, 0.6)),
            ];
            m.path("U-turn", 0, 0, 100, 56, nodes, false).stroke(dark, 2.4);
            m.shape("Head", Triangle, 93, 52, 14, 8).fill(dark).rotate(180);
        }
        "dasharrow" => {
            m = mk(10.0);
            m.line(48, 5, 92, 0, orange, 2).dashed(5, 3).arrow();
        }
        "cycle" => {
            m = mk(60.0);
            m.line(50, 16, 78, 0, blue, 2.6).arrow().bend(-0.5);
            m.line(50, 44, 78, 180, pink, 2.6).arrow().bend(-0.5);
        }

        // blobs and shapes
        "blob5" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.45, 0.36, 0.49, 0.42, 0.38, 0.47, 0.44, 0.4]), true).fill(purple);
        }
        "blob6" => {
            m = mk(100.0);
            m.path("Blob", 0, 0, 100, 100, blob(&[0.4, 0.48, 0.42, 0.5, 0.37, 0.46]), true).fill(sky);
        }
        "diamonds" => {
            m = mk(50.0);
            m.shape("Diamond", Diamond, 0, 14, 36, 36).fill(blue);
            m.shape("Diamond", Diamond, 32, 0, 36, 36).fill(pink);
            m.shape("Diamond", Diamond, 64, 14, 36, 36).fill(amber);
        }
        "hexagons" => {
            m = mk(60.0);
            m.shape("Hexagon", Hexagon, 0, 15, 34, 30).fill(blue);
            m.shape("Hexagon", Hexagon, 33, 0, 34, 30).fill(pink);
            m.shape("Hexagon", Hexagon, 33, 30, 34, 30).fill(amber);
        }
        "bubbles" => {
            m = mk(80.0);
            m.shape("Bubble", Ellipse, 0, 30, 40, 40).stroke(blue, 2.5);
            m.shape("Bubble", Ellipse, 30, 0, 28, 28).stroke(pink, 2.5);
            m.shape("Bubble", Ellipse, 60, 38, 36, 36).stroke(amber, 2.5);
            m.shape("Bubble", Ellipse, 44, 48, 12, 12).fill(sky);
        }
        "squares" => {
            m = mk(100.0);
            m.shape("Square", Rectangle, 8, 8, 52, 52).fill(blue).radius(6).rotate(15);
            m.shape("Square", Rectangle, 38, 38, 52, 52).fill(pink).radius(6).rotate(-10);
        }
        "orbit" => {
            m = mk(100.0);
            m.shape("Orbit", Ellipse, 5, 25, 90, 50).stroke(blue, 2).rotate(30);
            m.shape("Orbit", Ellipse, 5, 25, 90, 50).stroke(pink, 2).rotate(-30);
            m.shape("Planet", Ellipse, 38, 38, 24, 24).fill(amber);
        }

        // celebration
        "balloons" => {
            m = mk(100.0);
            for (x, color) in [(23.0, red), (50.0, blue), (77.0, amber)] {
                m.line(x, 76, 44, 90, dark, 0.8);
                let top = if x == 50.0 { 8.0 } else { 0.0 };
                m.shape("Balloon", Ellipse, x - 19.0, top, 38, 48).fill(color);
                m.shape("Knot", Triangle, x - 3.0, top + 47.0, 6, 5).fill(color).rotate(180);
            }
        }
        "present" => {
            m = mk(100.0);
            m.shape("Box", Rectangle, 6, 38, 88, 58).fill(red);
            m.shape("Lid", Rectangle, 0, 26, 100, 16).fill(c(220, 38, 38));
            m.shape("Ribbon", Rectangle, 44, 26, 12, 70).fill(amber);
            m.shape("Bow", Ellipse, 25, 6, 28, 20).fill(amber).rotate(-20);
            m.shape("Bow", Ellipse, 47, 6, 28, 20).fill(amber).rotate(20);
            m.shape("Knot", Ellipse, 44, 14, 12, 12).fill(orange);
        }
        "firework" => {
            m = mk(100.0);
            for i in 0..12 {
                let a = i as f64 * PI / 6.0;
                let color = rainbow[i % 8];
                m.line(50.0 + a.cos() * 29.0, 50.0 + a.sin() * 29.0, 30, i as f64 * 30.0, color, 3);
                m.shape("Spark", Ellipse, 50.0 + a.cos() * 46.0 - 3.0, 50.0 + a.sin() * 46.0 - 3.0, 6, 6).fill(color);
            }
        }
        "banner" => {
            m = mk(40.0);
            m.shape("Tail", Rectangle, 0, 12, 20, 26).fill(c(190, 24, 93));
            m.shape("Tail", Rectangle, 80, 12, 20, 26).fill(c(190, 24, 93));
            m.shape("Banner", Rectangle, 10, 2, 80, 28).fill(pink);
        }
        "partyhat" => {
            m = mk(90.0);
            m.poly("Hat", purple, &[(50.0, 8.0), (88.0, 88.0), (12.0, 88.0)]);
            m.shape("Pompom", Ellipse, 42, 0, 16, 16).fill(amber);
            m.shape("Dot", Ellipse, 45, 46, 8, 8).fill(pink);
            m.shape("Dot", Ellipse, 34, 66, 8, 8).fill(amber);
            m.shape("Dot", Ellipse, 58, 64, 8, 8).fill(sky);
        }
        "streamers" => {
            m = mk(60.0);
            for (r, color) in [pink, blue, amber, green].into_iter().enumerate() {
                m.path("Streamer", 0, r as f64 * 14.0, 100, 10, wave(6), false).stroke(color, 3);
            }
        }
        "starscatter" => {
            m = mk(80.0);
            for (x, y, s, color) in [
                (0.0, 24.0, 30.0, amber),
                (36.0, 0.0, 22.0, pink),
                (58.0, 30.0, 36.0, sky),
                (24.0, 56.0, 20.0, orange),
                (80.0, 0.0, 16.0, purple),
            ] {
                m.shape("Star", Star, x, y, s, s).fill(color);
            }
        }

        // nature
        "tree" => {
            m = mk(100.0);
            m.shape("Trunk", Rectangle, 44, 72, 12, 28).fill(brown);
            m.poly("Top", c(22, 163, 74), &[(50.0, 0.0), (78.0, 34.0), (22.0, 34.0)]);
            m.poly("Middle", green, &[(50.0, 20.0), (84.0, 56.0), (16.0, 56.0)]);
            m.poly("Bottom", c(22, 163, 74), &[(50.0, 38.0), (90.0, 78.0), (10.0, 78.0)]);
        }
        "mountains" => {
            m = mk(60.0);
            m.poly("Back", gray, &[(40.0, 60.0), (70.0, 22.0), (100.0, 60.0)]);
            m.poly("Front", c(100, 116, 139), &[(0.0, 60.0), (35.0, 10.0), (70.0, 60.0)]);
            m.poly("Snow", white, &[(35.0, 10.0), (44.0, 24.0), (35.0, 20.0), (26.0, 24.0)]);
        }
        "sunrise" => {
            m = mk(50.0);
            for i in 0..5 {
                let a = (-150.0 + f64::from(i) * 30.0) * PI / 180.0;
                m.line(50.0 + a.cos() * 38.0, 40.0 + a.sin() * 38.0, 10, -150.0 + f64::from(i) * 30.0, amber, 3);
            }
            m.shape("Sun", HalfCircle, 25, 15, 50, 25).fill(orange);
            m.line(50, 41, 96, 0, dark, 1.6);
        }
        "leaf" => {
            m = mk(100.0);
            m.path("Leaf", 0, 0, 100, 100, leaf(), true).fill(green);
            m.line(50, 50, 96, -45, white, 1.4);
        }
        "leaves" => {
            m = mk(100.0);
            m.path("Leaf", 0, 28, 62, 62, leaf(), true).fill(c(22, 163, 74));
            m.path("Leaf", 40, 0, 56, 56, leaf(), true).fill(c(74, 222, 128));
        }
        "cloudrain" => {
            m = mk(70.0);
            m.shape("Cloud", Cloud, 8, 0, 84, 44).fill(gray);
            for (x, y) in [(28.0, 56.0), (48.0, 60.0), (68.0, 56.0), (38.0, 68.0), (58.0, 70.0)] {
                m.line(x, y, 9, 70, sky, 2.4);
            }
        }
        "bolt" => {
            m = mk(100.0);
            m.shape("Bolt", Lightning, 22, 0, 56, 100).fill(amber).stroke(orange, 2);
        }
        "house" => {
            m = mk(90.0);
            m.shape("House", Home, 0, 0, 100, 90).fill(blue);
            m.shape("Door", Rectangle, 42, 54, 16, 32).fill(white).radius(2);
            m.shape("Window", Rectangle, 20, 50, 14, 14).fill(white).radius(2);
            m.shape("Window", Rectangle, 66, 50, 14, 14).fill(white).radius(2);
        }
        "waves" => {
            m = mk(46.0);
            for (r, color) in [c(186, 230, 253), sky, blue].into_iter().enumerate() {
                m.path("Wave", 0, r as f64 * 14.0, 100, 10, wave(8), false).stroke(color, 4);
            }
        }
        "daisy" => {
            m = mk(100.0);
            for i in 0..8 {
                let a = f64::from(i) * PI / 4.0;
                let (px, py) = (50.0 + a.sin() * 26.0, 50.0 - a.cos() * 26.0);
                m.shape("Petal", Ellipse, px - 9.0, py - 18.0, 18, 36)
                    .fill(white)
                    .stroke(c(203, 213, 225), 1)
                    .rotate(f64::from(i) * 45.0);
            }
            m.shape("Center", Ellipse, 36, 36, 28, 28).fill(amber);
        }
        "snowflake" => {
            m = mk(100.0);
            for i in 0..3 {
                m.line(50, 50, 92, f64::from(i) * 60.0, sky, 3);
            }
            m.shape("Center", Ellipse, 42, 42, 16, 16).fill(sky);
        }
        "rainbow" => {
            m = mk(50.0);
            for (r, color) in [(50.0, red), (42.0, orange), (34.0, amber), (26.0, green), (18.0, blue)].into_iter() {
                m.shape("Band", HalfCircle, 50.0 - r, 50.0 - r, 2.0 * r, r).fill(color);
            }
            m.shape("Band", HalfCircle, 40, 40, 20, 10).fill(white);
        }

        // business
        "bars" => {
            m = mk(80.0);
            for (i, (h, color)) in [(30.0, blue), (48.0, sky), (64.0, purple), (78.0, pink)].into_iter().enumerate() {
                m.shape("Bar", Rectangle, i as f64 * 24.0 + 2.0, 80.0 - h, 18, h).fill(color).radius(2);
            }
            m.line(50, 80, 100, 0, dark, 1.2);
        }
        "target" => {
            m = mk(100.0);
            m.shape("Ring", Ellipse, 0, 0, 100, 100).fill(red);
            m.shape("Ring", Ellipse, 16, 16, 68, 68).fill(white);
            m.shape("Ring", Ellipse, 32, 32, 36, 36).fill(red);
            m.shape("Ring", Ellipse, 44, 44, 12, 12).fill(white);
            m.line(74, 26, 52, -45, dark, 2.6);
            m.shape("Tip", Triangle, 42, 38, 12, 14).fill(dark).rotate(-135);
        }
        "lightbulb" => {
            m = mk(100.0);
            for i in 0..5 {
                let a = (-150.0 + f64::from(i) * 30.0) * PI / 180.0;
                m.line(50.0 + a.cos() * 46.0, 28.0 + a.sin() * 46.0, 10, -150.0 + f64::from(i) * 30.0, amber, 3);
            }
            m.shape("Bulb", Ellipse, 22, 8, 56, 56).fill(amber);
            m.shape("Neck", Rectangle, 36, 54, 28, 12).fill(amber);
            m.shape("Base", Rectangle, 36, 68, 28, 8).fill(gray).radius(2);
            m.shape("Base", Rectangle, 40, 78, 20, 8).fill(c(100, 116, 139)).radius(3);
        }
        "speech" => {
            m = mk(70.0);
            m.shape("Bubble", SpeechBubble, 0, 0, 62, 40).fill(blue);
            m.shape("Bubble", OvalBubble, 38, 28, 58, 40).fill(pink);
        }
        "checklist" => {
            m = mk(60.0);
            for r in 0..3 {
                let y = f64::from(r) * 24.0;
                m.shape("Check", Check, 0, y, 16, 16).fill(green);
                m.line(60, y + 8.0, 70, 0, gray, 3);
            }
        }
        "gear" => {
            m = mk(100.0);
            m.shape("Gear", Burst, 0, 0, 100, 100).fill(c(100, 116, 139));
            m.shape("Hole", Ellipse, 32, 32, 36, 36).fill(white);
        }
        "medal" => {
            m = mk(100.0);
            m.shape("Ribbon", Rectangle, 26, 0, 16, 52).fill(blue).rotate(12);
            m.shape("Ribbon", Rectangle, 58, 0, 16, 52).fill(red).rotate(-12);
            m.shape("Medal", Ellipse, 22, 40, 56, 56).fill(amber).stroke(orange, 4);
            m.shape("Star", Star, 36, 54, 28, 28).fill(white);
        }
        "calendar" => {
            m = mk(90.0);
            m.shape("Page", Rectangle, 0, 10, 100, 80).fill(white).stroke(dark, 2).radius(6);
            m.shape("Header", Rectangle, 0, 10, 100, 22).fill(red).radius(6);
            m.shape("Ring", Rectangle, 22, 0, 6, 18).fill(dark).radius(3);
            m.shape("Ring", Rectangle, 72, 0, 6, 18).fill(dark).radius(3);
            for r in 0..3 {
                for col in 0..4 {
                    m.shape("Day", Rectangle, 12.0 + f64::from(col) * 21.0, 42.0 + f64::from(r) * 15.0, 12, 8)
                        .fill(c(203, 213, 225))
                        .radius(1);
                }
            }
        }
        "envelope" => {
            m = mk(70.0);
            m.shape("Envelope", Rectangle, 0, 0, 100, 70).fill(white).stroke(dark, 1.8).radius(4);
            m.line(25, 18, 61.6, 35.8, dark, 1.6);
            m.line(75, 18, 61.6, -35.8, dark, 1.6);
        }

        // frames and borders
        "circleframe" => {
            m = mk(100.0);
            m.shape("Frame", Ellipse, 0, 0, 100, 100).stroke(blue, 3);
        }
        "ovalframe" => {
            m = mk(80.0);
            m.shape("Frame", Ellipse, 0, 0, 100, 80).stroke(pink, 3);
        }
        "heartframe" => {
            m = mk(90.0);
            m.shape("Frame", Heart, 0, 0, 100, 90).stroke(red, 3);
        }
        "archframe" => {
            m = mk(100.0);
            m.shape("Frame", Rectangle, 10, 0, 80, 100).stroke(dark, 2.4).radius(40);
        }
        "stampframe" => {
            m = mk(70.0);
            m.shape("Frame", Rectangle, 0, 0, 100, 70).stroke(dark, 2.4).dashed(4, 3);
        }
        "polaroid" => {
            m = mk(100.0);
            m.shape("Frame", Rectangle, 0, 0, 100, 100).fill(white).stroke(c(203, 213, 225), 1).radius(2);
            m.shape("Picture", Rectangle, 7, 7, 86, 70).fill(c(226, 232, 240));
        }
        "tape" => {
            m = mk(40.0);
            m.shape("Tape", Rectangle, 4, 8, 56, 20).fill(c(253, 224, 71)).rotate(-8);
            m.shape("Tape", Rectangle, 38, 14, 56, 20).fill(c(147, 197, 253)).rotate(6);
        }

        // patterns
        "slashes" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.line(x * 10.0 + 5.0, y * 10.0 + 5.0, 7, -45, amber, 1.6);
            });
        }
        "trianglesgrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Triangle", Triangle, x * 10.0 + 1.5, y * 10.0 + 1.5, 7, 7).fill(pink);
            });
        }
        "heartsgrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Heart", Heart, x * 10.0 + 1.0, y * 10.0 + 1.5, 8, 7).fill(if (x + y) % 2.0 == 0.0 {
                    red
                } else {
                    pink
                });
            });
        }
        "starsgrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Star", Star, x * 10.0 + 1.0, y * 10.0 + 1.0, 8, 8).fill(amber);
            });
        }
        "hexgrid" => {
            m = mk(46.0);
            for r in 0..4 {
                for col in 0..9 {
                    let x = f64::from(col) * 11.0 + if r % 2 == 1 { 5.5 } else { 0.0 };
                    if x + 10.0 <= 100.0 {
                        m.shape("Hexagon", Hexagon, x, f64::from(r) * 10.0, 10, 9).stroke(sky, 1.2);
                    }
                }
            }
        }
        "diamondsgrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Diamond", Diamond, x * 10.0 + 1.0, y * 10.0 + 1.0, 8, 8).fill(if (x + y) % 2.0 == 0.0 {
                    purple
                } else {
                    pink
                });
            });
        }
        "bubblesgrid" => {
            m = mk(60.0);
            let mut rng = Rng(11);
            for _ in 0..22 {
                let size = 5.0 + rng.next() * 12.0;
                let (x, y) = (rng.next() * (100.0 - size), rng.next() * (60.0 - size));
                m.shape("Bubble", Ellipse, x, y, size, size).stroke(rainbow[(rng.next() * 8.0) as usize % 8], 1.4);
            }
        }
        "gridlines" => {
            m = mk(50.0);
            for i in 0..11 {
                m.line(f64::from(i) * 10.0, 25, 50, 90, gray, 0.8);
            }
            for i in 0..6 {
                m.line(50, f64::from(i) * 10.0, 100, 0, gray, 0.8);
            }
        }
        "squaresgrid" => {
            m = mk(50.0);
            grid(&mut m, |m, x, y| {
                m.shape("Square", Rectangle, x * 10.0 + 1.0, y * 10.0 + 1.0, 8, 8)
                    .fill(if (x + y) % 2.0 == 0.0 { sky } else { blue })
                    .radius(2);
            });
        }
        "rainbowbands" => {
            m = mk(60.0);
            for (i, color) in rainbow.into_iter().enumerate() {
                m.shape("Band", Rectangle, 0, i as f64 * 7.5, 100, 7.5).fill(color);
            }
        }
        _ => return super::graphics_extra::build(id, canvas_w, canvas_h, width),
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
