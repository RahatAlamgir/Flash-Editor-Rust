//! The effects and resizing give the same pictures as the C# version. The reference pictures in
//! `tests/fixtures/effects` were made by the C# code from the test picture below.

use fe_core::effects::{self, Effect, Values};
use fe_core::imaging::{ColorRgba, PixelBuffer, PointD};
use fe_core::transform::{self, ResampleMode};

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

/// How many values differ by more than `tolerance`, and the biggest difference.
fn compare(name: &str, got: &PixelBuffer, tolerance: i32) -> (usize, i32) {
    let want = reference(name);
    assert_eq!((got.width(), got.height()), (want.width(), want.height()), "{name}: size");
    let (mut bad, mut worst) = (0, 0);
    for (a, b) in got.data().iter().zip(want.data()) {
        let d = (i32::from(*a) - i32::from(*b)).abs();
        worst = worst.max(d);
        if d > tolerance {
            bad += 1;
        }
    }
    (bad, worst)
}

type Settings = Vec<(&'static str, f64)>;

fn run(e: &Effect, set: &[(&'static str, f64)]) -> PixelBuffer {
    let mut v: Values = e.defaults();
    for (k, x) in set {
        v.set(k, *x);
    }
    e.apply(&source(), &v)
}

#[test]
fn the_adjustments_match_csharp() {
    let cases: Vec<(&str, &Effect, Settings)> = vec![
        ("auto_levels", &effects::AUTO_LEVELS, vec![]),
        ("brightness_contrast", &effects::BRIGHTNESS_CONTRAST, vec![("brightness", 30.0), ("contrast", 25.0)]),
        ("exposure", &effects::EXPOSURE_GAMMA, vec![("exposure", 0.7), ("gamma", 1.4)]),
        ("hue_sat", &effects::HUE_SATURATION, vec![("hue", 50.0), ("saturation", 30.0), ("lightness", -20.0)]),
        ("vibrance", &effects::VIBRANCE, vec![]),
        ("temperature", &effects::TEMPERATURE_TINT, vec![("temperature", 40.0), ("tint", -25.0)]),
        ("grayscale", &effects::GRAYSCALE, vec![]),
        ("sepia", &effects::SEPIA, vec![]),
        ("invert", &effects::INVERT, vec![]),
        ("threshold", &effects::THRESHOLD, vec![]),
        ("posterize", &effects::POSTERIZE, vec![]),
        (
            "recolor",
            &effects::RECOLOR,
            vec![("hue", 200.0), ("sat", 70.0), ("light", 40.0), ("refL", 0.45), ("refS", 0.4)],
        ),
    ];
    for (name, effect, set) in cases {
        let (bad, worst) = compare(name, &run(effect, &set), 1);
        assert!(bad == 0, "{name}: {bad} values differ by more than 1 (worst {worst})");
    }
}

#[test]
fn the_filters_match_csharp() {
    let cases: Vec<(&str, &Effect, Settings, i32)> = vec![
        ("blur", &effects::GAUSSIAN_BLUR, vec![("radius", 3.0)], 1),
        ("motion", &effects::MOTION_BLUR, vec![("angle", 30.0), ("distance", 9.0)], 1),
        ("sharpen", &effects::SHARPEN, vec![], 1),
        ("enhance", &effects::ENHANCE, vec![], 2),
        ("reduce_noise", &effects::REDUCE_NOISE, vec![], 1),
        ("pixelate", &effects::PIXELATE, vec![("size", 5.0)], 1),
        ("vignette", &effects::VIGNETTE, vec![], 1),
        ("emboss", &effects::EMBOSS, vec![], 1),
        ("emboss_color", &effects::EMBOSS, vec![("color", 1.0)], 1),
        ("edges", &effects::FIND_EDGES, vec![], 1),
    ];
    for (name, effect, set, tolerance) in cases {
        let (bad, worst) = compare(name, &run(effect, &set), tolerance);
        assert!(bad == 0, "{name}: {bad} values differ by more than {tolerance} (worst {worst})");
    }
}

#[test]
fn resizing_matches_csharp() {
    for (label, mode) in [
        ("Smooth", ResampleMode::Smooth),
        ("Bicubic", ResampleMode::Bicubic),
        ("Lanczos", ResampleMode::Lanczos),
        ("NearestNeighbor", ResampleMode::NearestNeighbor),
    ] {
        let up = transform::resize(&source(), 100, 70, mode);
        let down = transform::resize(&source(), 25, 17, mode);
        for (name, img) in [(format!("resize_up_{label}"), up), (format!("resize_down_{label}"), down)] {
            let (bad, worst) = compare(&name, &img, 1);
            assert!(bad == 0, "{name}: {bad} values differ (worst {worst})");
        }
    }
}

#[test]
fn rotating_straightening_and_perspective_match_csharp() {
    let s = source();
    for (name, img) in [
        ("rotate30", transform::rotate(&s, 30.0)),
        ("rotate90", transform::rotate(&s, 90.0)),
        ("straighten", transform::straighten(&s, 9.0, true)),
        (
            "persp",
            transform::perspective_crop(
                &s,
                &[PointD::new(5.0, 4.0), PointD::new(58.0, 8.0), PointD::new(52.0, 44.0), PointD::new(8.0, 38.0)],
                40,
                30,
            )
            .expect("usable quad"),
        ),
    ] {
        let (bad, worst) = compare(name, &img, 1);
        assert!(bad == 0, "{name}: {bad} values differ (worst {worst})");
    }
}
