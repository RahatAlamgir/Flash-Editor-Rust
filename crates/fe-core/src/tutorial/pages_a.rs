//! Tutorial pages 0 to 5: welcome, photo editing, selecting and text.

use super::page::{INK, Page, SUBTLE, WHITE, c, ca, edit_shape, set_radius, shadow};
use super::pictures::fx as fe_core_effects;
use super::pictures::{asset_picture, landscape, photo_png, photo_square, pic, recolor_red, resize};
use crate::document::Page as ProjectPage;
use crate::library::{badges, graphics};
use crate::objects::{GroupMember, ObjectBox, ShadowStyle, ShapeData, ShapeKind, StoredObject};
use crate::text::TEXT_PRESETS;

pub const TOTAL: usize = 13;

/// An empty frame with the photo in it.
pub fn photo_frame<'a>(
    p: &'a mut Page,
    name: &str,
    kind: ShapeKind,
    b: (f64, f64, f64, f64),
    label: &str,
    view: (f64, f64, f64),
) -> &'a mut StoredObject {
    let o = p.shape(name, kind, b.0, b.1, b.2, b.3, Some(c(226, 232, 240)));
    o.set_picture_fill(Some((photo_png(), label)));
    o.set_picture_view(view.0, view.1, view.2);
    o
}

pub fn welcome() -> ProjectPage {
    let (w, h) = (super::page::W, super::page::H);
    let mut p = Page::new("Welcome", c(30, 41, 99));
    p.place(graphics::build("blob2", w, h, 520.0), w / 2.0 + 400.0, h / 2.0 - 120.0, "Blob");
    p.place(graphics::build("blob1", w, h, 380.0), w / 2.0 - 440.0, h / 2.0 + 270.0, "Blob");
    p.place(graphics::build("sparkles", w, h, 160.0), w / 2.0 + 420.0, h / 2.0 - 210.0, "Sparkles");
    p.text("Welcome to Flash Editor", 80.0, 110.0, 66.0, WHITE, true, false);
    p.text_with(
        "This tutorial is a real project. Click, drag and change anything on these pages:\nnothing here can break, and you can always press Ctrl+Z.",
        84.0,
        205.0,
        24.0,
        ca(255, 255, 255, 225),
        false,
        false,
        1.3,
        |_| {},
    );
    let a = [
        "1  Cut out, enlarge and fix photos",
        "2  Adjust, filters and layers",
        "3  Select, move and resize",
        "4  Text and designs",
        "5  Shapes, lines and curves",
        "6  Graphics and icons",
        "7  Badges",
    ];
    let b = [
        "8  Charts",
        "9  SmartArt diagrams",
        "10  Photos and frames",
        "11  Shadows, glow and 3D",
        "12  QR codes and barcodes",
        "13  Layers, pages and export",
    ];
    for (i, t) in a.iter().enumerate() {
        p.text(t, 90.0, 306.0 + i as f64 * 36.0, 22.0, WHITE, false, false);
    }
    for (i, t) in b.iter().enumerate() {
        p.text(t, 500.0, 306.0 + i as f64 * 36.0, 22.0, WHITE, false, false);
    }
    let o = p.shape("Photo frame", ShapeKind::Ellipse, 880.0, 330.0, 300.0, 300.0, Some(c(226, 232, 240)));
    o.set_picture_fill(Some((photo_png(), "Face")));
    o.set_picture_view(2.2, 0.0, 1.0);
    edit_shape(o, |s| {
        s.stroke = Some(WHITE);
        s.stroke_width = 8.0;
    });
    shadow(o, ShadowStyle::DEFAULT);
    p.place(badges::build("new", w, h, 130.0), 1060.0, 240.0, "NEW badge");
    p.note("Move to the next page: click it in the page strip under the canvas (or press Ctrl+Tab).\nGrid view shows all pages at once.", 84.0, 575.0, 760.0, 70.0);
    p.text(
        "Tip: the search box at the top right finds any command by name.",
        84.0,
        660.0,
        17.0,
        ca(255, 255, 255, 200),
        false,
        false,
    );
    p.finish()
}

