//! Even more ready-made graphics: signs and buttons (check, cross, info, play...), everyday objects (phone, laptop, camera, lock,
//! book...), and fun pictures (rocket, planet, cupcake, paw...). Built like the others, as a group of shapes in a design space 100
//! wide.

use super::graphics::{GraphicInfo, wave};
use super::maker::{Maker, c};
use crate::objects::{ShapeKind::*, StoredObject};

const SIGNS: &str = "Signs and buttons";
const OBJECTS: &str = "Objects";
const FUN: &str = "Fun";
const LINES: &str = "Lines and dividers";
const DECO: &str = "Decorations";

const fn g(id: &'static str, name: &'static str, category: &'static str) -> GraphicInfo {
    GraphicInfo { id, name, category }
}

pub const EXTRA: [GraphicInfo; 33] = [
    g("wavyunderline", "Wavy underline", LINES),
    g("railway", "Railway line", LINES),
    g("filmstrip", "Film strip", LINES),
    g("speechoutline", "Speech outline", DECO),
    g("thought", "Thought cloud", DECO),
    g("starsoutline", "Outlined stars", DECO),
    g("heartoutline", "Two hearts", DECO),
    g("checkcircle", "Check mark", SIGNS),
    g("crosscircle", "Cross mark", SIGNS),
    g("infocircle", "Info sign", SIGNS),
    g("warning", "Warning sign", SIGNS),
    g("playbutton", "Play button", SIGNS),
    g("pausebutton", "Pause button", SIGNS),
    g("checkbadge", "Check badge", SIGNS),
    g("pin", "Map pin", SIGNS),
    g("eye", "Eye", OBJECTS),
    g("lock", "Padlock", OBJECTS),
    g("cloudup", "Cloud upload", OBJECTS),
    g("phone", "Phone", OBJECTS),
    g("laptop", "Laptop", OBJECTS),
    g("camera", "Camera", OBJECTS),
    g("coffee", "Coffee cup", OBJECTS),
    g("book", "Open book", OBJECTS),
    g("pencil", "Pencil", OBJECTS),
    g("music", "Music notes", OBJECTS),
    g("gem", "Gem", OBJECTS),
    g("paw", "Paw print", FUN),
    g("cupcake", "Cupcake", FUN),
    g("icecream", "Ice cream", FUN),
    g("rocket", "Rocket", FUN),
    g("planet", "Planet", FUN),
    g("butterfly", "Butterfly", FUN),
    g("fish", "Fish", FUN),
];

