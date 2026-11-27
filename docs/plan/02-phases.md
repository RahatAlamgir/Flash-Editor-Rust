# 02 · Build phases

17 phases (0 to 16). Each phase ends with a program that **builds, runs and passes `cargo test`**, so there is always
something to look at. A phase may take several sessions; log each one in `05-progress-log.md`.

Every phase has:
* **Prompt**: what to say to start it (after the general prompt in `PROMPT.md`).
* **C# to read**: where the behaviour is defined (paths under `D:\AiBuild\Flash Editor\src\`).
* **Done when**: what must be true before moving on. The user checks the visible parts.

---

## Phase 0 · Setup

**Prompt:** "Do Phase 0: set up the Rust workspace."

* Create the Cargo workspace and the five crates from `01-architecture.md` with empty modules.
* Workspace `Cargo.toml`: shared dependency versions, release profile, `clippy` + `rustfmt` settings.
* `fe-app` opens an empty eframe window titled "Flash Editor" with the app icon.
* Copy reference screenshots from the C# `docs/screenshots` into `docs/screenshots/csharp/`.
* Small script `scripts/check.ps1`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
* `git init`, `.gitignore` (target/, models/, *.onnx), first commit (ask the user first).

**Done when:** `cargo run -p fe-app` shows a window; `scripts/check.ps1` passes; README explains the structure.

---

## Phase 1 · Core engine basics

**Prompt:** "Do Phase 1: core imaging, document and layer model, CPU compositor."

**C# to read:** `Core/Imaging/*`, `Core/Documents/*`, `Core/Painting/Compositor.cs`, `Core/IO/ImageDecoder.cs`,
`ImageEncoder.cs`, `ImageFileFormat.cs`.

* `imaging/`: `ColorRgba`, `PixelRect`, `PointD`, `Tile` (256 x 256 RGBA8), `TileMap` (sparse, `Arc` copy-on-write,
  off-canvas tiles allowed), a plain `PixelBuffer` for small temporary images, conversion between them.
* Blend math for all modes (Normal, Multiply, Screen, Overlay, Soft light, Hard light, Darken, Lighten, Difference),
  color math (HSV, HSL), thumbnails.
* `document/`: `Document`, `Layer`, `LayerMask`, `AdjustmentStyle`, `CurveMath`, `LayerOrder`, `Page`, `Project`.
* `paint/compositor.rs`: flatten a document (opacity, blend, masks, adjustment layers) — the reference used by export.
* `io/`: decode PNG / JPG / WEBP / BMP / GIF / ICO / TIFF into a `TileMap`; encode PNG / JPG.
* Port the matching tests (`tests/.../Imaging`, `Documents`).

**Done when:** tests pass; a test opens a photo, adds two layers with different blend modes and the flattened
result matches the C# blend math; memory test shows an empty 4000 x 3000 layer uses (almost) no tile memory.

---

## Phase 2 · App shell and look

**Prompt:** "Do Phase 2: main window layout and theme, matching the C# screenshots."

**C# to read:** `App/Themes/*.xaml` (colors, styles, ribbon, panels, `Icons.xaml` = 95 icon path geometries),
`Views/MainWindow.xaml`, `Views/Panels/RibbonBar*`, `ToolboxPanel*`, `OptionsBar*`, `FileDrawer*`, `ViewAidsBar*`.
Look at `docs/screenshots/csharp/`.

* `fe-ui-kit/theme`: color tokens (see `04-design-guide.md`), fonts (Segoe UI from the system), spacing, rounding,
  egui `Visuals` and `Style`.
* `fe-ui-kit/icons`: convert every geometry in `Icons.xaml` to SVG path data (a small build script), render with
  `resvg` into a texture atlas at the right size; plain and colorful icon styles.
* Layout: ribbon (File button + tabs Edit, Select, Image, Layer, Insert, Adjust, Filter, View, Settings, search box),
  options/hint bar, toolbox column (with tool groups and the small triangle), project tabs, canvas area,
  right column (Properties, Layers, History sections), page strip, status bar. All empty / placeholder content.
