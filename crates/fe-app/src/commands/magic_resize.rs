//! Image > Magic Resize (C# `ImageCommands.MagicResizeDocument`): the design becomes another size and keeps its layout.
//! Objects stay editable (they keep their proportions and relative place; text scales), layers of plain pixels are
//! resampled. One undo step, for this page or every page.

use fe_core::document::canvas_ops::{self, transform_all};
use fe_core::layout::magic_resize::{map_object, map_pixels};

use crate::app::FlashApp;
use crate::dialogs::{Dialog, MagicResizeForm};

/// The ribbon button: opens the window.
pub fn open(app: &mut FlashApp) {
    if app.dialog.is_none() {
        let (w, h) = (app.doc().width(), app.doc().height());
        app.dialog = Some(Dialog::MagicResize(MagicResizeForm::new(w, h, app.project().pages().len())));
    }
}

/// OK in the window.
pub fn run(app: &mut FlashApp, width: u32, height: u32, all_pages: bool) {
    let max = canvas_ops::MAX_SIDE as u32;
    if width == 0 || height == 0 || width > max || height > max {
        app.status = format!("Enter a width and height between 1 and {max}.");
        return;
    }
    app.tools.reset_active();
    let active = app.project().active_index();
    let mut changed = 0;
    app.project_mut().edit_pages("Magic Resize", |pages, _| {
        for (i, page) in pages.iter_mut().enumerate() {
            if !all_pages && i != active {
                continue;
            }
            let doc = &mut page.doc;
            let (old_w, old_h) = (doc.width(), doc.height());
            if (old_w, old_h) == (width, height) {
                continue;
            }
            transform_all(
                doc,
                &|p| map_pixels(p, width, height),
                Some(&|o, _, _| map_object(o, old_w, old_h, width, height)),
            );
            changed += 1;
        }
    });
    app.current_view_mut().fit();
    app.status = if changed == 0 {
        "The picture already has that size.".into()
    } else {
        format!(
            "Magic Resize: {changed} page{} now {width} x {height} px. Ctrl+Z undoes it.",
            if changed == 1 { "" } else { "s" }
        )
    };
}
