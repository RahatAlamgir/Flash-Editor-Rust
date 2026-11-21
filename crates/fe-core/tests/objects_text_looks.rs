//! Text, looks (shadow, glow, 3D...), gradients, text designs, saved styles and frames (Phase 9).

mod common;

use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::objects::style::{ObjectStyle, SavedStyle, styles_from_json, styles_to_json};
use fe_core::objects::{
    COLLAGE_LAYOUTS, GRADIENT_PRESETS, Gradient, Looks, ObjectBox, ShadowStyle, ShapeData, ShapeKind, StoredObject,
    ThreeDStyle,
};
use fe_core::render;
use fe_core::text::{TEXT_PRESETS, TextData, fonts};

const CANVAS: PixelRect = PixelRect::new(0, 0, 300, 200);

fn has_fonts() -> bool {
    !fonts::families().is_empty()
}

fn text_object(words: &str) -> StoredObject {
    let data = TextData::new(words, "Segoe UI", 40.0, ColorRgba::BLACK);
    let (w, h) = render::measure_text(&data);
    StoredObject::new_text(ObjectBox::new(20.0, 40.0, w, h, 0.0), &data)
}

#[test]
fn text_is_drawn_inside_its_box_and_measured_from_the_letters() {
    if !has_fonts() {
        return;
    }
    let obj = text_object("Hello");
    let b = obj.box_().unwrap();
    assert!(b.width > 60.0 && b.height > 30.0, "{b:?}");
    let drawn = render::render_object(&obj, CANVAS).expect("text can be drawn");
    let area = drawn.visible_bounds();
    assert!(!area.is_empty());
    assert!(area.x >= 18 && area.right() <= (b.x + b.width) as i32 + 3, "{area:?}");
    assert!(area.y >= 38 && area.bottom() <= (b.y + b.height) as i32 + 3, "{area:?}");
    // More words make it wider; two lines make it taller.
    let longer = TextData::new("Hello Hello", "Segoe UI", 40.0, ColorRgba::BLACK);
    assert!(render::measure_text(&longer).0 > b.width * 1.7);
    let lines = TextData::new("Hello\nHello", "Segoe UI", 40.0, ColorRgba::BLACK);
    assert!(render::measure_text(&lines).1 > b.height * 1.8);
}

#[test]
fn letter_gaps_curves_and_alignment_change_the_layout() {
    if !has_fonts() {
        return;
    }
    let plain = TextData::new("Gap test", "Segoe UI", 30.0, ColorRgba::BLACK);
    let (w, _) = render::measure_text(&plain);
    let spaced = TextData { letter_spacing: 0.3, ..plain.clone() };
    assert!(render::measure_text(&spaced).0 > w + 8.0 * 30.0 * 0.3 * 0.8);
    let curved = TextData { arc: 240.0, ..plain.clone() };
    assert!(render::measure_text(&curved).1 > render::measure_text(&plain).1 + 4.0, "a curve is taller");
    // A centered second line starts further right than a left aligned one.
    let two = |align| {
        let t = TextData { text: "Wide line\nab".into(), align, ..plain.clone() };
        let b = ObjectBox::new(0.0, 0.0, render::measure_text(&t).0, render::measure_text(&t).1, 0.0);
        let drawn = render::render_object(&StoredObject::new_text(b, &t), CANVAS).unwrap();
        let second = PixelRect::new(0, 40, 300, 40);
        drawn
            .read_rect(second)
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p[3] > 100)
            .map(|(i, _)| i % 300)
            .min()
    };
    let (left, center) = (two(fe_core::text::TextAlign::Left).unwrap(), two(fe_core::text::TextAlign::Center).unwrap());
    assert!(center > left + 20, "left {left}, center {center}");
}

