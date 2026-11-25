//! Photoshop files (.psd) with 8-bit RGB color (C# `PsdFile`): opening keeps the layers (name, picture, position,
//! opacity, visibility, blend mode and, as see-through areas, the layer masks); saving writes every layer plus the
//! flattened picture, so Photoshop and other editors can open it. Layer effects, text layers, smart objects and
//! adjustment layers of Photoshop are not understood: they stay as the pictures Photoshop saved with them. Files in
//! other color modes (CMYK, 16 bit...) are opened as their flattened picture when possible.

use std::path::Path;

use super::IoError;
use super::decode::check_size;
use crate::document::canvas_ops::bake_mask;
use crate::document::{Document, Layer};
use crate::imaging::{BlendMode, PixelBuffer, TileMap};
use crate::paint::compositor;

/// The extension, with the dot.
pub const EXTENSION: &str = ".psd";

/// True for a path that ends in .psd.
pub fn is_psd(path: impl AsRef<Path>) -> bool {
    path.as_ref().extension().is_some_and(|e| e.eq_ignore_ascii_case("psd"))
}

/// The four-letter keys Photoshop uses for blend modes.
const BLEND_KEYS: [(&[u8; 4], BlendMode); 9] = [
    (b"norm", BlendMode::Normal),
    (b"mul ", BlendMode::Multiply),
    (b"scrn", BlendMode::Screen),
    (b"over", BlendMode::Overlay),
    (b"sLit", BlendMode::SoftLight),
    (b"hLit", BlendMode::HardLight),
    (b"dark", BlendMode::Darken),
    (b"lite", BlendMode::Lighten),
    (b"diff", BlendMode::Difference),
];

fn mode_of(key: &[u8]) -> BlendMode {
    BLEND_KEYS.iter().find(|(k, _)| k.as_slice() == key).map_or(BlendMode::Normal, |(_, m)| *m)
}

fn key_of(mode: BlendMode) -> &'static [u8; 4] {
    BLEND_KEYS.iter().find(|(_, m)| *m == mode).map_or(b"norm", |(k, _)| k)
}

fn damaged(text: &str) -> IoError {
    IoError::CannotOpen(text.to_owned())
}

// ---------------------------------------------------------------------------------------------------- reading

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], IoError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|e| *e <= self.data.len())
            .ok_or_else(|| damaged("The Photoshop file ends too early."))?;
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }
    fn skip(&mut self, n: usize) -> Result<(), IoError> {
        self.bytes(n).map(|_| ())
    }
    fn seek(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }
    fn u8(&mut self) -> Result<u8, IoError> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, IoError> {
        let b = self.bytes(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }
    fn i16(&mut self) -> Result<i16, IoError> {
        Ok(self.u16()? as i16)
    }
    fn u32(&mut self) -> Result<u32, IoError> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn i32(&mut self) -> Result<i32, IoError> {
        Ok(self.u32()? as i32)
    }
}

#[derive(Default)]
struct PsdLayer {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    name: String,
    opacity: u8,
    visible: bool,
    blend_key: [u8; 4],
    channels: Vec<(i16, Vec<u8>)>,
    mask: (i32, i32, i32, i32),
}

impl PsdLayer {
    fn width(&self) -> i32 {
        (self.right - self.left).max(0)
    }
    fn height(&self) -> i32 {
        (self.bottom - self.top).max(0)
    }
    fn channel(&self, id: i16) -> Option<&Vec<u8>> {
        self.channels.iter().find(|(i, _)| *i == id).map(|(_, d)| d)
    }
}

/// Run-length (PackBits) decoding of one row.
fn unpack_bits(source: &[u8], target: &mut [u8]) {
    let (mut s, mut t) = (0usize, 0usize);
    while s < source.len() && t < target.len() {
        let n = source[s] as i8;
        s += 1;
        if n >= 0 {
            for _ in 0..=n as usize {
                if s >= source.len() || t >= target.len() {
                    break;
                }
                target[t] = source[s];
                s += 1;
                t += 1;
            }
        } else if n != -128 {
            if s >= source.len() {
                break;
            }
            let value = source[s];
            s += 1;
            for _ in 0..(1 - i32::from(n)) as usize {
                if t >= target.len() {
                    break;
                }
                target[t] = value;
                t += 1;
            }
        }
    }
}

