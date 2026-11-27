//! The export / import formats against files written by the C# program's own code (`tests/fixtures/io`, made with
//! `tools/csharp-fixtures io`): Photoshop and SVG files from C# open in Rust with the same layers; the PDF and PowerPoint
//! files Rust writes have the same structure as the C# ones; the animated GIF decodes to nearly the same pictures.

use std::io::Read;
use std::path::PathBuf;

use fe_core::imaging::{ColorRgba, PixelBuffer, PixelRect};
use fe_core::io::{gif, pdf, pptx, psd, svg};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/../../tests/fixtures/io/{name}", env!("CARGO_MANIFEST_DIR")))
}

/// The first page of the tutorial page "Adjust" as the C# flattened it (RGBA, 1280 x 720).
fn flat() -> PixelBuffer {
    PixelBuffer::from_raw(1280, 720, std::fs::read(fixture("page-flat.rgba")).expect("the fixture")).expect("size")
}

fn zip_part(bytes: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip");
    let mut text = String::new();
    zip.by_name(name).expect("part").read_to_string(&mut text).expect("text");
    text
}

#[test]
fn a_photoshop_file_from_csharp_opens_with_the_same_layers() {
    let doc = psd::open(fixture("features.psd")).expect("opens");
    assert_eq!((doc.width(), doc.height()), (640, 400));
    // name, opacity, visible, bounds as the C# reader reports them
    let expected: [(&str, f64, bool, PixelRect); 5] = [
        ("Background", 1.0, true, PixelRect::new(0, 0, 640, 400)),
        ("Off canvas", 1.0, true, PixelRect::new(540, 40, 100, 120)),
        ("Masked", 0.749, true, PixelRect::new(3, 0, 637, 400)),
        ("Mask switched off", 1.0, true, PixelRect::new(20, 300, 200, 80)),
        ("Hidden", 1.0, false, PixelRect::new(0, 0, 640, 400)),
    ];
    assert_eq!(doc.layers().len(), expected.len());
    for (layer, (name, opacity, visible, bounds)) in doc.layers().iter().zip(expected) {
        assert_eq!(layer.name, name);
        assert!((layer.opacity() - opacity).abs() < 0.003, "{name}: {}", layer.opacity());
        assert_eq!(layer.visible, visible, "{name}");
        assert_eq!(layer.visible_bounds(), bounds, "{name}");
    }
    assert_eq!(doc.layers()[2].blend, fe_core::imaging::BlendMode::Multiply);
    assert_eq!(doc.layers()[4].blend, fe_core::imaging::BlendMode::SoftLight);
}

#[test]
fn a_photoshop_file_written_by_rust_reads_back_the_same_picture() {
    let doc = psd::open(fixture("features.psd")).expect("opens");
    let again = psd::read(&psd::write(&doc)).expect("reads its own file");
    assert_eq!(again.layers().len(), doc.layers().len());
    let a = fe_core::paint::compositor::flatten(&doc);
    let b = fe_core::paint::compositor::flatten(&again);
    let worst = a.data().iter().zip(b.data()).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0);
    assert!(worst <= 1, "the flattened picture is the same, worst difference {worst}");
}

#[test]
fn an_svg_file_from_csharp_opens_as_the_same_kinds_of_objects() {
    let text = std::fs::read_to_string(fixture("page.svg")).expect("the fixture");
    let doc = svg::import(&text).expect("opens");
    assert_eq!((doc.width(), doc.height()), (1280, 720));
    // C#: a Background layer, then the picture of the first layer, then one layer per element (44 in all).
    assert_eq!(doc.layers().len(), 44);
    let kinds: Vec<&str> = doc
        .layers()
        .iter()
        .skip(1)
        .take(10)
        .map(|l| l.object().map_or("-", fe_core::objects::StoredObject::kind))
        .collect();
    assert_eq!(kinds, ["image", "shape", "shape", "text", "text", "shape", "text", "shape", "text", "text"]);
    let header = doc.layers()[2].object().and_then(|o| o.box_()).expect("header box");
    assert_eq!((header.x, header.y, header.width, header.height), (0.0, 0.0, 1280.0, 112.0));
}

