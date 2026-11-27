# 06 · Progress log: making `.flash` files small

Separate log for the file-size work (the main log is `05-progress-log.md`). Newest entry on top, after the `---`.

---

## 2026-11-28 · PPTX import: freeform shapes and picture-filled shapes

- **Bug:** a deck from Canva ("Corporate Agile Project Workflow.pptx") lost most of its content: its arrows, cards and pins are
  freeform shapes (`custGeom`) and its pictures are shapes with a picture fill, and the importer skipped both. Now a freeform
  becomes one polygon per sub-path (curves flattened) and a picture-filled shape becomes the picture at its place. All 10 slides
  of that file now match the original look (checked slide by slide with `fe-core/examples/pptx_import.rs`, which saves a PNG of
  every slide). Pictures that have an SVG twin use the PNG, so large ones can look soft.

## 2026-11-28 · Open PowerPoint (.pptx) files, and a new crop in Export

- **Opening .pptx** (File > Open, the Project Manager's Open, drag and drop): every slide becomes a page, the project is a
  presentation. New `fe-core/src/io/pptx/import.rs`. Text boxes become text objects (long text is wrapped to the box with the
  real font widths; text with different looks in one box becomes one object per look), shapes become shapes (about 50 PowerPoint
  presets, rounded corners, stars, arrows...), lines/arrows/connectors become lines, pictures become pictures (with their crop
  and flips), tables become cells. The theme colors, the slide master text styles, the placeholders of the layout and the
  background (color or picture) are followed. Not brought in: charts, SmartArt, videos, curves drawn by hand (these are counted,
  and the status line says how many). Test: a deck made by our own export opens again with the same shapes, text and places.
  Not tested against decks made in real PowerPoint (none on this machine).
- **Export crop redesigned** (`dialogs/export_crop.rs`): the crop box is always on the picture with 8 handles. Drag the inside to
  move it, a corner or an edge to resize it, drag outside it to start a new one; thirds lines and a size label help placing it.
  Under the picture: exact numbers X, Y, W, H, the shape list (Free, Square, 4:5, 9:16, 16:9, 3:2, 4:3, Like the picture), and
  "Whole picture". A shape stays locked while resizing and while typing numbers. The old drag-only box and the "Crop shape" row
  on the right are gone.

## 2026-11-28 · Gradient rows cut off, red "ScrollArea" marks, Reshape button

- **Gradient:** the color rows had an empty label that took the whole label column, so the slider and its number box ran past the
  edge and were cut off ("24", "95"). The color now sits in the label column and the slider lines up with the other rows. All
  slider rows share one size (`SLIDER_W` 90, `VALUE_W` 46 in `properties/mod.rs`).
- **Red text "First use of ScrollArea ID ..." and red lines in History:** egui's warning that two scroll lists have the same id
  (only drawn in the debug build). The Layers list and the History list both had no name. Every scroll list in the right column
  now has its own (`layers-list`, `history-steps`, `props-scroll`, `props-adjustment`).
- **Reshape corners** is a real button (it showed as bare text).

## 2026-11-28 · The canvas and the right column no longer change size by a few pixels

- **Bug:** what was inside the right column could change its width by a couple of pixels (for example the Text tab against the
  Shape sections, or a scroll bar appearing when a section opened), and the canvas next to it moved with it. Fix: the column
  content is now a child that does not report its size back (`app/layout.rs`), so the column is always exactly its width, and the
  scroll bars float over the content instead of taking room. The page strip got the same treatment (its height).
- Measured on screenshots: the page's left and right edges on the canvas are the same (126 and 1211) with nothing selected, a
  text on the Text tab, a rectangle, and a rectangle with Transform and Gradient folded.

## 2026-11-28 · Alignment: Properties, Keyboard Shortcuts, right-click menu

- **Properties:** the label column is one width for every row (72 px, labels left aligned, so "Words gap" fits), so the controls
  start at the same place on every row. The Rotate slider now has the same slider and number box widths as the others, "Keep
  proportions" lines up with the controls, and the second "Outline" row of the Text tab is now called "Width".
- **Keyboard Shortcuts (F1):** the key buttons were placed right after each name, so they sat at different places. Now they are
  one column; the window is narrower.
- **Right-click menu:** the Align row (with the sub menu arrow) is the same kind of row as the others now, so its icon and text
  line up.
- Seen by screenshot: Text and Properties tabs, F1 window, right-click menu. Other sections (QR, chart, gradient, effects) use
  the same label column but were not looked at one by one.

## 2026-11-28 · Text tab in Properties, icons in menus, Chart picker

- **Properties tabs are now Properties, Text, Effects, Adjust.** The text sections (words, font, size, style, color, outline,
  spacing, Text designs) moved to their own **Text** tab, which only shows while text is selected (also several texts).
- **Icons in the right-click menu** (cut, copy, paste, duplicate, group, order, align, flip...) and **pictures of the shapes in the
  Insert > Frame menu**. Not every right-click item has a matching icon: the Align row sits 2 px off the others.
- **Insert > Chart opens a picture dialog** (like Icons / Badges / Graphics / SmartArt): all 11 chart kinds drawn as real charts
  with their sample numbers; click one to insert. Code: `dialogs/chart_picker.rs`; `commands/insert.rs` has `sample_chart`, shared
  with the previews. `scripts/screenshot.ps1` can right-click now (`rclick:x,y`).

## 2026-11-28 · Google (Material) icons, 33 more graphics, window no longer grows

- **Google icons:** Insert > Icons has a new tab **Google icons**: Google's Material Icons font (2,226 icons: home, search,
  settings, share, mail, heart, cart...). Downloaded with the user's OK from github.com/google/material-design-icons (font and
  name list, Apache License 2.0), kept in `assets/fonts/` (with a README about the license) and built into the program, so
  projects look right on a PC without the font. It starts on ~250 "Popular" icons; the search box looks through all of them by
  name. The grid draws only the rows on screen. Inserted as editable text in the family "Material Icons", like the Windows icons.
  Code: `fe-core/src/library/material.rs`, `fe-app/src/dialogs/icon_picker.rs`, `commands/insert.rs`. Seen by screenshot: the
  tab, the popular list, search for "cart", inserting the shopping cart. Brand logos (Google, etc.) are not in the font.