fn read_channel(r: &mut Reader, width: usize, height: usize) -> Result<Vec<u8>, IoError> {
    let compression = r.u16()?;
    let mut data = vec![0u8; width * height];
    if width == 0 || height == 0 {
        return Ok(data);
    }
    match compression {
        0 => {
            let raw = r.bytes(data.len().min(r.data.len() - r.pos))?;
            data[..raw.len()].copy_from_slice(raw);
        }
        1 => {
            let counts: Vec<usize> = (0..height).map(|_| r.u16().map(usize::from)).collect::<Result<_, _>>()?;
            for (y, count) in counts.into_iter().enumerate() {
                let packed = r.bytes(count)?;
                unpack_bits(packed, &mut data[y * width..(y + 1) * width]);
            }
        }
        _ => return Err(damaged("This Photoshop file uses a compression that is not supported.")),
    }
    Ok(data)
}

fn read_layers(r: &mut Reader) -> Result<Vec<PsdLayer>, IoError> {
    let info_length = r.u32()? as usize;
    if info_length == 0 {
        return Ok(Vec::new());
    }
    let info_end = r.pos + info_length;
    let count = r.i16()?.unsigned_abs() as usize;
    let mut layers = Vec::new();
    let mut lengths: Vec<Vec<(i16, usize)>> = Vec::new();
    for _ in 0..count {
        let mut layer =
            PsdLayer { top: r.i32()?, left: r.i32()?, bottom: r.i32()?, right: r.i32()?, ..PsdLayer::default() };
        let channel_count = r.u16()?;
        let mut these = Vec::new();
        for _ in 0..channel_count {
            these.push((r.i16()?, r.u32()? as usize));
        }
        if r.bytes(4)? != b"8BIM" {
            return Err(damaged("A layer in the Photoshop file is damaged."));
        }
        layer.blend_key.copy_from_slice(r.bytes(4)?);
        layer.opacity = r.u8()?;
        r.u8()?; // clipping
        layer.visible = r.u8()? & 2 == 0;
        r.u8()?; // filler
        let extra_end = r.u32()? as usize + r.pos;
        let mask_length = r.u32()? as usize;
        if mask_length >= 16 {
            let start = r.pos;
            layer.mask = (r.i32()?, r.i32()?, r.i32()?, r.i32()?);
            r.seek(start + mask_length);
        } else {
            r.skip(mask_length)?;
        }
        let ranges = r.u32()? as usize;
        r.skip(ranges)?;
        let name_length = usize::from(r.u8()?);
        layer.name = r.bytes(name_length)?.iter().map(|b| char::from(*b)).collect(); // Latin-1
        r.seek(extra_end);
        layers.push(layer);
        lengths.push(these);
    }
    for (layer, these) in layers.iter_mut().zip(lengths) {
        for (id, length) in these {
            let end = r.pos + length;
            let (w, h) = if id == -2 {
                ((layer.mask.3 - layer.mask.1).max(0) as usize, (layer.mask.2 - layer.mask.0).max(0) as usize)
            } else {
                (layer.width() as usize, layer.height() as usize)
            };
            let data = if length >= 2 { read_channel(r, w, h)? } else { Vec::new() };
            layer.channels.push((id, data));
            r.seek(end);
        }
    }
    r.seek(info_end);
    Ok(layers)
}

