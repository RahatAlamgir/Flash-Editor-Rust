//! Properties > Adjust (C# object `Tone`): brightness, contrast, saturation, hue, temperature and tint of the selected
//! object itself, without a dialog and without changing the picture inside it.

use egui::Ui;
use fe_core::document::AdjustmentStyle;

use super::shape::slider;
use crate::app::FlashApp;
use crate::commands::{objects::change_object, parts};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(tone) = parts::target_object(app).map(|o| o.tone().unwrap_or_default()) else { return };
    type Set = fn(&mut AdjustmentStyle, f64);
    let rows: [(&str, f64, f64, f64, &'static str, Set); 6] = [
        ("Brightness", tone.brightness, -100.0, 100.0, "Object Brightness", |s, v| s.brightness = v),
        ("Contrast", tone.contrast, -100.0, 100.0, "Object Contrast", |s, v| s.contrast = v),
        ("Saturation", tone.saturation, -100.0, 100.0, "Object Saturation", |s, v| s.saturation = v),
        ("Hue", tone.hue, -180.0, 180.0, "Object Hue", |s, v| s.hue = v),
        ("Warmth", tone.temperature, -100.0, 100.0, "Object Temperature", |s, v| s.temperature = v),
        ("Tint", tone.tint, -100.0, 100.0, "Object Tint", |s, v| s.tint = v),
    ];
    for (label, value, min, max, what, set) in rows {
        let mut v = value;
        if slider(ui, label, &mut v, min..=max, "") {
            let v = v.round();
            change_object(app, what, |o| {
                let mut t = o.tone().unwrap_or_default();
                set(&mut t, v);
                o.set_tone(Some(&t));
            });
        }
    }
    if tone != AdjustmentStyle::default() && ui.button("Reset").clicked() {
        change_object(app, "Reset Adjust", |o| o.set_tone(None));
    }
}
