//! Picture reading and writing in bands of rows, straight from / into tiles. Project files hold one cropped picture
//! per layer: lossless WebP (small), or PNG streamed in bands for very large ones (keeps memory low).
//! Old files hold PNG only; reading works out the kind from the first bytes.

use std::io::Write;

use crate::imaging::{PixelRect, TILE_SIZE, TileMap};
use crate::io::IoError;

/// Rows handled at once.
const BAND: u32 = TILE_SIZE as u32;

/// Pictures up to this many pixels are stored as WebP (needs the whole picture in memory); bigger ones as PNG.
const WEBP_LIMIT: u64 = 16_000_000;

/// True when a `width` x `height` picture is stored as lossless WebP.
pub fn use_webp(width: u32, height: u32) -> bool {
    u64::from(width) * u64::from(height) <= WEBP_LIMIT && width <= 16383 && height <= 16383
}

/// Lossless WebP, smaller than the default setting. (Trying even harder takes 10x longer for about 0.3% less.)
pub fn webp_lossless(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let encoder = webp::Encoder::from_rgba(rgba, width, height);
    if let Ok(mut config) = webp::WebPConfig::new() {
        config.lossless = 1;
        config.method = 4;
        config.quality = 90.0;
        if let Ok(data) = encoder.encode_advanced(&config) {
            return data.to_vec();
        }
    }
    encoder.encode_lossless().to_vec()
}

/// A PNG picture stored again as lossless WebP, when that is smaller (embedded photos). None = keep as it is.
pub fn recompress_png(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 4096 || bytes[..4] != [0x89, b'P', b'N', b'G'] {
        return None;
    }
    let mut rgba = Vec::new();
    let (w, h) = decode_bands(bytes, |_, rows, _| rgba.extend_from_slice(rows)).ok()?;
    if !use_webp(w, h) || rgba.len() != w as usize * h as usize * 4 {
        return None;
    }
    let packed = webp_lossless(&rgba, w, h);
    (packed.len() < bytes.len()).then_some(packed)
}

fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// Encodes a `width` x `height` RGBA picture whose rows come from `rows(y, count)` (RGBA bytes).
pub fn encode_rows(width: u32, height: u32, rows: impl Fn(u32, u32) -> Vec<u8>) -> Result<Vec<u8>, IoError> {
    if use_webp(width, height) {
        let all = rows(0, height);
        if all.len() == width as usize * height as usize * 4 {
            return Ok(webp_lossless(&all, width, height));
        }
    }
    let failed = |e: png::EncodingError| IoError::CannotSave(format!("Could not save a picture of the project: {e}"));
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Balanced);
        let mut writer = encoder.write_header().map_err(failed)?;
        let mut stream = writer.stream_writer().map_err(failed)?;
        let mut y = 0;
        while y < height {
            let count = BAND.min(height - y);
            stream.write_all(&rows(y, count)).map_err(|e| IoError::CannotSave(e.to_string()))?;
            y += count;
        }
        stream.finish().map_err(failed)?;
    }
    Ok(out)
}

/// Encodes the area `rect` of a tile map as a PNG.
pub fn encode_tiles(map: &TileMap, rect: PixelRect) -> Result<Vec<u8>, IoError> {
    encode_rows(rect.width as u32, rect.height as u32, |y, count| {
        map.read_rect(PixelRect::new(rect.x, rect.y + y as i32, rect.width, count as i32))
    })
}

/// Decodes a PNG band by band: `band(y, rows, width)` receives up to 256 RGBA rows at a time.
/// Returns the picture size.
pub fn decode_bands(bytes: &[u8], mut band: impl FnMut(u32, &[u8], u32)) -> Result<(u32, u32), IoError> {
    if is_webp(bytes) {
        return decode_webp(bytes, &mut band);
    }
    let bad = |e: png::DecodingError| IoError::CannotOpen(format!("A picture inside the project is damaged ({e})."));
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(bad)?;
    let (width, height) = (reader.info().width, reader.info().height);
    let (color, _) = reader.output_color_type();
    let mut rows: Vec<u8> = Vec::with_capacity(width as usize * 4 * BAND as usize);
    let (mut band_start, mut y) = (0u32, 0u32);
    while let Some(row) = reader.next_row().map_err(bad)? {
        to_rgba(row.data(), color, &mut rows);
        y += 1;
        if y - band_start == BAND || y == height {
            band(band_start, &rows, width);
            rows.clear();
            band_start = y;
        }
    }
    Ok((width, height))
}

/// Decodes a PNG into a tile map with its top-left corner at (x, y). Returns the map and the picture size.
pub fn decode_to_tiles(bytes: &[u8], x: i32, y: i32) -> Result<(TileMap, u32, u32), IoError> {
    let mut map = TileMap::empty();
    let (w, h) = decode_bands(bytes, |row_y, rows, width| {
        let count = (rows.len() / 4 / width.max(1) as usize) as i32;
        map.write_rect(PixelRect::new(x, y + row_y as i32, width as i32, count), rows);
    })?;
    Ok((map, w, h))
}

/// Converts one decoded row to RGBA8.
fn to_rgba(row: &[u8], color: png::ColorType, out: &mut Vec<u8>) {
    match color {
        png::ColorType::Rgba => out.extend_from_slice(row),
        png::ColorType::Rgb => {
            row.as_chunks::<3>().0.iter().for_each(|p| out.extend_from_slice(&[p[0], p[1], p[2], 255]))
        }
        png::ColorType::Grayscale => row.iter().for_each(|&v| out.extend_from_slice(&[v, v, v, 255])),
        png::ColorType::GrayscaleAlpha => {
            row.as_chunks::<2>().0.iter().for_each(|p| out.extend_from_slice(&[p[0], p[0], p[0], p[1]]))
        }
        png::ColorType::Indexed => out.extend_from_slice(row), // EXPAND turns palettes into RGB(A) before this
    }
}

fn decode_webp(bytes: &[u8], band: &mut impl FnMut(u32, &[u8], u32)) -> Result<(u32, u32), IoError> {
    let bad = || IoError::CannotOpen("A picture inside the project is damaged (WebP).".into());
    let image = webp::Decoder::new(bytes).decode().ok_or_else(bad)?;
    let (width, height) = (image.width(), image.height());
    let channels = if image.is_alpha() { 4 } else { 3 };
    let row_bytes = width as usize * channels;
    let mut rows: Vec<u8> = Vec::with_capacity(width as usize * 4 * BAND as usize);
    let mut y = 0u32;
    while y < height {
        let count = BAND.min(height - y);
        rows.clear();
        let part = &image[y as usize * row_bytes..(y + count) as usize * row_bytes];
        if channels == 4 {
            rows.extend_from_slice(part);
        } else {
            part.as_chunks::<3>().0.iter().for_each(|p| rows.extend_from_slice(&[p[0], p[1], p[2], 255]));
        }
        band(y, &rows, width);
        y += count;
    }
    Ok((width, height))
}