fn build_layer(p: &PsdLayer, canvas_w: u32, canvas_h: u32) -> Option<Layer> {
    if (p.width() == 0 || p.height() == 0)
        && (p.name.starts_with("</") || p.channels.is_empty() || p.name.to_lowercase().contains("group"))
    {
        return None; // group markers are not pictures
    }
    let (w, h) = (p.width().max(1) as u32, p.height().max(1) as u32);
    let mut content = PixelBuffer::new(w, h);
    let (red, green, blue, alpha) = (p.channel(0), p.channel(1), p.channel(2), p.channel(-1));
    let count = (p.width() * p.height()) as usize;
    for (i, px) in content.data_mut().chunks_exact_mut(4).take(count).enumerate() {
        px[0] = red.and_then(|c| c.get(i)).copied().unwrap_or(0);
        px[1] = green.and_then(|c| c.get(i)).copied().unwrap_or(0);
        px[2] = blue.and_then(|c| c.get(i)).copied().unwrap_or(0);
        px[3] = alpha.map_or(255, |c| c.get(i).copied().unwrap_or(0));
    }
    // A layer mask hides parts of the layer: it is applied to the see-through-ness of the layer's pixels.
    if let Some(mask) = p.channel(-2).filter(|m| !m.is_empty()) {
        let (top, left, bottom, right) = p.mask;
        let (mw, mh) = (right - left, bottom - top);
        for y in 0..p.height() {
            for x in 0..p.width() {
                let (mx, my) = (x + p.left - left, y + p.top - top);
                let m = if mx >= 0 && my >= 0 && mx < mw && my < mh { mask[(my * mw + mx) as usize] } else { 255 };
                if m < 255 {
                    let i = (y * p.width() + x) as usize * 4 + 3;
                    content.data_mut()[i] = (u32::from(content.data()[i]) * u32::from(m) / 255) as u8;
                }
            }
        }
    }
    let mut layer = Layer::new(p.name.clone());
    layer.set_opacity(f64::from(p.opacity) / 255.0);
    layer.visible = p.visible;
    layer.blend = mode_of(&p.blend_key);
    let _ = (canvas_w, canvas_h);
    layer.replace_pixels(TileMap::from_buffer(&content, p.left, p.top));
    Some(layer)
}

fn read_merged(
    r: &mut Reader,
    width: usize,
    height: usize,
    channels: usize,
    mode: u16,
) -> Result<PixelBuffer, IoError> {
    let compression = r.u16()?;
    let planes = channels.min(if mode == 3 { 4 } else { 2 });
    let mut plane: Vec<Vec<u8>> = (0..planes).map(|_| vec![0u8; width * height]).collect();
    match compression {
        0 => {
            for p in &mut plane {
                p.copy_from_slice(r.bytes(width * height)?);
            }
        }
        1 => {
            // Row lengths of every row of every channel come first, then the packed rows.
            let counts: Vec<usize> =
                (0..channels * height).map(|_| r.u16().map(usize::from)).collect::<Result<_, _>>()?;
            for c in 0..channels {
                for y in 0..height {
                    let packed = r.bytes(counts[c * height + y])?;
                    if c < planes {
                        unpack_bits(packed, &mut plane[c][y * width..(y + 1) * width]);
                    }
                }
            }
        }
        _ => return Err(damaged("This Photoshop file uses a compression that is not supported.")),
    }
    let mut result = PixelBuffer::new(width as u32, height as u32);
    for (i, px) in result.data_mut().chunks_exact_mut(4).enumerate() {
        if mode == 3 {
            px[0] = plane[0][i];
            px[1] = plane[1.min(planes - 1)][i];
            px[2] = plane[2.min(planes - 1)][i];
            px[3] = if planes >= 4 { plane[3][i] } else { 255 };
        } else {
            let g = plane[0][i];
            px[..3].fill(g);
            px[3] = if planes >= 2 { plane[1][i] } else { 255 };
        }
    }
    Ok(result)
}

/// Opens a .psd file as a document with its layers.
pub fn open(path: impl AsRef<Path>) -> Result<Document, IoError> {
    read(&std::fs::read(path)?)
}

