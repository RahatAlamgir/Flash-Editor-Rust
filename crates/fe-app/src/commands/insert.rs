//! The Insert menu (C# `InsertCommands`): QR codes, barcodes, charts, text, icons, badges, graphics and SmartArt,
//! each put in the middle of the picture on a new top layer.

use fe_core::codes::BarcodeKind;
use fe_core::imaging::ColorRgba;
use fe_core::library::{badges, graphics, icons, smartart};
use fe_core::objects::{BarcodeObjData, ChartData, ChartItem, ChartKind, ObjectBox, QrData, StoredObject};
use fe_core::render::measure_text;
use fe_core::text::TextData;

use super::objects::add_object;
use crate::app::FlashApp;

fn canvas_size(app: &FlashApp) -> (f64, f64) {
    (f64::from(app.doc().width()), f64::from(app.doc().height()))
}

/// Insert > QR Code: a code for a sample address in the middle of the picture; the text is changed in Properties.
pub fn add_qr(app: &mut FlashApp) {
    let (dw, dh) = canvas_size(app);
    let size = (dw.min(dh) * 0.35).max(40.0).round();
    let b = ObjectBox::new(((dw - size) / 2.0).round(), ((dh - size) / 2.0).round(), size, size, 0.0);
    add_object(app, StoredObject::new_qr(b, &QrData::new("https://example.com")), "QR code", "Insert QR Code");
    app.status =
        "Added a QR code. Type your own text, colors and style in Properties, and test it with your phone.".into();
}

/// Insert > Barcode: a barcode of the chosen kind with a sample number.
pub fn add_barcode(app: &mut FlashApp, kind: BarcodeKind) {
    let (dw, dh) = canvas_size(app);
    let w = (dw * 0.5).min(dh * 1.2).max(120.0).round();
    let h = (w * 0.42).round();
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - h) / 2.0).round(), w, h, 0.0);
    let code = BarcodeObjData::new(BarcodeObjData::sample(kind), kind);
    add_object(app, StoredObject::new_barcode(b, &code), "Barcode", "Insert Barcode");
    app.status = "Added a barcode. Change its text, type and colors in Properties. Test it with a scanner or your \
                  phone before printing."
        .into();
}

fn brand_colors(app: &FlashApp) -> Vec<ColorRgba> {
    app.settings.brand_colors.iter().filter_map(|h| ColorRgba::parse_hex(h)).collect()
}

/// A chart with sample numbers (the colors of the Brand Kit when there are at least two).
pub fn sample_chart(kind: ChartKind, brand: &[ColorRgba]) -> ChartData {
    let mut chart = ChartData::sample(kind);
    if brand.len() >= 2 {
        chart.colors = brand.to_vec();
    }
    let items = |list: &[(&str, f64)]| list.iter().map(|(l, v)| ChartItem::new(l, *v)).collect::<Vec<_>>();
    // Some kinds look best with their own kind of numbers.
    match kind {
        ChartKind::Progress => {
            chart.title = "Skills".into();
            chart.value_suffix = "%".into();
            chart.show_grid = false;
            chart.items = items(&[("Design", 85.0), ("Development", 64.0), ("Marketing", 48.0), ("Support", 72.0)]);
        }
        ChartKind::Funnel => {
            chart.title = "Sales funnel".into();
            chart.items = items(&[("Visitors", 1200.0), ("Leads", 640.0), ("Trials", 320.0), ("Customers", 140.0)]);
        }
        ChartKind::HalfDonut | ChartKind::RadialBars => {
            chart.title = "Share".into();
            chart.items = items(&[("Mobile", 46.0), ("Desktop", 34.0), ("Tablet", 20.0)]);
        }
        _ => {}
    }
    chart
}

/// Insert > Chart: a chart with sample numbers in the middle of the picture.
pub fn add_chart(app: &mut FlashApp, kind: ChartKind) {
    let (dw, dh) = canvas_size(app);
    let (w, h) = ((dw * 0.72).round(), (dh * 0.62).round());
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - h) / 2.0).round(), w, h, 0.0);
    let chart = sample_chart(kind, &brand_colors(app));
    let name = chart.layer_name();
    add_object(app, StoredObject::new_chart(b, &chart), &name, "Insert Chart");
    app.status =
        "Added a chart with sample numbers. Change the numbers, names and colors in Properties > Chart.".into();
}

