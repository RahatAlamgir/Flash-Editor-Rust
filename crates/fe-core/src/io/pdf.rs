//! PDF with one page per picture (C# `PdfWriter`). Each page is exactly as big as its picture (at the chosen
//! resolution) and holds the picture as a JPEG, so the file stays small. Written by hand (PDF 1.4), no library needed.

use std::fmt::Write as _;

use super::{IoError, encode, format::ImageFileFormat};
use crate::imaging::{ColorRgba, PixelBuffer};

fn number(v: f64) -> String {
    let text = format!("{v:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The bytes of a PDF. `quality` is the JPEG quality 1..100; `dpi` says how many pixels make one inch on paper (96 keeps the
/// on-screen size, 300 makes pages smaller on paper); `matte` is the color behind see-through parts.
pub fn build(pages: &[PixelBuffer], quality: u8, dpi: f64, matte: Option<ColorRgba>) -> Result<Vec<u8>, IoError> {
    if pages.is_empty() {
        return Err(IoError::CannotSave("A PDF needs at least one page.".into()));
    }
    let dpi = dpi.clamp(36.0, 1200.0);
    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new(); // byte position of object 1, 2, 3...
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let begin = |out: &mut Vec<u8>, offsets: &mut Vec<usize>| {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", offsets.len()).as_bytes());
    };
    // Object numbers: 1 = catalog, 2 = page tree, then three objects per page (page, contents, picture).
    let page_object = |i: usize| 3 + i * 3;
    begin(&mut out, &mut offsets);
    out.extend_from_slice(b"<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    begin(&mut out, &mut offsets);
    let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", page_object(i))).collect();
    out.extend_from_slice(
        format!("<< /Type /Pages /Count {} /Kids [{}] >>\nendobj\n", pages.len(), kids.join(" ")).as_bytes(),
    );
    for (i, picture) in pages.iter().enumerate() {
        let (w, h) = (f64::from(picture.width()) * 72.0 / dpi, f64::from(picture.height()) * 72.0 / dpi);
        let jpeg = encode::encode(picture, ImageFileFormat::Jpeg, quality, Some(matte.unwrap_or(ColorRgba::WHITE)))?;
        let (page, contents, image) = (page_object(i), page_object(i) + 1, page_object(i) + 2);
        begin(&mut out, &mut offsets);
        out.extend_from_slice(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /Im0 {image} 0 R >> >> /Contents {contents} 0 R >>\nendobj\n",
                number(w),
                number(h)
            )
            .as_bytes(),
        );
        let draw = format!("q {} 0 0 {} 0 0 cm /Im0 Do Q", number(w), number(h));
        begin(&mut out, &mut offsets);
        out.extend_from_slice(format!("<< /Length {} >>\nstream\n{draw}\nendstream\nendobj\n", draw.len()).as_bytes());
        begin(&mut out, &mut offsets);
        out.extend_from_slice(
            format!(
                "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
                picture.width(),
                picture.height(),
                jpeg.len()
            )
            .as_bytes(),
        );
        out.extend_from_slice(&jpeg);
        out.extend_from_slice(b"\nendstream\nendobj\n");
        let _ = page; // the page object number is only used for the page tree
    }
    let xref = out.len();
    let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
    for offset in &offsets {
        let _ = writeln!(table, "{offset:010} 00000 n ");
    }
    let _ = write!(table, "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1);
    out.extend_from_slice(table.as_bytes());
    Ok(out)
}

/// Writes the PDF (through a temporary file).
pub fn save(
    pages: &[PixelBuffer],
    path: impl AsRef<std::path::Path>,
    quality: u8,
    dpi: f64,
    matte: Option<ColorRgba>,
) -> Result<(), IoError> {
    let path = path.as_ref();
    let bytes = build(pages, quality, dpi, matte)?;
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pdf_has_a_page_per_picture_sized_by_the_resolution() {
        let page = PixelBuffer::filled(192, 96, ColorRgba::rgb(10, 20, 30));
        let pdf = build(&[page.clone(), page], 90, 96.0, None).expect("built");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("/Count 2"));
        assert!(text.contains("/MediaBox [0 0 144 72]"), "192 x 96 px at 96 dpi is 144 x 72 pt");
        assert!(text.trim_end().ends_with("%%EOF"));
        // The cross-reference table points at the right bytes.
        let find = |needle: &[u8]| pdf.windows(needle.len()).rposition(|w| w == needle).expect("found");
        let at = find(
            b"xref
0 ",
        );
        let after = &pdf[find(
            b"startxref
",
        ) + 10..];
        let start: usize =
            String::from_utf8_lossy(after).lines().next().and_then(|l| l.trim().parse().ok()).expect("number");
        assert_eq!(start, at);
        assert!(build(&[], 90, 96.0, None).is_err());
    }
}