- **33 more graphics** (148 in total) in `graphics_extra.rs`: signs and buttons (check, cross, info, warning, play, pause,
  badge, map pin), objects (eye, padlock, cloud upload, phone, laptop, camera, coffee, book, pencil, music, gem), fun (paw,
  cupcake, ice cream, rocket, planet, butterfly, fish), and a few lines / decorations. New groups: Signs and buttons, Objects, Fun.
- **Bug found on the way:** the window got a little bigger every time the program started (it saved the outer size and asked for
  it back as the inner size: +16 x +39 px per start). It now saves the inner size. A `window.json` already stretched stays as it is.

## 2026-11-28 · Even more icons, and 65 more graphics

- **Icons:** 88 more (about 395 in total), picked and named from contact sheets of the real icon font: thick arrows, filled
  symbols, tools (eyedropper, pens, eraser), more media, a new category **Numbers** (1 - 12) and more shapes.
- **Graphics:** 65 more (115 in total), in `fe-core/src/library/graphics_more.rs` (the old 50 stay in `graphics.rs`; use
  `graphics::all()` to go through all of them). New groups **Celebration** (balloons, present, firework, banner, party hat,
  streamers, stars), **Nature** (tree, mountains, sunrise, leaves, rain cloud, lightning, house, waves, daisy, snowflake,
  rainbow) and **Business** (bar chart, target, light bulb, speech bubbles, checklist, gear, medal, calendar, envelope), plus
  more lines, arrows, blobs, frames (circle, oval, heart, arch, stamp, polaroid, tape) and patterns (triangles, hearts, stars,
  hexagons, diamonds, bubbles, grid, squares, rainbow bands). All drawn on a contact sheet and looked at; the "Cycle arrows"
  and "Target" arrow are the plainest. Not seen in the picker window itself.

## 2026-11-28 · Keep proportions on, paint color for shapes, more icons

- **Keep proportions is on by default** (Properties > Transform checkbox; corner handles and the W / H boxes). Shift while
  dragging a corner does the opposite, as before. Code: `tools/state.rs`.
