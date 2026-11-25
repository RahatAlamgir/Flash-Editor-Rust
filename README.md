# Flash Editor (Rust)

A Rust + egui rebuild of **Flash Editor**, the Photoshop / Canva-style image editor with offline AI tools.
The original C# / WPF version lives in `D:\AiBuild\Flash Editor` and is the specification for this one.

**Why rebuild it?** The C# version can lag on low-end PCs. The Rust version keeps the same features and about the
same look, but is meant to be smoother, smaller and use less memory.

**Status:** Phase 14 done: File > Export (format, quality, size, crop, real file size), Export For, Export All Pages, PDF, animated GIF, PowerPoint, SVG, PSD, Batch Convert; Image > Magic Resize and Split Asset Sheet; open PSD / SVG / HEIC / AVIF / RAW. Phase 13: all AI features (Remove Background with live preview and editable cut-out, Refine brush, Select Subject, Fill (AI), Remove Brush, Click Select, AI Upscale, AI Models window); the AI runs on the graphics card (DirectML) or the processor and the models download once after asking. Phase 12: Clone Stamp, Healing Brush, Spot Healing, Patch, Dodge / Burn, Blur / Sharpen and Liquify. Phase 11: Image Size, Canvas Size, crop tools, rotate / flip, 12 adjustments and 10 filters with a live preview, adjustment layers (Levels, Curves). Phase 10: QR codes, barcodes, 11 chart types, icons, badges, graphics, SmartArt, Brand Kit and the 14-page tutorial project. Phase 9: text (any installed font, curved text, 16 designs), gradients, shadow / glow / inner shadow / outline / 3D / blur, saved styles, frames and photo grids, on top of the shapes, pictures, groups, Curve tool and snapping of Phase 8. Phase 7: painting (7 brush types, eraser, bucket, eyedropper), the color picker with swatches, layer masks with the Mask Brush and Clip to Layer Below. Phase 6: all selection tools with marching ants, moving pixels and layers, copy / cut / paste (also with other programs), Split, Reveal All. Before that: `.flash` projects open and save (fully compatible with the C# version both ways), pages, the Project Manager, autosave and crash recovery; layers, undo / redo and a smooth canvas. Editing tools follow phase by phase. See [`docs/plan/05-progress-log.md`](docs/plan/05-progress-log.md).

## The plan

| File | What it is |
|---|---|
| [`docs/PROMPT.md`](docs/PROMPT.md) | Prompt to start every build session with |
| [`docs/plan/00-overview.md`](docs/plan/00-overview.md) | Goal, decisions, speed and size targets, working rules |
| [`docs/plan/01-architecture.md`](docs/plan/01-architecture.md) | Crates, folders, libraries, performance design |
| [`docs/plan/02-phases.md`](docs/plan/02-phases.md) | 17 build phases, each with a prompt and "done when" |
| [`docs/plan/03-feature-checklist.md`](docs/plan/03-feature-checklist.md) | Every C# feature, its C# source and its phase |
| [`docs/plan/04-design-guide.md`](docs/plan/04-design-guide.md) | Colors, layout, icons, how the look is checked |
| [`docs/plan/05-progress-log.md`](docs/plan/05-progress-log.md) | What was done each session |

## How to build and run

Needs Rust (stable, MSVC toolchain), installed with [rustup](https://rustup.rs).

```powershell
cargo run -p fe-app
```

Release build (small, fast exe in `target\release\flash-editor.exe`):

```powershell
cargo build -p fe-app --release
```

Useful start options:

```powershell
& "target\release\flash-editor.exe" "C:\path\to\photo.jpg"   # open a picture
& "target\release\flash-editor.exe" --bench                      # speed test, results in target\release\settings\bench.txt
& "target\release\flash-editor.exe" --wgpu                       # use the wgpu renderer instead of OpenGL
```

In the app, F12 shows drawing statistics over the canvas.

### AI features (models and runtime)

The AI tools need the ONNX Runtime DLLs (`onnxruntime.dll`, `onnxruntime_providers_shared.dll`, `DirectML.dll`) next to
the program, and the models in a `models` folder next to it. The program asks before it downloads a model (one time,
about 480 MB for all five), and the AI Models window (Settings tab) lists, downloads and deletes them. While developing,
this copies the DLLs (and, with `-WithModels`, the models) from the C# build into `target\debug` and `target\release`:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\get-ai-runtime.ps1 -WithModels
```

Set `FLASH_AI_CPU=1` to force the processor instead of the graphics card, `FLASH_ORT_DLL` to use another runtime DLL.

Run every check (formatting, lints, tests) before finishing a change:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check.ps1
```

## Project structure

```
Flash Editor Rust/
├── Cargo.toml              Workspace: shared dependency versions, build profiles
├── rustfmt.toml            Code formatting rules
├── assets/icons/app/       App icon (.ico for the exe, .png for the window)
├── crates/
│   ├── fe-core/            Image engine, no UI (imaging, document, objects, render (tiny-skia), text (fontdb + rustybuzz), selection,
│   │                       history, paint, retouch, transform, effects, layout, codes, io)
│   ├── fe-ai/              AI models (models, runtime, segmentation, inpainting, upscaling, effects)
│   ├── fe-gpu/             Canvas renderer: combined tiles as textures, zoom levels, background jobs
│   ├── fe-ui-kit/          Look and feel: theme/ (colors, sizes, fonts, style), icons/ (line icons +
│   │                       Windows icon-font glyphs), widgets/ (ribbon button, tool button, sliders...)
│   └── fe-app/             The program
│       ├── build.rs        Puts the icon into the exe
│       └── src/
│           ├── main.rs     Starts logging and the window
│           ├── app/        Window start-up (OpenGL, wgpu as fallback), app state, window placement,
│           │               keyboard, clipboard, paths, logging
│           ├── commands/   What buttons and shortcuts do: edit (undo / redo), file, layers, pages,
│           │               selection (copy / paste, select all, split, reveal all), mask, objects
│           │               (add shape, group, flip, align, Properties changes), checks
│           ├── tools/      Tool catalog and toolbox state, options bar, selecting/ (one file per selection
│           │               tool), moving/ (Select / Move: pick, drag, nudge, move selected pixels),
│           │               painting/ (brush, eraser, mask brush, bucket, eyedropper, brush type pictures),
│           │               snap.rs (smart guides while dragging)
│           ├── canvas/     Canvas view: zoom / pan state, input, rulers, grid and guides, selection overlay
│           │               (marching ants), right-click menu, speed test
│           ├── panels/     ribbon/ (data + drawing), hint bar, toolbox, project tabs, right/ (properties,
│           │               layers/ (header, row, list, menu, footer), history), page strip, status bar,
│           │               pop-ups (ribbon lists, Undo Steps)
│           └── dialogs/    One file per dialog (ask, new project, project manager, color picker...)
├── docs/
│   ├── plan/               The porting plan (see table above)
│   └── screenshots/        csharp/ = reference pictures of the C# app, compare/ = Rust screenshots
├── scripts/
│   ├── check.ps1           Format + lint + test in one go
│   ├── screenshot.ps1      Start the app (optionally click / drag / press keys), save a screenshot, print
│   │                       start time / memory / idle CPU
│   └── convert_icons.py    Turn the C# Icons.xaml into crates/fe-ui-kit/src/icons/paths.rs
├── tests/fixtures/         .flash projects written by the C# app's own code (tutorial, features)
├── tools/csharp-fixtures/  .NET helper: writes those samples and prints what the C# reader sees in any
│                           .flash file (to prove Rust-saved files open identically in the C# app)
└── installer/              Installer script (Phase 16)
```

`fe-core/src/imaging`, `document`, `paint`, `selection`, `objects`, `render`, `layout` and `io` hold the engine; the other folders hold only a short description for now and fill up phase by phase. Engine tests live in `crates/fe-core/tests/` (run `cargo test -p fe-core`).
`fe-core` never depends on UI or GPU code, so the engine can be tested on its own.

While running, the app writes `settings\flash-editor.log` next to the exe.
