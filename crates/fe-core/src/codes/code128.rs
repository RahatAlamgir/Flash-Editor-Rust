//! Code 128 (any text; only digits in pairs make the shorter set C).

use super::barcode::{BarcodeData, bars};

/// Widths of bar, space, bar, space, bar, space for symbol values 0..105, then the stop pattern (value 106, seven
/// elements).
pub const C128: [&str; 107] = [
    "212222", "222122", "222221", "121223", "121322", "131222", "122213", "122312", "132212", "221213", "221312",
    "231212", "112232", "122132", "122231", "113222", "123122", "123221", "223211", "221132", "221231", "213212",
    "223112", "312131", "311222", "321122", "321221", "312212", "322112", "322211", "212123", "212321", "232121",
    "111323", "131123", "131321", "112313", "132113", "132311", "211313", "231113", "231311", "112133", "112331",
    "132131", "113123", "113321", "133121", "313121", "211331", "231131", "213113", "213311", "213131", "311123",
    "311321", "331121", "312113", "312311", "332111", "314111", "221411", "431111", "111224", "111422", "121124",
    "121421", "141122", "141221", "112214", "112412", "122114", "122411", "142112", "142211", "241211", "221114",
    "413111", "241112", "134111", "111242", "121142", "121241", "114212", "124112", "124211", "411212", "421112",
    "421211", "212141", "214121", "412121", "111143", "111341", "131141", "114113", "114311", "411113", "411311",
    "113141", "114131", "311141", "411131", "211412", "211214", "211232", "2331112",
];

pub fn encode(content: &str) -> Result<BarcodeData, String> {
    if content.is_empty() {
        return Err("Type something for the barcode.".into());
    }
    if content.chars().count() > 80 {
        return Err("Code 128 is limited to 80 characters here.".into());
    }
    let mut values: Vec<usize> = Vec::new();
    let digits_only =
        content.chars().all(|c| c.is_ascii_digit()) && content.len() >= 4 && content.len().is_multiple_of(2);
    if digits_only {
        values.push(105); // start C: two digits per symbol
        let d: Vec<usize> = content.bytes().map(|b| usize::from(b - b'0')).collect();
        values.extend(d.chunks(2).map(|p| p[0] * 10 + p[1]));
    } else {
        values.push(104); // start B: the characters from space to ~
        for c in content.chars() {
            if !(' '..='~').contains(&c) {
                return Err(format!(
                    "Code 128 cannot show \"{c}\". Use letters without accents, digits and common marks."
                ));
            }
            values.push(c as usize - 32);
        }
    }
    let mut sum = values[0] as u64;
    for (i, v) in values.iter().enumerate().skip(1) {
        sum += *v as u64 * i as u64;
    }
    values.push((sum % 103) as usize);
    values.push(106); // stop
    let mut modules = Vec::new();
    for v in values {
        let mut bar = true;
        for w in C128[v].bytes() {
            bars(&mut modules, bar, usize::from(w - b'0'));
            bar = !bar;
        }
    }
    let center = modules.len() as f64 / 2.0;
    Ok(BarcodeData { modules, labels: vec![(content.to_owned(), center)], guards: Vec::new() })
}