/// The three text buttons of the Insert tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Heading,
    Subheading,
    Body,
}

/// Insert > Heading / Subheading / Body Text: sample words in the middle of the picture with everything selected, so
/// the first key typed replaces them (right on the canvas, like double-clicking text).
pub fn add_text(app: &mut FlashApp, kind: TextKind) {
    let (dw, dh) = canvas_size(app);
    let (sample, scale, bold) = match kind {
        TextKind::Heading => ("Your heading", 0.11, true),
        TextKind::Subheading => ("A subheading", 0.06, true),
        TextKind::Body => ("A little bit of body text", 0.04, false),
    };
    let size = (dw.min(dh) * scale).round().max(14.0);
    let mut data = TextData::new(sample, "Segoe UI", size, app.foreground);
    data.bold = bold;
    crate::text_edit::create_centered(app, data);
    app.status =
        "Type to replace the words. Esc or a click outside finishes; font, color and designs are in Properties.".into();
}

/// Insert > Icons: the icon as text, so its color, size, outline and shadow can be changed.
pub fn add_icon(app: &mut FlashApp, icon: &icons::IconItem) {
    let (dw, dh) = canvas_size(app);
    let side = dw.min(dh);
    let data = TextData::new(&icon.glyph(), icons::FONT_FAMILY, (side * 0.2).round().max(24.0), app.foreground);
    let (w, h) = measure_text(&data);
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - h) / 2.0).round(), w, h, 0.0);
    add_object(app, StoredObject::new_text(b, &data), &format!("{} icon", icon.name), "Insert Icon");
    app.status = format!(
        "Added the {} icon. It is text: change its color, size, outline and shadow in Properties.",
        icon.name.to_lowercase()
    );
}

/// Insert > Icons > Google icons: the icon as text in the Material Icons font, like the Windows icons.
pub fn add_material_icon(app: &mut FlashApp, code: u32) {
    use fe_core::library::material;
    let Some(icon) = material::all().iter().find(|i| i.code == code) else { return };
    let (dw, dh) = canvas_size(app);
    let side = dw.min(dh);
    let data = TextData::new(&icon.glyph(), material::FONT_FAMILY, (side * 0.2).round().max(24.0), app.foreground);
    let (w, h) = measure_text(&data);
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - h) / 2.0).round(), w, h, 0.0);
    add_object(app, StoredObject::new_text(b, &data), &format!("{} icon", icon.name), "Insert Icon");
    app.status = format!(
        "Added the {} icon. It is text: change its color, size, outline and shadow in Properties.",
        icon.name.to_lowercase()
    );
}

/// Insert > Badges.
pub fn add_badge(app: &mut FlashApp, id: &str) {
    let (dw, dh) = canvas_size(app);
    let Some(group) = badges::build(id, dw, dh, (dw.min(dh) * 0.3).max(80.0)) else { return };
    let name = badges::name_of(id);
    add_object(app, group, name, "Insert Badge");
    app.status = format!("Added the \"{name}\" badge. Ungroup it (Ctrl+Shift+G) to change its colors and words.");
}

/// Insert > Graphics.
pub fn add_graphic(app: &mut FlashApp, id: &str) {
    let (dw, dh) = canvas_size(app);
    let side = dw.min(dh);
    let Some(group) = graphics::build(id, dw, dh, (dw * 0.6).min(side * 0.8).max(120.0)) else { return };
    let name = graphics::name_of(id);
    add_object(app, group, name, "Insert Graphic");
    app.status =
        format!("Added \"{name}\". It is a group of shapes: ungroup it (Ctrl+Shift+G) to recolor or change its parts.");
}

/// Insert > SmartArt.
pub fn add_smartart(app: &mut FlashApp, id: &str, words: &[String], scheme: &str) {
    let (dw, dh) = canvas_size(app);
    let side = dw.min(dh);
    let Some(group) = smartart::build(id, words, scheme, dw, dh, (dw * 0.8).min(side * 1.2).max(160.0)) else { return };
    let name = smartart::layout(id).map_or("SmartArt", |l| l.name);
    add_object(app, group, name, "Insert SmartArt");
    app.status =
        format!("Added the \"{name}\" diagram. Ungroup it (Ctrl+Shift+G) to move, recolor or retype single parts.");
}