pub fn cut_out() -> ProjectPage {
    let teal = c(20, 184, 166);
    let mut p = Page::new("Cut out and fix photos", c(248, 250, 252));
    p.header(
        1,
        TOTAL,
        "Photo editing: cut out, enlarge and fix",
        "The Image tab and the AI tools, on a real photo. Click any picture on this page and try the same.",
        teal,
    );
    p.steps(
        &[
            "Remove background: select a picture, then Image >\nRemove BG (AI). The subject stays, the rest becomes\nsee-through. Remove by Color works for flat colors.",
            "Recolor (Ctrl+Shift+R) and Magic wand change one color\nonly, like the T-shirt here. Select Subject (AI) selects\nthe main subject.",
            "Upscale 2x makes a picture bigger and cleaner; AI\nUpscale also rebuilds fine detail. Magic Resize turns a\ndesign into another size (post to story).",
            "Straighten levels a tilted photo; Crop (C) and\nPerspective Crop cut and flatten.",
            "Fill (AI): select something, then Select > Fill (AI)\npaints in what was behind it. The AI models download\nonce, after you say yes.",
        ],
        teal,
        140.0,
        110.0,
    );
    let small = photo_square(300);
    let tee = asset_picture("tee").unwrap_or_else(|| small.clone());
    let recolor = asset_picture("recolor").unwrap_or_else(|| recolor_red(&small, 215.0));
    let cutout = asset_picture("cutout").unwrap_or_else(|| small.clone());
    let before = asset_picture("fill-before").unwrap_or_else(|| landscape(320, 214, 1));
    let after = asset_picture("fill-after").unwrap_or_else(|| landscape(320, 214, 1));
    p.add(pic(&tee, 648.0, 140.0, 170.0, 170.0, "Original photo"), "Original photo");
    p.add(pic(&recolor, 840.0, 140.0, 170.0, 170.0, "Recolored T-shirt"), "Recolored T-shirt");
    let cut_w = 170.0 * f64::from(cutout.width()) / f64::from(cutout.height());
    let checker = p.shape("Checker", ShapeKind::Rectangle, 1032.0, 140.0, 170.0, 170.0, Some(c(226, 232, 240)));
    set_radius(checker, 4.0);
    p.add(
        pic(&cutout, 1032.0 + (170.0 - cut_w) / 2.0, 140.0, cut_w, 170.0, "Background removed"),
        "Background removed",
    );
    for (t, x) in [("Original", 733.0), ("Recolor (T-shirt only)", 925.0), ("Remove BG (AI)", 1117.0)] {
        p.text(t, x, 314.0, 14.0, SUBTLE, false, true);
    }
    p.add(pic(&before, 648.0, 360.0, 180.0, 120.0, "Before: Fill (AI)"), "Before: Fill (AI)");
    p.shape("Arrow", ShapeKind::Arrow, 836.0, 408.0, 30.0, 24.0, Some(teal));
    p.add(pic(&after, 874.0, 360.0, 180.0, 120.0, "After: Fill (AI)"), "After: Fill (AI)");
    p.text(
        "Fill (AI): the person is removed, the sea and rocks are painted in",
        851.0,
        484.0,
        14.0,
        SUBTLE,
        false,
        true,
    );
    let hint = p.shape("Hint", ShapeKind::Rectangle, 1076.0, 360.0, 184.0, 120.0, Some(c(254, 240, 138)));
    set_radius(hint, 8.0);
    shadow(hint, ShadowStyle { color: ca(0, 0, 0, 55), distance: 5.0, angle: 60.0, blur: 10.0, size: 1.0 });
    p.text(
        "Try it: pick a\npicture, then Image >\nRemove BG (AI), or\nSelect > Fill (AI).",
        1090.0,
        372.0,
        14.5,
        c(113, 63, 18),
        false,
        false,
    );
    let tiny = resize(&photo_square(40), 300, 300, false);
    pair(&mut p, 648.0, 530.0, "Upscale: small to big", &tiny, &small, teal, 0.0, 0.0);
    pair(&mut p, 960.0, 530.0, "Straighten", &small, &small, teal, 9.0, 10.0);
    p.finish()
}