- **Shapes use the paint color** (the swatch at the bottom of the toolbox), both from the Shape tool and Insert > Shapes; a line
  is drawn in it too. Before they were always blue. The Brush already used it, and icons already did (the black star).
- **More icons:** 139 more (about 170 -> 309 in total), checked one by one on a contact sheet of the real font: more arrows, symbols
  (heart, bell, menu, more, sliders, QR...), communication, media, objects, places, weather, shapes (triangles, pills), and a
  new category **Text and editing** (bold, italic, underline, alignment, lists, font size...). Code: `fe-core/src/library/icons.rs`.
  Not tried in the app: the picker with the new category.

## 2026-11-28 · Icon broke the Properties column; Shape tool

- **Bug:** selecting (or adding) an icon made the right column too wide, so the canvas area slid over it and cut off the left
  of Properties and Layers. Cause: icons are text in the font "Segoe Fluent Icons, Segoe MDL2 Assets"; the font box showed that
  whole long name and grew to fit it. Found by logging the used width of each Properties section (only "Text" was wider).
  Fix: `font_picker` (options bar and Properties) shows only the first family name and cuts it to the box width.
- **Shape tool** in the toolbox (key U, above Text): the options bar has a button showing the chosen shape as a picture; it opens the gallery of
  shape pictures (the same one as Insert > Shapes), then drag on the picture to draw (Shift = square / circle; a line snaps to 15 degrees; Esc cancels; a click puts
  a default-size shape around the click). A blue outline previews the drag; the new shape is selected and the tool goes back to
  Select / Move. Code: `tools/shape.rs`, `commands/objects.rs` (`add_shape_at`, `add_shape_box`). Seen by screenshot:
  the button, the options bar, a dragged rectangle. Not tried: other shapes, a line, Shift, Esc.

## 2026-11-28 · Start-up: a small Projects window, like the C# version

- I had misread "only the project manager": it is **its own small window** (as in the C# `ProjectManagerWindow`: 1000 x 680,
  centered, title "Flash Editor - Projects", app icon), not the big editor window with a box on black. Now the program starts as
  that small window. Opening or making a project turns it into the editor window (saved size and place, maximized on the first
  start). **Close** / Esc / the X in the Projects window ends the program, as in C#. The window place saved for the editor is not
  touched by the small window. Starting with a file on the command line opens the editor directly.
