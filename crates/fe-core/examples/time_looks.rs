//! Times the drawing of objects with looks (for the progress log).
use fe_core::imaging::{ColorRgba, PixelRect};
use fe_core::objects::{Looks, ObjectBox, ShadowStyle, ShapeData, ShapeKind, StoredObject, ThreeDStyle};
use std::time::Instant;

fn main() {
    let canvas = PixelRect::new(0, 0, 1920, 1080);
    for (name, w, h) in [("small 200x200", 200.0, 200.0), ("big 1600x900", 1600.0, 900.0)] {
        let mut o = StoredObject::new_shape(
            ObjectBox::new(100.0, 100.0, w, h, 0.0),
            &ShapeData::new(ShapeKind::Heart, Some(ColorRgba::rgb(220, 40, 60)), None, 0.0),
        );
        for (look, looks) in [
            ("plain", Looks::default()),
            ("shadow", Looks { shadow: Some(ShadowStyle::DEFAULT), ..Looks::default() }),
            ("3D", Looks { three_d: Some(ThreeDStyle::DEFAULT), ..Looks::default() }),
            (
                "all",
                Looks {
                    shadow: Some(ShadowStyle::DEFAULT),
                    three_d: Some(ThreeDStyle::DEFAULT),
                    glow: Some(fe_core::objects::GlowStyle::DEFAULT),
                    inner_shadow: Some(fe_core::objects::InnerShadowStyle::DEFAULT),
                    outline: Some(fe_core::objects::OutlineStyle::DEFAULT),
                    blur: 0.0,
                },
            ),
        ] {
            o.set_looks(&looks);
            let t = Instant::now();
            let _ = fe_core::render::render_object(&o, canvas);
            println!("{name} {look}: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
        }
    }
}