/// A before picture, an arrow and an after picture, with what was done written under them.
#[allow(clippy::too_many_arguments)]
fn pair(
    p: &mut Page,
    x: f64,
    y: f64,
    label: &str,
    before: &crate::imaging::PixelBuffer,
    after: &crate::imaging::PixelBuffer,
    accent: crate::imaging::ColorRgba,
    tilt: f64,
    inset: f64,
) {
    let (size, bx) = (128.0, 128.0 - inset * 2.0);
    let mut b = pic(before, x + inset, y + inset, bx, bx, &format!("Before: {label}"));
    b.set_box(ObjectBox::new(x + inset, y + inset, bx, bx, tilt));
    p.add(b, &format!("Before: {label}"));
    p.shape("Arrow", ShapeKind::Arrow, x + size + 6.0, y + size / 2.0 - 11.0, 28.0, 22.0, Some(accent));
    p.add(
        pic(after, x + size + 40.0 + inset, y + inset, bx, bx, &format!("After: {label}")),
        &format!("After: {label}"),
    );
    p.text(label, x + size + 20.0, y + size + 8.0, 14.0, SUBTLE, false, true);
}

pub fn photo_look() -> ProjectPage {
    let orange = c(249, 115, 22);
    let mut p = Page::new("Adjust and filters", c(248, 250, 252));
    p.header(
        2,
        TOTAL,
        "Photo editing: adjust, filters and layers",
        "Photoshop-style tools on the same photo: tone and color, filters, retouching, masks and blend modes.",
        orange,
    );
    p.steps(
        &[
            "Adjust tab: brightness, contrast, hue, saturation,\ntemperature, levels, curves. As an Adjustment layer it\nis non-destructive: change or delete it any time.",
            "Properties > Adjust (for the selected picture): quick\nbrightness, contrast, saturation, hue, temperature and\ntint sliders, and blur. No dialog needed.",
            "Filter tab: blur, motion blur, sharpen, noise, pixelate,\nvignette, emboss, find edges. Select an area first to\nlimit the effect to it.",
            "Retouch tools in the toolbox: Healing, Clone stamp,\nPatch, Dodge / Burn, Liquify. Selections, layer masks,\nblend modes and opacity are in the Layer tab.",
        ],
        orange,
        140.0,
        135.0,
    );
    let base = photo_square(300);
    use fe_core_effects as fx;
    let items: Vec<(&str, crate::imaging::PixelBuffer)> = vec![
        ("Original", base.clone()),
        ("Grayscale", fx::run(&fx::GRAYSCALE, &base, &[])),
        ("Sepia", fx::run(&fx::SEPIA, &base, &[])),
        ("Warm (temperature +60)", fx::run(&fx::TEMPERATURE_TINT, &base, &[("temperature", 60.0)])),
        ("Cool (temperature -60)", fx::run(&fx::TEMPERATURE_TINT, &base, &[("temperature", -60.0)])),
        ("More contrast", fx::run(&fx::BRIGHTNESS_CONTRAST, &base, &[("contrast", 40.0)])),
        ("Hue shift", fx::run(&fx::HUE_SATURATION, &base, &[("hue", 60.0)])),
        ("Blur", fx::run(&fx::GAUSSIAN_BLUR, &base, &[("radius", 5.0)])),
        ("Sharpen", fx::run(&fx::SHARPEN, &base, &[])),
        ("Vignette", fx::run(&fx::VIGNETTE, &base, &[])),
        ("Pixelate", fx::run(&fx::PIXELATE, &base, &[("size", 10.0)])),
        ("Emboss", fx::run(&fx::EMBOSS, &base, &[])),
    ];
    for (i, (label, image)) in items.iter().enumerate() {
        let (x, y) = (648.0 + (i % 4) as f64 * 152.0, 140.0 + (i / 4) as f64 * 180.0);
        p.add(pic(image, x, y, 140.0, 140.0, label), label);
        p.text(label, x + 70.0, y + 144.0, 12.5, SUBTLE, false, true);
    }
    p.finish()
}

