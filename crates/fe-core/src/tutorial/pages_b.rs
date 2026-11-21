//! Tutorial pages 6 to 13: shapes, graphics, badges, charts, SmartArt, photos, effects, codes and the wrap-up.

use super::page::{H, Page, W, c, ca, edit_shape, looks, set_radius, shadow};
use super::pages_a::{TOTAL, photo_frame};
use crate::codes::BarcodeKind;
use crate::document::Page as ProjectPage;
use crate::imaging::{ColorRgba, PointD};
use crate::library::{badges, graphics, icons, smartart};
use crate::objects::{
    BarcodeObjData, BendStyle, ChartData, ChartItem, ChartKind, CurveNode, GlowStyle, Gradient, InnerShadowStyle,
    ObjectBox, OutlineStyle, QrData, QrStyle, ShadowStyle, ShapeData, ShapeKind, StoredObject, StrokeStyle,
    ThreeDStyle,
};
use crate::text::TextData;

const PAPER: ColorRgba = ColorRgba::rgb(248, 250, 252);

pub fn shapes_page() -> ProjectPage {
    let green = c(34, 197, 94);
    let mut p = Page::new("Shapes", PAPER);
    p.header(
        5,
        TOTAL,
        "Shapes, lines and curves",
        "Insert > Shapes has dozens of shapes. Lines can bend, and the Curve tool draws your own.",
        green,
    );
    p.steps(
        &[
            "Click a shape in the Insert tab, or open Shapes\nfor all of them. Change color and border in Properties.",
            "A line has a Bend slider (arc or wave) and arrow heads.\n'Edit as curve' lets you drag its points.",
            "Curve tool (P): click to place points, drag for\nsmooth bends. Double-click a point to switch it.",
            "Reshape corners (Properties) drags the corners of any\nshape: a square becomes a trapezium.",
        ],
        green,
        150.0,
        110.0,
    );
    use ShapeKind::*;
    let kinds =
        [Rectangle, Ellipse, Triangle, Diamond, Pentagon, Hexagon, Star, Heart, Arrow, SpeechBubble, Cloud, Shield];
    let palette = [
        c(59, 130, 246),
        c(244, 114, 182),
        c(245, 158, 11),
        c(34, 197, 94),
        c(124, 58, 237),
        c(249, 115, 22),
        c(239, 68, 68),
        c(20, 184, 166),
        c(14, 165, 233),
        c(79, 70, 229),
        c(148, 163, 184),
        c(51, 65, 85),
    ];
    for (i, kind) in kinds.iter().enumerate() {
        let o = p.shape(
            kind.name(),
            *kind,
            690.0 + (i % 4) as f64 * 135.0,
            150.0 + (i / 4) as f64 * 120.0,
            100.0,
            92.0,
            Some(palette[i]),
        );
        if *kind == Rectangle {
            set_radius(o, 16.0);
        }
    }
    let mut bent = ShapeData::new(Line, None, Some(c(59, 130, 246)), 6.0);
    bent.arrow_end = true;
    bent.bend = 0.5;
    p.add(StoredObject::new_shape(ObjectBox::new(690.0, 520.0, 260.0, 8.0, 0.0), &bent), "Bent arrow line");
    let mut dashed = ShapeData::new(Line, None, Some(c(249, 115, 22)), 5.0);
    dashed.dash = StrokeStyle::Dashed;
    p.add(StoredObject::new_shape(ObjectBox::new(690.0, 616.0, 260.0, 8.0, 0.0), &dashed), "Dashed line");
    p.place(graphics::build("wave", W, H, 260.0), 820.0, 665.0, "Wavy line");
    let n = |p: (f64, f64), i: (f64, f64), o: (f64, f64)| CurveNode {
        point: PointD::new(p.0, p.1),
        handle_in: PointD::new(i.0, i.1),
        handle_out: PointD::new(o.0, o.1),
    };
    let mut curve = ShapeData::new(Path, None, Some(c(236, 72, 153)), 6.0);
    curve.nodes = Some(vec![
        n((0.0, 0.9), (0.0, 0.9), (0.4, 0.9)),
        n((0.55, 0.45), (0.35, 0.45), (0.75, 0.45)),
        n((1.0, 0.0), (0.85, 0.0), (1.0, 0.0)),
    ]);
    curve.arrow_end = true;
    let _ = BendStyle::Arc;
    p.add(StoredObject::new_shape(ObjectBox::new(990.0, 500.0, 220.0, 120.0, 0.0), &curve), "Curve");
    p.caption("Bent line with an arrow", 820.0, 590.0, 13.0);
    p.caption("A drawn curve", 1100.0, 636.0, 13.0);
    p.finish()
}

