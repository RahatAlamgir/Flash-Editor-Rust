# 04 · Design guide

Target: the Rust app should look like the same program as the C# app (about 95 %). Reference screenshots are in
`docs/screenshots/csharp/` (copied in Phase 0). The source of truth for colors and sizes is
`D:\AiBuild\Flash Editor\src\FlashEditor.App\Themes\*.xaml`.

The main reference is `docs/screenshots/csharp/01-editor-tutorial.jpg` (the current C# app with the **ribbon**:
File button, tabs, big icon buttons with labels, search box at the right; "Projects" button left of the project
tabs; view-aid buttons right of the tabs; Properties with collapsible sections; Layers with Mix / Opacity and
Select all / Lock all / Unlock all / Merge; page strip with large page cards and "Grid view"). The C# screenshots
folder changes over time: copy it again at the start of Phase 2 and Phase 16 so the reference is current.

## Color tokens (from `Themes/Colors.xaml`)

| Token | Value | Used for |
|---|---|---|
| `window` | `#1E1E1E` | window background, ribbon |
| `panel` | `#262626` | side panels |
| `panel_header` | `#2E2E2E` | panel section headers |
| `canvas_area` | `#161616` | area around the picture |
| `divider` | `#3A3A3A` | lines between areas |
| `hover` | `#363636` | hovered buttons / rows |
| `accent` | `#3B82F6` | active tool, sliders, focus |
| `accent_soft` | `#3B82F6` at 33 % | selected rows |
| `selection_tint` | `#3D8BFF` at 15 % | selection fill on canvas |
| `selection_accent` | `#4FC3FF` | handles, selection outline |
| `text` | `#E6E6E6` | normal text |
| `subtle_text` | `#9A9A9A` | hints, section titles, status bar |
| `icon` | `#D8D8D8` | plain icons |

Put them in `fe-ui-kit/src/theme/colors.rs` and use only the tokens in UI code (no hard-coded colors).
Colorful icon style colors come from `App/Services/IconTheme.cs`.

## Layout (from the screenshots, at 100 % Windows scaling)

* Title bar: normal Windows title bar ("<project> * - Flash Editor").
* Ribbon / menu row, then the tool hint bar (tool icon + bold tool name + gray hint text).
* Left: toolbox column (~40 px wide, one icon per tool group, active tool on accent-soft square), FG/BG color
  squares at the bottom.
* Center: project tabs (name, `*` when unsaved, close cross; `+` and folder buttons at the right), canvas area,
  page strip under the canvas.
* Right: ~290 px column with PROPERTIES, LAYERS (opacity slider + number box + %, Remove Background button,
  layer rows ~48 px with eye, thumbnail, name, lock; buttons row at the bottom), HISTORY (collapsible).
* Status bar: left tool info ("Move: +47, 0 px"), right canvas size (underlined, clickable) and zoom %.
* Section titles are small uppercase gray text.
* Font: Segoe UI from the system, ~12–13 px for normal text.

Measure exact paddings, heights and corner radii from `Themes/Styles.xaml`, `Panels.xaml`, `Ribbon.xaml`,
`TextBoxes.xaml` when building each part, and write them as constants in `fe-ui-kit/src/theme/metrics.rs`.

## Icons

`Themes/Icons.xaml` holds ~95 vector icons as path geometries. Convert them once (script in `scripts/`) to
`assets/icons/*.svg`, keep the same names, and draw them with `resvg` into a texture atlas at the needed sizes
(crisp at 100 %, 125 %, 150 %, 200 % scaling). Never use emoji or icon fonts instead.

## Widgets to build in `fe-ui-kit`

NumberSlider (slider + number box + unit), ColorSwatch / ColorGraph, CurveEditor, SizeEditor, tool button with
group triangle, ribbon button (large / small / with dropdown), panel section (collapsible header), layer row,
thumbnail with checkerboard, tabs with close cross and drag, icon menu items, dialog frame matching
`MessageDialog.xaml`.

## How each screen is checked

1. Build the screen in Rust.
2. Take a screenshot of the Rust app at the same window size as the C# reference.
3. Put both side by side in `docs/screenshots/compare/<screen>.png` and list the differences.
4. Fix what is easy; show the user the rest and let them decide.

Accepted differences (agreed with the user): slightly different text rendering, simpler or no animations, small
details of menus and dropdowns, on-canvas text editing that feels a little different at first.
