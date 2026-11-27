# 06 · Progress log: making `.flash` files small

Separate log for the file-size work (the main log is `05-progress-log.md`). Newest entry on top, after the `---`.

---

## 2026-11-28 · Preview for every template, big catalogs, project types

Done:
- **Preview for every template.** Clicking any card opens the preview window (one-page templates show the big picture only; the
  height follows the window size so tall posters fit). The double-click shortcut was removed.
- **Big catalogs.** The template grid now draws and loads only the cards on screen (plus one row around). Tested with a made-up
  catalog of 1000 templates: opens at once, 82 MB memory, 0% idle CPU. Pictures scrolled far away are dropped from memory
  (more than 120 kept) and come back from the disk cache. The catalog list itself is one small file (about 0.4 MB for 1000).
  If it ever reaches tens of thousands, the catalog can be split per type; not needed now.
- **New Project window.** "Start from" removed. New: project type buttons (Custom, Presentation, Poster, Card, Thumbnail,
  Instagram post, Story, Flyer, Resume, Banner) that set the size; changing width or height by hand makes it Custom;
  "Browse <type> templates" opens the online templates on that type.
- **My templates.** Because "Start from" is gone, the templates saved with File > Save as Template now live in
  Project Manager > Templates > My templates (a list with "Use template").
Not done: the ready-made layouts (Title slide, Quote card...) are no longer offered anywhere (the code is still in
`fe-core/src/library/layouts.rs`); they could become online templates.

## 2026-11-27 · Templates repo converted, and the page preview window

Done: tool `crates/fe-app/examples/build_templates.rs` re-saves every template as `template.fe`, makes `page-N.jpg` for every
page and `thumb.jpg`, and rebuilds `catalog.json` (size, SHA-256, pages). Run on `D:\AiBuild\Flash Editor Templates`: 9 templates,
12.6 MB -> 1.4 MB (repo folder 2.9 MB). A `CHANGELOG.md` was added there. Nothing is committed in either repo.
New window: clicking a template with more than one page opens a preview (big page, arrows, strip of pages, Use template, Esc /
Close). One-page templates work as before. Checked by screenshots against the converted folder (`FLASHEDITOR_TEMPLATES`).
Next: commit and push the templates repo (user's call; old big files stay in its history unless the history is rewritten).

## 2026-11-27 · Second round: embedded photos, and `.flash` no longer offered

Found: the user's tutorial template was 5.7 MB (my 1 MB estimate was for the 9 poster/resume templates, which hold no
photos). In the tutorial, 21 embedded photos (the original pictures behind picture layers) were 4.1 MB of 5.4 MB, still as PNG.
Done: embedded PNG photos are now stored as lossless WebP when that is smaller (about 40% less, nothing lost). Layer pictures use
a slightly stronger WebP setting. (Trying the strongest setting made saving 6x slower for 0.3% gain, so it is not used.)
`.flash` is no longer offered in Open / Project Manager / drag and drop; the program only knows `.fe`. The test files from the
C# version (`tests/fixtures/*.flash`) are still read by the code, as test data only.
Measured: tutorial 8.2 MB (C# file) -> 5.4 MB (first round) -> 3.8 MB; save takes about 3 s.
Next: photos are now 2.5 MB of the 3.8 MB. Going lower needs lossy (JPEG-like) photos, which changes the picture a little.

## 2026-11-27 · Smaller files: version 3 format, and `.flash` becomes `.fe`

Done:
- **Cropped layers.** Each layer now saves only the rectangle that has paint on it (plus where it sits on the canvas:
  `PixelsX/PixelsY`, `RestoreX/RestoreY` in `project.json`). Empty layers save no picture at all. The empty canvas around a
  small shape no longer costs anything. Hidden layers, shapes, text objects and masks are all still saved in full.
- **Lossless WebP** instead of PNG for these pictures (no quality loss). Pictures over 16 million pixels still use PNG, written
  in bands, so memory stays low for huge photos. Reading picks PNG or WebP by looking at the file, so old files open.
- **Version 3.** Files with version 2 (and the C# version's files) still open; version 3 files cannot be opened by the C# version.
- **Name `.fe`.** Saving uses `.fe`. Old `.flash` files still open (Open dialog, Project Manager list, drag and drop).
  Autosave recovery file and the template catalog path (`template.fe`) use the new name.
Measured: the 9 templates, re-saved by this program: 12.6 MB -> 1.43 MB total (art-exhibition 690 KB -> 84 KB, new-year-party
2.2 MB -> 143 KB, modern-resume 2.4 MB -> 167 KB). `scripts\check.ps1` passes (existing round-trip tests cover save and open).
Next: convert the templates repo (`D:\AiBuild\Flash Editor Templates`) to `.fe`; try the new files by hand in the app.
Problems / questions for the user: git history of the templates repo still holds the old big files (see chat).

## 2026-11-27 · Investigation: where the bytes go

Looked at the 9 templates in `D:\AiBuild\Flash Editor Templates` (12.6 MB of pictures, 17 MB repo with git).

Found:
- A `.flash` is a zip: `project.json` (small, 13-110 KB) + one PNG per layer. **The pictures are 99% of the size.**
- Every layer is saved as a PNG the **size of the whole canvas** (2700x3600 on posters), even when the layer is a
  97x97 px sparkle. The empty area alone costs about 39 KB per layer. Shapes and text layers (381 of 398 layers) make
  up 12.2 of the 12.6 MB; real photo layers are only 0.4 MB.
- Test with the real files (crop each layer to what is visible, then re-pack):
  | Way of storing the layer pictures | All 9 templates |
  |---|---|
  | Now (whole canvas, PNG) | 12.63 MB |
  | Cropped to the visible part, PNG | 3.38 MB (-73%) |
  | Cropped to the visible part, lossless WebP | 1.00 MB (-92%) |
- The shape / text objects (`Object` in `project.json`) are kept as data only; this program does not redraw them, so the
  saved picture is still needed. Nothing is dropped, so PowerPoint animation later can still use the objects.

Next: user chooses the way (see the options asked in chat), then implement, measure, and rebuild the templates repo.
