//! The 16 ready-made text designs (C# `TextPreset`): font, color or gradient and effects in one click. They never
//! change the words, the place, the rotation or the alignment. The effects are made for text of about 64 px and
//! are scaled to the size of the text they go on.

use crate::imaging::ColorRgba;
use crate::objects::{
    GlowStyle, Gradient, InnerShadowStyle, Looks, OutlineStyle, ShadowStyle, StoredObject, ThreeDStyle,
};

pub struct TextPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub font: &'static str,
    pub color: ColorRgba,
    pub bold: bool,
    pub italic: bool,
    pub gradient: Option<(&'static [ColorRgba], f64)>,
    pub shadow: Option<ShadowStyle>,
    pub glow: Option<GlowStyle>,
    pub inner_shadow: Option<InnerShadowStyle>,
    pub outline: Option<OutlineStyle>,
    pub three_d: Option<ThreeDStyle>,
    pub letter_outline: f64,
    pub letter_outline_color: Option<ColorRgba>,
    pub preview_background: Option<ColorRgba>,
}

/// The size the effects were made for.
pub const REFERENCE_SIZE: f64 = 64.0;

const fn c(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::new(r, g, b, 255)
}

const fn ca(r: u8, g: u8, b: u8, a: u8) -> ColorRgba {
    ColorRgba::new(r, g, b, a)
}

const fn shadow(color: ColorRgba, distance: f64, angle: f64, blur: f64) -> Option<ShadowStyle> {
    Some(ShadowStyle { color, distance, angle, blur, size: 1.0 })
}

const fn glow(color: ColorRgba, size: f64, intensity: f64) -> Option<GlowStyle> {
    Some(GlowStyle { color, size, intensity })
}

const BASE: TextPreset = TextPreset {
    name: "",
    description: "",
    font: "Segoe UI",
    color: c(0, 0, 0),
    bold: false,
    italic: false,
    gradient: None,
    shadow: None,
    glow: None,
    inner_shadow: None,
    outline: None,
    three_d: None,
    letter_outline: 0.0,
    letter_outline_color: None,
    preview_background: None,
};

