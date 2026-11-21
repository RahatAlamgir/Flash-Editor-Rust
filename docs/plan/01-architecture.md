# 01 · Architecture

## Cargo workspace

One workspace, several crates. Same split as the C# solution (Core / AI / App), plus a few small helper crates so
the heavy parts compile and test on their own.

```
Flash Editor Rust/
├── Cargo.toml                    workspace, shared dependency versions, release profile
├── docs/PROMPT.md                start-of-session prompt
├── README.md
├── docs/plan/                    this plan
├── docs/screenshots/             reference screenshots (C#) and comparison screenshots (Rust)
├── assets/                       app icon, tutorial photos, fonts that ship with the app, icon path data
├── installer/                    Inno Setup script + build script (Phase 16)
├── crates/
│   ├── fe-core/                  image engine, no UI, no GPU. Mirrors FlashEditor.Core
│   │   └── src/
│   │       ├── imaging/          tiles, pixel buffers, colors, rects, blend math, resampling
│   │       ├── document/         Document, Page, Project, Layer, LayerMask, AdjustmentStyle
│   │       ├── objects/          object model (shape, text, image, group, qr, chart, barcode) + styles
│   │       ├── render/           CPU rendering of objects (tiny-skia), text layout, effects (shadow, glow, 3D)
│   │       ├── selection/        selection masks, polygon fill, outline tracing, quick select, magic wand
│   │       ├── history/          undo / redo engine and step kinds
│   │       ├── paint/            brush engine, flood fill, compositor (reference CPU path)
│   │       ├── retouch/          clone, heal, spot heal, patch, dodge/burn, blur/sharpen, liquify
│   │       ├── transform/        crop, rotate, flip, resize, perspective, straighten
│   │       ├── effects/          effect framework + adjustments/ + filters/
│   │       ├── layout/           align, distribute, magic resize, collage layouts, snapping
│   │       ├── codes/            QR and barcode encoders
│   │       └── io/               decode / encode, project (.flash), psd, svg, pdf, pptx, gif, ico, batch
│   ├── fe-ai/                    ONNX models: store/download/verify, sessions, segmentation, inpaint, upscale, SAM
│   ├── fe-gpu/                   canvas renderer: combined-tile textures, zoom levels, background tile jobs
│   ├── fe-ui-kit/                theme, icons, reusable egui widgets (NumberSlider, ColorGraph, CurveEditor...)
│   └── fe-app/                   the program: app state, commands, tools, panels, dialogs, main()
│       └── src/
│           ├── app/              app state, project tabs, job system, settings, autosave
│           ├── commands/         one file per menu area (file, edit, image, layer, insert, adjust...)
│           ├── tools/            one file per tool, grouped like the C# Tools folder
│           ├── canvas/           canvas view, input mapping, overlays (handles, guides, rulers, ants)
│           ├── panels/           ribbon, file drawer, toolbox, options bar, properties, layers, history, pages
│           └── dialogs/          one file per dialog
└── tests/                        cross-crate integration tests + sample .flash files from the C# app
```

`fe-core` must never depend on egui or wgpu. `fe-app` is the only crate that knows about windows.

## Libraries (pick the latest stable version when adding; pin in the workspace `Cargo.toml`)

