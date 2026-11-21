//! Barcodes: Code 128 (any text), EAN-13, UPC-A and EAN-8 (shop codes, digits with a check digit) and Code 39.

use super::{code39, code128};

/// The kinds of barcode, numbered like the C# `BarcodeKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarcodeKind {
    #[default]
    Code128 = 0,
    Ean13 = 1,
    UpcA = 2,
    Ean8 = 3,
    Code39 = 4,
}

impl BarcodeKind {
    pub const ALL: [BarcodeKind; 5] =
        [BarcodeKind::Code128, BarcodeKind::Ean13, BarcodeKind::UpcA, BarcodeKind::Ean8, BarcodeKind::Code39];

    pub fn from_number(n: u64) -> Self {
        Self::ALL.get(n as usize).copied().unwrap_or_default()
    }

    pub fn name(self) -> &'static str {
        match self {
            BarcodeKind::Code128 => "Code 128",
            BarcodeKind::Ean13 => "EAN-13",
            BarcodeKind::UpcA => "UPC-A",
            BarcodeKind::Ean8 => "EAN-8",
            BarcodeKind::Code39 => "Code 39",
        }
    }

    /// What the kind accepts, in plain words (shown under the text box).
    pub fn describe(self) -> &'static str {
        match self {
            BarcodeKind::Ean13 => "12 digits (the 13th, the check digit, is added) or all 13 digits.",
            BarcodeKind::UpcA => "11 digits (the 12th, the check digit, is added) or all 12 digits.",
            BarcodeKind::Ean8 => "7 digits (the 8th, the check digit, is added) or all 8 digits.",
            BarcodeKind::Code39 => "Capital letters, digits, space and - . $ / + %",
            BarcodeKind::Code128 => {
                "Any text with letters, digits and common marks (no accents). Only digits make a shorter code."
            }
        }
    }
}

/// A barcode as a row of modules (true = a dark bar) and the numbers or letters printed under it.
#[derive(Clone, Debug, PartialEq)]
pub struct BarcodeData {
    pub modules: Vec<bool>,
    /// Printed text with the middle of each piece, counted in modules from the left end of the bars (can be
    /// negative: outside the bars).
    pub labels: Vec<(String, f64)>,
    /// Modules of the long guard bars of EAN / UPC codes (they reach down into the text), `(from, to)`.
    pub guards: Vec<(usize, usize)>,
}

impl BarcodeData {
    pub fn width(&self) -> usize {
        self.modules.len()
    }
}

/// Adds `count` modules, dark or light.
pub(super) fn bars(modules: &mut Vec<bool>, dark: bool, count: usize) {
    modules.extend(std::iter::repeat_n(dark, count));
}

/// The barcode for `content`. The error says in plain words what does not fit the kind.
pub fn encode(content: &str, kind: BarcodeKind) -> Result<BarcodeData, String> {
    match kind {
        BarcodeKind::Ean13 => ean_upc(content, 13, false),
        BarcodeKind::UpcA => ean_upc(content, 12, true),
        BarcodeKind::Ean8 => ean8(content),
        BarcodeKind::Code39 => code39::encode(content),
        BarcodeKind::Code128 => code128::encode(content),
    }
}

/// `None` when `content` can be made into this kind of barcode, otherwise what is wrong.
pub fn validate(content: &str, kind: BarcodeKind) -> Option<String> {
    encode(content, kind).err()
}

/// The Code 128 symbol patterns (for the tests).
pub fn code128_patterns() -> &'static [&'static str] {
    &code128::C128
}

/// The Code 39 characters (without the start / stop mark).
pub fn code39_characters() -> Vec<char> {
    code39::characters()
}

const L: [&str; 10] =
    ["0001101", "0011001", "0010011", "0111101", "0100011", "0110001", "0101111", "0111011", "0110111", "0001011"];
const G: [&str; 10] =
    ["0100111", "0110011", "0011011", "0100001", "0011101", "0111001", "0000101", "0010001", "0001001", "0010111"];
const R: [&str; 10] =
    ["1110010", "1100110", "1101100", "1000010", "1011100", "1001110", "1010000", "1000100", "1001000", "1110100"];
const PARITY: [&str; 10] =
    ["LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG", "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL"];

/// The check digit for the digits before it (EAN / UPC: weights 3 and 1 counted from the right).
pub fn check_digit(digits: &str) -> u32 {
    let sum: u32 = digits
        .bytes()
        .rev()
        .enumerate()
        .map(|(i, b)| {
            let d = u32::from(b - b'0');
            if i % 2 == 0 { d * 3 } else { d }
        })
        .sum();
    (10 - sum % 10) % 10
}

fn digits(content: &str, length: usize, name: &str) -> Result<String, String> {
    let text: String = content.trim().replace(' ', "");
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{name} needs digits only."));
    }
    if text.len() != length && text.len() != length - 1 {
        return Err(format!("{name} needs {} or {length} digits (you typed {}).", length - 1, text.len()));
    }
    let body = &text[..length - 1];
    let check = check_digit(body);
    if text.len() == length && u32::from(text.as_bytes()[length - 1] - b'0') != check {
        let typed = &text[length - 1..];
        return Err(format!(
            "The check digit of {name} should be {check}, not {typed}. Leave it out and it is added for you."
        ));
    }
    Ok(format!("{body}{check}"))
}

fn add(modules: &mut Vec<bool>, pattern: &str) {
    modules.extend(pattern.bytes().map(|c| c == b'1'));
}

fn digit(s: &str, i: usize) -> usize {
    usize::from(s.as_bytes()[i] - b'0')
}

fn ean_upc(content: &str, length: usize, upc: bool) -> Result<BarcodeData, String> {
    let full = digits(content, length, if upc { "UPC-A" } else { "EAN-13" })?;
    let ean = if upc { format!("0{full}") } else { full.clone() }; // a UPC-A is an EAN-13 with a 0 in front
    let mut modules = Vec::new();
    add(&mut modules, "101");
    let parity = PARITY[digit(&ean, 0)].as_bytes();
    for i in 0..6 {
        let table = if parity[i] == b'L' { &L } else { &G };
        add(&mut modules, table[digit(&ean, 1 + i)]);
    }
    add(&mut modules, "01010");
    for i in 0..6 {
        add(&mut modules, R[digit(&ean, 7 + i)]);
    }
    add(&mut modules, "101");
    let labels: Vec<(String, f64)> = if upc {
        vec![
            (full[..1].into(), -5.0),
            (full[1..6].into(), 27.5),
            (full[6..11].into(), 67.5),
            (full[11..].into(), 100.0),
        ]
    } else {
        vec![(ean[..1].into(), -5.0), (ean[1..7].into(), 24.0), (ean[7..].into(), 71.0)]
    };
    Ok(BarcodeData { modules, labels, guards: vec![(0, 3), (45, 50), (92, 95)] })
}

fn ean8(content: &str) -> Result<BarcodeData, String> {
    let full = digits(content, 8, "EAN-8")?;
    let mut modules = Vec::new();
    add(&mut modules, "101");
    for i in 0..4 {
        add(&mut modules, L[digit(&full, i)]);
    }
    add(&mut modules, "01010");
    for i in 4..8 {
        add(&mut modules, R[digit(&full, i)]);
    }
    add(&mut modules, "101");
    Ok(BarcodeData {
        modules,
        labels: vec![(full[..4].into(), 17.0), (full[4..].into(), 50.0)],
        guards: vec![(0, 3), (31, 36), (64, 67)],
    })
}
