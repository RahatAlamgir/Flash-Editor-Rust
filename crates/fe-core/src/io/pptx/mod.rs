//! PowerPoint files (.pptx), written by hand: a .pptx is a zip of small XML files (C# `PptxWriter`). One slide per page.
//! Picture mode puts each page on its slide as one picture (JPEG or PNG), so it looks the same everywhere; editable mode
//! (`editable.rs`) keeps text and simple shapes as PowerPoint text boxes and shapes.

mod editable;
mod slide;

use std::io::Write;

use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

pub use editable::{EditableOptions, build_editable, build_editable_with, save_editable, save_editable_with};
pub use slide::transition_xml;
pub use slide::{Media, Place, PptxSlide, ShapeLook, TextLook, escape};

use super::{IoError, encode, format::ImageFileFormat};
use crate::imaging::{ColorRgba, PixelBuffer};

const EMU_PER_PIXEL: f64 = 9525.0; // 96 dpi
/// The biggest slide PowerPoint accepts (56 in).
const MAX_SLIDE_EMU: f64 = 51_206_400.0;

const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
const NS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const NS_P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn failed(e: impl std::fmt::Display) -> IoError {
    IoError::CannotSave(format!("Could not write the presentation: {e}"))
}

fn rels(items: &[(String, String, String)]) -> String {
    let body: String = items
        .iter()
        .map(|(id, kind, target)| format!("<Relationship Id=\"{id}\" Type=\"{kind}\" Target=\"{target}\"/>"))
        .collect();
    format!(
        "{DECL}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{body}</Relationships>"
    )
}

fn rel(id: &str, kind: &str, target: &str) -> (String, String, String) {
    (id.to_owned(), format!("{REL}/{kind}"), target.to_owned())
}

fn content_types(slides: usize) -> String {
    let overrides: String = (1..=slides)
        .map(|i| format!("<Override PartName=\"/ppt/slides/slide{i}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>"))
        .collect();
    format!(
        "{DECL}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"jpg\" ContentType=\"image/jpeg\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Override PartName=\"/ppt/presentation.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml\"/><Override PartName=\"/ppt/slideMasters/slideMaster1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml\"/><Override PartName=\"/ppt/slideLayouts/slideLayout1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml\"/><Override PartName=\"/ppt/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>{overrides}</Types>"
    )
}

fn presentation(slides: usize, w: i64, h: i64) -> String {
    let ids: String = (1..=slides).map(|i| format!("<p:sldId id=\"{}\" r:id=\"rId{}\"/>", 255 + i, i + 2)).collect();
    format!(
        "{DECL}<p:presentation xmlns:a=\"{NS_A}\" xmlns:r=\"{NS_R}\" xmlns:p=\"{NS_P}\"><p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rId1\"/></p:sldMasterIdLst><p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx=\"{w}\" cy=\"{h}\"/><p:notesSz cx=\"6858000\" cy=\"9144000\"/></p:presentation>"
    )
}

const GROUP_HEAD: &str = "<p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>";

fn slide_xml(slide: &PptxSlide, scale: f64) -> String {
    format!(
        "{DECL}<p:sld xmlns:a=\"{NS_A}\" xmlns:r=\"{NS_R}\" xmlns:p=\"{NS_P}\"><p:cSld><p:spTree>{GROUP_HEAD}{}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>{}</p:sld>",
        slide.to_xml(scale),
        slide.transition_xml()
    )
}

fn slide_layout() -> String {
    format!(
        "{DECL}<p:sldLayout xmlns:a=\"{NS_A}\" xmlns:r=\"{NS_R}\" xmlns:p=\"{NS_P}\" type=\"blank\" preserve=\"1\"><p:cSld name=\"Blank\"><p:spTree>{GROUP_HEAD}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"
    )
}

fn slide_master() -> String {
    format!(
        "{DECL}<p:sldMaster xmlns:a=\"{NS_A}\" xmlns:r=\"{NS_R}\" xmlns:p=\"{NS_P}\"><p:cSld><p:bg><p:bgRef idx=\"1001\"><a:schemeClr val=\"bg1\"/></p:bgRef></p:bg><p:spTree>{GROUP_HEAD}</p:spTree></p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/><p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rId1\"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz=\"4400\"/></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz=\"2800\"/></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz=\"1800\"/></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>"
    )
}