/// Reads a .psd file from bytes.
pub fn read(data: &[u8]) -> Result<Document, IoError> {
    let mut r = Reader { data, pos: 0 };
    if r.bytes(4)? != b"8BPS" {
        return Err(damaged("This is not a Photoshop file."));
    }
    if r.u16()? != 1 {
        return Err(damaged("Large Photoshop files (.psb) are not supported."));
    }
    r.skip(6)?;
    let channels = usize::from(r.u16()?);
    let (height, width) = (r.u32()?, r.u32()?);
    let depth = r.u16()?;
    let mode = r.u16()?;
    check_size(width, height)?;
    let color_mode = r.u32()? as usize;
    r.skip(color_mode)?;
    let resources = r.u32()? as usize;
    r.skip(resources)?;
    let rgb8 = depth == 8 && mode == 3;
    let section_length = r.u32()? as usize;
    let section_end = r.pos + section_length;
    let layers = if section_length > 0 && rgb8 { read_layers(&mut r)? } else { Vec::new() };
    r.seek(section_end);
    let mut document = Document::new(width, height);
    if !layers.is_empty() {
        let built: Vec<Layer> = layers.iter().filter_map(|l| build_layer(l, width, height)).collect();
        if let Some(last) = built.last().map(Layer::id) {
            document.set_layers(built, Some(last));
            return Ok(document);
        }
    }
    // No usable layers: use the flattened picture at the end of the file.
    if !(depth == 8 && (mode == 3 || mode == 1)) {
        return Err(damaged("This Photoshop file uses a color mode that is not supported (only 8-bit RGB and grey)."));
    }
    let merged = read_merged(&mut r, width as usize, height as usize, channels, mode)?;
    document.set_layers(vec![Layer::from_buffer("Background", &merged)], None);
    Ok(document)
}

// ---------------------------------------------------------------------------------------------------- writing

fn planes_of(pixels: &PixelBuffer) -> [Vec<u8>; 4] {
    let n = (pixels.width() * pixels.height()) as usize;
    let (mut a, mut r, mut g, mut b) =
        (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
    for p in pixels.data().chunks_exact(4) {
        r.push(p[0]);
        g.push(p[1]);
        b.push(p[2]);
        a.push(p[3]);
    }
    [a, r, g, b]
}

/// The bytes of a .psd file with every layer (adjustment layers have no picture and are left out; masks are applied).
pub fn write(document: &Document) -> Vec<u8> {
    let (width, height) = (document.width(), document.height());
    let canvas = document.bounds();
    let mut out = Vec::new();
    out.extend_from_slice(b"8BPS");
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&[0; 6]);
    out.extend_from_slice(&4u16.to_be_bytes());
    out.extend_from_slice(&height.to_be_bytes());
    out.extend_from_slice(&width.to_be_bytes());
    out.extend_from_slice(&8u16.to_be_bytes());
    out.extend_from_slice(&3u16.to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes()); // color mode data
    out.extend_from_slice(&0u32.to_be_bytes()); // image resources
    let layers: Vec<&Layer> = document.layers().iter().filter(|l| !l.is_adjustment()).collect();
    let mut info: Vec<u8> = Vec::new();
    info.extend_from_slice(&(-(layers.len() as i16)).to_be_bytes()); // negative: the merged picture has transparency
    let mut channel_data = Vec::new();
    for layer in &layers {
        let baked = match layer.effective_mask() {
            Some(m) => bake_mask(layer.pixels(), m),
            None => layer.pixels().clone(),
        };
        let planes = planes_of(&baked.to_buffer(canvas));
        channel_data.push(planes);
        for v in [0i32, 0, height as i32, width as i32] {
            info.extend_from_slice(&v.to_be_bytes());
        }
        info.extend_from_slice(&4u16.to_be_bytes());
        for id in [-1i16, 0, 1, 2] {
            info.extend_from_slice(&id.to_be_bytes());
            info.extend_from_slice(&(2 + width * height).to_be_bytes());
        }
        info.extend_from_slice(b"8BIM");
        info.extend_from_slice(key_of(layer.blend));
        info.push((layer.opacity().clamp(0.0, 1.0) * 255.0).round_ties_even() as u8);
        info.push(0);
        info.push(if layer.visible { 0 } else { 2 });
        info.push(0);
        let mut extra: Vec<u8> = Vec::new();
        extra.extend_from_slice(&0u32.to_be_bytes()); // layer mask data
        extra.extend_from_slice(&0u32.to_be_bytes()); // blending ranges
        let name: String = layer.name.chars().map(|c| if c.is_ascii() { c } else { '?' }).take(255).collect();
        extra.push(name.len() as u8);
        extra.extend_from_slice(name.as_bytes());
        let padded = (1 + name.len()).div_ceil(4) * 4; // the name takes a multiple of four bytes
        extra.extend(std::iter::repeat_n(0u8, padded - (1 + name.len())));
        info.extend_from_slice(&(extra.len() as u32).to_be_bytes());
        info.extend_from_slice(&extra);
    }
    // Channel pixels, in the same order: alpha, red, green, blue (stored raw).
    for planes in &channel_data {
        for plane in planes {
            info.extend_from_slice(&0u16.to_be_bytes());
            info.extend_from_slice(plane);
        }
    }
    if !info.len().is_multiple_of(2) {
        info.push(0);
    }
    let mut section: Vec<u8> = Vec::new();
    section.extend_from_slice(&(info.len() as u32).to_be_bytes());
    section.extend_from_slice(&info);
    section.extend_from_slice(&0u32.to_be_bytes()); // global layer mask info
    out.extend_from_slice(&(section.len() as u32).to_be_bytes());
    out.extend_from_slice(&section);
    // The flattened picture (raw, planar: red, green, blue, alpha).
    let flat = planes_of(&compositor::flatten(document));
    out.extend_from_slice(&0u16.to_be_bytes());
    for plane in &flat[1..] {
        out.extend_from_slice(plane);
    }
    out.extend_from_slice(&flat[0]);
    out
}