* Window size, position and maximized state saved in `settings\window.json` next to the exe (as C#).

**Done when:** a side-by-side screenshot of the Rust window and the C# screenshot look like the same program
(user approves). Idle CPU is ~0 %.

---

## Phase 3 · Fast canvas

**Prompt:** "Do Phase 3: the GPU canvas with zoom, pan, rulers, grid and guides."

**C# to read:** `Views/Editor/CanvasView*`, `RulerControl.cs`, `AidsOverlay.cs`, `Rendering/CheckerboardBrush.cs`,
`Tools/Navigation/*`, `ViewModels/ViewAids.cs`.

* `fe-gpu`: per-layer tile textures, dirty-tile upload, compositing shader with opacity, blend modes, masks and
  adjustment layers; mip levels for zoomed-out views; checkerboard behind transparent areas.
* Canvas view in `fe-app/canvas`: zoom (wheel, Z tool, fit, 100 %), pan (Space+drag, H tool), status bar zoom %
  and size, rulers, grid, guide lines dragged from the rulers.
* `glow` fallback: if wgpu cannot start, composite the visible area on the CPU (reference compositor).
* Benchmarks: open a 4000 x 3000 photo with 10 layers; measure frame time while zooming/panning and memory.
  Compare with the C# app on the same file; record numbers in the progress log.

**Done when:** smooth 60 fps zoom/pan on a large picture; numbers recorded; fallback path tested.

---

## Phase 4 · Undo, layers panel, history panel

**Prompt:** "Do Phase 4: undo/redo engine, Layers panel and History panel."

**C# to read:** `Core/History/*`, `ViewModels/LayerViewModel.cs`, `Commands/LayerCommands.cs`,
`Views/Panels/LayersPanel*`, `HistoryPanel*`, `Core/Documents/LayerState.cs`.

* History engine with all step kinds, merging, step limit (setting, default 30, 1–60), ~1 GB cap, one history per
  project (all pages share it), dirty flag.
* Layers panel: list with thumbnails (debounced, background), eye, lock, rename (double-click), opacity slider +
  number box, blend list, add / duplicate / delete / move up/down / merge down / merge selected / flatten,
  multi-select (Ctrl / Shift), drag to reorder with the drop line, groups with expandable parts, right-click menu.
* History panel: click a step to jump.

**Done when:** every layer operation works and undoes; tests for history merge, limits and memory cap.

---

## Phase 5 · Projects, pages, Project Manager, saving

**Prompt:** "Do Phase 5: .flash files, pages, project tabs, Project Manager, autosave and crash recovery."

**C# to read:** `Core/IO/Project/*`, `App/Services/ProjectLibrary.cs`, `ProjectEntry.cs`, `AutoSaveService.cs`,
`AppSettings.cs`, `DocumentTemplates.cs`, `UserTemplates.cs`, `Views/Dialogs/ProjectManagerWindow*`,
`NewImageDialog*`, `Views/Panels/PageGridView*`, `ViewModels/ProjectGroup.cs`, `PageActions.cs`,
`Commands/FileCommands.cs`, `PageCommands.cs`.

* `.flash` reader/writer compatible with the C# version (see architecture). **First ask the user to save sample
  projects from the C# app** into `tests/fixtures/` (or create them with the C# test helpers).
* Several projects as tabs; page strip with thumbnails, + Page, reorder by drag, rename, duplicate, delete,
  multi-select and copy/cut/paste of pages (also between projects), Grid view.
* Project Manager window (cards with preview, page count, date; search; new project with sizes and layouts;
  rename / show in Explorer / remove / delete to Recycle Bin), opened at start.
* New picture dialog with the size templates and "Start from" layouts; Save as Template.
* Open by drag and drop (Ctrl = add as layers). Autosave every minute in the background, crash recovery prompt.
* Settings (`settings\app.json`) next to the exe: recent colors, swatches, undo steps, snapping, icon style...

**Done when:** every sample C# project opens and looks the same, saves, and reopens in the C# app.

---

## Phase 6 · Selections

**Prompt:** "Do Phase 6: all selection tools, moving pixels, copy/paste."

**C# to read:** `Core/Selections/*`, `Tools/Selecting/*` (except MoveTool's object parts), `Commands/EditCommands.cs`,
`MaskCommands.cs`, `App/Services/ClipboardService.cs`, `Core/Painting/SelectionPixels.cs`.

* Rectangle, ellipse, lasso, polygon lasso, pen (curves), magic wand, quick-select brush, select layer content;
  Shift adds, Alt subtracts; marching ants (animated only while visible).
* Move selected pixels (nothing cut at the canvas edge; Reveal All), copy, cut, paste, delete, fill, invert,
  deselect, Split to New Layer. Clipboard with other programs (images).

**Done when:** all tools work with undo; ported selection tests pass.

---

## Phase 7 · Painting and masks

**Prompt:** "Do Phase 7: brush engine, painting tools, color picker, layer masks."

**C# to read:** `Core/Painting/*`, `Tools/Painting/*`, `Tools/Options/BrushOptions.cs`, `BucketOptions.cs`,
`Views/Dialogs/ColorPickerDialog*`, `ViewModels/ColorsViewModel.cs`, `Core/Documents/LayerMask.cs`.

* Brush types: round, square, calligraphy, pencil, chalk, spray, splatter (with previews); eraser; bucket;
  eyedropper; foreground / background colors; HSV color picker with recent colors, swatches and Brand Kit colors.
* Painting writes only touched tiles; strokes are smooth at 60 fps on large pictures (measure).
* Layer masks: add, Mask Brush (K), enable/disable, Clip to Layer Below. Painting on an object turns it into pixels.

**Done when:** painting feels instant on a 4000 x 3000 picture; one undo step per stroke.

---

## Phase 8 · Objects, part 1: shapes and transforms

**Prompt:** "Do Phase 8: object layers, shapes, the Select/Move tool, handles, snapping, groups."

**C# to read:** `Core/Objects/*` (ObjectBox, ShapeObject, ShapeVertices, CurvePath, LineCurves, GroupObject,
ImageObject, ObjectStyle, ObjectTransforms), `App/Rendering/ShapeGeometryFactory.cs`, `ObjectRenderer.cs`,
`Tools/Selecting/MoveTool.cs`, `MultiMover.cs`, `TransformHandles.cs`, `Tools/SnapDrag.cs`,
`Core/Snapping/SnapSession.cs`, `Core/Layout/Alignment.cs`, `Tools/Shapes/CurveTool.cs`,
`Views/MainWindow.ShapeMenu.cs`, `Commands/InsertCommands.cs`, `AlignCommands.cs`.

