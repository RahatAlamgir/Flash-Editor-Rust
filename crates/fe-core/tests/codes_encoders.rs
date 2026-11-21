//! QR and barcode encoders (ports of the C# `QrEncoderTests` and `BarcodeEncoderTests`).

use fe_core::codes::barcode::{self, BarcodeKind, check_digit, code39_characters, code128_patterns, validate};
use fe_core::codes::{QrLevel, encode_qr, max_bytes};

#[test]
fn the_largest_text_matches_the_standard() {
    for (level, bytes) in
        [(QrLevel::Low, 2953), (QrLevel::Medium, 2331), (QrLevel::Quartile, 1663), (QrLevel::High, 1273)]
    {
        assert_eq!(max_bytes(level), bytes);
    }
}

#[test]
fn a_short_text_makes_a_version_1_code_and_longer_texts_make_bigger_ones() {
    assert_eq!(encode_qr("HELLO", QrLevel::Low).unwrap().len(), 21);
    let a = encode_qr(&"x".repeat(40), QrLevel::Medium).unwrap().len();
    let b = encode_qr(&"x".repeat(400), QrLevel::Medium).unwrap().len();
    assert!(b > a);
    assert_eq!((a - 17) % 4, 0);
}

#[test]
fn the_three_finder_patterns_and_the_timing_lines_are_in_place() {
    let m = encode_qr("https://example.com", QrLevel::Quartile).unwrap();
    let n = m.len();
    for (ox, oy) in [(0, 0), (n - 7, 0), (0, n - 7)] {
        for i in 0..7 {
            assert!(m[oy][ox + i] && m[oy + 6][ox + i]); // outer ring
            assert!(m[oy + i][ox] && m[oy + i][ox + 6]);
        }
        assert!(m[oy + 3][ox + 3]); // middle
        assert!(!m[oy + 1][ox + 1]); // light ring
    }
    for (i, module) in m[6].iter().enumerate().take(n - 8).skip(8) {
        assert_eq!(*module, i % 2 == 0); // timing
    }
}

#[test]
fn the_same_text_always_gives_the_same_code_and_different_levels_differ() {
    let a = encode_qr("flash", QrLevel::Medium).unwrap();
    assert_eq!(a, encode_qr("flash", QrLevel::Medium).unwrap());
    assert_ne!(a, encode_qr("flash", QrLevel::High).unwrap());
}

#[test]
fn a_text_that_is_too_long_is_refused() {
    assert!(encode_qr(&"a".repeat(max_bytes(QrLevel::Low) + 1), QrLevel::Low).is_err());
    assert!(encode_qr(&"a".repeat(max_bytes(QrLevel::Low)), QrLevel::Low).is_ok());
}

#[test]
fn letters_outside_ascii_are_stored_as_utf8() {
    let a = encode_qr(&"é€".repeat(20), QrLevel::Low).unwrap(); // 100 bytes in 40 letters
    let b = encode_qr(&"e".repeat(40), QrLevel::Low).unwrap();
    assert!(a.len() > b.len());
}

#[test]
fn known_ean13_numbers_have_the_right_check_digit() {
    for (number, check) in [("5901234123457", 7), ("4006381333931", 1), ("9780201379624", 4)] {
        assert_eq!(check_digit(&number[..12]), check);
    }
}

#[test]
fn an_ean13_is_95_modules_wide_with_guards_and_a_check_digit_added() {
    let d = barcode::encode("590123412345", BarcodeKind::Ean13).unwrap();
    assert_eq!(d.width(), 95);
    assert!(d.modules[0] && !d.modules[1] && d.modules[2]);
    assert!(!d.modules[45] && d.modules[46] && !d.modules[47] && d.modules[48] && !d.modules[49]);
    assert!(d.modules[92] && !d.modules[93] && d.modules[94]);
    assert_eq!(d.labels[0].0, "5");
    assert_eq!(d.labels[2].0, "123457");
}

#[test]
fn a_wrong_check_digit_is_refused_with_the_right_one_named() {
    let message = validate("5901234123458", BarcodeKind::Ean13).expect("refused");
    assert!(message.contains('7'));
    assert!(validate("5901234123457", BarcodeKind::Ean13).is_none());
}

#[test]
fn the_number_of_digits_and_the_kind_of_characters_are_checked() {
    assert!(validate("12345", BarcodeKind::Ean13).is_some());
    assert!(validate("12345678901A", BarcodeKind::Ean13).is_some());
    assert!(validate("", BarcodeKind::Code128).is_some());
    assert!(validate("café", BarcodeKind::Code128).is_some());
    assert!(validate("abc*", BarcodeKind::Code39).is_some());
    assert!(validate("036000291452", BarcodeKind::UpcA).is_none());
    assert!(validate("9638507", BarcodeKind::Ean8).is_none());
}

#[test]
fn a_upc_a_is_an_ean13_with_a_zero_in_front_and_shows_twelve_digits() {
    let upc = barcode::encode("03600029145", BarcodeKind::UpcA).unwrap();
    let ean = barcode::encode("003600029145", BarcodeKind::Ean13).unwrap();
    assert_eq!(upc.modules, ean.modules);
    let shown: String = upc.labels.iter().map(|l| l.0.as_str()).collect();
    assert_eq!(shown, "036000291452");
}

#[test]
fn every_code128_symbol_follows_the_rules_of_the_standard() {
    let patterns = code128_patterns();
    assert_eq!(patterns.len(), 107);
    let mut seen = std::collections::HashSet::new();
    for p in &patterns[..106] {
        let w: Vec<u32> = p.bytes().map(|b| u32::from(b - b'0')).collect();
        assert_eq!(w.len(), 6);
        assert_eq!(w.iter().sum::<u32>(), 11);
        assert_eq!((w[0] + w[2] + w[4]) % 2, 0);
        assert!(w.iter().all(|x| (1..=4).contains(x)));
        assert!(seen.insert(*p), "{p} repeats another");
    }
    assert_eq!(patterns[106].bytes().map(|b| u32::from(b - b'0')).sum::<u32>(), 13);
}

#[test]
fn code128_adds_a_check_symbol_and_a_stop_mark() {
    let d = barcode::encode("A", BarcodeKind::Code128).unwrap();
    assert_eq!(d.width(), 11 + 11 + 11 + 13);
    assert!(d.modules[0] && *d.modules.last().unwrap());
}

#[test]
fn only_digits_in_pairs_make_the_shorter_code128_c() {
    let digits = barcode::encode("1234567890", BarcodeKind::Code128).unwrap().width();
    let text = barcode::encode("12345678901", BarcodeKind::Code128).unwrap().width();
    assert!(digits < text);
    assert_eq!(digits, 11 * (1 + 5 + 1) + 13);
}

#[test]
fn every_code39_character_has_three_wide_elements_out_of_nine() {
    let chars = code39_characters();
    assert_eq!(chars.len(), 43);
    let mut patterns = std::collections::HashSet::new();
    for c in chars {
        let d = barcode::encode(&c.to_string(), BarcodeKind::Code39).unwrap();
        assert_eq!(d.width(), 3 * 15 + 2);
        assert!(patterns.insert(d.modules));
    }
}

#[test]
fn lowercase_letters_are_made_capital_for_code39() {
    let a = barcode::encode("HELLO 1", BarcodeKind::Code39).unwrap();
    assert_eq!(a, barcode::encode("hello 1", BarcodeKind::Code39).unwrap());
}
