//! Phase 10: charts, QR codes, barcodes, badges, graphics, SmartArt and the tutorial project.

use fe_core::codes::BarcodeKind;
use fe_core::imaging::PixelRect;
use fe_core::library::{badges, graphics, icons, smartart};
use fe_core::objects::{BarcodeObjData, ChartData, ChartKind, ObjectBox, QrData, StoredObject};
use fe_core::render::render_object;

fn drawn(o: &StoredObject, w: i32, h: i32) -> usize {
    let canvas = PixelRect::new(0, 0, w, h);
    let tiles = render_object(o, canvas).expect("the object can be drawn");
    let buf = tiles.to_buffer(canvas);
    (0..buf.height())
        .flat_map(|y| (0..buf.width()).map(move |x| (x, y)))
        .filter(|(x, y)| buf.pixel(*x, *y).a > 0)
        .count()
}

#[test]
fn every_chart_kind_draws_something() {
    for kind in ChartKind::ALL {
        let o = StoredObject::new_chart(ObjectBox::new(10.0, 10.0, 400.0, 300.0, 0.0), &ChartData::sample(kind));
        assert!(o.can_draw());
        assert!(drawn(&o, 420, 320) > 500, "{kind:?} drew nothing");
    }
}

#[test]
fn a_chart_without_numbers_says_so_and_does_not_crash() {
    let mut d = ChartData::sample(ChartKind::Pie);
    d.items.clear();
    let o = StoredObject::new_chart(ObjectBox::new(0.0, 0.0, 300.0, 200.0, 0.0), &d);
    assert!(drawn(&o, 300, 200) > 0);
}

#[test]
fn qr_codes_and_barcodes_draw_and_bad_text_shows_a_crossed_box() {
    let qr = StoredObject::new_qr(ObjectBox::new(0.0, 0.0, 200.0, 200.0, 0.0), &QrData::new("https://example.com"));
    assert!(drawn(&qr, 200, 200) > 5000);
    let too_long = StoredObject::new_qr(ObjectBox::new(0.0, 0.0, 200.0, 200.0, 0.0), &QrData::new(&"x".repeat(4000)));
    assert!(drawn(&too_long, 200, 200) > 100);
    for kind in BarcodeKind::ALL {
        let code = BarcodeObjData::new(BarcodeObjData::sample(kind), kind);
        let o = StoredObject::new_barcode(ObjectBox::new(0.0, 0.0, 300.0, 120.0, 0.0), &code);
        assert!(drawn(&o, 300, 120) > 3000, "{kind:?}");
    }
}

#[test]
fn every_badge_graphic_and_diagram_builds_and_draws() {
    assert_eq!(badges::ALL.len(), 52);
    for info in &badges::ALL {
        let g = badges::build(info.id, 300.0, 300.0, 120.0).unwrap_or_else(|| panic!("badge {}", info.id));
        assert!(g.can_draw() && drawn(&g, 300, 300) > 100, "badge {}", info.id);
    }
    for info in graphics::all() {
        let g = graphics::build(info.id, 400.0, 400.0, 200.0).unwrap_or_else(|| panic!("graphic {}", info.id));
        assert!(g.can_draw() && drawn(&g, 400, 400) > 20, "graphic {}", info.id);
    }
    for layout in &smartart::LAYOUTS {
        let words: Vec<String> = layout.sample.iter().map(|s| (*s).to_owned()).collect();
        for scheme in ["Colorful", "Gray"] {
            let g = smartart::build(layout.id, &words, scheme, 500.0, 500.0, 300.0).expect(layout.id);
            assert!(drawn(&g, 500, 500) > 500, "{}", layout.id);
        }
        // Too few words are filled up, too many cut off.
        assert!(smartart::build(layout.id, &[], "Blue", 500.0, 500.0, 300.0).is_some());
        let many: Vec<String> = (0..20).map(|i| format!("W{i}")).collect();
        assert!(smartart::build(layout.id, &many, "Blue", 500.0, 500.0, 300.0).is_some());
    }
    assert!(icons::ALL.len() >= 150);
}

#[test]
fn the_tutorial_has_14_pages_that_all_draw() {
    let pages = fe_core::tutorial::build_pages();
    assert_eq!(pages.len(), 14);
    for page in &pages {
        assert_eq!((page.doc.width(), page.doc.height()), (1280, 720));
        assert!(page.doc.layers().len() > 5, "{}", page.name);
    }
    // Every object on every page can be drawn by the Rust version.
    for page in &pages {
        for layer in page.doc.layers() {
            if let Some(o) = layer.object() {
                assert!(o.can_draw(), "{}: {}", page.name, layer.name);
            }
        }
    }
}
