//! Objects: the shape description round trip, drawing shapes / pictures / groups, reshape, group and ungroup,
//! flip, align and distribute (C# `ShapeObjectTests`, `GroupObjectTests`, `AlignmentTests` ideas).

mod common;

use common::*;
use fe_core::document::{Layer, object_ops};
use fe_core::imaging::{ColorRgba, PixelRect, PointD};
use fe_core::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject, StrokeStyle, picture_bytes};
use fe_core::render;

const RED: ColorRgba = ColorRgba::rgb(255, 0, 0);
const CANVAS: PixelRect = PixelRect::new(0, 0, 200, 100);

fn shape(kind: ShapeKind, b: ObjectBox) -> StoredObject {
    StoredObject::new_shape(b, &ShapeData::new(kind, Some(RED), None, 0.0))
}

#[test]
fn a_shape_description_is_written_like_the_csharp_files() {
    let obj = shape(ShapeKind::Star, ObjectBox::new(10.0, 20.0, 30.0, 40.0, 0.0));
    let keys: Vec<&String> = obj.json.as_object().unwrap().keys().collect();
    assert_eq!("kind", keys[0]);
    assert_eq!("shape", obj.kind());
    assert_eq!("3", obj.json["Shape"].to_string()); // enums are numbers
    assert_eq!("10", obj.json["Box"]["X"].to_string()); // whole numbers without ".0"
    let mut data = obj.shape().unwrap();
    assert_eq!(ShapeKind::Star, data.kind);
    data.dash = StrokeStyle::Dotted;
    data.points = 7;
    let mut changed = obj.clone();
    changed.set_shape(&data);
    assert_eq!(Some(data), changed.shape());
    assert_eq!(ShapeKind::Path, ShapeKind::from_number(37).unwrap());
}

#[test]
fn a_rectangle_fills_its_box_and_an_ellipse_leaves_the_corners() {
    let rect = render::render_object(&shape(ShapeKind::Rectangle, ObjectBox::new(10.0, 10.0, 50.0, 30.0, 0.0)), CANVAS)
        .unwrap();
    assert_eq!(RED, rect.pixel(12, 12));
    assert_eq!(RED, rect.pixel(58, 38));
    assert_eq!(0, rect.pixel(61, 20).a);
    let ellipse =
        render::render_object(&shape(ShapeKind::Ellipse, ObjectBox::new(10.0, 10.0, 50.0, 30.0, 0.0)), CANVAS).unwrap();
    assert_eq!(0, ellipse.pixel(11, 11).a);
    assert_eq!(RED, ellipse.pixel(35, 25));
}

#[test]
fn every_shape_kind_draws_something_inside_its_box() {
    for kind in ShapeKind::ALL {
        let b = ObjectBox::new(20.0, 10.0, 80.0, 60.0, 0.0);
        let mut data = ShapeData::new(*kind, Some(RED), Some(ColorRgba::BLACK), 3.0);
        if *kind == ShapeKind::Line {
            data.fill = None;
        }
        let drawn = render::render_object(&StoredObject::new_shape(b, &data), CANVAS).unwrap();
        let bounds = drawn.visible_bounds();
        assert!(!bounds.is_empty(), "{kind:?} drew nothing");
        assert!(bounds.x >= 16 && bounds.right() <= 104, "{kind:?} drew outside its box: {bounds}");
    }
}

#[test]
fn a_turned_shape_turns_its_drawing() {
    // A thin bar, turned 90 degrees: it now stands upright.
    let obj = shape(ShapeKind::Rectangle, ObjectBox::new(50.0, 45.0, 60.0, 10.0, 90.0));
    let drawn = render::render_object(&obj, CANVAS).unwrap();
    let b = drawn.visible_bounds();
    assert!((b.width - 10).abs() <= 1 && (b.height - 60).abs() <= 1, "{b}");
}

#[test]
fn a_dashed_line_has_gaps_and_arrow_heads_stick_out() {
    let mut data = ShapeData::new(ShapeKind::Line, None, Some(RED), 4.0);
    data.dash = StrokeStyle::Dashed;
    let b = ObjectBox::new(10.0, 40.0, 150.0, 20.0, 0.0);
    let drawn = render::render_object(&StoredObject::new_shape(b, &data), CANVAS).unwrap();
    let row: Vec<u8> = (10..160).map(|x| drawn.pixel(x, 50).a).collect();
    assert!(row.contains(&0) && row.contains(&255), "dashes and gaps");
    data.dash = StrokeStyle::Solid;
    data.arrow_end = true;
    let arrow = render::render_object(&StoredObject::new_shape(b, &data), CANVAS).unwrap();
    assert!(arrow.pixel(150, 45).a > 0, "the head is wider than the line");
    assert_eq!(0, arrow.pixel(120, 45).a);
}

