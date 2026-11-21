//! Properties changes of QR codes, barcodes and charts (C# `ObjectPropertiesViewModel`, code and chart parts).

use fe_core::imaging::ColorRgba;
use fe_core::objects::{BarcodeObjData, ChartData, QrData};

use super::objects::change_object;
use crate::app::FlashApp;

/// A change of the selected QR code.
pub fn change_qr(app: &mut FlashApp, what: &'static str, mut change: impl FnMut(&mut QrData)) {
    change_object(app, what, |o| {
        let Some(mut q) = o.qr() else { return };
        change(&mut q);
        o.set_qr(&q);
    });
}

/// A change of the selected barcode.
pub fn change_barcode(app: &mut FlashApp, what: &'static str, mut change: impl FnMut(&mut BarcodeObjData)) {
    change_object(app, what, |o| {
        let Some(mut c) = o.barcode() else { return };
        change(&mut c);
        o.set_barcode(&c);
    });
}

/// A change of the selected chart.
pub fn change_chart(app: &mut FlashApp, what: &'static str, mut change: impl FnMut(&mut ChartData)) {
    change_object(app, what, |o| {
        let Some(mut c) = o.chart() else { return };
        change(&mut c);
        o.set_chart(&c);
    });
}

/// Which color of a code or chart is picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectColor {
    QrForeground,
    QrBackground,
    BarcodeForeground,
    BarcodeBackground,
    ChartBackground,
    ChartText,
    /// The color of the n-th bar or slice.
    ChartItem(usize),
}

/// Uses a picked color.
pub fn apply_color(app: &mut FlashApp, which: ObjectColor, color: ColorRgba) {
    match which {
        ObjectColor::QrForeground => change_qr(app, "QR Color", |q| q.foreground = color),
        ObjectColor::QrBackground => change_qr(app, "QR Background", |q| {
            q.background = color;
            q.transparent_background = false;
        }),
        ObjectColor::BarcodeForeground => change_barcode(app, "Barcode Color", |c| c.foreground = color),
        ObjectColor::BarcodeBackground => change_barcode(app, "Barcode Background", |c| {
            c.background = color;
            c.transparent_background = false;
        }),
        ObjectColor::ChartBackground => change_chart(app, "Chart Background", |c| c.background = Some(color)),
        ObjectColor::ChartText => change_chart(app, "Chart Text Color", |c| c.text_color = Some(color)),
        ObjectColor::ChartItem(i) => change_chart(app, "Chart Color", |c| {
            // A chart that has fewer colors than items shares them: make one for every item first.
            let n = c.items.len().max(i + 1);
            let colors: Vec<ColorRgba> = (0..n).map(|k| c.color_of(k)).collect();
            c.colors = colors;
            c.colors[i] = color;
        }),
    }
}