fn theme() -> String {
    let fills = "<a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill>".repeat(3);
    let lines: String = [6350, 12700, 19050]
        .iter()
        .map(|w| format!("<a:ln w=\"{w}\"><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:ln>"))
        .collect();
    let effects = "<a:effectStyle><a:effectLst/></a:effectStyle>".repeat(3);
    format!(
        "{DECL}<a:theme xmlns:a=\"{NS_A}\" name=\"Flash Editor\"><a:themeElements><a:clrScheme name=\"Flash Editor\"><a:dk1><a:srgbClr val=\"000000\"/></a:dk1><a:lt1><a:srgbClr val=\"FFFFFF\"/></a:lt1><a:dk2><a:srgbClr val=\"1F2937\"/></a:dk2><a:lt2><a:srgbClr val=\"F3F4F6\"/></a:lt2><a:accent1><a:srgbClr val=\"2563EB\"/></a:accent1><a:accent2><a:srgbClr val=\"F97316\"/></a:accent2><a:accent3><a:srgbClr val=\"14B8A6\"/></a:accent3><a:accent4><a:srgbClr val=\"EC4899\"/></a:accent4><a:accent5><a:srgbClr val=\"8B5CF6\"/></a:accent5><a:accent6><a:srgbClr val=\"F59E0B\"/></a:accent6><a:hlink><a:srgbClr val=\"2563EB\"/></a:hlink><a:folHlink><a:srgbClr val=\"7C3AED\"/></a:folHlink></a:clrScheme><a:fontScheme name=\"Flash Editor\"><a:majorFont><a:latin typeface=\"Segoe UI\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:majorFont><a:minorFont><a:latin typeface=\"Segoe UI\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:minorFont></a:fontScheme><a:fmtScheme name=\"Flash Editor\"><a:fillStyleLst>{fills}</a:fillStyleLst><a:lnStyleLst>{lines}</a:lnStyleLst><a:effectStyleLst>{effects}</a:effectStyleLst><a:bgFillStyleLst>{fills}</a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"
    )
}

/// A presentation of ready slides on a page of the given size in pixels.
pub fn build_slides(slides: &[PptxSlide], page_width: u32, page_height: u32) -> Result<Vec<u8>, IoError> {
    if slides.is_empty() {
        return Err(IoError::CannotSave("A presentation needs at least one slide.".into()));
    }
    let scale = (MAX_SLIDE_EMU / (f64::from(page_width.max(page_height)) * EMU_PER_PIXEL)).min(1.0);
    let slide_w = (f64::from(page_width) * EMU_PER_PIXEL * scale).round_ties_even() as i64;
    let slide_h = (f64::from(page_height) * EMU_PER_PIXEL * scale).round_ties_even() as i64;
    let mut out = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut out));
        let mut add = |name: &str, data: &[u8]| -> Result<(), IoError> {
            let method =
                if name.starts_with("ppt/media/") { CompressionMethod::Stored } else { CompressionMethod::Deflated };
            zip.start_file(name, SimpleFileOptions::default().compression_method(method)).map_err(failed)?;
            zip.write_all(data).map_err(failed)
        };
        add("[Content_Types].xml", content_types(slides.len()).as_bytes())?;
        add("_rels/.rels", rels(&[rel("rId1", "officeDocument", "ppt/presentation.xml")]).as_bytes())?;
        add("ppt/presentation.xml", presentation(slides.len(), slide_w, slide_h).as_bytes())?;
        let mut presentation_rels =
            vec![rel("rId1", "slideMaster", "slideMasters/slideMaster1.xml"), rel("rId2", "theme", "theme/theme1.xml")];
        for i in 0..slides.len() {
            presentation_rels.push(rel(&format!("rId{}", i + 3), "slide", &format!("slides/slide{}.xml", i + 1)));
        }
        add("ppt/_rels/presentation.xml.rels", rels(&presentation_rels).as_bytes())?;
        add("ppt/theme/theme1.xml", theme().as_bytes())?;
        add("ppt/slideMasters/slideMaster1.xml", slide_master().as_bytes())?;
        add(
            "ppt/slideMasters/_rels/slideMaster1.xml.rels",
            rels(&[
                rel("rId1", "slideLayout", "../slideLayouts/slideLayout1.xml"),
                rel("rId2", "theme", "../theme/theme1.xml"),
            ])
            .as_bytes(),
        )?;
        add("ppt/slideLayouts/slideLayout1.xml", slide_layout().as_bytes())?;
        add(
            "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
            rels(&[rel("rId1", "slideMaster", "../slideMasters/slideMaster1.xml")]).as_bytes(),
        )?;
        let mut media_number = 0;
        for (i, slide) in slides.iter().enumerate() {
            let mut slide_rels = vec![rel("rId1", "slideLayout", "../slideLayouts/slideLayout1.xml")];
            for (m, media) in slide.media.iter().enumerate() {
                media_number += 1;
                let file = format!("image{media_number}.{}", if media.png { "png" } else { "jpg" });
                add(&format!("ppt/media/{file}"), &media.data)?;
                slide_rels.push(rel(&format!("rId{}", m + 2), "image", &format!("../media/{file}")));
            }
            add(&format!("ppt/slides/slide{}.xml", i + 1), slide_xml(slide, scale).as_bytes())?;
            add(&format!("ppt/slides/_rels/slide{}.xml.rels", i + 1), rels(&slide_rels).as_bytes())?;
        }
        zip.finish().map_err(failed)?;
    }
    Ok(out)
}

