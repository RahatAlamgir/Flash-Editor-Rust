# 06 · Progress log: making `.flash` files small

Separate log for the file-size work (the main log is `05-progress-log.md`). Newest entry on top, after the `---`.

---

## 2026-11-28 · Animation pane moved into Properties, smaller fold handles

- The **Animation pane popup is gone**. Everything it did is in the **Animation** tab of the Properties panel: Add Animation and
  Preview at the top, **On this slide** (every animation in play order with its layer, effect, start and click number; click to
  choose, the cross removes), and the settings of the chosen one (start, direction, duration, delay, earlier / later).
  Adding an animation switches the right column to that tab (and unfolds it). The "Animation Pane" button of the ribbon is gone;
  Preview Animations stays.
- The fold handles are a little smaller (14 x 48 px on the right column, 48 x 14 px on the page strip, thinner chevron).

## 2026-11-28 · Right column in tabs, panels that fold away

- The right column (was one long Properties list) now has tabs side by side under its title: **Properties** (Transform, Align,
  Text, Gradient, Picture in frame, QR / Barcode / Chart, Shape), **Effects** (Styles and Effects), **Adjust** (blur, tone...) and, for
  presentations, **Animation** (the animations of the selected layer: add, start, duration, delay, direction, order, remove,
  preview, Animation pane). Effects and Adjust say so when the layer is plain pixels.
- **Fold away:** a handle (a small pill with a chevron that turns blue on hover) in the **middle** of the right column's left edge
  folds the column to a thin strip, and one in the middle of the page strip's top edge folds the pages to a thin "Pages (n)"
  bar; the same handle (or the bar) unfolds them. The panels **glide** (about a quarter of a second, eased) instead of jumping: the
  full panel slides out of view while the space it leaves goes to the canvas. (Not remembered between runs.)

## 2026-11-28 · Presentations stage B: object animations

Done:
- **Two tabs** for presentations: **Transitions** (the slide transition, as before) and **Animations** (new).
- **Animations tab:** "Add Animation" (a list of 17 effects: Entrance = Appear, Fade In, Fly In, Float In, Zoom In, Wipe In,
  Grow & Turn; Emphasis = Pulse, Spin, Grow / Shrink, Teeter; Exit = Disappear, Fade Out, Fly Out, Float Out, Zoom Out, Wipe Out)
  adds the effect to the selected layer; Start (On click / With previous / After previous), Duration, Delay and, for Fly and Wipe,
  the direction; Move Earlier / Later, Remove; **Animation Pane** (a window listing the slide's animations in order with the
  layer, effect, start and click number; click one to choose it; the cross removes it); **Preview Animations** (plays the slide
  once inside the window); From Beginning / From Current Slide. Every change is one undo step.
- **Slideshow** plays them: a click plays the next animation step, and when the slide has no more, goes to the next slide. Steps
  that do not start on a click play by themselves when the slide appears. Going back shows the earlier slide as it ended.
  Layers with an entrance effect are hidden until it plays.
- **Saved** in the `.fe` file per page (`Animations`, each pointing at its layer by position); copying a page keeps them on the
  copied layers; a deleted layer's animations are ignored.
- **PowerPoint export** (the editable option) writes `<p:timing>`: click steps, with-previous / after-previous, delay, duration,
  and each effect with PowerPoint's own behaviors (visibility, fade, wipe, fly, zoom, spin, scale). Tests check the XML is
  well formed with unique ids.
Limits: an animated layer is drawn as a normal picture (its blend mode and mask are not applied while it animates); in the
picture-mode PowerPoint export there are no object animations (the whole slide is one picture); slides that use blend modes or
adjustment layers are exported as one picture too, so they have no animations either. Effects apply to one layer at a time.
Checked by screenshots: the tabs, adding Fly In to a layer, the pane, the timing boxes, the animation preview. NOT checked: how
each effect looks while moving, and opening the exported .pptx in PowerPoint (the exact XML may need a fix there).

## 2026-11-28 · Presentations: Animation tab, slide transitions, slideshow (plan: 07-animation.md)

Done (stage A of the plan):
- **Project type.** Projects can be presentations (saved in the file as `ProjectType`). New Project > Presentation, the online
  Presentation templates (rebuilt, see the templates CHANGELOG) and File > Presentation (on / off) set it.
- **Animation tab** (shown only for presentations, between Filter and View): a transition for the shown slide (None, Fade, Push,
  Wipe, Cover, Uncover, Zoom, Split), direction, duration, "On click or key", "After N s", Apply to All, Preview Transition,
  From Beginning, From Current Slide. Choosing a transition plays a preview at once. Every change is one undo step.
- **Slideshow.** F5 (from start), Ctrl+F5 (from this slide): full screen; click / Right / Space / Enter / Page Down = next, Left /
  Backspace / Page Up / right-click = back, Esc = stop; automatic advance when "After" is set; a click skips a running movement.
- **PowerPoint export** writes `<p:transition>` for each slide (fade, push, wipe, cover, pull, zoom, split; speed, advance on
  click, advance time), in the editable and the picture export. Unit tests cover the XML and the save / open round trip.
Checked by screenshots: the tab appears for a Presentation project and not for others; the slideshow opens full screen.
NOT checked by eye: how each of the 8 transitions looks while moving (the preview is too short to catch with the screenshot
tool), and opening the exported .pptx in PowerPoint.
Not done (stage B): object animations (entrance / emphasis / exit effects, the animation pane, `<p:timing>` export). Waiting for
the user to confirm.

## 2026-11-28 · Toolbox cleanup

Removed the tools Paint Bucket (G), Eyedropper (I), Hand (H) and Zoom (Z), with their options and code. Panning still works with
Space + drag or the middle mouse button; zooming with the mouse wheel and View > Zoom In / Zoom Out (Ctrl+plus / Ctrl+minus).
The color-picker's own Eyedropper button stays (it is a different thing).
The two overlapping squares (foreground / background, swap, default colors, keys X and D) became one square, the **paint color**
(tooltip "Paint color"), because it is still used by the Brush, the Curve pen, the Mask Brush, the starting color of new text and
icons, and Adjust > Recolor. The background color had no use left (only the Bucket / Eyedropper and the default matte of
JPEG export, which is now always white), so it is gone. No "fill with foreground / background" commands exist; Edit > Fill Color
(which opens the color picker) is unchanged.

## 2026-11-28 · One color picker everywhere, with an eyedropper

Done: the New Project background used a basic built-in picker (no swatches, no compare). It now opens the program's own color
picker (new / current compare, hex and RGB, brand, recent, saved and quick swatches), shown over the New Project window.
New **Eyedropper** button in that picker, so every place that picks a color has it (toolbox colors, fill, shape, text, effects,
gradient, charts / codes, Brand Kit, New Project). How it works: the dialogs are hidden for a moment, the program takes a
picture of its own window, shows it full size with a magnifier (11 x 11 pixels, hex value) at the mouse, and the clicked pixel
becomes the color. Esc or right-click cancels. It picks from anything inside the Flash Editor window (not from other programs).
Files: `app/eyedropper.rs`, `dialogs/color_picker.rs`, `dialogs/new_project.rs`.
Checked by screenshots: picking the blue slider knob gave #4CC2FF in the picker. The magnifier itself was not seen in a screenshot.

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