| Job | Crate | Notes |
|---|---|---|
| Window, UI | `eframe`, `egui`, `egui_extras` | OpenGL (`glow`) renderer by default, `wgpu` as fallback |
| GPU | `wgpu` (through eframe) | custom paint callback for the canvas |
| Vector drawing (shapes, effects) | `tiny-skia` | CPU, fast, small; draws objects into cached tiles |
| SVG icons / SVG import | `usvg` + `resvg` | the C# icons are XAML path data: convert once to SVG path strings |
| Text shaping & fonts | `cosmic-text` (uses `fontdb`, `swash`) | system fonts, bold/italic, line/letter/word spacing |
| Image files | `image` (png, jpeg, webp, gif, bmp, ico, tiff) | `zune-jpeg` / `png` fast paths where needed |
| HEIC / AVIF / JXL / RAW | `windows` crate → WIC | same approach as the C# `WicImageCodec` |
| Fast resize | `fast_image_resize` | SIMD Lanczos / bicubic / bilinear / nearest |
| Parallel work | `rayon` | filters, effects, encoding |
| Project file | `zip`, `serde`, `serde_json` | compatible with the C# `project.json` |
| AI | `ort` (load-dynamic, DirectML) | DLLs shipped next to the exe, loaded only on first AI use |
| Downloads | `ureq` + `sha2` | model downloads with progress and SHA-256 check |
| File dialogs | `rfd` | native Windows dialogs |
| Clipboard | `arboard` | images and text; objects as our own format |
| GIF encoding | `gif` + `color_quant` (or a port of the C# quantizer) | |
| PDF | `pdf-writer` | or port the small C# `PdfWriter` |
| PSD | port the C# `PsdFile` (read + write) | |
| Errors / logging | `thiserror`, `anyhow` (app only), `log` + `simplelog` | log file next to the exe |
| Benchmarks | `criterion` | Phase 3 onward |

Avoid: Electron-style web views, big async runtimes (no `tokio`), anything that pulls hundreds of crates for a small job.

## Performance design (the reason for the port)

The C# version keeps **every layer as a full canvas-sized BGRA buffer**, composites on the CPU and lets WPF redraw.
That is what makes it slow and memory-hungry. The Rust version changes the engine, not the features:

1. **Tiled, sparse layers.** Pixels live in 64 x 64 RGBA8 tiles in a hash map (64 px measured best in Phase 5:
   small objects waste little memory; the canvas still draws 256 px screen tiles). Empty (fully transparent) tiles are
   not stored at all. A 4000 x 3000 layer with a small drawing on it costs a few tiles, not 48 MB.
   Pixels moved off the canvas are just tiles outside the canvas rectangle (replaces the C# "extended" buffer).
2. **Copy-on-write tiles (`Arc<Tile>`).** Undo snapshots share unchanged tiles, so an undo step stores only the
   tiles that changed. Duplicating a layer or page is almost free until one of them is edited.
3. **Canvas tiles on the graphics card** (as built in Phase 3, see `fe-gpu`). The canvas is cut into 256 px
   tiles; each tile's layers are combined on the CPU (in parallel) into one picture and uploaded once as a
   texture. A cheap fingerprint per tile (`fe_core::paint::signature`: tile change stamps + layer settings,
   never pixels) says exactly which tiles changed. Zoom and pan only move textures. Small updates (≤ 4
   full-size tiles, e.g. a brush dab) are combined in the frame; bigger ones run on background threads while
   the old picture (or another level) stays on screen. Only on-screen tiles are made; unused textures are
   dropped after ~5 s. Works the same on wgpu and OpenGL. *Changed from the first plan (layer compositing in
   custom wgpu shaders): simpler, no custom GPU code, measured 60 fps with ~1 ms CPU per frame. Shader
   compositing stays an option if painting on huge pictures ever needs it.*
4. **Zoom levels.** When zoomed out, tiles cover 2, 4, 8... times more canvas at the same 256 px texture size,
   so a 6000 px picture at 25 % costs what a 1500 px one does.
5. **Objects are cached drawings.** A shape/text/picture object is drawn by tiny-skia once into tiles and redrawn
   only when the object changes. While dragging, the cached texture is just moved/rotated on the GPU.
6. **Redraw only when needed.** egui runs in reactive mode: no frames while idle. Animations (marching ants,
   progress) request repaints only while they are visible.
7. **Background jobs.** A small job system (rayon pool + channels) runs filters, AI, thumbnails, autosave, file
   loading and saving. The UI shows progress and stays responsive; results come back as messages.
8. **Thumbnails debounced.** Layer and page thumbnails are rebuilt a moment after a change, in the background, at
   thumbnail size only.
9. **Lazy AI.** `onnxruntime.dll` / `DirectML.dll` are loaded on the first AI use; a model session is dropped after
   a few idle minutes to give the memory back.
10. **Release build**: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.

## Data model sketch

```rust
// fe-core
struct Project { pages: Vec<Page>, active_page: usize, history: History, path: Option<PathBuf> }
struct Page { name: String, doc: Document, guides: Guides }
struct Document { width: u32, height: u32, layers: Vec<Layer>, active: LayerId, selection: Option<Selection> }
struct Layer {
    id: LayerId, name: String, opacity: f32, visible: bool, locked: bool,
    blend: BlendMode, adjustment: Option<AdjustmentStyle>,
    pixels: TileMap,                 // sparse Arc tiles; also holds off-canvas pixels
    object: Option<Arc<LayerObject>>,// immutable description; pixels are its cached drawing
    restore_source: Option<TileMap>, // original for the Refine brush
    mask: Option<Mask>, mask_enabled: bool,
}
enum LayerObject { Shape(..), Text(..), Image(..), Group(..), Qr(..), Chart(..), Barcode(..) } // + common styles
```

Undo steps are an enum (`PixelChange`, `LayerProperty`, `LayerStructure`, `ObjectChange`, `Mask`, `Canvas`,
`Composite`) and keep the C# rules: one history per project, step limit (default 30, 1 to 60) and a memory cap
(~1 GB), merge of consecutive small steps.

## `.flash` compatibility

* Zip with `project.json`, `preview.png` (480 px) and `images/N.png`.
* JSON written by `System.Text.Json`: PascalCase property names, nulls left out, objects tagged with `"kind"`
  (`shape`, `text`, `image`, `group`, `qr`, `chart`, `barcode`), pixel buffers as image file names.
* Read `Format = "flash-project"`, `Version` 1 and 2. Write version 2 so the C# app can still open Rust files.
* Phase 5 must collect sample `.flash` files saved by the C# app (all object kinds, masks, adjustments, extended
  pixels, several pages) into `tests/fixtures/` and test a load → save → load round trip.

## Threads and state (app)

* UI thread: egui, app state, tools. Owns the `Project`s.
* Job pool: gets `Arc` snapshots of what it needs (cheap thanks to Arc tiles), sends back results.
* GPU: tile textures cached per layer id + tile key + version.