#[test]
fn a_picture_is_drawn_into_its_box_and_can_be_reshaped() {
    let picture = picture_bytes(&solid(20, 20, RED));
    let obj = StoredObject::new_image(ObjectBox::new(10.0, 10.0, 40.0, 40.0, 0.0), picture, "Photo");
    let drawn = render::render_object(&obj, CANVAS).unwrap();
    assert_eq!(RED, drawn.pixel(30, 30));
    assert_eq!(0, drawn.pixel(55, 30).a);
    // Drag the bottom-right corner out: the picture now reaches further.
    let mut warped = obj.clone();
    warped.set_warp(Some(&[
        PointD::new(0.0, 0.0),
        PointD::new(1.0, 0.0),
        PointD::new(2.0, 2.0),
        PointD::new(0.0, 1.0),
    ]));
    let w = render::render_object(&warped, CANVAS).unwrap();
    assert!(w.pixel(75, 75).a > 200);
}

#[test]
fn grouping_keeps_the_look_and_ungrouping_puts_the_parts_back() {
    let a =
        object_ops::object_layer("A", shape(ShapeKind::Rectangle, ObjectBox::new(10.0, 10.0, 20.0, 20.0, 0.0)), CANVAS);
    let b =
        object_ops::object_layer("B", shape(ShapeKind::Ellipse, ObjectBox::new(60.0, 30.0, 30.0, 30.0, 0.0)), CANVAS);
    let ids = [a.id(), b.id()];
    let mut doc = doc_with(200, 100, vec![solid_layer("Bg", 200, 100, ColorRgba::WHITE), a, b]);
    let group = object_ops::group(&mut doc, &ids).expect("grouped");
    assert_eq!(2, doc.layers().len());
    let layer = doc.layer(group).unwrap();
    assert_eq!(RED, layer.pixels().pixel(20, 20));
    assert_eq!(RED, layer.pixels().pixel(75, 45));
    let gb = layer.object().unwrap().box_().unwrap();
    assert_eq!((10.0, 10.0, 80.0, 50.0), (gb.x, gb.y, gb.width, gb.height));

    // Move the group, then take it apart: the parts are where the group put them.
    let mut moved = layer.object().unwrap().clone();
    moved.set_box(gb.offset(50.0, 0.0));
    object_ops::set_object(doc.layer_mut(group).unwrap(), moved, CANVAS);
    let parts = object_ops::ungroup(&mut doc, group).expect("ungrouped");
    assert_eq!(3, doc.layers().len());
    let first = doc.layer(parts[0]).unwrap();
    assert_eq!("A", first.name);
    assert_eq!(Some(60.0), first.object().unwrap().box_().map(|b| b.x));
    assert_eq!(RED, first.pixels().pixel(70, 20));
}

#[test]
fn a_pixel_layer_becomes_a_picture_and_keeps_its_pixels() {
    let mut layer = Layer::new("Paint");
    layer.pixels_mut().fill_rect(PixelRect::new(5, 5, 10, 10), RED);
    let id = layer.id();
    let mut doc = doc_with(200, 100, vec![layer]);
    assert!(object_ops::make_picture(&mut doc, id));
    let l = doc.layer(id).unwrap();
    assert_eq!("image", l.object().unwrap().kind());
    assert_eq!(Some((5.0, 5.0, 10.0, 10.0)), l.object().unwrap().bounds());
    assert_eq!(RED, l.pixels().pixel(6, 6));
}

#[test]
fn objects_not_drawn_yet_keep_their_saved_drawing_when_moved_or_stretched() {
    // A chart (drawn in Phase 10): its saved drawing is stretched with its box.
    let json = serde_json::json!({"kind": "mystery", "Title": "Hi", "Box": {"X": 10, "Y": 10, "Width": 20, "Height": 10, "Rotation": 0}});
    let text = StoredObject { json, images: Default::default() };
    let mut layer = Layer::new("Text");
    let mut pixels = fe_core::imaging::TileMap::empty();
    pixels.fill_rect(PixelRect::new(10, 10, 20, 10), RED);
    layer.set_pixels_with_object(pixels, Some(text.clone()));
    let mut bigger = text.clone();
    bigger.set_box(ObjectBox::new(10.0, 10.0, 40.0, 20.0, 0.0));
    object_ops::set_object(&mut layer, bigger, CANVAS);
    assert!(layer.pixels().pixel(45, 25).a > 200);
    assert_eq!(0, layer.pixels().pixel(55, 35).a);
}

