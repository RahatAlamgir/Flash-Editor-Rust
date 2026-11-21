# 00 · Overview: porting Flash Editor from C# to Rust

## Goal

Rebuild **Flash Editor** (the C# / .NET 10 / WPF app in `D:\AiBuild\Flash Editor`) in **Rust**, with the same
features, about **95 % the same look**, and a clearly **smoother, smaller and lighter** program than the C# version,
especially on low-end PCs.

The C# project is the **specification**. When this plan and the C# code disagree about *what a feature does*,
the C# code wins. When they disagree about *how it is built*, this plan wins (the Rust version must be faster).

## Decisions already made (do not reopen)

| Topic | Decision |
|---|---|
| Language | Rust (stable, MSVC toolchain on Windows; installed: rustc 1.100) |
| UI toolkit | **egui / eframe**, **OpenGL (`glow`) renderer by default**, `wgpu` (DirectX 12 / Vulkan) as fallback — changed with the user's OK in Phase 3: OpenGL measured as smooth and used the least memory |
| Look | Same dark theme, layout, icons and wording as the C# app; 95 % fidelity is the target, not pixel-perfect |
| Platform | Windows 10 / 11, 64-bit (keep code portable where it costs nothing) |
| AI | Same ONNX models, same download URLs and SHA-256 checks, through the `ort` crate (DirectML, CPU fallback) |
| Project files | Must open `.flash` files saved by the C# version (zip + `project.json` + PNGs) |
| C# folder | **Read only.** Never edit anything in `D:\AiBuild\Flash Editor` |

## Quality targets (checked in Phase 16, measured from Phase 3 on)

| What | Target (C# for comparison) |
|---|---|
| `flash-editor.exe` size, no AI | under 20 MB (C# install is ~60 MB) |
| Start-up to usable window | under 1 s on a mid PC, under 2 s on a low-end PC |
| Memory, one empty 1920 x 1080 project | under 150 MB |
| Memory, 4000 x 3000 photo with 10 layers | under half of the C# version |
| Idle CPU (nothing happening) | ~0 % (no redraws while idle) |
| Brush / drag / zoom on a 4000 x 3000 picture | steady 60 fps on a mid PC, no visible stutter on a low-end PC |
| AI DLLs and models | loaded only when an AI tool is first used; freed again after a while unused |

## How to use these files

| File | What it is |
|---|---|
| [`../PROMPT.md`](../PROMPT.md) | The prompt to start every build session with |
| [`01-architecture.md`](01-architecture.md) | Crates, folders, libraries and the performance design |
| [`02-phases.md`](02-phases.md) | The step-by-step build plan: 17 phases, each with its own prompt and "done when" |
| [`03-feature-checklist.md`](03-feature-checklist.md) | Every feature of the C# app, with the C# source to read and the phase that ports it. Ticked off as we go |
| [`04-design-guide.md`](04-design-guide.md) | Colors, sizes, layout and how we check the look against the C# screenshots |
| [`05-progress-log.md`](05-progress-log.md) | One short entry per finished session: what was done, what is next, known problems |

## Working rules for every session

1. **Start** by reading `docs/PROMPT.md`, this file, `05-progress-log.md` and the section of `02-phases.md` for the phase.
2. **Read the C# source** for the feature before writing Rust (paths are in `03-feature-checklist.md`).
   Port the behaviour and the tests, not the WPF/MVVM structure.
3. **Small files, one job each** (aim for under ~300 lines; split when bigger). Folders per area. No "utils" dumping ground.
4. **Tests**: port the matching xUnit tests from `D:\AiBuild\Flash Editor\tests\FlashEditor.Core.Tests`
   (76 files) and add new ones. `cargo test` must pass at the end of every session.
5. **Never block the UI thread** with work that can take more than ~8 ms: use the job system (see architecture).
6. **Measure** performance-sensitive work (`cargo bench` / the built-in frame timer) instead of guessing.
7. **Run the app** at the end of a session and look at it (screenshot) when the session changed anything visible.
8. **Update** `03-feature-checklist.md` (tick what is done) and add an entry to `05-progress-log.md`.
9. Ask the user before: adding a big dependency not listed in the architecture, changing the file format,
   dropping or changing a feature, or anything that touches the network besides model downloads.
10. Explain things to the user in plain words; they are not a Rust expert.