#[test]
fn a_pdf_has_the_same_structure_as_the_csharp_one() {
    let csharp = std::fs::read(fixture("pages.pdf")).expect("the fixture");
    let ours = pdf::build(&[flat(), flat()], 90, 96.0, None).expect("built");
    // Everything outside the JPEG pictures is the same text.
    let text_of = |bytes: &[u8]| -> String {
        let s = String::from_utf8_lossy(bytes).into_owned();
        s.lines()
            .filter(|l| {
                l.starts_with("<<")
                    || l.contains(" 0 obj")
                    || l.contains("endobj")
                    || *l == "endstream"
                    || l.starts_with("q ")
                    || *l == "%%EOF"
            })
            .map(|l| l.chars().take(260).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let strip = |s: String| -> String {
        // The JPEG stream length differs, so it is left out of the comparison.
        s.split(" /Length ").map(|p| p.split_once(" >>").map_or(p, |(_, rest)| rest)).collect::<Vec<_>>().join("")
    };
    assert_eq!(strip(text_of(&ours)), strip(text_of(&csharp)));
}

#[test]
fn a_picture_presentation_matches_the_csharp_xml_part_by_part() {
    let csharp = std::fs::read(fixture("picture.pptx")).expect("the fixture");
    let ours = pptx::build_pictures(&[flat()], 90, false, None).expect("built");
    for part in [
        "[Content_Types].xml",
        "_rels/.rels",
        "ppt/presentation.xml",
        "ppt/_rels/presentation.xml.rels",
        "ppt/theme/theme1.xml",
        "ppt/slideMasters/slideMaster1.xml",
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        "ppt/slideLayouts/slideLayout1.xml",
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        "ppt/slides/slide1.xml",
        "ppt/slides/_rels/slide1.xml.rels",
    ] {
        assert_eq!(zip_part(&ours, part), zip_part(&csharp, part), "{part}");
    }
}

#[test]
fn an_editable_presentation_has_the_same_shapes_and_text_as_the_csharp_one() {
    let csharp = std::fs::read(fixture("editable.pptx")).expect("the fixture");
    let pages = fe_core::tutorial::build_pages();
    let ours = pptx::build_editable(&[&pages[2].doc], pptx::EditableOptions::default()).expect("built");
    let (a, b) = (zip_part(&ours, "ppt/slides/slide1.xml"), zip_part(&csharp, "ppt/slides/slide1.xml"));
    for needle in
        ["<p:sp>", "<p:pic>", "txBox=\"1\"", "prst=\"rect\"", "prst=\"roundRect\"", "prst=\"ellipse\"", "<a:r>"]
    {
        assert_eq!(a.matches(needle).count(), b.matches(needle).count(), "{needle}");
    }
    assert_eq!(zip_part(&ours, "ppt/presentation.xml"), zip_part(&csharp, "ppt/presentation.xml"));
    // The texts of the text boxes are the same, in the same order.
    let texts = |s: &str| {
        s.split("<a:t>").skip(1).filter_map(|p| p.split("</a:t>").next()).map(str::to_owned).collect::<Vec<_>>()
    };
    assert_eq!(texts(&a), texts(&b));
}

#[test]
fn an_animated_gif_keeps_the_picture_and_the_settings_of_the_csharp_one() {
    let csharp = std::fs::read(fixture("anim.gif")).expect("the fixture");
    let original = flat();
    let ours = gif::encode_animation(&[flat(), flat()], 500, true).expect("built");
    let (a, b) = (
        fe_core::io::decode::load_bytes(&ours, Some("gif")).expect("ours"),
        fe_core::io::decode::load_bytes(&csharp, Some("gif")).expect("csharp"),
    );
    assert_eq!((a.width(), a.height()), (b.width(), b.height()));
    // 256 colors cannot be exact, but the picture must stay close to the original (C# dithers; this encoder does not).
    let error = a.data().iter().zip(original.data()).map(|(x, y)| u64::from(x.abs_diff(*y))).sum::<u64>() as f64
        / original.data().len() as f64;
    assert!(error < 6.0, "the 256-color picture differs from the page by {error:.2} on average");
    let has = |bytes: &[u8], what: &[u8]| bytes.windows(what.len()).any(|w| w == what);
    assert_eq!(has(&ours, b"NETSCAPE2.0"), has(&csharp, b"NETSCAPE2.0"), "both repeat forever");
    assert_eq!(has(&ours, &[0xF9, 0x04]), has(&csharp, &[0xF9, 0x04]), "both write frame delays");
    // 500 ms = 50 hundredths of a second in the first frame's delay.
    let at = ours.windows(2).position(|w| w == [0xF9, 0x04]).expect("graphic control");
    assert_eq!(u16::from_le_bytes([ours[at + 3], ours[at + 4]]), 50);
    let _ = ColorRgba::WHITE;
}

/// Writes the files the C# reader is then asked to open (`tools/csharp-fixtures readdoc`): set FE_WRITE_IO to a folder.
#[test]
#[ignore = "only writes files for a manual check with the C# program"]
fn write_files_for_the_csharp_reader() {
    let Some(folder) = std::env::var_os("FE_WRITE_IO") else { return };
    let folder = PathBuf::from(folder);
    std::fs::create_dir_all(&folder).expect("folder");
    let pages = fe_core::tutorial::build_pages();
    psd::save(&psd::open(fixture("features.psd")).expect("opens"), folder.join("features-rust.psd")).expect("saved");
    svg::save(&pages[2].doc, folder.join("page-rust.svg")).expect("saved");
}

#[test]
fn remove_by_color_matches_the_csharp_result_on_a_photo() {
    use fe_core::effects::REMOVE_BY_COLOR;
    // The photo as the C# program decoded it (JPEG decoders differ by a level or two, which moves the edge).
    let tee = PixelBuffer::from_raw(
        440,
        439,
        std::fs::read(fixture("tee-remove-by-color.rgba.input")).expect("the C# input"),
    )
    .expect("size");
    let expected = std::fs::read(fixture("tee-remove-by-color.rgba")).expect("the C# result");
    let ours = REMOVE_BY_COLOR.apply(&tee, &REMOVE_BY_COLOR.defaults());
    assert_eq!(ours.data().len(), expected.len());
    // The alpha channel (what the effect changes) matches within 2 / 255 on nearly every pixel; colors are untouched.
    let alpha_diff: Vec<u8> = ours
        .data()
        .iter()
        .skip(3)
        .step_by(4)
        .zip(expected.iter().skip(3).step_by(4))
        .map(|(a, b)| a.abs_diff(*b))
        .collect();
    let worst = alpha_diff.iter().copied().max().unwrap_or(0);
    let off = alpha_diff.iter().filter(|d| **d > 2).count();
    assert!(
        off * 1000 < alpha_diff.len(),
        "{off} of {} pixels differ by more than 2 (worst {worst})",
        alpha_diff.len()
    );
    assert_eq!(&ours.data()[..3], &expected[..3], "colors stay");
}
