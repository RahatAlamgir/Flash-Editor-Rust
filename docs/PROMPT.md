# Build prompt

Paste this at the start of every build session, then add the phase line, for example
"**Do Phase 3.**" (each phase's own prompt is in `docs/plan/02-phases.md`).

---

You are porting **Flash Editor**, a Photoshop / Canva-style image editor, from C# / .NET 10 / WPF
(`D:\AiBuild\Flash Editor`, read only) to **Rust + egui** in `D:\AiBuild\Flash Editor Rust`.

Before writing code:
1. Read `docs/plan/00-overview.md` (goal, decisions, rules), `docs/plan/01-architecture.md` (crates, libraries,
   performance design) and the newest entries in `docs/plan/05-progress-log.md`.
2. Read the section of `docs/plan/02-phases.md` for the phase you are asked to do, and its rows in
   `docs/plan/03-feature-checklist.md`.
3. Read the C# source listed for that phase so the Rust version behaves the same. Port the behaviour and the tests,
   not the WPF / MVVM structure. For anything visible, follow `docs/plan/04-design-guide.md`.

While working:
* The Rust version must be **smoother, smaller and lighter** than the C# one: sparse `Arc` tiles, GPU compositing,
  redraw only when needed, heavy work on background jobs, lazy AI. Measure instead of guessing.
* Many small files with one job each, in folders per area. `fe-core` never depends on UI or GPU crates.
* Keep `cargo fmt`, `cargo clippy -- -D warnings` and `cargo test` clean (`scripts/check.ps1`).
* Ask me before adding a large dependency not in the architecture, changing the `.flash` format, or dropping /
  changing a feature. Never edit the C# folder.
* I am not a Rust expert: explain choices and results in plain words.

When the session ends:
* Run the app and check anything visible (screenshot it).
* Tick finished rows in `03-feature-checklist.md` and add an entry to `05-progress-log.md`
  (done, measurements, next step, open questions).
* Tell me what to look at and what is next.