pub fn graphics_page() -> ProjectPage {
    let orange = c(249, 115, 22);
    let mut p = Page::new("Graphics and icons", PAPER);
    p.header(
        6,
        TOTAL,
        "Graphics and icons",
        "Insert > Graphics has 50 ready-made pieces, Icons has about 150 symbols.",
        orange,
    );
    p.steps(
        &[
            "Insert > Graphics: lines, arrows, blobs, decorations,\nframes and patterns, in categories with a search box.",
            "Every graphic is a group of shapes. Ungroup it\n(Ctrl+Shift+G) to recolor or change single parts.",
            "Insert > Icons: each icon is text, so color, size,\noutline and shadow all work.",
        ],
        orange,
        150.0,
        120.0,
    );
    for (id, width, cx, cy, name) in [
        ("blob3", 250.0, 800.0, 270.0, "Blob"),
        ("flower", 170.0, 800.0, 270.0, "Flower"),
        ("sparkles", 150.0, 1080.0, 230.0, "Sparkles"),
        ("confetti", 300.0, 1050.0, 400.0, "Confetti"),
        ("rings", 140.0, 690.0, 520.0, "Rings"),
        ("crown", 160.0, 860.0, 520.0, "Crown"),
        ("moon", 120.0, 1010.0, 520.0, "Moon and stars"),
        ("starsrow", 150.0, 1150.0, 520.0, "Stars"),
    ] {
        p.place(graphics::build(id, W, H, width), cx, cy, name);
    }
    let picks = [
        (0xE735, c(245, 158, 11)),
        (0xE722, c(59, 130, 246)),
        (0xE719, c(244, 114, 182)),
        (0xE774, c(20, 184, 166)),
        (0xE706, c(249, 115, 22)),
        (0xE7B3, c(239, 68, 68)),
        (0xE8BE, c(34, 197, 94)),
        (0xE945, c(245, 158, 11)),
    ];
    for (i, (code, color)) in picks.iter().enumerate() {
        let glyph = char::from_u32(*code).map(String::from).unwrap_or_default();
        p.text_with(&glyph, 700.0 + i as f64 * 70.0, 610.0, 46.0, *color, false, false, 1.15, |t| {
            t.family = icons::FONT_FAMILY.into()
        });
    }
    p.caption("Icons are text: change the color like any text", 920.0, 672.0, 13.0);
    p.finish()
}

pub fn badges_page() -> ProjectPage {
    let red = c(239, 68, 68);
    let mut p = Page::new("Badges", PAPER);
    p.header(7, TOTAL, "Badges", "52 ready-made labels for sales, awards, social posts and info.", red);
    p.steps(
        &[
            "Insert > Badges. Pick a category on the left, or\ntype in the search box.",
            "The badge lands in the middle of the picture.\nDrag it where you want it.",
            "Ungroup (Ctrl+Shift+G), then double-click the text\nto write your own words and change the colors.",
        ],
        red,
        150.0,
        125.0,
    );
    let ids = [
        "sale",
        "off",
        "bestchoice",
        "toprated",
        "freeship",
        "subscribe",
        "tip",
        "step1",
        "premium",
        "bogo",
        "live",
        "done",
    ];
    for (i, id) in ids.iter().enumerate() {
        p.place(badges::build(id, W, H, 130.0), 735.0 + (i % 4) as f64 * 160.0, 215.0 + (i / 4) as f64 * 175.0, id);
    }
    p.finish()
}

fn chart(
    p: &mut Page,
    b: (f64, f64, f64, f64),
    kind: ChartKind,
    title: &str,
    name: &str,
    more: impl FnOnce(&mut ChartData),
) {
    let mut d = ChartData::sample(kind);
    d.title = title.into();
    more(&mut d);
    p.add(StoredObject::new_chart(ObjectBox::new(b.0, b.1, b.2, b.3, 0.0), &d), name);
}

