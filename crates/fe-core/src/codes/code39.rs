//! Code 39 (capital letters, digits and a few marks).

use super::barcode::{BarcodeData, bars};

/// Each character: nine elements (bar, space, bar...) where `w` is wide (3 modules) and `n` narrow (1 module).
const C39: [(char, &str); 44] = [
    ('0', "nnnwwnwnn"),
    ('1', "wnnwnnnnw"),
    ('2', "nnwwnnnnw"),
    ('3', "wnwwnnnnn"),
    ('4', "nnnwwnnnw"),
    ('5', "wnnwwnnnn"),
    ('6', "nnwwwnnnn"),
    ('7', "nnnwnnwnw"),
    ('8', "wnnwnnwnn"),
    ('9', "nnwwnnwnn"),
    ('A', "wnnnnwnnw"),
    ('B', "nnwnnwnnw"),
    ('C', "wnwnnwnnn"),
    ('D', "nnnnwwnnw"),
    ('E', "wnnnwwnnn"),
    ('F', "nnwnwwnnn"),
    ('G', "nnnnnwwnw"),
    ('H', "wnnnnwwnn"),
    ('I', "nnwnnwwnn"),
    ('J', "nnnnwwwnn"),
    ('K', "wnnnnnnww"),
    ('L', "nnwnnnnww"),
    ('M', "wnwnnnnwn"),
    ('N', "nnnnwnnww"),
    ('O', "wnnnwnnwn"),
    ('P', "nnwnwnnwn"),
    ('Q', "nnnnnnwww"),
    ('R', "wnnnnnwwn"),
    ('S', "nnwnnnwwn"),
    ('T', "nnnnwnwwn"),
    ('U', "wwnnnnnnw"),
    ('V', "nwwnnnnnw"),
    ('W', "wwwnnnnnn"),
    ('X', "nwnnwnnnw"),
    ('Y', "wwnnwnnnn"),
    ('Z', "nwwnwnnnn"),
    ('-', "nwnnnnwnw"),
    ('.', "wwnnnnwnn"),
    (' ', "nwwnnnwnn"),
    ('$', "nwnwnwnnn"),
    ('/', "nwnwnnnwn"),
    ('+', "nwnnnwnwn"),
    ('%', "nnnwnwnwn"),
    ('*', "nwnnwnwnn"),
];

fn pattern(c: char) -> Option<&'static str> {
    C39.iter().find(|(k, _)| *k == c).map(|(_, p)| *p)
}

/// The Code 39 characters (without the start / stop mark).
pub fn characters() -> Vec<char> {
    C39.iter().map(|(c, _)| *c).filter(|c| *c != '*').collect()
}

pub fn encode(content: &str) -> Result<BarcodeData, String> {
    let text = content.to_uppercase();
    if text.is_empty() {
        return Err("Type something for the barcode.".into());
    }
    for c in text.chars() {
        if c == '*' || pattern(c).is_none() {
            return Err(format!("Code 39 cannot show \"{c}\". Use capital letters, digits, space and - . $ / + %"));
        }
    }
    if text.chars().count() > 40 {
        return Err("Code 39 is limited to 40 characters here.".into());
    }
    let all: Vec<char> = std::iter::once('*').chain(text.chars()).chain(std::iter::once('*')).collect();
    let mut modules = Vec::new();
    for (ch, c) in all.iter().enumerate() {
        let p = pattern(*c).unwrap_or("nnnnnnnnn");
        for (i, w) in p.bytes().enumerate() {
            bars(&mut modules, i % 2 == 0, if w == b'w' { 3 } else { 1 });
        }
        if ch + 1 < all.len() {
            modules.push(false); // the gap between two characters
        }
    }
    let center = modules.len() as f64 / 2.0;
    Ok(BarcodeData { modules, labels: vec![(text, center)], guards: Vec::new() })
}