#[test]
fn looks_are_kept_in_the_description_and_drawn_outside_the_box() {
    let mut obj = StoredObject::new_shape(
        ObjectBox::new(100.0, 60.0, 60.0, 60.0, 0.0),
        &ShapeData::new(ShapeKind::Rectangle, Some(ColorRgba::rgb(255, 0, 0)), None, 0.0),
    );
    let plain = render::render_object(&obj, CANVAS).unwrap().visible_bounds();
    assert_eq!(PixelRect::new(100, 60, 60, 60), plain);
    let looks = Looks { shadow: Some(ShadowStyle::DEFAULT), three_d: Some(ThreeDStyle::DEFAULT), ..Looks::default() };
    obj.set_looks(&looks);
    assert_eq!(looks, obj.looks());
    assert_eq!("8", obj.json["Shadow"]["Distance"].to_string());
    let with = render::render_object(&obj, CANVAS).unwrap().visible_bounds();
    assert!(with.right() > plain.right() + 15 && with.bottom() > plain.bottom() + 15, "{with:?}");
    // Switching it off again gives the plain drawing back.
    obj.set_looks(&Looks::default());
    assert!(obj.json.get("Shadow").is_none());
    assert_eq!(plain, render::render_object(&obj, CANVAS).unwrap().visible_bounds());
}

#[test]
fn a_glow_and_an_outline_follow_the_outline_of_the_object() {
    let mut obj = StoredObject::new_shape(
        ObjectBox::new(100.0, 60.0, 60.0, 60.0, 0.0),
        &ShapeData::new(ShapeKind::Ellipse, Some(ColorRgba::rgb(0, 0, 255)), None, 0.0),
    );
    let looks = Looks {
        outline: Some(fe_core::objects::OutlineStyle { color: ColorRgba::rgb(255, 255, 255), width: 6.0 }),
        ..Looks::default()
    };
    obj.set_looks(&looks);
    let drawn = render::render_object(&obj, CANVAS).unwrap();
    // The ring is white outside the circle's edge, not at its middle.
    let edge = drawn.pixel(100 - 3, 90);
    assert!(edge.a > 200 && edge.r > 200, "{edge:?}");
    assert_eq!(0, drawn.pixel(100 - 12, 90).a);
}

#[test]
fn gradients_round_trip_and_get_more_colors() {
    let g = GRADIENT_PRESETS[8].create(); // Rainbow: five colors
    assert_eq!(5, g.stops.len());
    let back = Gradient::from_json(&g.to_json()).unwrap();
    assert_eq!(g, back);
    let two = GRADIENT_PRESETS[0].create();
    let three = two.with_stop_added();
    assert_eq!(3, three.stops.len());
    assert!((three.stops[2].1 - 0.5).abs() < 1e-9);
    assert_eq!(2, three.with_stop_removed(2).stops.len());
    assert_eq!(2, two.with_stop_removed(0).stops.len(), "two colors is the least");
    assert_eq!(8, (0..10).fold(three, |g, _| g.with_stop_added()).stops.len(), "eight colors is the most");
}

#[test]
fn a_text_design_changes_the_look_but_not_the_words() {
    if !has_fonts() {
        return;
    }
    let mut obj = text_object("Keep me");
    let words_before = obj.text().unwrap().text;
    let neon = TEXT_PRESETS.iter().find(|p| p.name == "Neon Pink").unwrap();
    neon.apply(&mut obj);
    assert_eq!(words_before, obj.text().unwrap().text);
    assert!(obj.looks().glow.is_some());
    // The effects were made for 64 px text: 40 px text gets smaller ones.
    assert!((obj.looks().glow.unwrap().size - 18.0 * 40.0 / 64.0).abs() < 1e-9);
    let sunset = TEXT_PRESETS.iter().find(|p| p.name == "Sunset").unwrap();
    sunset.apply(&mut obj);
    assert!(obj.looks().glow.is_none(), "the old effects go");
    assert!(obj.gradient().is_some());
    assert!(render::render_object(&obj, CANVAS).is_some());
}

#[test]
fn saved_styles_round_trip_and_go_onto_other_objects() {
    let mut shape = StoredObject::new_shape(
        ObjectBox::new(10.0, 10.0, 80.0, 40.0, 0.0),
        &ShapeData::new(ShapeKind::Rectangle, Some(ColorRgba::rgb(10, 200, 30)), Some(ColorRgba::BLACK), 3.0),
    );
    shape.set_looks(&Looks { shadow: Some(ShadowStyle::DEFAULT), blur: 5.0, ..Looks::default() });
    shape.set_gradient(Some(&GRADIENT_PRESETS[1].create()));
    let style = ObjectStyle::from(&shape);
    assert_eq!(0.0, style.looks.blur, "blur is not part of a look");
    let saved = vec![SavedStyle { name: "Mine".into(), style: style.clone() }];
    let back = styles_from_json(&styles_to_json(&saved));
    assert_eq!(saved, back);

    let mut other = StoredObject::new_shape(
        ObjectBox::new(0.0, 0.0, 20.0, 20.0, 0.0),
        &ShapeData::new(ShapeKind::Star, Some(ColorRgba::WHITE), None, 0.0),
    );
    style.apply_to(&mut other);
    assert_eq!(Some(ColorRgba::rgb(10, 200, 30)), other.shape().unwrap().fill);
    assert_eq!(3.0, other.shape().unwrap().stroke_width);
    assert!(other.looks().shadow.is_some() && other.gradient().is_some());
    // A text only takes the effects (the kinds differ).
    if has_fonts() {
        let mut text = text_object("Hi");
        style.apply_to(&mut text);
        assert!(text.looks().shadow.is_some());
        assert!(text.gradient().is_none());
    }
}