* Object model in `fe-core/objects` and CPU drawing in `fe-core/render` (tiny-skia), cached as tiles.
* All 36 shapes with their special settings; fill, border (solid/dashed/dotted, dash/gap), corners, arrow heads,
  bent lines; placed pictures (resizable without quality loss).
* Select / Move: click, Shift+click, Ctrl+drag box, handles, rotate (Shift = 15°), lock, duplicate, group /
  ungroup, edit inside a group, bring to front/back, flip, Reshape corners, Keep proportions.
* Snapping with smart guides and equal spacing (Alt = no snap); align / distribute; Curve tool (P).
* Properties panel first version: Transform, Align, Shape sections with NumberSlider widgets.
* "Every layer has the same Properties": pixel layers turn into movable pictures on first transform.

**Done when:** the "shapes" C# screenshot can be rebuilt in the Rust app and looks the same; dragging is smooth.

---

## Phase 9 · Objects, part 2: text and looks

**Prompt:** "Do Phase 9: text objects and all object effects (gradient, shadow, glow, outline, 3D, styles, frames)."

**C# to read:** `Core/Objects/TextObject.cs`, `TextPreset.cs`, `GradientStyle.cs`, `ShadowFx.cs`, `ShadowPreset.cs`,
`ShadowStyle.cs`, `SurfaceFx.cs`, `ThreeDFx.cs`, `ThreeDStyle.cs`, `EffectStyles.cs`, `PictureFill.cs`,
`App/Rendering/ArcText.cs`, `SpacedText.cs`, `TextPresetPreviews.cs`, `ShadowPresetPreviews.cs`,
`Tools/Text/TextTool.cs`, `ViewModels/ObjectPropertiesViewModel.cs`, `Views/Panels/PropertiesPanel.cs`,
`App/Services/StyleLibrary.cs`, `Core/Layout/CollageLayouts.cs`, `Views/Dialogs/PhotoGridDialog*`.

* Text with cosmic-text: any installed font, size, bold, italic, alignment, color, outline, letter/word/line gap,
  text on a curve, editing on the canvas (double-click), 16 text designs.
* Gradients (2–8 colors, angle, radial, 11 presets, on border, overlay on pictures), drop shadow (10 presets, size),
  glow, inner shadow, outline, 3D look (5 presets), Properties > Adjust per object, saved Styles.
