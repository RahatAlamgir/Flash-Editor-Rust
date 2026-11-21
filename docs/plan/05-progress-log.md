# 05 · Progress log

Newest entry on top. One entry per session, short:

```
## YYYY-MM-DD · Phase N · <title>
Done: ...
Measured: ... (frame time, memory, sizes, if relevant)
Next: ...
Problems / questions for the user: ...
```

---

## 2026-11-21 � Phase 13 � AI model store and all AI features

Done: everything the AI does, running on the real models. `fe-ai` has the model store (same URLs, sizes and SHA-256 as C#,
`models\` next to the program, asks first, progress + Cancel, deletes half-finished files), the ONNX runtime loaded only on
first use (graphics card through DirectML, processor as fallback; sessions are dropped when a model file is deleted),
IS-Net (Remove Background, Select Subject), LaMa (Fill, in 256 px pieces for big selections), Real-ESRGAN (tiled, seamless
upscaling) and MobileSAM (Click Select, the picture is studied once and remembered). In the program: Image > Remove BG (AI)
with the three sliders and a live preview (the AI looks at the picture once, moving a slider is quick); OK makes one undo
step and leaves an editable picture that remembers the original; Refine brush (R) restores or removes on that picture's own
pixels (works after moving / resizing / turning / mirroring); Select > Subject (AI) and Fill (AI) (result on a new "AI Fill"
layer); Remove Brush (J); Click Select (W: click, or drag a box; Shift adds, Alt subtracts); Image > AI Upscale (size 1-4x,
strength, time estimate; size 1 cleans only the selected layer, bigger enlarges every picture layer); Settings > AI Models
(list, size, installed, download, delete, show folder). While the AI works the window stays usable: a progress bar and
Cancel sit in the status bar. Before a download the user is asked ("one-time download of ... Download it now?").
Checked against C#: the C# code and the Rust code select the same 1530 pixels for the same click, and the real-model tests
(subject, cut-out, click, fill, upscale) pass; all 13 helper tests of the AI math are ported. The tests that need a model
skip themselves when the models are not on the PC. Tried by hand in the window: Remove BG dialog + live preview, Select
Subject, Fill (AI) (new "AI Fill" layer) and the AI Models window.
Measured: the first use of a model is slow (IS-Net on the graphics card took about 60 s once while DirectML prepared it,
then 1-3 s); the window never froze. Memory goes from about 210 MB to 550-880 MB while a model is loaded.
Added: three small libraries: `ort` (ONNX Runtime, loaded as a DLL only when an AI feature is first used), `ureq` (downloads)
and `sha2` (checks the download). The three DLLs (onnxruntime, onnxruntime_providers_shared, DirectML) must sit next to the
program: `scripts\get-ai-runtime.ps1` copies them (and the models with `-WithModels`) from the C# build; the installer in
Phase 16 will carry them.
Not yet: "Remove by Color" (the colour-based background removal) and Magic Resize; the Refine brush on layers that are not
pictures says to use the Eraser (the C# "RestoreSource" fallback is not ported); Upscale, Click Select, Remove Brush and the
Refine brush were not tried by hand with the mouse (their AI core is tested).
Next: Phase 14.
Problems / questions for the user: none.

## 2026-11-21 · Phase 12 · Retouching

Done: Clone Stamp (S), Healing Brush and Spot Healing (J), Patch (J), Dodge / Burn (O), Blur / Sharpen and Liquify (Q).
The six pixel operations, the stroke (always recomputed from the untouched pixels, so a stroke never builds up), the
Patch area, Spot Healing's search for a clean place and Liquify (Push, Bloat, Pucker) are ported from C#; I ran the C#
code on the test picture and all of them match within 1 / 255, and Spot Healing picks the same place. One undo step per
stroke; strokes stay inside the selection (Liquify, like in C#, only uses the layer). Alt+click chooses the copy-from
place (a cross shows it), Alt while painting swaps Dodge / Burn and Blur / Sharpen, [ ] change the brush size, Esc cancels a
patch. The options bar shows size, hardness and what each tool needs (opacity, Aligned, Dodge / Burn and range, Blur /
Sharpen, Liquify mode). Painting on an object layer turns it into pixels, like the other pixel tools.
Not yet: Remove Brush is an AI tool (Phase 13). Not tried by hand: Clone, Healing, Spot Healing and Patch with the mouse
(Liquify was; the Alt+click could not be sent by my screenshot script).
Next: Phase 13 (AI tools).
Problems / questions for the user: none.

## 2026-11-21 · Phase 11 · Image commands, adjustments and filters

Done: all 12 adjustments (Auto Levels, Brightness / Contrast, Exposure / Gamma, Hue / Saturation, Vibrance,
Temperature / Tint, Grayscale, Sepia, Invert, Threshold, Posterize, Recolor) and 10 filters (Gaussian / Motion Blur,
Sharpen, Enhance, Add / Reduce Noise, Pixelate, Vignette, Emboss, Find Edges) with the same math as C#. I ran the C#
code on a test picture and kept its results in `tests/fixtures/effects`: every effect, all 4 resize methods, turn,
straighten and perspective crop match within 1 / 255 (Enhance 2; the noise grain is random in both, so it is not compared).
Image tab: Image Size (4 methods, % and keep proportions), Canvas Size with the 3 x 3 anchor, Trim Edges, Upscale 2x,
Straighten, and a new "Rotate ▾" menu (90 right / left, 180, any angle, flip) because the C# commands had no button.
Crop tool (7 shapes) and Perspective Crop cut the selected layer onto a new layer and hide the original; Layer >
Crop Layer ▾ has "Crop This Object" and "Crop Layer to Selection". Effects with settings open a window with a live
preview made on a background thread (a newer slider move replaces a running preview; OK is one undo step, Cancel puts the
layer back) and only the selection changes. Adjustment layers (Adjustment, Levels, Curves) are made from the Adjust tab and
edited in Properties (sliders; a curve editor with All / Red / Green / Blue). Properties > Adjust changes the colors of one
object (the C# "Tone"; Phase 9 leftover). The tutorial page 2 now uses the real filters (and the hue shift of the
tutorial page 1 photo was wrong before: fixed).
Speed (12 megapixels, 4000 x 3000): adjustments 10-50 ms, blur 330 ms, sharpen 370 ms, Enhance 850 ms, resize to half 50-60 ms.
Different from C#: the preview is made at full size, not first at screen size (the slowest, Enhance, takes under a second
at 12 megapixels); Recolor starts from the foreground color with its three sliders (no color graph yet); the Layers panel
edits of adjustment layers moved into Properties.
Not tried by hand: Image Size / Canvas Size windows, Rotate menu, Perspective Crop with the mouse, Levels sliders.
Next: Phase 12 (retouching).
Problems / questions for the user: none.

## 2026-11-21 · Phase 10 · Insert extras

Done: QR encoder (versions 1-40, 4 levels, 3 square styles) and barcode encoders (Code 128, EAN-13, UPC-A, EAN-8,
Code 39), both with the C# tests ported; three QR codes were scanned with OpenCV and all decode. Charts (11 kinds) drawn
as vectors with a number table in Properties; 150+ icons (font symbols as text), 52 badges, 50 graphics (the C# list
has 50, not 51), 16 SmartArt layouts, each a group of shapes and text; Insert > Heading / Subheading / Body Text (text
appears with all words selected, type to replace it, right on the canvas); Brand Kit (colors, fonts, "add colors from
this picture"); the 14-page tutorial project, made on first start and opened from View > Tutorial.
Different from C#: QR codes and barcodes are inserted with a sample text and edited in Properties (no input dialog),
like the in-canvas text change. The flower graphic's petals are placed around the middle (the C# ones piled up).
Confetti uses its own random numbers, so the pieces sit elsewhere.
Tutorial page 2 first showed quick looks I made; Phase 11 replaced them with the real filters. Not tried by hand: the Icons / SmartArt dialogs by mouse, Brand Kit.
Next: Phase 11 (image size, crop, adjustments, filters).
Problems / questions for the user: none.

## 2026-11-21 · Phase 9 change · Typing text right on the canvas
The user asked for editing like Canva / Office instead of the C# text box, so the dialog is gone:
* Double-click text with Select / Move (or a text inside a group), click existing text with the Text tool, press Enter
  on a selected text, or "Edit text" in Properties: a blinking cursor appears in the text. A click with the Text tool
  on empty canvas makes a new empty text you type into (thrown away again when left empty).
* Typing, Backspace / Delete (Ctrl = word), arrows (Ctrl = by word, Shift = select, up / down keep the place on the
  line), Home / End (Ctrl = whole text), Enter = new line, Ctrl+A / C / X / V, mouse click / drag to place the cursor
  and select, double-click picks a word, Esc or a click outside finishes, Ctrl+Z takes the whole edit back. The text
  and its box update while typing; the whole edit is one undo step. Works for curved text too.
* Code: `fe-core/text` gives the places between letters (`CaretPos`) for straight and curved text and the cursor logic
  (`editing.rs`, 3 tests); `fe-app/text_edit/` holds the editor (keys, mouse, painting). 1 more test for the places.
Not tried by hand: copy / paste, drag selecting, curved text editing, editing a text inside a group.

## 2026-11-21 · Phase 9 · Text and looks
Done:
* Text (`fe-core/text`): installed fonts are found with `fontdb`, letters are shaped with `rustybuzz` (kerning) and
  drawn as outlines with tiny-skia, so fill, outline, gradient and curved text share one path. Ported: line spacing,
  letter and word gaps, left / center / right, text on a curve (`SpacedText`, `ArcText`), the box that follows the text.
  Names like "Segoe UI Black" are found as a weight of "Segoe UI" (like WPF). The size of every text box in the
  tutorial equals the size the C# app saved (width and height), and the drawing matches the C# one closely.
  (Plan said cosmic-text; the user chose the smaller fontdb + rustybuzz pair.)
* Looks (`objects/looks.rs`, `render/fx/`): drop shadow (10 presets, size), glow, inner shadow, outline, 3D with
  bevel (5 presets) and blur, all ported from `ShadowFx`, `SurfaceFx`, `ThreeDFx`. Drawn for every object and for the
  parts of a group. Compared with the C# drawings of the tutorial: shapes with shadow / glow / inner shadow / 3D /
  outline differ by 0.1-0.5 levels on average (of 255). Drawing time on a 1920 x 1080 canvas: a 200 x 200 object
  with all looks 11 ms, a 1600 x 900 object 182 ms (shadow alone 45 ms).
* Gradients: 2-8 colors, angle, radial, 11 presets, on the border, see-through overlay on pictures, Insert >
  Gradient Background. 16 text designs with pictures made by the real renderer. Saved styles ("Save this look...")
  in `settings\styles.json`, same JSON as C#.
* Frames: Insert > Frame (5 kinds), Photo Grid (10 layouts, gap / border / corners, preview), Fill Frames (pick
  pictures, one undo step), double-click a filled frame then drag to move the picture and the wheel to zoom it,
  Properties > Picture in frame.
* Text tool (T) with font / size / bold / italic in the options bar, a multi-line text box (Ctrl+Enter = OK),
  double-click text with Select / Move to change the words. Properties: Text, Text designs, Gradient, Styles, Effects.
* With several layers selected Properties shows the sections that fit all of them and a change goes to all in
  one undo step. Groups that hold text can now be taken apart (Phase 8 leftover).
* Tests: 8 new (`objects_text_looks.rs`: text size and place, gaps / curve / alignment, looks round trip and reach,
  outline, gradient editing, text designs, saved styles, collage layouts and frames). Examples: `text_sheet`
  (all 16 designs on one picture), `dump_text`, `time_looks`.
Not done yet: Properties > Adjust colors per object (brightness ... tint) comes with the Adjust tab (Phase 11);
Heading / Subheading / Body Text buttons come with the Insert tab (Phase 12). Not tried by hand: Fill Frames (file
dialog), moving the picture inside a frame, the Frame menu.
Next: Phase 10 (charts, QR codes, barcodes, icons, graphics, badges, SmartArt).
Problems / questions for the user: none.

## 2026-11-21 · Phase 8 leftovers · Curve tool, editing inside a group
Done:
* Curve tool (P), port of `CurveTool` (`fe-app/tools/curve/`, maths in `fe-core/objects/curve.rs`): click = corner,
  click and drag = smooth point, click the first point to close a filled shape, double-click or Enter ends a line,
  Backspace / Esc. On an existing curve: drag points and handles (Alt bends one side), double-click a point = smooth /
  sharp, double-click the line = new point (curve unchanged), Delete removes a point. Double-clicking a curve with
  Select / Move opens it in the Curve tool ("Edit as curve"). The result is a normal shape (fill, border, dashes, arrows).
* Editing one part inside a group (`commands/parts.rs`, `StoredObject::part / with_part / without_part`): double-click a
  group, then click parts to pick them; handles, rotate, Properties (Transform, Shape) and Delete work on the part, the
  group is fitted around its parts again. Esc leaves. The Layers panel shows an arrow on groups that lists the parts.
* Crop one object belongs to the Crop tool (Phase 11, same box and Enter / Esc handling), so it moves there.
* Tests: 3 for the curve maths, 2 for group parts.
Not done: ungrouping groups that hold text (Phase 9); renaming a part in the Layers panel.

## 2026-11-21 · Phase 8 · Objects: shapes, pictures, groups, handles, snapping
Done:
* Object model (`fe-core/objects`): objects stay stored as their C# description (`StoredObject`: JSON + pictures),
  with typed views on top: `ObjectBox`, `ShapeData` (all 38 shape kinds, fill, border, dash / dot, dash and gap,
  arrow heads, bend arc / wave, corners, inner, sweep, points, custom corners, curve nodes, picture fill),
  gradients, pictures, groups (create, members as they are now), scaling of borders / text / looks, handle maths
  (`handles.rs`: resize with the opposite side fixed, rotate with 15 degree steps, Reshape corners). Keeping the
  description means nothing the Rust version does not know yet (text, charts, shadows, 3D...) is ever lost.
* Renderer (`fe-core/render`, tiny-skia, agreed in the plan): every shape outline (`geometry.rs`, port of
  `ShapeGeometryFactory` / `ShapeVertices` / `LineCurves`), fills and gradients, borders along the outside of
  combined shapes (cloud, bubbles, crescent), dashes and round dots like WPF, arrow heads, pictures with smooth
  smaller copies, picture frames, groups with member opacity, Reshape (perspective warp, port of `QuadWarp`).
  Objects the Rust version cannot draw yet (text, charts, codes, shadows / glow / 3D) keep their saved drawing,
  which is moved, stretched and turned with them until Phases 9-10.
* Compared with the C# drawings of the tutorial: every shape, picture and group without shadows draws within 1-6
  levels (of 255) on average (`examples/compare_objects.rs`). Fixed on the way: dashed lines had round caps on
  every dash (C# rounds only the line's ends). A project with Rust-made shapes, a dashed bent arrow, a reshaped
  picture and a group opens in the C# reader as real objects (`examples/make_objects.rs`).
* `document/object_ops`: object layers, drawing on change, pixel layer -> picture, convert to pixels, group /
  ungroup, flip (objects in place, pixels across the canvas), align / distribute (port of `Alignment`).
  `layout/snapping` (port of `SnapSession`: edges, centers, canvas, grid, guides, equal spacing).
* App: Select / Move tool with the box, 8 resize handles (lines: 2, text: 4) and the rotate handle (Shift = 15
  degrees; Shift on a corner switches Keep proportions), cursors per handle, Reshape corners, picking objects by
  their box (pictures by their pixels, like C#), rubber band by turned box. Snapping with pink smart guides and gap
  labels while dragging layers or handles (Alt = no snap). Insert > Shapes gallery (pictures drawn by the
  renderer). Ctrl+G / Ctrl+Shift+G, ribbon Group / Ungroup, right-click: Group, Ungroup, Align, Distribute, Flip,
  Convert to Pixels. Paste now makes a resizable picture, like C#.
* Properties panel (`panels/right/properties/`): Transform (X, Y, W, H, Keep proportions, rotation, 90 degree
  turns, flips, Reshape / Reset), Align (3 x 3 grid, six align buttons, distribute), Shape (fill and border colors
  with the color picker, width, line style, dash, gap, bend, arc / wave, arrows, corners, hole / bite / arms, angle,
  points). Changing the size of a pixel layer turns it into a picture first; slider drags are one undo step.
* Tests: 12 new (description format, every shape draws inside its box, turned / dashed / arrow, picture and
  reshape, group / ungroup round trip, pixel -> picture, text keeps its drawing when stretched, flip, align /
  distribute, snapping). 186 in total, all pass. Screenshots: `phase8-select.png`, `phase8-transform.png`
  (photo resized and turned 37 degrees), `phase8-shapes.png` (gallery, then a star and a ring added).
Measured: memory unchanged (~210 MB on the tutorial), idle CPU 0 %. Resizing a shape or the tutorial photo
redraws it every mouse move without visible lag (release build).
Not done yet: the Curve tool (P) and "Edit as curve"; editing a part inside a group and groups listed in the Layers
panel; Crop one object; ungrouping groups that hold text (comes with text, Phase 9). Text, effects and their
Properties sections: Phase 9. Not tried by hand: snapping guides on screen (the test script cannot hold the mouse
still mid-drag), Reshape corners, every Properties control.
Next: the Phase 8 leftovers or Phase 9 (text and looks), as the user prefers.
Problems / questions for the user: none.

## 2026-11-21 · Phase 7 · Painting and masks
Done:
* `fe-core/paint/`: `brush` (the 7 brush types, settings), `coverage` (where a stroke painted: dabs every 12 % of
  the size, the maximum is kept so overlapping dabs never darken; pencil / chalk grain fixed per pixel; spray and
  splatter), `stroke` (Brush / Eraser stroke on a layer, Mask Brush stroke on a mask: always recomputed from the
  pixels at the start of the stroke, only in the touched 64 px tiles; respects the selection), `bucket` (paint
  bucket with tolerance, contiguous, selection). `document/mask_ops` (add show all / hide all, from selection,
  hide selection, invert, on / off, apply, delete, Clip to Layer Below), `LayerMask::from_selection`,
  `apply_selection`, `multiply`. Fixed `bake_mask`: pixels off the canvas now stay as they are, like C#.
* App: `tools/painting/` (Brush B, Eraser E, Mask Brush K, Paint Bucket G, Eyedropper I with Alt / right-click for
  the background color; Shift+click draws a straight line from the last stroke; [ ] change the size; brush circle on
  the canvas). Options bar: brush type list with a picture of each type (drawn by the real brush engine), size,
  hardness (dimmed for types that ignore it), opacity; bucket tolerance and contiguous. X swaps, D resets the
  colors. Color picker (`dialogs/color_picker.rs`): saturation / brightness square, hue strip, hex and RGB boxes,
  new / current, Brand Kit colors, recent colors (14), My swatches (28, right-click removes), quick swatches;
  opened from the toolbox squares and from Edit > Fill Color (which now asks for the color, like C#).
* Masks: `commands/mask.rs`; Select tab mask buttons, Layer > Clip Below and the Layer Mask ▾ list; the layer row
  shows the mask (click = on / off, red cross when off); layer right-click menu: Clip to Layer Below, Add / Delete
  Layer Mask. One undo step per stroke and per mask command (a first Mask Brush stroke adds the mask as its own
  step, like C#).
* Tests: 19 new (ported C# `BrushTypeTests`, `PaintingTests` and the selection parts of `LayerMaskTests`, plus mask
  commands, clip, mask stroke, bracket sizes). 174 in total, all pass. Screenshots: `phase7-paint.png`,
  `phase7-mask.png` (Mask Brush hiding part of the photo), `phase7-colors.png`.
Measured (release, `speed_compositor` test, 4000 x 3000 white picture, a quick drag with a mouse move every 25 px):
round 30 px 0.15 ms per move, round 150 px 0.64 ms (worst 1.0), chalk 150 px 1.0 ms (worst 1.6), spray 150 px
0.26 ms. The screen then redraws only the touched 256 px screen tiles. Memory and idle CPU unchanged (~208 MB, 0 %).
Not done yet: the Refine brush and the retouch brushes reuse this engine in Phase 12 (the C# restore strokes are
ported then). Not tried by hand: painting with every brush type in the app, the eyedropper, the bucket (covered
by tests), the Layer Mask list.
Next: Phase 8 (objects).
Problems / questions for the user: none.

## 2026-11-21 · Phase 6 · Selections, moving pixels, copy / paste
Done:
* `fe-core/selection/`: `Selection` (mask with soft values + outline for the ants; unlike C# the mask only covers
  the selection's bounds, so a small selection on a big picture costs little), rectangle / ellipse / polygon /
  mask, add / subtract / replace, invert, offset; even-odd polygon fill; outline tracing (holes included);
  Bezier pen paths; magic wand (connected or everywhere, all layers or one); quick-select dabs.
* `fe-core/paint/selection_pixels` (copy, clear, fill, keep-only for Split), `paint/pixel_move` (floating piece:
  nothing is cut at the canvas edge, a piece moved off and back comes back whole; whole-layer moves keep objects
  editable by moving their `Box`), `document/canvas_ops` (Reveal All: new canvas size, layers shifted, masks
  baked like C#). `Document` holds the selection (shared `Arc`, so undo copies cost nothing). Selecting is not
  an undo step (like C#); undoing a step brings back the selection it had.
* App: tools in `tools/selecting/` (one file per tool: marquee, lasso + polygon lasso, pen, wand + quick select)
  and `tools/moving/` (Select / Move: click to pick a layer by its pixels, Shift+click, Ctrl+drag rubber band,
  drag one or several layers, drag inside a selection moves the pixels, Shift locks the axis, arrow keys nudge
  1 / 10 px). Options bar for Magic Wand and Quick Select (tolerance, connected only, all layers, brush size,
  [ ]). Marching ants (dark line + walking white dashes, only animated while on screen), light blue tint (mask
  texture, max 2048 px), outlines while drawing, Pen handles, brush circle, frame around the picked layers.
* Commands (`commands/selection.rs`): copy / cut / paste (Windows clipboard too, via `arboard`, which eframe
  already used; pictures from other programs paste in; object layers copy as editable objects, also between
  projects), Delete / Backspace, Fill Color (foreground color for now), Select All (Ctrl+A with non-Move tools),
  Deselect (Ctrl+Shift+D, Esc), Invert (Ctrl+Shift+I), Layer Content, Split to New Layer, Reveal All. Ribbon
  buttons and a right-click menu on the canvas.
* Fixed two Phase 5 bugs found while testing: after a drag on the canvas all tool keys stopped working (the canvas
  kept the keyboard focus and the shortcut code ignored keys whenever anything had focus; now only a text box
  blocks them), and Ctrl+C / X / V for pages never arrived (egui turns them into Copy / Cut / Paste events).
* `scripts/screenshot.ps1 -Steps` drives the mouse and keys (click, drag, Shift+drag, keys, wait) for test shots.
* Tests: 22 new (ported C# `SelectionTests` and `MaskSelectionTests` + pen, wand, quick select, pixels, moving,
  Reveal All, object move). 155 in total, all pass. Screenshots: `phase6-select.png` (magic wand with holes +
  Shift-added rectangle), `phase6-move.png` (selected pixels moved half off the canvas).
Measured (release, tutorial project): memory unchanged (~208 MB). Idle CPU with marching ants on screen
~4 % of one core (125 ms over 3 s, 8 frames a second like the C# timer); without a selection ~0.5 %.
Not done yet (later phases): Click Select (AI) → Phase 13; snapping while selecting / moving → Phase 8 (with
the object handles); Paste makes a plain layer instead of a resizable picture object → Phase 8; Fill Color asks
for a color in C# → Phase 7 (color picker); "Mask from selection" buttons → Phase 7 (masks). Moving an object
far off the canvas and back may show its drawing cut at the old edge until objects are drawn by Rust (Phase 8).
Not tried by hand: Pen curves, Quick Select dragging, pasting from another program (all built, the test script
cannot draw them well).
Next: Phase 7 (painting and masks).
Problems / questions for the user: none.

## 2026-11-21 · Phase 5 · Projects, pages, Project Manager, saving
Done:
* **Real C# samples without the user's help**: `tools/csharp-fixtures` (.NET 10 console app that references
  the *built* C# libraries in `installer\build\app`, so nothing is written into the C# folder) writes
  `tests/fixtures/tutorial.flash` (the real 14-page tutorial, 421 layers, 407 objects of all 7 kinds) and
  `features.flash` (off-canvas pixels, masks on / off, Levels / Curves / basic adjustment layers, blend modes,
  locked / hidden layers, guides, two page sizes). Its `summary` command prints what the C# reader sees.
* `fe-core/io/project`: `.flash` reader (versions 1 and 2) and writer. PNGs are streamed in bands straight
  from / into tiles (no canvas-sized copies), encoded in parallel; temp file + rename; 480 px preview;
  `read_info` / `read_preview` for the Project Manager. Objects are kept as `StoredObject` (JSON + the PNGs
  they refer to) and written back unchanged; painting on such a layer turns it into pixels (like C#).
  JSON needs `serde_json` "preserve_order" (C# wants `"kind"` first) and "float_roundtrip" (exact numbers).
  **Rust-saved files open in the C# app identically: 0 differences** in the C# summary (pixels hashed,
  objects, masks, adjustments, guides) for both samples. Sizes match (tutorial 8.4 MB vs 7.8 MB, PNG
  "Balanced"). Tutorial opens in ~0.1–0.2 s and saves in ~0.4–0.55 s (release).
* History: page-list steps (`Project::edit_pages`) next to page steps; `mark_saved_at` for background saves.
  `document/page_ops.rs`: add, duplicate (new document identity, shared tiles), paste, delete (keeps one),
  move (several, order kept), rename. Project has `uid`, `path`, `is_dirty`, `from_pages`.
* **Tile size 256 → 64 px** after measuring the tutorial: tiles 288 MB → 114 MB for 79 MB of real drawing,
  same speed (flatten 66 ms either way). Screen tiles stay 256 px (`fe-gpu` `RENDER_TILE`).
* App: `settings\app.json` (undo steps, icon style, view aids, projects folder, recent list...; written when
  something changes); background `jobs` for open / save / autosave / project list; `commands/file.rs` (Open
  dialog with pictures + projects, open in background, already-open project just switches tab, Save,
  Save As, Save as Template, Close with "Save changes?", window close asks once and can "Save all", Ctrl+drop
  adds pictures as layers); `commands/pages.rs`; page strip and Grid view (`panels/pages/`: cards with real
  thumbnails, "+" in every gap, click / Ctrl / Shift selection, double-click rename, drag to reorder, "..."
  and right-click menu with the C# items, Ctrl+C / X / V / D / A and Delete while the page row has the
  keyboard); Project Manager (`dialogs/project_manager/`: cards with preview, pages, size, local date;
  search; New project; Open file; change folder; rename / Show in Explorer / remove from list / delete to the
  Recycle Bin); New Project dialog (C# sizes, background, "Start from" Blank or ★ user templates); autosave
  every minute to `settings\recovery\` and the recovery question at start; temporary File menu (the sliding
  File drawer is Phase 15); `*` on unsaved tabs; shortcuts Ctrl+N / O / S / Shift+S / W / Tab, Ctrl+Alt+N.
  Start-up shows the Project Manager (like C#), or opens the files given on the command line.
* New libraries (small, single job): `rfd` (Windows file dialogs), `trash` (Recycle Bin), `dirs` (real
  Documents folder, e.g. inside OneDrive), `chrono` (local dates), `zip`, `png`.
* Tests: 133 in total (new: 6 project-file tests on the C# samples, 5 page tests, settings tests).
Measured (release): the C# tutorial project open: Rust 208–212 MB vs C# 214 MB (before the tile change Rust
was 372 MB). Speed test still 60 fps (CPU per frame 1.7 ms, was 0.8 ms: more, smaller tiles to fingerprint).
Not done / later: renaming a project tab by double-click; the built-in "Start from" layouts and the tutorial
project at first start need objects (Phases 8–10); the File drawer (Phase 15). Not tried by hand yet: the
recovery question after a real crash, the window-close question with several unsaved projects.
Next: Phase 6 (selections).

## 2026-11-20 · Phase 4 · Undo, Layers panel, History panel
Done:
* `fe-core/history`: one `History` per `Project` (all pages share it). A step stores the page's document
  before and after (cheap: shared `Arc` tiles; `size.rs` counts only tiles not shared between the two).
  30 steps by default (1–60), ~1 GB cap (oldest go first, the newest always stays), merge of repeated steps
  with the same `MergeKey` (opacity drags), dirty flag / mark saved, jump to any step, steps of a removed
  page are forgotten and the others renumbered. `Project::edit(name, merge, |doc| ...)` wraps any change.
* `fe-core`: `Document` keeps the selected layers (active always included); `document/layer_ops.rs`: new
  ("Layer N" above the active one), duplicate selected ("X copy", above the topmost, tiles shared), delete
  (skips locked, keeps one), move up / down / front / back, drag into a gap, Merge Down (also bakes adjustment
  layers), Merge Layers (selected visible unlocked → "Merged"), Flatten, lock / unlock selected.
  `compositor::draw_layer_into_tiles` merges tile by tile (no canvas-sized buffer). `paint/thumbnails.rs`
  samples layer and page previews; `signature::layer_signature` / `page_signature`.
* App: `commands/edit.rs` (undo / redo with "Undo: <step>" in the status bar), `commands/layers.rs` (every
  layer command as one undo step, C# step names and messages). Layers panel rebuilt in
  `panels/right/layers/` (header, row, list, menu, footer): real thumbnails (made in the background after the
  mouse is released), click / Ctrl+click / Shift+click selection, double-click or F2 to rename, drag to
  reorder with a blue drop line (several selected layers move together), right-click menu, Mix and Opacity with
  undo, Select all / Lock all / Unlock all / Merge, footer buttons dimmed when they cannot act. History panel
  lists "Open" + every step; click jumps; redo-able steps dimmed. Draggable divider between Properties and
  Layers. Page strip shows real page thumbnails. Ribbon buttons are dimmed when they cannot run (Undo, Redo,
  Delete, Flatten); Edit / Layer tab layer buttons work; Order ▾ opens its list; Settings > Undo Steps window.
  Shortcuts: Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z, Ctrl+D, Ctrl+Shift+N, Ctrl+E, Ctrl+L, Ctrl+] / [, Ctrl+Shift+] / [,
  Ctrl+A (Select / Move: all layers), F2.
* Tests: 119 in total (new: 14 history tests ported from the C# HistoryTests / HistoryMemoryTests /
  ProjectHistoryTests, 11 layer-command tests).
Measured: an undo step for a dot on a 4000 x 3000 white picture holds 512 KB (two copies of one tile),
not 2 x 48 MB like a canvas-sized snapshot. Debug build with a photo and 3 layers: 68 MB working set, idle 0 ms.
Not done / later: groups, masks, rotate / flip in the layer menu (Phases 7–8); the Undo Steps setting is not
saved yet (settings file in Phase 5); other "▾" lists come with their commands.
Next: Phase 5 (.flash files, pages, Project Manager, autosave). Needs sample `.flash` files saved by the C# app.

## 2026-11-20 · Phase 3 · Fast canvas
Done:
* `fe-core`: every tile now has a **change stamp** (`TileGrid::tile_stamp`); `paint::signature::region_signature`
  fingerprints an area from stamps + layer settings (never reads pixels); `compositor::composite_region`
  (parallel bands); `Document::uid()` for caches.
* `fe-gpu` (rewritten as the canvas renderer, see architecture item 3): combined-tile textures, zoom levels
  (level n = 2^n smaller, alpha-weighted box filter), only on-screen tiles, fingerprint-based updates,
  ≤ 4 small tiles in-frame / bigger updates on background threads with the old picture or another level shown
  meanwhile, textures dropped after ~5 s unused, screen-space checkerboard. Same code on wgpu and OpenGL.
* `fe-app/canvas`: `ViewState` per page (fit-to-window with 40 px margin and max 800 % that follows the window
  until the user zooms or pans; C# zoom steps; 1.2x per wheel notch around the pointer; resize scales the
  chosen zoom), panning (Space, middle button, Hand tool), Zoom tool (click / Alt or right-click), rulers
  (C# tick rules, rotated numbers on the left ruler, blue mouse marker), grid (C# colors, every 5th line
  stronger, thinned when dense), guides (drag out of a ruler, move with Select / Move, drop outside to delete),
  grid / guides / snap settings cards (`fe-ui-kit/widgets/popup.rs`), black page border, cursor position in the
  status bar, F12 statistics overlay. Ctrl+0 / Ctrl+1 / Ctrl++ / Ctrl+- / Ctrl+R / Ctrl+' / Ctrl+; /
  Ctrl+Shift+; and the View tab buttons work. egui's own Ctrl +/- interface zoom is switched off.
* Opening pictures: drag files onto the window or pass them on the command line (new project tab; the untouched
  "Untitled" start project is replaced). `--bench` (speed test → `settings\bench.txt`), `--opengl`.
* `scripts/screenshot.ps1` gained `-AppArgs` and `-Keys` (SendKeys).
* Tests: 93 in total (new: view state, levels, tile rendering, signatures, start-up options).
Measured (release, Intel Arc A750, 4000 x 3000 test picture with 10 layers, 480 frames of zoom + pan):

| Renderer | Frame interval avg / 95 % / max | CPU per frame avg / max | Peak memory |
|---|---|---|---|
| wgpu | 16.7 / 17.1 / 29.3 ms | 0.9 / 8.0 ms | 216 MB |
| OpenGL | 16.7 / 17.2 / 33.0 ms | 0.8 / 18.1 ms | 184 MB |

(Before moving big tile updates to background threads, one zoom-level change cost a 68 ms frame.)
Same 4000 x 3000 JPEG opened in the C# app vs Rust (wgpu): working set **213 MB vs 158 MB**; idle CPU 0 for
both. Opening the JPEG took 57–64 ms. Exe 12.0 MB. At 400 % only 4 tiles are drawn.
Graphics back-ends (same photo, working set / private): default wgpu (Vulkan) 158 / 405 MB, wgpu DX12 201 /
568 MB, wgpu Vulkan 139 / 388 MB, **OpenGL 120 / 180 MB**. The large private numbers are graphics-driver
reservations. Window appears ~0.1 s after start with either renderer.
**Decided with the user: OpenGL is the default renderer, wgpu the fallback** (`--wgpu` forces wgpu; `--opengl`
is still accepted). Re-measured after the switch: OpenGL bench 16.7 ms avg frames, 0.7 ms CPU, 186 MB peak.
Not done / later: page strip and layer thumbnails are still placeholders (Phases 4–5); the Open dialog, `.flash`
files and Ctrl+drop come in Phase 5; snapping itself comes with moving objects (Phase 8).
Next: Phase 4 (undo / redo, Layers panel, History panel).

## 2026-11-20 · Phase 2 · App shell and look
Done:
* `fe-ui-kit`: color tokens (+ the 11 colorful-icon colors from `Ribbon.xaml`), `metrics.rs` (sizes from the
  XAML), fonts loaded from the Windows font folder (Segoe UI, Segoe UI Semibold, Segoe UI Symbol for ▾ ⌂, and
  **Segoe Fluent Icons / MDL2** for the ribbon glyphs — the C# ribbon uses the same font, so its icons match
  exactly), egui style (dark, round slider knob). 99 line icons converted from `Icons.xaml` by
  `scripts/convert_icons.py` into `icons/paths.rs`, drawn as tinted SVG via `egui_extras`. Widgets:
  ribbon button, tool button (active bar, corner triangle), icon button, aid toggle, panel title bar,
  number slider, checkerboard.
* `fe-app`: full window layout like `MainWindow.xaml`: ribbon (File button, 9 tabs with every C# group and
  button, search box, hide/show arrow, double-click a tab to hide), tool hint bar, toolbox (16 groups, group
  lists via triangle or right-click, FG/BG swatches with swap/reset), project tabs + Projects / New / Open +
  view-aid toggles, placeholder canvas (white page fitted, zoom shown in the status bar), page strip (PAGES,
  count, Grid view, page card, Add page), Properties / Layers (Mix, Opacity, quick buttons, layer row with eye
  and lock, footer buttons) / History (collapsible). Tool keys (V, M, L, B, J...) switch tools, cycling tools
  that share a key. Window title "<project> - Flash Editor". Window placement saved to `settings\window.json`
  on close and restored at start (first start maximized). Buttons whose commands come later say so in the
  status bar. `scripts/screenshot.ps1` starts the app, screenshots it and prints start time / memory / idle CPU.
* Tests: 5 new (tool keys and groups, every tool icon exists, window.json format).
* Screenshots: `docs/screenshots/compare/phase2-main.png`, `phase2-side-by-side.png` (C# on top, Rust below).
Measured (release, Intel Arc A750): exe **11.6 MB**; window visible after ~0.6 s; 108 MB working set;
**idle CPU 0 ms over 3 s**.
Follow-up (user feedback): the egui slider looked wrong, so `fe-ui-kit/widgets/slider.rs` now draws the
Windows 11 slider of the C# app (4 px light-blue track #59BEF6, 20 px grey ring #454545 around a 12 px blue dot
#4CC2FF that grows on hover). The user prefers the Rust number box over the C# one: keep it.
Known differences (small): Undo/Paste/etc. are not greyed out yet (they will
be when commands know whether they can run, Phase 4+); C# layer quick buttons are flatter; the Properties /
Layers divider cannot be dragged yet (Phase 4); grid / guide / snap arrow popups come in Phase 3; File drawer,
search results and menus under "▾" buttons come in Phase 15.
egui 0.37 notes: `Panel::top/left/...` replace TopBottomPanel/SidePanel; `Id::new` → `Id::unique`;
`ui.id()` → `ui.scope_id()`; clippy wants `as_chunks::<N>()` instead of `chunks_exact(N)`.
Next: Phase 3 (fast GPU canvas: zoom, pan, rulers, grid, guides). Also look at the GPU-driver start-up memory.
Problems / questions for the user: please compare the side-by-side screenshot and say what looks off.

## 2026-11-20 · Phase 1 · Core engine basics
Done (`fe-core`, no UI change):
* `imaging/`: `ColorRgba`, `PixelRect`, `PointD`, `color_math` (HSV/HSL, C#-style rounding), `BlendMode` (saved
  names match C#), blend math for all 9 modes, dense `PixelBuffer`, thumbnails, and the **sparse tile storage**
  `TileGrid<C>` (256 x 256 tiles, `Arc` copy-on-write, missing tiles = fill value, negative / off-canvas
  coordinates allowed). `TileMap` = RGBA grid; masks use a 1-byte grid.
* `document/`: `AdjustmentStyle` (brightness ... tint, Levels, Curves with tone tables), `curve` (monotone curve,
  levels table, add / remove points), `LayerMask` (sparse), `Layer` (unique `LayerId`, version counter, no
  separate "extended" buffer: off-canvas pixels just live in other tiles), `Document`, `layer_order` (drag gaps),
  `Page`, `Project`.
* `paint/compositor`: `flatten` (bands in parallel with rayon), `flatten_region`, `sample_merged`, `draw_layer`.
* `io/`: decode PNG / JPG / WEBP / BMP / GIF / ICO / TIFF / DDS with EXIF orientation and the 16384 px limit;
  encode PNG / JPG (matte) / WEBP (lossless only for now) / BMP / TIFF; `save` via temp file; format list and the
  user-facing "cannot open" hints. ICO / GIF saving → Phase 14.
* Tests: 74 in `crates/fe-core/tests/` (ported C# BlendMode, PixelBuffer, Thumbnail, Adjustment, ToneAdjustment,
  LayerMask, LayerOrder, ImageIo tests + new tile / memory / off-canvas / EXIF tests). Parts of the C# tests that
  need selections, undo or project files are left for Phases 4–6.
Measured (release, `speed_compositor` test): flatten 4000 x 3000 with 10 layers (photo, 8 blended shapes,
1 adjustment layer) = **67 ms**; one 256 px region = 9.7 ms (single thread; fine for now, the GPU does live
drawing). Layer memory **89 MB** vs **480 MB** for canvas-sized buffers like the C# version. An empty
4000 x 3000 layer or an untouched mask uses 0 bytes.
Decisions: pixels stay straight-alpha RGBA (not premultiplied) so results match the C# math exactly; C#
`Math.Round` (round half to even) is copied where results must match.
Next: Phase 2 (main window layout and theme). Copy the C# screenshots folder again first.
Problems / questions for the user: none.

## 2026-11-20 · Phase 0 · Setup
Done: Cargo workspace with the five crates (`fe-core`, `fe-ai`, `fe-gpu`, `fe-ui-kit`, `fe-app`) and their module
folders (one doc line each). eframe / egui **0.37** (note: the app trait now uses `fn ui(&mut self, ui, frame)` and
panels use `.show(ui, ..)`). Window "Flash Editor" opens maximized with the app icon (window + exe via
`winresource`); starts on wgpu and falls back to OpenGL (glow) if wgpu fails. Color tokens from `Colors.xaml` in
`fe-ui-kit/theme/colors.rs`. Log file `settings\flash-editor.log` next to the exe. `scripts/check.ps1`
(fmt + clippy -D warnings + tests) passes. `rustfmt.toml`, `.gitignore`, `git init` (nothing committed yet).
Copied the C# icon and reference screenshot (`docs/screenshots/csharp/01-editor-tutorial.jpg`; the C# folder now
has only this one, showing the new ribbon UI). Screenshot: `docs/screenshots/compare/phase0-window.png`.
Measured (debug build, Intel Arc A750): window visible ~0.4 s after start; ~102 MB memory (mostly the GPU driver /
wgpu start-up); idle CPU ~0.5 % (16 ms over 3 s). Release exe: **9.6 MB** (target under 20 MB), release build takes ~3 min
(fat LTO). Release memory: 112 MB working set, but 346 MB private (committed) memory, almost all from the GPU
driver / wgpu start-up. **Phase 3 to-do:** compare wgpu backends (DX12 vs Vulkan), low-power adapter choice and
the glow renderer to bring the start-up memory down.
Next: Phase 1 (core imaging, document / layer model, CPU compositor).
Problems / questions for the user: first git commit waits for the user's OK.

## 2026-11-20 · Planning
Done: Read the C# project (about 390 source files, 76 test files). Picked Rust + egui (user approved; 95 % look
fidelity accepted). Wrote this plan: overview, architecture, 17 phases, feature checklist, design guide, PROMPT.md.
Rust 1.100 (MSVC) is already installed on the PC.
Next: Phase 0 (workspace setup), when the user says go.
Problems / questions for the user: none yet. Phase 5 will need sample `.flash` files saved with the C# app.
