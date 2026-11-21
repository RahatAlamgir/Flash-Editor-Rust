//! Draws the 16 text designs and the effects on a contact sheet (a PNG) to look at. Usage: `text_sheet out.png`.

use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::objects::{ObjectBox, StoredObject};
use fe_core::text::{TEXT_PRESETS, TextAlign, TextData};

fn main() {
    let out = std::env::args().nth(1).expect("usage: text_sheet <out.png>");
    let (cols, cell_w, cell_h) = (4usize, 300i32, 130i32);
    let rows = TEXT_PRESETS.len().div_ceil(cols);
    let (w, h) = (cols as i32 * cell_w, rows as i32 * cell_h);
    let canvas = PixelRect::new(0, 0, w, h);
    let mut sheet = PixelBuffer::new(w as u32, h as u32);
    for (i, preset) in TEXT_PRESETS.iter().enumerate() {
        let (cx, cy) = ((i % cols) as i32 * cell_w, (i / cols) as i32 * cell_h);
        let bg = preset.preview_background.unwrap_or(ColorRgba::new(238, 238, 242, 255));
        let cell = PixelRect::new(cx, cy, cell_w, cell_h);
        for y in cell.y..cell.y + cell_h {
            for x in cell.x..cell.x + cell_w {
                sheet.set_pixel(x as u32, y as u32, bg);
            }
        }
        let mut t = TextData::new(preset.name, "Segoe UI", 44.0, ColorRgba::BLACK);
        t.align = TextAlign::Center;
        let mut o = StoredObject::new_text(ObjectBox::new(0.0, 0.0, 10.0, 10.0, 0.0), &t);
        preset.apply(&mut o);
        let b = o.box_().unwrap();
        o.set_box(ObjectBox::from_center(
            fe_core::imaging::PointD::new(f64::from(cx + cell_w / 2), f64::from(cy + cell_h / 2)),
            b.width,
            b.height,
            0.0,
        ));
        let tiles = fe_core::render::render_object(&o, canvas).unwrap_or_default();
        let top = tiles.to_buffer(canvas);
        for y in cell.y..cell.y + cell_h {
            for x in cell.x..cell.x + cell_w {
                let i = sheet.index_of(x as u32, y as u32);
                let src = top.pixel(x as u32, y as u32);
                let a = f64::from(src.a) / 255.0;
                let d = &mut sheet.data_mut()[i..i + 3];
                for (dc, sc) in d.iter_mut().zip([src.r, src.g, src.b]) {
                    *dc = (f64::from(sc) * a + f64::from(*dc) * (1.0 - a)).round() as u8;
                }
            }
        }
    }
    fe_core::io::encode::save(&sheet, &out, fe_core::io::ImageFileFormat::Png, 90, None).expect("save");
    println!("saved {out}");
}