* Frames and Photo Grid, Fill Frames, move/zoom the picture inside a frame.
* Multi-selection Properties rules (common sections only, one undo step).

**Done when:** the "shapes and text" C# screenshot can be rebuilt; effect rendering matches the C# look closely.

---

## Phase 10 · Insert extras

**Prompt:** "Do Phase 10: QR codes, barcodes, charts, icons, badges, graphics, SmartArt, tutorial project."

**C# to read:** `Core/Qr/QrEncoder.cs`, `Core/Barcodes/BarcodeEncoder.cs`, `Core/Objects/QrCodeObject.cs`,
`BarcodeObject.cs`, `ChartObject.cs`, `App/Rendering/QrRenderer.cs`, `BarcodeRenderer.cs`, `ChartRenderer.cs`,
`Badges.cs`, `Graphics.cs`, `SmartArt.cs`, `App/Services/IconCatalog.cs`, `DemoProject.cs`,
`Views/Dialogs/IconPickerDialog*`, `SmartArtDialog*`, `BrandKitDialog*`, `ViewModels/ChartEditorItems.cs`.

* Port the QR and barcode encoders **with their tests**; all chart types (11) with the data table editor;
  ~150 icons, 52 badges, 51 graphics, 15 SmartArt layouts; Heading / Subheading / Body text buttons; Brand Kit.
* Tutorial project (14 pages) built on first start, like `DemoProject.cs`.

**Done when:** every Insert item works and the tutorial project matches the C# one.

---

## Phase 11 · Image commands, adjustments and filters

**Prompt:** "Do Phase 11: image size, canvas size, crop, rotate, all adjustments, filters and adjustment layers."

**C# to read:** `Core/Transforms/*`, `Core/Effects/**`, `Core/Documents/AdjustmentStyle.cs`, `CurveMath.cs`,
`Tools/Transform/*`, `Commands/ImageCommands.cs`, `EffectCommands.cs`, `App/Services/EffectPreviewSession.cs`,
`Views/Dialogs/ImageSizeDialog*`, `CanvasSizeDialog*`, `ParametersDialog*`, `RecolorDialog*`,
`Views/Controls/ColorGraph*`, `CurveEditor.cs`.

* Image Size (4 resize methods), Canvas Size with anchor, Crop tool (ratios, new layer, original hidden),
  Perspective Crop, Straighten, Trim, rotate 90/180/any, flip, Reveal All.
* 12 adjustments, Recolor with color graph, 11 filters + Enhance, all with **live preview on a background job**
  (preview at screen resolution first, full resolution on OK), limited to the selection.
* Adjustment layers: brightness/contrast/saturation/hue/temperature/tint, Levels, Curves (per channel).
* Effects run in parallel with rayon; benchmark against C#.

**Done when:** every effect matches the C# output (pixel tests with small tolerance) and previews stay smooth.

---

## Phase 12 · Retouching

**Prompt:** "Do Phase 12: clone stamp, healing tools, patch, dodge/burn, blur/sharpen, liquify."

**C# to read:** `Core/Retouch/*`, `Core/Painting/Liquify.cs`, `Tools/Retouch/*`, `Tools/Options/RetouchOptions.cs`.

**Done when:** all retouch tools work with one undo step per stroke and respect the selection.

---

## Phase 13 · AI tools

**Prompt:** "Do Phase 13: AI model store and all AI features."

**C# to read:** all of `FlashEditor.AI/`, `Core/Effects/Background/*`, `Tools/Painting/RefineTool.cs`,
`Tools/Retouch/RemoveBrushTool.cs`, `Tools/Selecting/ClickSelectTool.cs`, `Commands/InpaintCommands.cs`,
`UpscaleCommands.cs`, `ViewModels/AiModelItem.cs`, `AiModelsViewModel.cs`, `Views/Dialogs/AiModelsWindow*`.