pub fn charts_page() -> ProjectPage {
    let teal = c(20, 184, 166);
    let mut p = Page::new("Charts", PAPER);
    p.header(8, TOTAL, "Charts", "Charts stay editable: change the numbers, type and colors at any time.", teal);
    p.steps(
        &[
            "Insert > Chart and pick a type: columns, bars, line,\npie, donut, area, lollipop, half donut, radial, progress, funnel.",
            "Select the chart. In Properties > Chart change the\nnumbers, names and colors in the table.",
            "Change type, colors (ready-made palettes), what is\nshown and the size of the text.",
        ],
        teal,
        150.0,
        120.0,
    );
    chart(&mut p, (650.0, 140.0, 290.0, 230.0), ChartKind::Columns, "Columns", "Columns chart", |_| {});
    chart(&mut p, (950.0, 140.0, 290.0, 230.0), ChartKind::Donut, "Donut", "Donut chart", |_| {});
    chart(&mut p, (650.0, 390.0, 290.0, 290.0), ChartKind::Progress, "Progress", "Progress chart", |d| {
        d.value_suffix = "%".into();
        d.show_grid = false;
        d.items = [("Design", 85.0), ("Code", 64.0), ("Docs", 48.0), ("Tests", 72.0)]
            .iter()
            .map(|(l, v)| ChartItem::new(l, *v))
            .collect();
    });
    chart(&mut p, (950.0, 390.0, 290.0, 290.0), ChartKind::Area, "Area", "Area chart", |_| {});
    p.finish()
}

pub fn smartart_page() -> ProjectPage {
    let indigo = c(79, 70, 229);
    let mut p = Page::new("SmartArt", PAPER);
    p.header(9, TOTAL, "SmartArt diagrams", "Type your words, get a diagram. Like the SmartArt of Office.", indigo);
    p.steps(
        &[
            "Insert > SmartArt. Pick a layout: processes, lists,\ncycles, hierarchies, Venn, funnel, matrix, pyramid.",
            "Type your words, one per line, and choose a color set.",
            "The diagram is a group: ungroup it to move, recolor\nor retype single parts.",
        ],
        indigo,
        150.0,
        120.0,
    );
    let words = |w: &[&str]| w.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    p.place(
        smartart::build("process", &words(&["Plan", "Design", "Build", "Launch"]), "Colorful", W, H, 290.0),
        800.0,
        215.0,
        "Process",
    );
    p.place(
        smartart::build("cycle", &words(&["Plan", "Do", "Check", "Act"]), "Blue", W, H, 230.0),
        1075.0,
        245.0,
        "Cycle",
    );
    p.place(
        smartart::build("org", &words(&["Team lead", "Design", "Sales", "Support"]), "Warm", W, H, 320.0),
        810.0,
        500.0,
        "Organization chart",
    );
    p.place(
        smartart::build("pyramid", &words(&["Vision", "Goals", "Tasks"]), "Purple", W, H, 250.0),
        1100.0,
        500.0,
        "Pyramid",
    );
    p.finish()
}

