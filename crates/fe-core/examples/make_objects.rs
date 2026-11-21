//! Writes a project with objects made by the Rust version (shapes, a picture, a group, a reshaped picture), to
//! check that the C# app reads them. Usage: `cargo run -p fe-core --example make_objects -- out.flash`

use fe_core::document::{Document, Page, object_ops};
use fe_core::imaging::{ColorRgba, PixelBuffer, PointD};
use fe_core::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject, StrokeStyle, picture_bytes};

fn main() {
    let path = std::env::args().nth(1).expect("usage: make_objects <out.flash>");
    let mut doc = Document::blank(800, 500, ColorRgba::WHITE);
    let canvas = doc.bounds();
    let mut star = ShapeData::new(ShapeKind::Star, Some(ColorRgba::rgb(250, 200, 30)), Some(ColorRgba::BLACK), 4.0);
    star.points = 6;
    let a = object_ops::object_layer(
        "Star",
        StoredObject::new_shape(ObjectBox::new(40.0, 40.0, 160.0, 160.0, 15.0), &star),
        canvas,
    );
    let mut line = ShapeData::new(ShapeKind::Line, None, Some(ColorRgba::rgb(20, 120, 220)), 6.0);
    (line.dash, line.arrow_end, line.bend) = (StrokeStyle::Dashed, true, 0.3);
    let b = object_ops::object_layer(
        "Line",
        StoredObject::new_shape(ObjectBox::new(250.0, 80.0, 220.0, 40.0, 0.0), &line),
        canvas,
    );
    let mut photo = PixelBuffer::new(64, 48);
    for (i, p) in photo.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
        *p = [(i % 64 * 4) as u8, (i / 64 * 5) as u8, 128, 255];
    }
    let mut pic =
        StoredObject::new_image(ObjectBox::new(500.0, 250.0, 200.0, 150.0, -10.0), picture_bytes(&photo), "Photo");
    pic.set_warp(Some(&[PointD::new(0.1, 0.0), PointD::new(1.0, 0.0), PointD::new(1.0, 1.0), PointD::new(0.0, 0.9)]));
    let c = object_ops::object_layer("Photo", pic, canvas);
    let ring = object_ops::object_layer(
        "Ring",
        StoredObject::new_shape(
            ObjectBox::new(80.0, 280.0, 120.0, 120.0, 0.0),
            &ShapeData::new(ShapeKind::Ring, Some(ColorRgba::rgb(220, 40, 90)), None, 0.0),
        ),
        canvas,
    );
    let heart = object_ops::object_layer(
        "Heart",
        StoredObject::new_shape(
            ObjectBox::new(220.0, 300.0, 100.0, 90.0, 0.0),
            &ShapeData::new(ShapeKind::Heart, Some(ColorRgba::rgb(240, 60, 60)), None, 0.0),
        ),
        canvas,
    );
    let group_ids = [ring.id(), heart.id()];
    for l in [a, b, c, ring, heart] {
        doc.push_layer(l);
    }
    object_ops::group(&mut doc, &group_ids).expect("grouped");
    let flat = fe_core::paint::compositor::flatten(&doc);
    let png = format!("{path}.png");
    fe_core::io::encode::save(&flat, &png, fe_core::io::ImageFileFormat::Png, 90, None).expect("png");
    fe_core::io::project::save(&[Page::new("Objects", doc)], 0, &path).expect("saved");
    println!("saved {path}");
}