/// One slide per picture, each slide the size of the first page and holding the page as one picture.
pub fn build_pictures(
    pages: &[PixelBuffer],
    quality: u8,
    lossless: bool,
    matte: Option<ColorRgba>,
) -> Result<Vec<u8>, IoError> {
    build_pictures_with(pages, quality, lossless, matte, &[])
}

/// Like `build_pictures`, with a slide transition for each slide (a shorter list leaves the rest without one).
pub fn build_pictures_with(
    pages: &[PixelBuffer],
    quality: u8,
    lossless: bool,
    matte: Option<ColorRgba>,
    transitions: &[crate::document::Transition],
) -> Result<Vec<u8>, IoError> {
    let Some(first) = pages.first() else {
        return Err(IoError::CannotSave("A presentation needs at least one slide.".into()));
    };
    let mut slides = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let data = if lossless {
            encode::encode(page, ImageFileFormat::Png, 100, matte)?
        } else {
            encode::encode(page, ImageFileFormat::Jpeg, quality, Some(matte.unwrap_or(ColorRgba::WHITE)))?
        };
        let mut slide = PptxSlide::new();
        slide.add_picture(
            data,
            lossless,
            &format!("Page {}", i + 1),
            Place { x: 0.0, y: 0.0, w: f64::from(first.width()), h: f64::from(first.height()), ..Place::default() },
            1.0,
        );
        if let Some(t) = transitions.get(i) {
            slide.set_transition(t);
        }
        slides.push(slide);
    }
    build_slides(&slides, first.width(), first.height())
}

/// Writes bytes through a temporary file.
pub fn write_file(path: &std::path::Path, bytes: Vec<u8>) -> Result<(), IoError> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    Ok(())
}

/// Saves the picture-mode presentation.
pub fn save_pictures(
    pages: &[PixelBuffer],
    path: impl AsRef<std::path::Path>,
    quality: u8,
    lossless: bool,
    matte: Option<ColorRgba>,
) -> Result<(), IoError> {
    save_pictures_with(pages, path, quality, lossless, matte, &[])
}

/// Like `save_pictures`, with a slide transition for each slide.
pub fn save_pictures_with(
    pages: &[PixelBuffer],
    path: impl AsRef<std::path::Path>,
    quality: u8,
    lossless: bool,
    matte: Option<ColorRgba>,
    transitions: &[crate::document::Transition],
) -> Result<(), IoError> {
    write_file(path.as_ref(), build_pictures_with(pages, quality, lossless, matte, transitions)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn part(bytes: &[u8], name: &str) -> String {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip");
        let mut text = String::new();
        zip.by_name(name).expect("part").read_to_string(&mut text).expect("text");
        text
    }

    #[test]
    fn a_picture_presentation_has_a_slide_per_page_with_the_page_size() {
        let page = PixelBuffer::filled(192, 108, ColorRgba::rgb(10, 20, 30));
        let bytes = build_pictures(&[page.clone(), page], 90, false, None).expect("built");
        let presentation = part(&bytes, "ppt/presentation.xml");
        assert!(
            presentation.contains("<p:sldSz cx=\"1828800\" cy=\"1028700\"/>"),
            "192 x 108 px = 1828800 x 1028700 EMU"
        );
        assert!(presentation.contains("rId4"), "two slides use rId3 and rId4");
        let slide = part(&bytes, "ppt/slides/slide1.xml");
        assert!(slide.contains("<a:ext cx=\"1828800\" cy=\"1028700\"/>") && slide.contains("r:embed=\"rId2\""));
        assert!(part(&bytes, "[Content_Types].xml").contains("slide2.xml"));
        assert!(build_pictures(&[], 90, false, None).is_err());
    }

    #[test]
    fn text_is_escaped_and_scaled_with_the_slide() {
        assert_eq!(escape("a < b & \"c\"\u{1}"), "a &lt; b &amp; &quot;c&quot;");
        let mut slide = PptxSlide::new();
        slide.add_shape(
            "rect",
            "Box",
            Place { x: 10.0, y: 20.0, w: 100.0, h: 50.0, ..Place::default() },
            &ShapeLook::default(),
        );
        assert!(slide.to_xml(1.0).contains("<a:off x=\"95250\" y=\"190500\"/>"));
        assert!(slide.to_xml(0.5).contains("<a:off x=\"47625\" y=\"95250\"/>"));
    }
}
