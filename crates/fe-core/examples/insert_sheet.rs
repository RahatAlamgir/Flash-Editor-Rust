//! Draws the Insert extras on a contact sheet (a PNG) to look at.
//! Usage: `insert_sheet <charts|codes|badges|graphics|smartart|icons> out.png`.

use fe_core::codes::BarcodeKind;
use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::library;
use fe_core::objects::{BarcodeObjData, ChartData, ChartItem, ChartKind, ObjectBox, QrData, QrStyle, StoredObject};
use fe_core::text::TextData;

fn chart(kind: ChartKind) -> StoredObject {
    let mut c = ChartData::sample(kind);
    match kind {
        ChartKind::Progress => {
            c.title = "Skills".into();
            c.value_suffix = "%".into();
            c.show_grid = false;
            c.items = [("Design", 85.0), ("Development", 64.0), ("Marketing", 48.0), ("Support", 72.0)]
                .iter()
                .map(|(l, v)| ChartItem::new(l, *v))
                .collect();
        }
        ChartKind::Funnel => {
            c.title = "Sales funnel".into();
            c.items = [("Visitors", 1200.0), ("Leads", 640.0), ("Trials", 320.0), ("Customers", 140.0)]
                .iter()
                .map(|(l, v)| ChartItem::new(l, *v))
                .collect();
        }
        ChartKind::HalfDonut | ChartKind::RadialBars => {
            c.title = "Share".into();
            c.items = [("Mobile", 46.0), ("Desktop", 34.0), ("Tablet", 20.0)]
                .iter()
                .map(|(l, v)| ChartItem::new(l, *v))
                .collect();
        }
        _ => {}
    }
    StoredObject::new_chart(ObjectBox::new(0.0, 0.0, 420.0, 300.0, 0.0), &c)
}

fn objects(what: &str) -> (Vec<StoredObject>, (i32, i32)) {
    match what {
        "charts" => (ChartKind::ALL.iter().map(|k| chart(*k)).collect(), (420, 300)),
        "badges" => {
            let list =
                library::badges::ALL.iter().filter_map(|i| library::badges::build(i.id, 150.0, 150.0, 130.0)).collect();
            (list, (150, 150))
        }
        "graphics" => {
            let list =
                library::graphics::all().filter_map(|i| library::graphics::build(i.id, 150.0, 150.0, 130.0)).collect();
            (list, (150, 150))
        }
        "smartart" => {
            let list = library::smartart::LAYOUTS
                .iter()
                .filter_map(|l| {
                    let words: Vec<String> = l.sample.iter().map(|s| (*s).to_owned()).collect();
                    library::smartart::build(l.id, &words, "Colorful", 300.0, 300.0, 280.0)
                })
                .collect();
            (list, (300, 300))
        }
        "icons" => {
            let list = library::icons::ALL
                .iter()
                .map(|i| {
                    let t = TextData::new(&i.glyph(), library::icons::FONT_FAMILY, 40.0, ColorRgba::rgb(30, 41, 59));
                    let (w, h) = fe_core::render::measure_text(&t);
                    StoredObject::new_text(ObjectBox::new(0.0, 0.0, w, h, 0.0), &t)
                })
                .collect();
            (list, (60, 60))
        }
        _ => {
            let mut list = Vec::new();
            for style in QrStyle::ALL {
                let mut qr = QrData::new("https://example.com/flash-editor");
                qr.style = style;
                list.push(StoredObject::new_qr(ObjectBox::new(0.0, 0.0, 300.0, 300.0, 0.0), &qr));
            }
            for kind in BarcodeKind::ALL {
                let code = BarcodeObjData::new(BarcodeObjData::sample(kind), kind);
                list.push(StoredObject::new_barcode(ObjectBox::new(0.0, 0.0, 300.0, 126.0, 0.0), &code));
            }
            (list, (300, 300))
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let what = args.next().expect("usage: insert_sheet <charts|codes|badges|graphics|smartart|icons> <out.png>");
    let out = args.next().expect("usage: insert_sheet <charts|codes|badges|graphics|smartart|icons> <out.png>");
    let (list, (cw, ch)) = objects(&what);
    let cols = if what == "icons" {
        14
    } else if matches!(what.as_str(), "smartart" | "charts" | "codes") {
        4
    } else {
        8
    };
    let rows = list.len().div_ceil(cols);
    let (w, h) = (cols as i32 * (cw + 10) + 10, rows as i32 * (ch + 10) + 10);
    let mut sheet = PixelBuffer::new(w as u32, h as u32);
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            sheet.set_pixel(x, y, ColorRgba::rgb(210, 214, 222));
        }
    }
    for (i, o) in list.into_iter().enumerate() {
        let (x, y) = (10 + (i % cols) as i32 * (cw + 10), 10 + (i / cols) as i32 * (ch + 10));
        let o = o.offset(f64::from(x), f64::from(y));
        let tiles = fe_core::render::render_object(&o, PixelRect::new(0, 0, w, h)).unwrap_or_default();
        let top = tiles.to_buffer(PixelRect::new(0, 0, w, h));
        for yy in y.max(0)..(y + ch).min(h) {
            for xx in x.max(0)..(x + cw).min(w) {
                let idx = sheet.index_of(xx as u32, yy as u32);
                let src = top.pixel(xx as u32, yy as u32);
                let a = f64::from(src.a) / 255.0;
                let d = &mut sheet.data_mut()[idx..idx + 3];
                for (dc, sc) in d.iter_mut().zip([src.r, src.g, src.b]) {
                    *dc = (f64::from(sc) * a + f64::from(*dc) * (1.0 - a)).round() as u8;
                }
            }
        }
    }
    fe_core::io::encode::save(&sheet, &out, fe_core::io::ImageFileFormat::Png, 90, None).expect("save");
    println!("saved {out}");
}