pub const TEXT_PRESETS: [TextPreset; 16] = [
    TextPreset { name: "Clean", description: "Plain dark title, no effects", color: c(31, 41, 55), bold: true, ..BASE },
    TextPreset {
        name: "Impact",
        description: "Big white letters with a black edge, like a meme",
        font: "Impact",
        color: c(255, 255, 255),
        letter_outline: 4.0,
        letter_outline_color: Some(c(0, 0, 0)),
        preview_background: Some(c(110, 120, 140)),
        ..BASE
    },
    TextPreset {
        name: "Neon Pink",
        description: "Glowing pink neon sign",
        font: "Bahnschrift",
        color: c(255, 190, 230),
        bold: true,
        glow: glow(c(255, 40, 170), 18.0, 140.0),
        preview_background: Some(c(24, 18, 40)),
        ..BASE
    },
    TextPreset {
        name: "Neon Blue",
        description: "Glowing cyan neon sign",
        font: "Bahnschrift",
        color: c(200, 250, 255),
        bold: true,
        glow: glow(c(0, 200, 255), 18.0, 140.0),
        preview_background: Some(c(14, 20, 40)),
        ..BASE
    },
    TextPreset {
        name: "Retro 3D",
        description: "Chunky yellow letters with a red side",
        font: "Arial Black",
        color: c(255, 214, 0),
        three_d: Some(ThreeDStyle { depth: 9.0, angle: 45.0, color: Some(c(214, 62, 40)), bevel: 0.3 }),
        shadow: shadow(ca(0, 0, 0, 90), 14.0, 45.0, 6.0),
        ..BASE
    },
    TextPreset {
        name: "Sticker",
        description: "White letters, thick dark edge and a hard shadow",
        font: "Arial Black",
        color: c(255, 255, 255),
        outline: Some(OutlineStyle { color: c(30, 41, 90), width: 7.0 }),
        shadow: shadow(c(0, 0, 0), 6.0, 45.0, 0.0),
        preview_background: Some(c(255, 200, 80)),
        ..BASE
    },
    TextPreset {
        name: "Sunset",
        description: "Pink to orange fade",
        font: "Segoe UI Black",
        color: c(255, 120, 100),
        gradient: Some((&[c(255, 70, 110), c(255, 140, 70), c(255, 205, 100)], 0.0)),
        shadow: shadow(ca(120, 30, 40, 70), 5.0, 90.0, 10.0),
        ..BASE
    },
    TextPreset {
        name: "Ocean",
        description: "Cool blue fade",
        font: "Segoe UI Black",
        color: c(0, 150, 255),
        gradient: Some((&[c(0, 220, 255), c(30, 80, 230)], 90.0)),
        shadow: shadow(ca(10, 30, 90, 70), 5.0, 90.0, 10.0),
        ..BASE
    },
    TextPreset {
        name: "Gold",
        description: "Shiny gold lettering",
        font: "Georgia",
        color: c(212, 160, 40),
        bold: true,
        gradient: Some((&[c(255, 240, 170), c(214, 160, 40), c(255, 235, 150), c(176, 120, 20)], 90.0)),
        shadow: shadow(ca(0, 0, 0, 110), 4.0, 90.0, 6.0),
        preview_background: Some(c(40, 34, 30)),
        ..BASE
    },
    TextPreset {
        name: "Fire",
        description: "Flames from yellow to red with an orange glow",
        font: "Impact",
        color: c(255, 120, 0),
        gradient: Some((&[c(255, 245, 120), c(255, 140, 0), c(200, 20, 0)], 90.0)),
        glow: glow(ca(255, 90, 0, 200), 12.0, 110.0),
        preview_background: Some(c(30, 20, 20)),
        ..BASE
    },
    TextPreset {
        name: "Elegant",
        description: "Slim serif italics with a soft shadow",
        font: "Georgia",
        color: c(70, 45, 35),
        italic: true,
        shadow: shadow(ca(0, 0, 0, 55), 3.0, 90.0, 6.0),
        ..BASE
    },
    TextPreset {
        name: "Script",
        description: "Flowing script handwriting",
        font: "Segoe Script",
        color: c(40, 40, 60),
        bold: true,
        ..BASE
    },
    TextPreset {
        name: "Comic",
        description: "Yellow comic letters with a black edge",
        font: "Comic Sans MS",
        color: c(255, 232, 40),
        bold: true,
        letter_outline: 3.5,
        letter_outline_color: Some(c(0, 0, 0)),
        shadow: shadow(c(0, 0, 0), 5.0, 45.0, 0.0),
        preview_background: Some(c(80, 110, 200)),
        ..BASE
    },
    TextPreset {
        name: "Long Shadow",
        description: "White letters with a long, flat shadow",
        font: "Segoe UI Black",
        color: c(255, 255, 255),
        shadow: shadow(ca(20, 30, 60, 110), 38.0, 45.0, 0.0),
        preview_background: Some(c(90, 130, 230)),
        ..BASE
    },
    TextPreset {
        name: "Letterpress",
        description: "Pressed into the paper",
        color: c(150, 150, 156),
        bold: true,
        inner_shadow: Some(InnerShadowStyle { color: ca(0, 0, 0, 160), distance: 3.0, angle: 90.0, blur: 3.0 }),
        shadow: shadow(ca(255, 255, 255, 230), 2.0, 90.0, 0.0),
        ..BASE
    },
    TextPreset {
        name: "Terminal",
        description: "Green computer-screen text",
        font: "Consolas",
        color: c(90, 255, 140),
        bold: true,
        glow: glow(ca(40, 255, 110, 190), 8.0, 100.0),
        preview_background: Some(c(10, 20, 14)),
        ..BASE
    },
];

impl TextPreset {
    /// The text object with this design (words, box, alignment and flips stay as they were).
    pub fn apply(&self, o: &mut StoredObject) {
        let Some(mut t) = o.text() else { return };
        let k = (t.size / REFERENCE_SIZE).max(0.1);
        t.family = self.font.to_owned();
        t.color = self.color;
        t.bold = self.bold;
        t.italic = self.italic;
        t.outline_width = self.letter_outline * k;
        t.outline_color = (self.letter_outline > 0.0).then_some(self.letter_outline_color.unwrap_or(ColorRgba::BLACK));
        o.set_text(&t);
        let gradient = self.gradient.map(|(colors, angle)| Gradient::evenly(colors, angle, false));
        o.set_gradient(gradient.as_ref());
        let blur = o.looks().blur;
        let looks = Looks {
            shadow: self.shadow,
            glow: self.glow,
            inner_shadow: self.inner_shadow,
            outline: self.outline,
            three_d: self.three_d,
            blur,
        }
        .scaled(k);
        o.set_looks(&looks);
        crate::text::refit(o, &t);
    }
}