pub fn photos_page() -> ProjectPage {
    let pink = c(236, 72, 153);
    let mut p = Page::new("Photos and frames", PAPER);
    p.header(10, TOTAL, "Photos and frames", "Put a picture inside any shape. The picture is part of the frame.", pink);
    p.steps(
        &[
            "Insert > Frame adds an empty frame. Select it and\nchoose Insert > Fill Frames to pick your picture.",
            "Photo Grid makes several frames in a collage layout.",
            "Double-click a filled frame, then drag to move the picture\nand turn the mouse wheel to zoom it. Esc when done.",
            "Properties > Picture in frame has zoom and position.\nMoving the frame takes the picture along.",
        ],
        pink,
        150.0,
        105.0,
    );
    use ShapeKind::*;
    shadow(
        photo_frame(&mut p, "Circle frame", Ellipse, (690.0, 150.0, 230.0, 230.0), "Face", (2.2, 0.0, 1.0)),
        ShadowStyle::DEFAULT,
    );
    let r =
        photo_frame(&mut p, "Rounded frame", Rectangle, (950.0, 150.0, 270.0, 200.0), "Whole photo", (1.0, 0.0, 0.0));
    set_radius(r, 28.0);
    shadow(r, ShadowStyle::DEFAULT);
    photo_frame(&mut p, "Heart frame", Heart, (690.0, 420.0, 230.0, 210.0), "Plants", (2.4, -1.0, 0.1));
    let ph = photo_frame(&mut p, "Photo", Rectangle, (950.0, 400.0, 270.0, 200.0), "T-shirt", (2.0, 0.0, -0.1));
    edit_shape(ph, |s| {
        s.stroke = Some(ColorRgba::WHITE);
        s.stroke_width = 10.0;
        s.corner_radius = 4.0;
    });
    if let Some(b) = ph.box_() {
        ph.set_box(ObjectBox::new(b.x, b.y, b.width, b.height, 4.0));
    }
    shadow(ph, ShadowStyle { color: ca(0, 0, 0, 90), distance: 10.0, angle: 60.0, blur: 16.0, size: 1.0 });
    let mut p2 = p;
    p2.caption("Frames can be any shape", 805.0, 645.0, 14.0);
    p2.caption("With a border and a shadow", 1085.0, 645.0, 14.0);
    p2.finish()
}

pub fn effects_page() -> ProjectPage {
    let amber = c(245, 158, 11);
    let white = ColorRgba::WHITE;
    let mut p = Page::new("Effects", PAPER);
    p.header(
        11,
        TOTAL,
        "Shadows, glow and 3D",
        "Every object can get effects. They are all in Properties and all editable.",
        amber,
    );
    p.steps(
        &[
            "Select an object and open the effect sections in\nProperties: Shadow, Glow, Outline, Inner shadow, 3D.",
            "Gradient (shapes and text) fades through several colors.",
            "Adjust (blur, tone) changes how an object looks without\nchanging the picture inside it.",
            "Turn an effect off again at any time: nothing is baked in.",
        ],
        amber,
        150.0,
        105.0,
    );
    type Effect = fn(&mut StoredObject);
    let tiles: [(&str, Effect); 6] = [
        ("Shadow", |o| {
            shadow(o, ShadowStyle { color: ca(0, 0, 0, 120), distance: 14.0, angle: 55.0, blur: 18.0, size: 1.0 })
        }),
        ("Glow", |o| {
            looks(o, |l| l.glow = Some(GlowStyle { color: ca(255, 214, 90, 235), size: 26.0, intensity: 100.0 }))
        }),
        ("Outline", |o| {
            looks(o, |l| {
                l.outline = Some(OutlineStyle { color: ColorRgba::WHITE, width: 8.0 });
                l.shadow = Some(ShadowStyle::DEFAULT);
            })
        }),
        ("Inner shadow", |o| {
            looks(o, |l| {
                l.inner_shadow =
                    Some(InnerShadowStyle { color: ca(0, 0, 0, 170), distance: 8.0, angle: 45.0, blur: 12.0 })
            })
        }),
        ("3D", |o| {
            looks(o, |l| {
                l.three_d = Some(ThreeDStyle { depth: 26.0, angle: 45.0, color: Some(c(30, 58, 138)), bevel: 0.5 })
            })
        }),
        ("Gradient", |o| o.set_gradient(Some(&Gradient::evenly(&[c(244, 114, 182), c(79, 70, 229)], 45.0, false)))),
    ];
    let _ = white;
    for (i, (label, effect)) in tiles.iter().enumerate() {
        let (x, y) = (690.0 + (i % 3) as f64 * 190.0, 170.0 + (i / 3) as f64 * 250.0);
        let o = p.shape(label, ShapeKind::Rectangle, x, y, 140.0, 140.0, Some(c(59, 130, 246)));
        set_radius(o, 30.0);
        effect(o);
        p.caption(label, x + 70.0, y + 178.0, 16.0);
    }
    p.finish()
}