#[test]
fn flipping_mirrors_objects_in_place_and_pixels_across_the_canvas() {
    let mut paint = Layer::new("Paint");
    paint.pixels_mut().fill_rect(PixelRect::new(0, 0, 10, 10), RED);
    let id = paint.id();
    let mut doc = doc_with(200, 100, vec![paint]);
    assert!(object_ops::flip(&mut doc, id, true));
    assert_eq!(RED, doc.layer(id).unwrap().pixels().pixel(195, 5));
    assert_eq!(0, doc.layer(id).unwrap().pixels().pixel(5, 5).a);
}

#[test]
fn align_and_distribute_like_the_csharp_version() {
    let canvas = PixelRect::new(0, 0, 100, 100);
    let one = [PixelRect::new(10, 10, 20, 20)];
    assert_eq!(vec![(30, 0)], object_ops::align_moves(&one, canvas, object_ops::AlignKind::CenterHorizontally));
    let three = [PixelRect::new(0, 0, 10, 10), PixelRect::new(15, 0, 10, 10), PixelRect::new(90, 0, 10, 10)];
    assert_eq!(vec![(0, 0), (30, 0), (0, 0)], object_ops::distribute_moves(&three, true));
    assert_eq!(
        vec![(0, 0), (0, 0), (0, -90)].len(),
        object_ops::align_moves(&three, canvas, object_ops::AlignKind::Top).len()
    );
}

#[test]
fn snapping_lines_up_edges_and_equal_gaps() {
    use fe_core::layout::snapping::{SnapSession, SnapSettings};
    let a =
        object_ops::object_layer("A", shape(ShapeKind::Rectangle, ObjectBox::new(10.0, 10.0, 20.0, 20.0, 0.0)), CANVAS);
    let b =
        object_ops::object_layer("B", shape(ShapeKind::Rectangle, ObjectBox::new(50.0, 10.0, 20.0, 20.0, 0.0)), CANVAS);
    let doc = doc_with(200, 100, vec![a, b]);
    let settings =
        SnapSettings { objects: true, grid: false, canvas: true, guides: false, grid_size: 50.0, spacing: true };
    let s = SnapSession::new(&doc, &[], &[], &[], settings);
    // A box whose left edge is 2 px from B's right edge snaps onto it.
    let r = s.snap_box(72.0, 60.0, 82.0, 70.0, 6.0);
    assert_eq!(-2.0, r.dx);
    assert_eq!(vec![70.0], r.lines_x);
    // A box 3 px away from the equal gap (A-B gap is 20) to the right of B snaps there and shows the gaps.
    let r = s.snap_box(93.0, 10.0, 113.0, 30.0, 6.0);
    assert_eq!(-3.0, r.dx);
    assert!(!r.gaps.is_empty());
    // Too far: no snap.
    assert_eq!(0.0, s.snap_box(140.0, 50.0, 150.0, 55.0, 2.0).dx);
}

fn two_part_group() -> StoredObject {
    use fe_core::objects::GroupMember;
    let part = |x: f64, name: &str| GroupMember {
        object: shape(ShapeKind::Rectangle, ObjectBox::new(x, 10.0, 20.0, 20.0, 0.0)),
        name: name.into(),
        opacity: 1.0,
    };
    StoredObject::new_group(&[part(10.0, "Left"), part(60.0, "Right")]).expect("group")
}

#[test]
fn one_part_of_a_group_can_be_moved_renamed_and_removed() {
    let group = two_part_group();
    let right = group.part(1).expect("part").object;
    assert_eq!(60.0, right.box_().unwrap().x);

    // Moving a part refits the group around its parts and keeps the other part where it was.
    let mut moved = right.clone();
    moved.set_box(right.box_().unwrap().offset(40.0, 0.0));
    let changed = group.with_part(1, moved).expect("changed");
    assert_eq!(10.0, changed.box_().unwrap().x);
    assert_eq!(110.0, changed.box_().unwrap().width);
    assert_eq!(10.0, changed.part(0).unwrap().object.box_().unwrap().x);
    assert_eq!(100.0, changed.part(1).unwrap().object.box_().unwrap().x);

    let renamed = changed.with_part_name(0, "Start").expect("renamed");
    assert_eq!("Start", renamed.part(0).unwrap().name);

    // A part goes away; the group shrinks. The last part cannot be removed.
    let smaller = changed.without_part(0).expect("removed");
    assert_eq!(1, smaller.group_members().unwrap().len());
    assert!(smaller.without_part(0).is_none());
}

#[test]
fn a_part_follows_the_group_when_it_is_moved() {
    let group = two_part_group();
    let moved = group.offset(5.0, 7.0);
    let right = moved.part(1).unwrap().object.box_().unwrap();
    assert_eq!((65.0, 17.0), (right.x, right.y));
}
