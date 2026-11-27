# Flash Editor

A fast image editor and design tool for Windows, built with Rust and [egui](https://github.com/emilk/egui). Think
Photoshop and Canva in one small program, with offline AI tools, and presentations with slide transitions and animations.

It is the Rust rebuild of the C# / WPF version (`D:\AiBuild\Flash Editor`, the specification, never changed). Same
features, about the same look, but smaller, quicker to start and light on memory.

## What it does

- **Pictures and pages:** layers (with masks, clipping, groups, adjustment layers), several pages per project, undo / redo
  with a history list, autosave and crash recovery, the Project Manager with online templates.
- **Objects that stay editable:** shapes, text (any installed font, curved text, 16 designs), pictures in frames and photo
  grids, QR codes, barcodes, 11 chart types, icons, badges, SmartArt, a Curve (pen) tool, shadow / glow / outline / 3D effects,
  gradients, saved styles and a Brand Kit. About 400 Windows icons, 2,200 Google (Material) icons, 52 badges and 148 graphics to insert. Smart guides, snapping, rulers, grid and guides. Hold Alt to see the distances
  around the selected object.
- **Painting and retouching:** 7 brush types, eraser, mask brush, Clone Stamp, Healing, Spot Healing, Patch, Dodge / Burn,
  Blur / Sharpen, Liquify.
- **Selections:** every selection tool with marching ants, Click Select, Select Subject, Remove by Color.
- **Adjustments and filters:** Levels, Curves and 10+ adjustments and filters with a live preview, Image Size, Canvas Size,
  crop tools, Magic Resize, Split Asset Sheet.
- **AI, offline:** Remove Background (editable cut-out), Refine brush, Fill (AI), Remove Brush, Click Select, AI Upscale.
  Runs on the graphics card (DirectML) or the processor. The models download once, after asking, and are freed again when
  not used for 5 minutes.
- **Presentations:** a project can be a presentation: slide transitions, object animations (entrance, emphasis, exit) listed
  on the Animations tab of the Layers panel, slideshow (F5), export to PowerPoint with the transitions and animations.
- **Files:** projects are saved as `.fe` (a small zip with lossless pictures). Open and export PNG, JPEG, WebP, GIF (animated),
  PDF, SVG, PSD, PowerPoint, and open HEIC, AVIF, RAW. Batch Convert and Export All Pages.
- **Everything searchable:** command search, editable keyboard shortcuts (F1), Quick Tour.

## Numbers (release build, Intel Arc A750 PC)

| What | Target | Measured |
|---|---|---|
| `flash-editor.exe` | under 20 MB | 19.2 MiB (20.1 MB) |
| Start to a usable window | under 1 s | about 0.2 s |
| Memory, empty start | under 150 MB | 85 MB working set |
| Memory, 4000 x 3000 picture with 10 layers, zooming and panning | half of the C# version | 173 MB peak |
| Idle CPU | about 0 % | 0 ms of CPU in 3 s |
| Zoom / pan on 4000 x 3000 with 10 layers | steady 60 fps | 16.7 ms per frame (95 % 17.2 ms) |

Measure them again with `scripts\measure.ps1` and `flash-editor.exe --bench` (see below).

## Install

Run `FlashEditor-Setup-<version>.exe`. The installer puts the program and the AI runtime into `Program Files\Flash Editor`,
and can download the AI models for you (about 470 MB, optional, checked with SHA-256). Nothing is written outside the install
folder; uninstalling removes it all.

Build the installer yourself (needs [Inno Setup 6](https://jrsoftware.org/isinfo.php), `winget install JRSoftware.InnoSetup`):

```powershell
powershell -ExecutionPolicy Bypass -File installer\build-installer.ps1
```

The result is `installer\output\FlashEditor-Setup-<version>.exe` (add `-TestInstall` for one that needs no administrator).

## Build and run

Needs Rust (stable, MSVC toolchain), installed with [rustup](https://rustup.rs).

```powershell
cargo run -p fe-app
```

Release build (small, fast exe in `target\release\flash-editor.exe`):

```powershell
cargo build -p fe-app --release
```

Start options:

```powershell
& "target\release\flash-editor.exe" "C:\path\to\photo.jpg"   # open a picture or a .fe project
& "target\release\flash-editor.exe" --bench                      # speed test, results in target\release\settings\bench.txt
& "target\release\flash-editor.exe" --wgpu                       # use the wgpu renderer instead of OpenGL
```

In the app, F12 shows drawing statistics over the canvas.

### AI features while developing

The AI tools need the ONNX Runtime DLLs (`onnxruntime.dll`, `onnxruntime_providers_shared.dll`, `DirectML.dll`) next to the
program, and the models in a `models` folder next to it. This copies the DLLs (and, with `-WithModels`, the models) from the
C# build into `target\debug` and `target\release`:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\get-ai-runtime.ps1 -WithModels
```

Set `FLASH_AI_CPU=1` to force the processor instead of the graphics card, `FLASH_ORT_DLL` to use another runtime DLL.

## Test

Run every check (formatting, lints, tests) before finishing a change:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\check.ps1
```

Engine tests live in `crates\fe-core\tests\` (`cargo test -p fe-core`). `scripts\screenshot.ps1` starts the app, can click and
drag, saves a screenshot and prints start-up time, memory and idle CPU. `scripts\measure.ps1` samples memory and CPU while the
app runs (for example with `-AppArgs "--bench"`).

## Project structure

```
Flash Editor Rust/
├── Cargo.toml              Workspace: shared dependency versions, build profiles (fat LTO, stripped)
├── rustfmt.toml            Code formatting rules
├── assets/icons/app/       App icon (.ico for the exe and installer, .png for the window)
├── crates/
│   ├── fe-core/            Image engine, no UI: imaging (tiles), document (pages, layers, animations, transitions), objects,
│   │                       render (tiny-skia), text (fontdb + rustybuzz), selection, history, paint, retouch, transform,
│   │                       effects, layout (snapping), codes (QR, barcodes), io (.fe, PNG, PSD, PDF, PPTX, SVG...), library
│   ├── fe-ai/              AI models: download and checks, ONNX runtime, segmentation, inpainting, upscaling
│   ├── fe-gpu/             Canvas renderer: combined tiles as textures, zoom levels, background jobs
│   ├── fe-ui-kit/          Look and feel: theme (colors, sizes, fonts), icons, widgets (ribbon button, sliders...)
│   └── fe-app/             The program
│       ├── build.rs        Puts the icon into the exe
│       ├── examples/       build_templates: makes the template projects
│       └── src/
│           ├── main.rs     Starts logging and the window
│           ├── app/        Start-up, app state, layout (collapsible panels), keyboard, clipboard, paths, settings
│           ├── commands/   What buttons and shortcuts do (edit, file, layers, pages, objects, presentation...)
│           ├── tools/      Tool catalog, options bar, selecting/, moving/, painting/, retouching, snapping
│           ├── canvas/     Canvas view: zoom / pan, input, rulers, grid, guides, selection overlay, distance guides
│           ├── panels/     ribbon/, toolbox, project tabs, right/ (properties, layers + animations, history), page strip
│           ├── dialogs/    One file per dialog (new project, project manager, color picker, export...)
│           ├── ai/         AI jobs, the AI Models window, loaded models (released when idle)
│           ├── slideshow/  Slideshow playback, transitions and animations
│           ├── templates/  Online template catalog with lazy loading
│           └── text_edit/  Editing text on the canvas
├── docs/
│   ├── plan/               The porting plan, progress logs (05 main, 06 file size and later work, 07 animation)
│   └── screenshots/        csharp/ = reference pictures of the C# app
├── installer/              Inno Setup script and build-installer.ps1 (output goes to installer\output)
├── scripts/                check.ps1, screenshot.ps1, measure.ps1, get-ai-runtime.ps1, convert_icons.py
├── tests/fixtures/         Projects written by the C# app, used by the file tests
└── tools/csharp-fixtures/  .NET helper that writes those fixtures
```

`fe-core` never depends on UI or GPU code, so the engine is tested on its own. While running, the app writes
`settings\flash-editor.log` next to the exe.

## The plan

| File | What it is |
|---|---|
| [`docs/PROMPT.md`](docs/PROMPT.md) | Prompt to start every build session with |
| [`docs/plan/00-overview.md`](docs/plan/00-overview.md) | Goal, decisions, speed and size targets, working rules |
| [`docs/plan/01-architecture.md`](docs/plan/01-architecture.md) | Crates, folders, libraries, performance design |
| [`docs/plan/02-phases.md`](docs/plan/02-phases.md) | The build phases, each with a prompt and "done when" |
| [`docs/plan/03-feature-checklist.md`](docs/plan/03-feature-checklist.md) | Every C# feature, its C# source and its phase |
| [`docs/plan/04-design-guide.md`](docs/plan/04-design-guide.md) | Colors, layout, icons, how the look is checked |
| [`docs/plan/05-progress-log.md`](docs/plan/05-progress-log.md) | What was done each session |
| [`docs/plan/06-flash-size-log.md`](docs/plan/06-flash-size-log.md) | `.fe` file size, presentations, UI changes after Phase 15 |