pub fn codes_page() -> ProjectPage {
    let slate = c(51, 65, 85);
    let ink = c(30, 41, 59);
    let mut p = Page::new("QR codes and barcodes", PAPER);
    p.header(
        12,
        TOTAL,
        "QR codes and barcodes",
        "Real codes that stay editable. Scan them with your phone to test.",
        slate,
    );
    p.steps(
        &[
            "Insert > QR Code: type a web address, phone number\nor any text. Change colors and dot style in Properties.",
            "Insert > Barcode: Code 128, EAN-13, UPC-A, EAN-8 or\nCode 39. Wrong digits show a clear message.",
            "Keep the colors high-contrast (dark on light) and\ntest with a scanner before printing.",
        ],
        slate,
        150.0,
        125.0,
    );
    let mut q1 = QrData::new("https://example.com");
    q1.foreground = ink;
    p.add(StoredObject::new_qr(ObjectBox::new(700.0, 160.0, 230.0, 230.0, 0.0), &q1), "QR code");
    let mut q2 = QrData::new("Hello from Flash Editor");
    q2.foreground = c(79, 70, 229);
    q2.background = c(238, 242, 255);
    q2.level = crate::codes::QrLevel::High;
    q2.style = QrStyle::Dots;
    p.add(StoredObject::new_qr(ObjectBox::new(980.0, 160.0, 230.0, 230.0, 0.0), &q2), "Dotted QR code");
    p.caption("A web address", 815.0, 396.0, 15.0);
    p.caption("Dots, with a color", 1095.0, 396.0, 15.0);
    let mut b1 = BarcodeObjData::new("590123412345", BarcodeKind::Ean13);
    b1.foreground = ink;
    p.add(StoredObject::new_barcode(ObjectBox::new(700.0, 470.0, 250.0, 110.0, 0.0), &b1), "EAN-13 barcode");
    let mut b2 = BarcodeObjData::new("FLASH-12345", BarcodeKind::Code128);
    b2.foreground = ink;
    p.add(StoredObject::new_barcode(ObjectBox::new(980.0, 470.0, 230.0, 110.0, 0.0), &b2), "Code 128 barcode");
    p.caption("EAN-13 (shop products)", 825.0, 590.0, 15.0);
    p.caption("Code 128 (any text)", 1095.0, 590.0, 15.0);
    p.finish()
}

pub fn wrap_up() -> ProjectPage {
    let blue = c(59, 130, 246);
    let mut p = Page::new("Layers, pages and export", PAPER);
    p.header(
        13,
        TOTAL,
        "Layers, pages and export",
        "The last lesson: keep things tidy, work with many pages, and save your result.",
        blue,
    );
    p.steps(
        &[
            "Layers panel: drag to reorder, eye hides, lock protects.\nLayer tab has masks, clip, adjustment layers and groups.",
            "Pages (under the canvas): + adds one, Ctrl+click picks\nseveral, Ctrl+C / V copies pages, even into another project.",
            "File > Export: picture (PNG, JPG, WEBP), ready-made\nsizes for social media, PDF, animated GIF, SVG, PSD.",
            "Ctrl+S saves the whole project; everything stays editable.\nF1 lists all keyboard shortcuts, and you can change them.",
        ],
        blue,
        150.0,
        112.0,
    );
    let back = p.shape("Back layer", ShapeKind::Rectangle, 700.0, 170.0, 300.0, 200.0, Some(c(14, 165, 233)));
    set_radius(back, 24.0);
    shadow(
        p.shape("Middle layer", ShapeKind::Ellipse, 850.0, 240.0, 230.0, 230.0, Some(c(236, 72, 153))),
        ShadowStyle::DEFAULT,
    );
    let star = p.shape("Front layer", ShapeKind::Star, 760.0, 330.0, 170.0, 170.0, Some(c(245, 158, 11)));
    if let Some(b) = star.box_() {
        star.set_box(ObjectBox::new(b.x, b.y, b.width, b.height, 12.0));
    }
    shadow(star, ShadowStyle::DEFAULT);
    p.caption("Three layers: the last one added is on top.", 890.0, 520.0, 14.0);
    let words = ["Make", "Check", "Export"].map(String::from);
    p.place(smartart::build("steps", &words, "Colorful", W, H, 250.0), 1010.0, 622.0, "Steps");
    p.place(badges::build("done", W, H, 120.0), 1150.0, 180.0, "Done badge");
    p.text_with("You are ready. Have fun!", 56.0, 660.0, 30.0, blue, true, false, 1.15, |_: &mut TextData| {});
    p.finish()
}