pub fn selecting() -> ProjectPage {
    let blue = c(59, 130, 246);
    let mut p = Page::new("Select and move", c(248, 250, 252));
    p.header(
        3,
        TOTAL,
        "Select, move and resize",
        "Everything you add is an object on its own layer. Click it and it stays fully editable.",
        blue,
    );
    p.steps(
        &[
            "Click an object with the Select / Move tool (V) to select it.",
            "Drag it to move. Drag a corner to resize it: the link\nbutton in Properties keeps the proportions.",
            "Drag the round handle above it to turn it\n(hold Shift for 15° steps).",
            "Shift+click, or Ctrl+drag a box, to select several.\nCtrl+G groups them, Ctrl+Shift+G ungroups.",
            "Arrow keys nudge by 1 pixel, Shift+arrows by 10.",
        ],
        blue,
        150.0,
        100.0,
    );
    let o = p.shape("Try me", ShapeKind::Rectangle, 700.0, 170.0, 230.0, 150.0, Some(blue));
    set_radius(o, 22.0);
    shadow(o, ShadowStyle::DEFAULT);
    shadow(
        p.shape("Turn me", ShapeKind::Ellipse, 990.0, 190.0, 190.0, 190.0, Some(c(236, 72, 153))),
        ShadowStyle::DEFAULT,
    );
    let star = p.shape("Star", ShapeKind::Star, 740.0, 400.0, 190.0, 190.0, Some(c(245, 158, 11)));
    if let Some(b) = star.box_() {
        star.set_box(ObjectBox::new(b.x, b.y, b.width, b.height, 18.0));
    }
    shadow(star, ShadowStyle::DEFAULT);
    let g1 = StoredObject::new_shape(
        ObjectBox::new(1010.0, 430.0, 90.0, 90.0, 0.0),
        &ShapeData::new(ShapeKind::Hexagon, Some(c(34, 197, 94)), None, 0.0),
    );
    let g2 = StoredObject::new_shape(
        ObjectBox::new(1070.0, 490.0, 90.0, 90.0, 0.0),
        &ShapeData::new(ShapeKind::Heart, Some(c(239, 68, 68)), None, 0.0),
    );
    if let Some(group) = StoredObject::new_group(&[
        GroupMember { object: g1, name: "Hexagon".into(), opacity: 1.0 },
        GroupMember { object: g2, name: "Heart".into(), opacity: 1.0 },
    ]) {
        p.add(group, "A group");
    }
    p.caption("A group: it moves as one.\nDouble-click it to edit a single part.", 1085.0, 585.0, 14.0);
    p.note("Try it on these shapes.\nThe Layers panel on the right lists every object.", 700.0, 640.0, 480.0, 62.0);
    p.finish()
}

pub fn text_page() -> ProjectPage {
    let purple = c(124, 58, 237);
    let mut p = Page::new("Text", c(248, 250, 252));
    p.header(
        4,
        TOTAL,
        "Text and designs",
        "Add words, then change font, color, spacing, curve and ready-made designs in Properties.",
        purple,
    );
    p.steps(
        &[
            "Insert > Heading, Subheading or Body Text.\nDouble-click any text to change its words.",
            "Properties: font, size, bold, color, alignment,\nspace between letters, words and lines.",
            "Curve slider bends the text around a circle.",
            "Text designs: one click for shadow, outline,\nneon, 3D and gradient looks.",
        ],
        purple,
        150.0,
        110.0,
    );
    for (i, preset) in TEXT_PRESETS.iter().take(8).enumerate() {
        let (x, y) = (690.0 + (i % 4) as f64 * 142.0, 150.0 + (i / 4) as f64 * 130.0);
        let o = p.text_with("Aa", x + 60.0, y, 64.0, INK, true, true, 1.0, |_| {});
        preset.apply(o);
        let h = o.box_().map_or(70.0, |b| b.height);
        if let Some(b) = o.box_() {
            o.set_box(ObjectBox::new(x + 60.0 - b.width / 2.0, y, b.width, b.height, 0.0));
        }
        p.caption(preset.name, x + 60.0, y + h + 2.0, 13.0);
    }
    p.text_with("Curved text", 930.0, 440.0, 44.0, purple, true, true, 1.15, |t| t.arc = 70.0);
    p.text_with("Bold and italic", 930.0, 548.0, 34.0, INK, true, true, 1.15, |t| t.italic = true);
    p.text_with("Outlined", 930.0, 606.0, 50.0, WHITE, true, true, 1.15, |t| {
        t.outline_width = 4.0;
        t.outline_color = Some(purple);
    });
    p.finish()
}
