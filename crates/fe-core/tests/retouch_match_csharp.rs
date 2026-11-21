//! The retouching operations give the same pictures as the C# version (references in `tests/fixtures/effects`).

use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect, PointD, TileMap};
use fe_core::retouch::{self, LiquifyMode, RetouchOp, ToneRange, apply_region};
use fe_core::selection::Selection;

fn source() -> PixelBuffer {
    let mut p = PixelBuffer::new(64, 48);
    for y in 0..48u32 {
        for x in 0..64u32 {
            let a = if x >= 40 && y < 12 {
                0
            } else if (20..36).contains(&x) && y >= 24 {
                128
            } else {
                255
            };
            p.set_pixel(
                x,
                y,
                ColorRgba::new(((x * 4 + y) % 256) as u8, ((y * 5 + x * 3) % 256) as u8, ((x * y) % 256) as u8, a),
            );
        }
    }
    p
}

fn reference(name: &str) -> PixelBuffer {
    let path = format!("{}/../../tests/fixtures/effects/{name}.bin", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let w = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let h = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    PixelBuffer::from_raw(w, h, bytes[8..].to_vec()).expect("size")
}

fn check(name: &str, got: &PixelBuffer, tolerance: i32) {
    let want = reference(name);
    assert_eq!((got.width(), got.height()), (want.width(), want.height()));
    let worst =
        got.data().iter().zip(want.data()).map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs()).max().unwrap_or(0);
    assert!(worst <= tolerance, "{name}: differs by {worst}");
}

fn region_run(op: RetouchOp) -> PixelBuffer {
    let canvas = PixelRect::new(0, 0, 64, 48);
    let tiles = TileMap::from_buffer(&source(), 0, 0);
    let mut target = tiles.clone();
    let inner = Selection::from_rect(PixelRect::new(16, 12, 30, 22), 64, 48).unwrap();
    apply_region(&mut target, &tiles, op, &inner, None, PixelRect::new(10, 8, 40, 30), canvas);
    target.to_buffer(canvas)
}

#[test]
fn the_retouch_operations_match_csharp() {
    check("rt_clone", &region_run(RetouchOp::Clone { dx: 6, dy: -4 }), 1);
    check("rt_heal", &region_run(RetouchOp::Heal { dx: 8, dy: 5, radius: 8 }), 1);
    check("rt_dodge", &region_run(RetouchOp::Tone { burn: false, range: ToneRange::Midtones }), 1);
    check("rt_burn", &region_run(RetouchOp::Tone { burn: true, range: ToneRange::Shadows }), 1);
    check("rt_blur", &region_run(RetouchOp::Blur { radius: 3 }), 1);
    check("rt_sharpen", &region_run(RetouchOp::Sharpen { radius: 2, amount: 1.6 }), 1);
}

#[test]
fn liquify_matches_csharp() {
    let canvas = PixelRect::new(0, 0, 64, 48);
    for mode in LiquifyMode::ALL {
        let mut tiles = TileMap::from_buffer(&source(), 0, 0);
        retouch::liquify::apply(&mut tiles, canvas, 30.0, 24.0, 14.0, 0.8, mode, 6.0, -3.0);
        check(&format!("rt_liquify_{mode:?}"), &tiles.to_buffer(canvas), 1);
    }
}

#[test]
fn spot_healing_picks_the_same_clean_place_as_csharp() {
    let canvas = PixelRect::new(0, 0, 64, 48);
    let tiles = TileMap::from_buffer(&source(), 0, 0);
    let found = retouch::find_spot_source(&tiles, PixelRect::new(28, 20, 8, 6), canvas).expect("a place");
    let want = std::fs::read_to_string(format!("{}/../../tests/fixtures/effects/spot.txt", env!("CARGO_MANIFEST_DIR")))
        .unwrap();
    assert_eq!(format!("{} {}", found.0, found.1), want.trim());
    let _ = PointD::new(0.0, 0.0);
}

#[test]
fn a_stroke_stays_inside_the_selection_and_never_builds_up() {
    use fe_core::paint::brush::{BrushSettings, BrushType};
    use fe_core::retouch::RetouchStroke;
    let canvas = PixelRect::new(0, 0, 64, 48);
    let tiles = TileMap::from_buffer(&PixelBuffer::filled(64, 48, ColorRgba::rgb(100, 100, 100)), 0, 0);
    let sel = Selection::from_rect(PixelRect::new(0, 0, 32, 48), 64, 48);
    let brush = BrushSettings { size: 20.0, hardness: 1.0, opacity: 0.5, kind: BrushType::Round };
    let mut target = tiles.clone();
    let mut stroke =
        RetouchStroke::new(&tiles, brush, RetouchOp::Tone { burn: false, range: ToneRange::Midtones }, sel, canvas);
    // The same place many times over: the effect stays that of one dab.
    for _ in 0..5 {
        stroke.add_point(&mut target, PointD::new(30.0, 24.0));
    }
    stroke.add_point(&mut target, PointD::new(34.0, 24.0));
    assert!(target.pixel(30, 24).r > 100, "lightened inside the selection");
    assert_eq!(target.pixel(40, 24), tiles.pixel(40, 24), "untouched outside the selection");
    let once = target.pixel(30, 24).r;
    stroke.add_point(&mut target, PointD::new(30.0, 24.0));
    assert_eq!(target.pixel(30, 24).r, once);
}