/// The graphic, centered on a canvas, about `width` pixels wide. `None` for an id that is not in this list.
pub fn build(id: &str, canvas_w: f64, canvas_h: f64, width: f64) -> Option<StoredObject> {
    let (blue, pink, amber, green, purple) =
        (c(59, 130, 246), c(244, 114, 182), c(250, 204, 21), c(34, 197, 94), c(139, 92, 246));
    let (dark, orange, red, white, sky, gray) =
        (c(30, 41, 59), c(249, 115, 22), c(239, 68, 68), c(255, 255, 255), c(56, 189, 248), c(148, 163, 184));
    let brown = c(146, 92, 46);
    let mk = |h: f64| Maker::new(canvas_w, canvas_h, width, h);
    let mut m;
    match id {
        // lines
        "wavyunderline" => {
            m = mk(10.0);
            m.path("Wave", 0, 1, 100, 8, wave(16), false).stroke(red, 2.6);
        }
        "railway" => {
            m = mk(14.0);
            m.line(50, 3, 100, 0, dark, 1.6);
            m.line(50, 11, 100, 0, dark, 1.6);
            for i in 0..14 {
                m.line(3.5 + f64::from(i) * 7.6, 7, 14, 90, brown, 2.4);
            }
        }
        "filmstrip" => {
            m = mk(24.0);
            m.shape("Strip", Rectangle, 0, 0, 100, 24).fill(dark);
            for i in 0..12 {
                let x = 3.0 + f64::from(i) * 8.0;
                m.shape("Hole", Rectangle, x, 2, 5, 3.5).fill(white).radius(1);
                m.shape("Hole", Rectangle, x, 18.5, 5, 3.5).fill(white).radius(1);
            }
            for i in 0..4 {
                m.shape("Frame", Rectangle, 3.0 + f64::from(i) * 24.0, 7, 21, 10).fill(gray);
            }
        }

        // decorations
        "speechoutline" => {
            m = mk(70.0);
            m.shape("Bubble", SpeechBubble, 0, 0, 100, 70).stroke(dark, 2.4);
        }
        "thought" => {
            m = mk(100.0);
            m.shape("Cloud", Cloud, 10, 0, 90, 62).stroke(dark, 2.4);
            m.shape("Dot", Ellipse, 22, 70, 14, 14).stroke(dark, 2.4);
            m.shape("Dot", Ellipse, 6, 88, 8, 8).stroke(dark, 2.4);
        }
        "starsoutline" => {
            m = mk(70.0);
            m.shape("Star", Star, 0, 18, 44, 44).stroke(amber, 4);
            m.shape("Star", Star, 34, 0, 36, 36).stroke(pink, 4);
            m.shape("Star", Star, 58, 28, 40, 40).stroke(sky, 4);
        }
        "heartoutline" => {
            m = mk(80.0);
            m.shape("Heart", Heart, 0, 20, 60, 56).stroke(pink, 3.4);
            m.shape("Heart", Heart, 38, 0, 62, 58).stroke(red, 3.4);
        }

        // signs and buttons
        "checkcircle" => {
            m = mk(100.0);
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(green);
            m.shape("Check", Check, 24, 28, 52, 44).fill(white);
        }
        "crosscircle" => {
            m = mk(100.0);
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(red);
            m.shape("Cross", XMark, 28, 28, 44, 44).fill(white);
        }
        "infocircle" => {
            m = mk(100.0);
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(blue);
            m.shape("Dot", Ellipse, 43, 20, 14, 14).fill(white);
            m.shape("Stem", Rectangle, 43, 42, 14, 38).fill(white).radius(3);
        }
        "warning" => {
            m = mk(90.0);
            m.shape("Triangle", Triangle, 0, 0, 100, 90).fill(amber);
            m.shape("Stem", Rectangle, 45, 32, 10, 30).fill(dark).radius(3);
            m.shape("Dot", Ellipse, 44, 68, 12, 12).fill(dark);
        }
        "playbutton" => {
            m = mk(100.0);
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(blue);
            m.shape("Play", Triangle, 33, 30, 40, 40).fill(white).rotate(90);
        }
        "pausebutton" => {
            m = mk(100.0);
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(orange);
            m.shape("Bar", Rectangle, 32, 28, 13, 44).fill(white).radius(3);
            m.shape("Bar", Rectangle, 55, 28, 13, 44).fill(white).radius(3);
        }
        "checkbadge" => {
            m = mk(100.0);
            m.shape("Badge", Burst, 0, 0, 100, 100).fill(green);
            m.shape("Check", Check, 26, 30, 48, 40).fill(white);
        }
        "pin" => {
            m = mk(100.0);
            m.poly("Point", red, &[(24.0, 48.0), (76.0, 48.0), (50.0, 100.0)]);
            m.shape("Head", Ellipse, 15, 0, 70, 70).fill(red);
            m.shape("Hole", Ellipse, 36, 21, 28, 28).fill(white);
        }

        // objects
        "eye" => {
            m = mk(60.0);
            m.shape("Eye", Ellipse, 0, 8, 100, 44).fill(white).stroke(dark, 2.6);
            m.shape("Iris", Ellipse, 34, 8, 32, 44).fill(blue);
            m.shape("Pupil", Ellipse, 43, 20, 14, 20).fill(dark);
        }
        "lock" => {
            m = mk(100.0);
            m.shape("Shackle", Rectangle, 26, 4, 48, 56).stroke(gray, 9).radius(24);
            m.shape("Body", Rectangle, 10, 40, 80, 60).fill(amber).radius(8);
            m.shape("Keyhole", Ellipse, 43, 56, 14, 14).fill(dark);
            m.shape("Keyhole", Rectangle, 47, 66, 6, 16).fill(dark);
        }
        "cloudup" => {
            m = mk(70.0);
            m.shape("Cloud", Cloud, 0, 0, 100, 70).fill(blue);
            m.shape("Arrow", Arrow, 30, 25, 40, 24).fill(white).rotate(270);
        }
        "phone" => {
            m = mk(100.0);
            m.shape("Body", Rectangle, 24, 0, 52, 100).fill(dark).radius(9);
            m.shape("Screen", Rectangle, 30, 10, 40, 74).fill(sky).radius(2);
            m.shape("Button", Ellipse, 45, 88, 10, 8).fill(gray);
        }
        "laptop" => {
            m = mk(70.0);
            m.shape("Screen", Rectangle, 10, 0, 80, 54).fill(dark).radius(4);
            m.shape("Display", Rectangle, 15, 5, 70, 44).fill(sky);
            m.shape("Base", Rectangle, 0, 56, 100, 8).fill(gray).radius(4);
        }
        "camera" => {
            m = mk(80.0);
            m.shape("Top", Rectangle, 30, 0, 26, 16).fill(dark).radius(3);
            m.shape("Body", Rectangle, 0, 12, 100, 68).fill(dark).radius(10);
            m.shape("Lens", Ellipse, 30, 22, 44, 44).fill(white);
            m.shape("Lens", Ellipse, 38, 30, 28, 28).fill(blue);
            m.shape("Flash", Ellipse, 82, 20, 8, 8).fill(amber);
        }
        "coffee" => {
            m = mk(100.0);
            m.shape("Handle", Ellipse, 62, 40, 32, 34).stroke(brown, 7);
            m.shape("Cup", Rectangle, 6, 30, 64, 64).fill(brown).radius(10);
            m.shape("Saucer", Rectangle, 0, 92, 80, 8).fill(gray).radius(4);
            for x in [24.0, 44.0] {
                m.path("Steam", x, 0, 10, 24, wave(4), false).stroke(gray, 2.4);
            }
        }
        "book" => {
            m = mk(80.0);
            m.shape("Left", Rectangle, 0, 6, 48, 74).fill(blue).radius(3);
            m.shape("Right", Rectangle, 52, 6, 48, 74).fill(sky).radius(3);
            for r in 0..4 {
                let y = 20.0 + f64::from(r) * 12.0;
                m.line(24, y, 32, 0, white, 1.6);
                m.line(76, y, 32, 0, white, 1.6);
            }
        }
        "pencil" => {
            m = mk(24.0);
            m.shape("Body", Rectangle, 20, 0, 70, 24).fill(amber);
            m.poly("Tip", c(253, 224, 171), &[(20.0, 0.0), (0.0, 12.0), (20.0, 24.0)]);
            m.shape("Eraser", Rectangle, 90, 0, 10, 24).fill(pink);
            m.shape("Band", Rectangle, 84, 0, 6, 24).fill(gray);
        }
        "music" => {
            m = mk(90.0);
            m.shape("Head", Ellipse, 6, 66, 26, 20).fill(dark).rotate(-20);
            m.shape("Head", Ellipse, 58, 56, 26, 20).fill(dark).rotate(-20);
            m.shape("Stem", Rectangle, 28, 12, 5, 60).fill(dark);
            m.shape("Stem", Rectangle, 80, 2, 5, 60).fill(dark);
            m.poly("Beam", dark, &[(28.0, 12.0), (85.0, 2.0), (85.0, 16.0), (28.0, 26.0)]);
        }
        "gem" => {
            m = mk(80.0);
            m.poly("Gem", sky, &[(26.0, 0.0), (74.0, 0.0), (100.0, 28.0), (50.0, 80.0), (0.0, 28.0)]);
            m.poly("Facet", blue, &[(0.0, 28.0), (100.0, 28.0), (50.0, 80.0)]);
            m.line(50, 28, 100, 0, white, 1.2);
            m.line(38, 14, 26, -64, white, 1.2);
            m.line(62, 14, 26, 64, white, 1.2);
        }

        // fun
        "paw" => {
            m = mk(100.0);
            m.shape("Pad", Ellipse, 28, 46, 44, 44).fill(brown);
            for (x, y) in [(6.0, 34.0), (28.0, 8.0), (54.0, 8.0), (76.0, 34.0)] {
                m.shape("Toe", Ellipse, x, y, 18, 24).fill(brown);
            }
        }
        "cupcake" => {
            m = mk(100.0);
            m.poly("Cup", pink, &[(14.0, 52.0), (86.0, 52.0), (74.0, 100.0), (26.0, 100.0)]);
            m.shape("Frosting", Ellipse, 6, 24, 88, 44).fill(c(253, 242, 248)).stroke(pink, 1.6);
            m.shape("Frosting", Ellipse, 22, 8, 56, 36).fill(white).stroke(pink, 1.6);
            m.shape("Cherry", Ellipse, 42, 0, 16, 16).fill(red);
            for i in 0..4 {
                m.line(26.0 + f64::from(i) * 16.0, 76, 40, 80, c(244, 160, 200), 1.4);
            }
        }
        "icecream" => {
            m = mk(100.0);
            m.poly("Cone", orange, &[(24.0, 50.0), (76.0, 50.0), (50.0, 100.0)]);
            m.shape("Scoop", Ellipse, 18, 14, 64, 46).fill(pink);
            m.shape("Scoop", Ellipse, 30, 0, 40, 36).fill(c(253, 224, 71));
            m.shape("Cherry", Ellipse, 44, 0, 12, 12).fill(red);
        }
        "rocket" => {
            m = mk(100.0);
            m.poly("Flame", orange, &[(40.0, 72.0), (60.0, 72.0), (50.0, 100.0)]);
            m.poly("Fin", red, &[(32.0, 48.0), (12.0, 82.0), (32.0, 72.0)]);
            m.poly("Fin", red, &[(68.0, 48.0), (88.0, 82.0), (68.0, 72.0)]);
            m.shape("Body", Rectangle, 30, 0, 40, 76).fill(white).stroke(gray, 1.6).radius(20);
            m.shape("Window", Ellipse, 41, 24, 18, 18).fill(sky).stroke(dark, 1.6);
        }
        "planet" => {
            m = mk(100.0);
            m.shape("Planet", Ellipse, 16, 16, 68, 68).fill(orange);
            m.shape("Stripe", Ellipse, 30, 40, 40, 10).fill(c(253, 186, 116));
            m.shape("Ring", Ellipse, 0, 34, 100, 32).stroke(purple, 3.4).rotate(-20);
        }
        "butterfly" => {
            m = mk(80.0);
            m.shape("Wing", Ellipse, 4, 2, 44, 40).fill(purple).rotate(-25);
            m.shape("Wing", Ellipse, 52, 2, 44, 40).fill(purple).rotate(25);
            m.shape("Wing", Ellipse, 12, 36, 34, 32).fill(pink).rotate(20);
            m.shape("Wing", Ellipse, 54, 36, 34, 32).fill(pink).rotate(-20);
            m.shape("Body", Rectangle, 46, 12, 8, 56).fill(dark).radius(4);
            m.line(46, 6, 16, 40, dark, 1);
            m.line(54, 6, 16, -40, dark, 1);
        }
        "fish" => {
            m = mk(60.0);
            m.poly("Tail", orange, &[(70.0, 30.0), (100.0, 8.0), (100.0, 52.0)]);
            m.shape("Body", Ellipse, 0, 6, 80, 48).fill(sky);
            m.shape("Fin", Triangle, 28, 0, 22, 16).fill(blue);
            m.shape("Eye", Ellipse, 14, 20, 10, 10).fill(white);
            m.shape("Pupil", Ellipse, 17, 23, 5, 5).fill(dark);
            for i in 0..3 {
                m.shape("Scale", Ellipse, 34.0 + f64::from(i) * 9.0, 22.0, 6, 14).fill(blue);
            }
        }
        _ => return None,
    }
    m.done()
}
