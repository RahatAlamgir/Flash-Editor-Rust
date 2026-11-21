//! QR codes (ISO 18004, byte mode, versions 1 to 40) from text. The result is a square grid of dark / light modules
//! without the empty border (the quiet zone is added when drawing).

use super::qr_grid::Builder;
use super::qr_tables::{block_count, data_codewords, ecc_per_block, raw_modules};

/// How much damage a QR code survives (more protection = a bigger, denser code). Numbered like the C# `QrLevel`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QrLevel {
    Low = 0,
    #[default]
    Medium = 1,
    Quartile = 2,
    High = 3,
}

impl QrLevel {
    pub const ALL: [QrLevel; 4] = [QrLevel::Low, QrLevel::Medium, QrLevel::Quartile, QrLevel::High];

    pub fn from_number(n: u64) -> Self {
        Self::ALL.get(n as usize).copied().unwrap_or_default()
    }

    pub fn name(self) -> &'static str {
        match self {
            QrLevel::Low => "Low (7 %)",
            QrLevel::Medium => "Medium (15 %)",
            QrLevel::Quartile => "Quartile (25 %)",
            QrLevel::High => "High (30 %)",
        }
    }
}

/// The text is too long for a QR code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TooLong;

impl std::fmt::Display for TooLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The text is too long for a QR code. Shorten it or lower the error correction.")
    }
}

/// The largest text (in bytes) that fits at the given level.
pub fn max_bytes(level: QrLevel) -> usize {
    data_codewords(40, level) - 3
}

/// The modules of the QR code for `text` (UTF-8), row by row; true = dark.
pub fn encode(text: &str, level: QrLevel) -> Result<Vec<Vec<bool>>, TooLong> {
    encode_with_mask(text, level, None)
}

/// Like [`encode`], with the mask pattern forced (tests and reproducible output).
pub fn encode_with_mask(text: &str, level: QrLevel, mask: Option<usize>) -> Result<Vec<Vec<bool>>, TooLong> {
    let data = text.as_bytes();
    let version = (1..=40usize)
        .find(|&v| {
            let count_bits = if v < 10 { 8 } else { 16 };
            data.len() < (1 << count_bits) && 4 + count_bits + 8 * data.len() <= data_codewords(v, level) * 8
        })
        .ok_or(TooLong)?;

    // Bits: mode (byte = 0100), length, the bytes, an end mark, then padding to the capacity.
    let mut bits: Vec<bool> = Vec::new();
    put(&mut bits, 0b0100, 4);
    put(&mut bits, data.len(), if version < 10 { 8 } else { 16 });
    for b in data {
        put(&mut bits, *b as usize, 8);
    }
    let capacity = data_codewords(version, level) * 8;
    let end = 4.min(capacity - bits.len());
    put(&mut bits, 0, end);
    let align = (8 - bits.len() % 8) % 8;
    put(&mut bits, 0, align);
    let mut pad = 0xEC;
    while bits.len() < capacity {
        put(&mut bits, pad, 8);
        pad ^= 0xEC ^ 0x11;
    }

    let mut codewords = vec![0u8; bits.len() / 8];
    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            codewords[i >> 3] |= 0x80 >> (i & 7);
        }
    }
    let all = add_error_correction(&codewords, version, level);
    Ok(Builder::new(version, level, all).build(mask))
}

fn put(bits: &mut Vec<bool>, value: usize, count: usize) {
    for i in (0..count).rev() {
        bits.push((value >> i) & 1 != 0);
    }
}

// ---------------------------------------------------------------- Reed-Solomon

fn add_error_correction(data: &[u8], version: usize, level: QrLevel) -> Vec<u8> {
    let blocks = block_count(version, level);
    let ecc_len = ecc_per_block(version, level);
    let raw = raw_modules(version) / 8;
    let short_blocks = blocks - raw % blocks;
    let short_len = raw / blocks;
    let divisor = divisor(ecc_len);

    let mut data_blocks: Vec<&[u8]> = Vec::new();
    let mut ecc_blocks: Vec<Vec<u8>> = Vec::new();
    let mut k = 0;
    for i in 0..blocks {
        let len = short_len - ecc_len + usize::from(i >= short_blocks);
        let block = &data[k..k + len];
        k += len;
        data_blocks.push(block);
        ecc_blocks.push(remainder(block, &divisor));
    }
    let mut result = Vec::with_capacity(raw);
    let longest = data_blocks.last().map_or(0, |b| b.len());
    for i in 0..longest {
        // The long blocks have one more data byte.
        result.extend(data_blocks.iter().filter_map(|b| b.get(i)));
    }
    for i in 0..ecc_len {
        result.extend(ecc_blocks.iter().map(|b| b[i]));
    }
    result
}

fn divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root = 1u8;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = multiply(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = multiply(root, 2);
    }
    result
}

fn remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for b in data {
        let factor = b ^ result[0];
        result.rotate_left(1);
        *result.last_mut().expect("the divisor is not empty") = 0;
        for (r, d) in result.iter_mut().zip(divisor) {
            *r ^= multiply(*d, factor);
        }
    }
    result
}

/// Multiplication in GF(2^8) with the polynomial x^8 + x^4 + x^3 + x^2 + 1.
fn multiply(x: u8, y: u8) -> u8 {
    let mut z: u32 = 0;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x11D);
        z ^= u32::from((y >> i) & 1) * u32::from(x);
    }
    z as u8
}