/// Saves the document as a .psd file.
pub fn save(document: &Document, path: impl AsRef<Path>) -> Result<(), IoError> {
    super::pptx::write_file(path.as_ref(), write(document))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn layers_survive_a_save_and_open() {
        let mut doc = Document::blank(40, 30, ColorRgba::rgb(10, 20, 30));
        let mut top = Layer::new("Red dot");
        top.pixels_mut().fill_rect(crate::imaging::PixelRect::new(5, 6, 4, 4), ColorRgba::rgb(255, 0, 0));
        top.set_opacity(0.5);
        top.blend = BlendMode::Multiply;
        top.visible = false;
        doc.push_layer(top);
        let back = read(&write(&doc)).expect("readable");
        assert_eq!((back.width(), back.height()), (40, 30));
        assert_eq!(back.layers().len(), 2);
        let dot = &back.layers()[1];
        assert_eq!(dot.name, "Red dot");
        assert!((dot.opacity() - 0.5).abs() < 0.01);
        assert_eq!(dot.blend, BlendMode::Multiply);
        assert!(!dot.visible);
        assert_eq!(dot.pixels().pixel(6, 7), ColorRgba::rgb(255, 0, 0));
        assert_eq!(dot.pixels().pixel(30, 20).a, 0);
        assert_eq!(back.layers()[0].pixels().pixel(1, 1), ColorRgba::rgb(10, 20, 30));
    }

    #[test]
    fn packbits_rows_decode() {
        let mut row = [0u8; 6];
        unpack_bits(&[0xFE, 7, 0x01, 1, 2], &mut row); // three 7s, then 1 and 2
        assert_eq!(row, [7, 7, 7, 1, 2, 0]);
    }

    #[test]
    fn a_file_that_is_not_photoshop_is_refused() {
        assert!(read(b"not a psd file").is_err());
        assert!(is_psd("a/B.PSD") && !is_psd("a.png"));
    }
}