* Model store: same URLs, sizes, SHA-256, `models\` next to the exe, ask before downloading, progress + cancel.
* `ort` with DirectML (GPU) and CPU fallback, DLLs loaded on first use, sessions dropped when idle.
* Remove Background (with sliders and live preview, editable cut-out object), Refine brush (R), Select Subject,
  Remove / Fill Selection (LaMa) + Remove Brush, AI Upscale / Clean Up (tiled, seamless, progress, cancel),
  Click Select (MobileSAM, CPU, cached image embedding).
* AI Models dialog (list, size, download, delete, show folder).
* Port the AI helper tests; real-model tests run only when the models are present.

**Done when:** results match the C# app on the same pictures; the UI never freezes during AI work.

---

## Phase 14 · Import and export

**Prompt:** "Do Phase 14: export dialog and every import/export format."

**C# to read:** `Core/IO/*` (all), `Core/Sheets/SheetSplitter.cs`, `Core/Layout/MagicResize.cs`,
`App/Services/WicImageCodec.cs`, `Views/Dialogs/ExportDialog*`, `ExportCropBox.cs`, `ExportForDialog*`,
`BatchDialog*`, `SplitSheetDialog*`, `MagicResizeDialog*`, `Commands/PagesExportCommands.cs`.

* Export dialog: format, quality, size, crop ratios, **real file size before saving** (computed on a job).
* PNG, JPG, WEBP, ICO (16–256), BMP, TIFF, GIF (256 colors); Export All Pages; Export For presets;
  PDF; animated GIF; SVG export + import; PSD import + export; PPTX (editable and picture modes).
* HEIC / AVIF / JPEG XL / camera RAW via WIC when the Windows codec is installed.
* Batch Convert Folder, Split Asset Sheet, Magic Resize (optionally every page).

**Done when:** each format round-trips in tests and opens in other programs (user spot-checks).

---

## Phase 15 · The rest of the UI

**Prompt:** "Do Phase 15: file drawer, shortcuts editor, command search, context menus, settings, tour."

**C# to read:** `Views/Panels/FileDrawer*`, `App/Services/Shortcuts.cs`, `Views/Dialogs/ShortcutsDialog*`,
`SettingsDialog*`, `MessageDialog*`, `Views/Editor/CanvasView.ContextMenu.cs`, `LayersPanel.ContextMenu.cs`,
`App/Services/IconTheme.cs`, `Views/Panels/RibbonBar.cs` (search), quick tour code in `MainWindow.cs`,
`App/Services/Templates/*` (new in the C# version, commit "load template from online"), `Views/Dialogs/Controls/TemplatesView*`,
the Templates part of `ProjectManagerWindow*`.

* File drawer (slides in from the left), every ribbon command wired, right-click menus with icons,
  editable keyboard shortcuts (F1), command search, icon style Plain / Colorful, quick tour, About.
* **Online templates** (added to the C# version after this plan was written; the Project Manager of Phase 5 gets a
  **Templates** button): the browser swaps the project list for template cards with a search box (name, type, keywords),
  one button per project type ("All" first), thumbnails loaded 4 at a time in the background, **Use template** / double-click /
  Enter. The list comes from the public GitHub repository `RahatAlamgir/Flash-Editor-Templates` (`catalog.json`, `types.json`,
  `templates/<type>/<id>/template.flash` + `thumb.jpg`); the web address can be changed with the `FLASHEDITOR_TEMPLATES`
  environment variable (a web address or a folder, for testing). Every download is checked against the catalog (size +
  SHA-256), cached in `settings\online-templates` so it works offline ("Offline: showing the templates saved earlier"),
  and saved as a new project in the projects folder, then opened. Needs no new library (`ureq` and `sha2` are in since Phase 13);
  downloads run on a job, never on the UI thread; Esc goes back from Templates to Projects; messages for no internet, no match,
  retry.
* Walk the whole `03-feature-checklist.md`: anything still unticked gets done or explicitly listed for the user.

**Done when:** the checklist has no unticked item that the user has not agreed to drop.

---

## Phase 16 · Polish, speed and release

**Prompt:** "Do Phase 16: design pass, performance pass, installer and README."

* Design pass: screenshot every main screen and dialog next to the C# one; fix differences with the user.
* Performance pass: measure all targets in `00-overview.md` on the user's PC and, if possible, a low-end PC;
  profile and fix the worst spots; record final numbers.
* Memory pass: long editing session, check undo memory cap and that AI memory is freed.
* Installer (Inno Setup, like the C# one): one edition only, named Flash Editor (models downloaded on demand; no AI edition).
* README with features, structure, build and test instructions.

**Done when:** all targets met (or the user accepts the gap), installers built and tested on a clean Windows.