#[test]
fn collage_layouts_fill_the_canvas_and_frames_hold_pictures() {
    let grid = &COLLAGE_LAYOUTS[4]; // 2 x 2
    let cells = grid.place(200.0, 100.0, 10.0, 10.0);
    assert_eq!(4, cells.len());
    assert_eq!((10.0, 10.0, 85.0, 35.0), cells[0]);
    assert_eq!((105.0, 55.0, 85.0, 35.0), cells[3]);
    let big = COLLAGE_LAYOUTS[6].place(200.0, 100.0, 0.0, 0.0); // one large, two small
    assert_eq!((0.0, 0.0, 100.0, 100.0), big[0]);

    let mut frame = StoredObject::new_shape(
        ObjectBox::new(0.0, 0.0, 80.0, 60.0, 0.0),
        &ShapeData::new(ShapeKind::Ellipse, Some(ColorRgba::rgb(226, 232, 240)), None, 0.0),
    );
    assert!(frame.picture_fill().is_none());
    let picture = common::solid(40, 30, ColorRgba::rgb(200, 0, 0));
    frame.set_picture_fill(Some((fe_core::objects::picture_bytes(&picture), "red")));
    let fill = frame.picture_fill().unwrap();
    assert_eq!((1.0, 0.0, 0.0, "red"), (fill.zoom, fill.pan_x, fill.pan_y, fill.name.as_str()));
    assert!(frame.picture().is_some());
    frame.set_picture_view(9.0, -3.0, 0.5);
    let view = frame.picture_fill().unwrap();
    assert_eq!((5.0, -1.0, 0.5), (view.zoom, view.pan_x, view.pan_y), "kept in range");
    // The picture shows inside the ellipse, not in the corners.
    let drawn = render::render_object(&frame, PixelRect::new(0, 0, 100, 80)).unwrap();
    assert!(drawn.pixel(40, 30).r > 150);
    assert_eq!(0, drawn.pixel(1, 1).a);
    frame.set_picture_fill(None);
    assert!(frame.picture_fill().is_none());
}

#[test]
fn cursor_places_follow_the_letters_lines_and_curves() {
    if !has_fonts() {
        return;
    }
    let mut t = TextData::new("ab\ncde", "Segoe UI", 30.0, ColorRgba::BLACK);
    let laid = fe_core::text::lay_out(&t).unwrap();
    // 2 letters + break + 3 letters: places 0..=6, left to right on a line, the second line lower.
    assert_eq!(6 + 1, laid.carets.len());
    assert!(laid.carets.windows(2).take(2).all(|w| w[1].a.0 > w[0].a.0), "{:?}", laid.carets);
    assert_eq!(0, laid.carets[2].line);
    assert_eq!(1, laid.carets[3].line);
    assert!(laid.carets[3].a.1 > laid.carets[2].b.1 - 1.0);
    assert!(laid.carets[3].a.0 < laid.carets[2].a.0, "a new line starts at the left");
    // An empty text has one place, and a curved text has places on its curve.
    t.text.clear();
    assert_eq!(1, fe_core::text::lay_out(&t).unwrap().carets.len());
    t.text = "Curved text".into();
    t.arc = 180.0;
    let curved = fe_core::text::lay_out(&t).unwrap();
    assert_eq!(12, curved.carets.len());
    assert!(curved.carets[5].a.1 < curved.carets[0].a.1 - 5.0 || curved.carets[5].a.1 > curved.carets[0].a.1 + 5.0);
}