- (The icon next to the manager's title was tried and removed again on request.)
- Code: `app/window.rs` (`launcher`), `FlashApp::leave_launcher`, `dialogs/frame.rs::fullscreen`. `scripts/screenshot.ps1` now
  finds the window whatever its title.

## 2026-11-28 · Start-up screen, Play button, outside fold handles, Grid view fixes

- (Replaced by the entry above.) **Start-up shows only the Project Manager.** If it is closed another way (a cancelled dialog, a file that would not open) it comes back; its **Close** button (or
  Esc) is the way into the empty editor. Starting with a file on the command line goes straight to the editor.
  Code: `app.launcher`, `app/layout.rs`.
- **The tutorial is a presentation.** New tutorial files are saved as presentations; an old plain one is made again, once.
  `read_info` now also says whether a file is a presentation.
- **Grid view:** dragging pages did nothing. Cause: the page strip under the Grid view also handled the drop (with its own
  gap, so every drop went to "first place"). Now only the Grid view handles it while it is open. Also new: a **New page**
  button in the Grid view header and a "New page" card after the last page.
- **Fold handles are outside the panels:** the right column's handle hangs on the canvas side of its left edge, the pages
  handle sits just above the strip (drawn in a layer above the canvas). Both folds are unchanged otherwise.
- **Play button** (presentations only; a line icon like its neighbours, `Play` in `icons/paths.rs`) in the row above the canvas, next to the Open button and the ruler / grid / snap
  buttons: plays from the first slide, Ctrl+click from the shown slide. The **New project** button was removed from that
  row (File > New, Ctrl+N and the Project Manager still make projects).
- Seen by screenshot: start-up screen, Play button, both handles, Grid view drag (Welcome moved to the 7th place), New page
  (15 pages). Not checked: the handles' look next to a dialog, folding while the Grid view is open.

## 2026-11-28 · Phase 16: performance pass, memory pass, installer, README

- **Performance (release build, Intel Arc A750):** exe 19.2 MiB (20.1 MB; target "under 20 MB" met in MiB, just over in
  decimal MB). Window visible 0.17 - 0.25 s after start (target 1 s). Idle CPU 0 - 16 ms in 3 s. `--bench` (4000 x 3000, 10
  layers, zoom and pan): 16.7 ms per frame average, 95 % 17.2 ms, max 35 ms = steady 60 fps. Not measured on a low-end PC.
- **Memory:** empty start 85 MB working set (private 144 MB); the bench picture peaks at 173 MB working set (271 MB private).
  Layers are stored as tiles, so a step in the undo list only costs the tiles that changed (a test proves it). Changes:
  the undo memory cap went from 1 GB to **512 MB** (the 60-step cap is unchanged), and **AI models are now let go after
  5 minutes without use** (`ai/services.rs: release_if_idle`, checked once a minute, never while a job holds a model).
  Before this they stayed loaded until the app closed, which the target forbids. A real hours-long editing session was not
  run; the cap is covered by `whole_picture_edits_cannot_pile_up_past_the_memory_limit`.
- **Installer:** one edition only, named **Flash Editor** (no AI edition). `installer/FlashEditor.iss` + `installer/build-installer.ps1`
  build `installer/output/FlashEditor-Setup-<version>.exe` (version read from `Cargo.toml`). It installs `Flash Editor.exe`,
  the three AI runtime DLLs, and `models\` / `settings\` folders in the install folder; the optional page downloads the AI
  models with SHA-256 checks; shortcuts, desktop icon, clean uninstall. New AppId, so it never overwrites the C# install.
  Built: `FlashEditor-Setup-0.1.0.exe`, 20.4 MB. Tested with a silent install into a folder (all files and the `models` /
  `settings` folders appeared), the installed exe started and ran, the silent uninstall removed the whole folder. Not tested:
  the wizard pages by eye, the model-download page, an admin install into Program Files, a clean Windows.
- **README** rewritten: features, numbers, install, build, test, structure. `scripts/measure.ps1` added (memory / CPU sampling).

## 2026-11-28 · A click no longer moves things

- Fixed: clicking an object could shift it by a pixel (any mouse jitter moved it). Now the Select / Move tool waits until the
  mouse has travelled 4 screen pixels before a press becomes a drag (move, resize, rotate, reshape, picture in frame). A click
  that never gets there changes nothing and leaves no undo step. Arrow keys still nudge by 1 px. Code: `tools/moving/mod.rs`
  (`DEAD_ZONE`).

## 2026-11-28 · Alt shows distances

- Holding **Alt** with something selected draws pink lines from the selection to the nearest object on each side (the one that
  overlaps it in that direction), or to the canvas border when nothing is there, with the distance in pixels. Code:
  `canvas/distance_guides.rs`. Built and clippy-clean; not yet checked by screenshot (needs a held key).

## 2026-11-28 · Layer type icons, Animations tab in the Layers panel, numbers on the canvas

- **Layer type icons:** every row of the layer list has a small drawn icon before the name: T (text), shape, picture, group, QR
  code, barcode, chart, plain pixels (checker), adjustment (half-filled circle). Hover for the name of the type.
- **Animations belong to the page, so they moved next to the Layers** (they are no longer in Properties): in a presentation the
  Layers panel title bar has two tabs, **Layers** and **Animations**. The Animations tab has Add Animation + Preview, the list of
  everything that plays on the slide (number, layer, effect, start, click number), **drag a row up or down to change the order**
  (a blue line shows where it lands), click to choose, the cross removes, and the settings of the chosen one (start, direction,
  duration, delay) below the list.
- **Numbers on the canvas:** while the Animations tab is open, each animated object carries small numbers at its top left corner
  (1, 2, 3 = the place in the play order; an object with two animations shows two); the chosen animation's number is blue, the
  others orange. They are not part of the picture or the export.
- Properties has three tabs again (Properties, Effects, Adjust).

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
