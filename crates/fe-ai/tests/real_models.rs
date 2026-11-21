//! Tests that run the real AI models. They only run when the models and the runtime DLL are there (the folder of the C#
//! installer build is tried; or set FLASH_MODELS_DIR and FLASH_ORT_DLL); otherwise each test says it was skipped.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use fe_ai::models::{ISNET, LAMA, MOBILE_SAM_DECODER, MOBILE_SAM_ENCODER, ModelStore, REAL_ESRGAN};
use fe_core::imaging::{ColorRgba, PixelBuffer};

const DEFAULT_MODELS: &str = r"D:\AiBuild\Flash Editor\installer\build\models";
const DEFAULT_DLL: &str = r"D:\AiBuild\Flash Editor\installer\build\app\onnxruntime.dll";

#[allow(unsafe_code)]
fn setup() -> Option<ModelStore> {
    let models = std::env::var("FLASH_MODELS_DIR").unwrap_or_else(|_| DEFAULT_MODELS.into());
    if std::env::var_os("FLASH_ORT_DLL").is_none() {
        // The tests share one process: setting it before any model is opened is safe.
        unsafe { std::env::set_var("FLASH_ORT_DLL", DEFAULT_DLL) };
    }
    let dll_ok = std::env::var("FLASH_ORT_DLL").is_ok_and(|p| PathBuf::from(p).exists());
    let store = ModelStore::new(Some(PathBuf::from(models)));
    if !dll_ok {
        eprintln!("skipped: no onnxruntime.dll");
        return None;
    }
    Some(store)
}

fn photo() -> PixelBuffer {
    let path = format!("{}/../../assets/tutorial/TutorialPhoto.jpg", env!("CARGO_MANIFEST_DIR"));
    fe_core::io::decode::load(path).expect("the tutorial photo")
}

#[test]
fn the_subject_finder_keeps_the_person_and_drops_the_corner() {
    let Some(store) = setup() else { return };
    if !store.is_installed(&ISNET) {
        eprintln!("skipped: model missing");
        return;
    }
    let image = photo();
    let seg = fe_ai::segmentation::SubjectSegmenter::new(&ISNET, store.path_of(&ISNET));
    let mask = seg.predict_mask(&image, &AtomicBool::new(false)).expect("a mask");
    let (w, h) = (image.width() as usize, image.height() as usize);
    assert_eq!(mask.len(), w * h);
    let at = |x: usize, y: usize| mask[y * w + x];
    assert!(at(w / 2, h * 3 / 5) > 0.8, "the person is kept: {}", at(w / 2, h * 3 / 5));
    assert!(at(2, 2) < 0.3, "the corner goes: {}", at(2, 2));
    println!("device: {}", seg.device_name());
}

#[test]
fn click_select_outlines_the_object_under_the_click() {
    let Some(store) = setup() else { return };
    if !store.is_installed(&MOBILE_SAM_ENCODER) || !store.is_installed(&MOBILE_SAM_DECODER) {
        eprintln!("skipped: model missing");
        return;
    }
    let image = photo();
    let (w, h) = (image.width() as f64, image.height() as f64);
    let seg = fe_ai::segmentation::ClickSegmenter::new(
        store.path_of(&MOBILE_SAM_ENCODER),
        store.path_of(&MOBILE_SAM_DECODER),
    );
    let prompt = fe_ai::segmentation::ClickPrompt { points: vec![(w / 2.0, h * 0.75, true)], boxed: None };
    let mask = seg.predict(&image, &prompt, &AtomicBool::new(false)).expect("a mask");
    let (wi, hi) = (image.width() as usize, image.height() as usize);
    assert_eq!(mask.len(), wi * hi);
    assert_eq!(mask[(hi * 3 / 4) * wi + wi / 2], 255, "the clicked place is selected");
    let selected = mask.iter().filter(|m| **m == 255).count();
    // The C# version selects 340 099 pixels for this click on this picture.
    assert!((300_000..380_000).contains(&selected), "about what C# selects: {selected}");
}

#[test]
fn lama_fills_a_hole_with_the_colors_around_it() {
    let Some(store) = setup() else { return };
    if !store.is_installed(&LAMA) {
        eprintln!("skipped: model missing");
        return;
    }
    // A plain blue picture with a red square: removing the square must give blue again.
    let mut image = PixelBuffer::filled(300, 300, ColorRgba::rgb(40, 90, 200));
    let mut mask = vec![0u8; 300 * 300];
    for y in 120..180u32 {
        for x in 120..180u32 {
            image.set_pixel(x, y, ColorRgba::rgb(220, 30, 30));
            mask[(y * 300 + x) as usize] = 255;
        }
    }
    let inp = fe_ai::inpainting::Inpainter::new(&LAMA, store.path_of(&LAMA));
    let filled = inp.fill(&image, &mask, &|_| {}, &AtomicBool::new(false)).expect("ran").expect("something filled");
    let p = filled.pixel(150, 150);
    assert!(p.a > 200 && p.b > 120 && p.r < 120, "blue again: {p:?}");
    assert_eq!(filled.pixel(10, 10).a, 0, "nothing outside the hole");
}

#[test]
fn the_upscaler_makes_the_picture_bigger_and_keeps_the_colors() {
    let Some(store) = setup() else { return };
    if !store.is_installed(&REAL_ESRGAN) {
        eprintln!("skipped: model missing");
        return;
    }
    let image = PixelBuffer::filled(64, 48, ColorRgba::rgb(200, 100, 50));
    let up = fe_ai::upscaling::AiUpscaler::new(&REAL_ESRGAN, store.path_of(&REAL_ESRGAN));
    let out = up.upscale(&image, 2.0, &|_| {}, &AtomicBool::new(false)).expect("ran");
    assert_eq!((out.width(), out.height()), (128, 96));
    let p = out.pixel(64, 48);
    assert!(
        (i32::from(p.r) - 200).abs() < 25 && (i32::from(p.g) - 100).abs() < 25 && (i32::from(p.b) - 50).abs() < 25,
        "{p:?}"
    );
}

#[test]
fn background_removal_keeps_the_person_and_makes_the_corner_see_through() {
    let Some(store) = setup() else { return };
    if !store.is_installed(&ISNET) {
        eprintln!("skipped: model missing");
        return;
    }
    let image = photo();
    let seg = fe_ai::segmentation::SubjectSegmenter::new(&ISNET, store.path_of(&ISNET));
    let cut = fe_ai::effects::remove_background(&seg, &image, 0.3, 0.0, 0.0, &AtomicBool::new(false)).expect("ran");
    let (w, h) = (cut.width(), cut.height());
    assert_eq!((w, h), (image.width(), image.height()));
    assert!(cut.pixel(w / 2, h * 3 / 5).a > 200, "the person stays");
    assert!(cut.pixel(2, 2).a < 80, "the corner goes");
    let gone = cut.data().iter().skip(3).step_by(4).filter(|a| **a < 128).count();
    assert!(gone > 1000, "something was removed: {gone}");
}
