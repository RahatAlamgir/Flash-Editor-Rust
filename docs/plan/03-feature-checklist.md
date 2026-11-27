# 03 · Feature checklist

Every feature of the C# Flash Editor (from its README and source). Tick `[x]` when the Rust version does it and it
has been checked. "Ph" = phase in `02-phases.md`. C# paths are under `D:\AiBuild\Flash Editor\src\`.

## Files, projects and pages
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Open PNG, JPG, WEBP, ICO, BMP, GIF, TIFF, JPEG XR, DDS | 1, 14 | `Core/IO/ImageDecoder.cs`, `App/Services/WicImageCodec.cs` |
| [x] | Open HEIC, AVIF, JPEG XL, camera RAW (via Windows codecs) | 14 | `WicImageCodec.cs` |
| [x] | Drag files onto the window; Ctrl+drop adds as layers | 5 | `Views/MainWindow.cs` |
| [x] | Save / open `.flash` projects (compatible with C# files) | 5 | `Core/IO/Project/*` |
| [x] | Several projects as tabs, rename by double-click, Ctrl+Tab, Ctrl+W (double-click a tab to rename it) | 5 | `ViewModels/ProjectGroup.cs`, `MainViewModel.cs` |
| [x] | One undo history per project (all pages), step setting 1–60 (default 30) | 4 | `Core/History/ProjectHistory.cs` |
| [x] | Project Manager (cards, search, new, rename, show in Explorer, remove, delete to Recycle Bin) | 5 | `Views/Dialogs/ProjectManagerWindow*`, `Services/ProjectLibrary.cs` |
| [x] | Pages: strip with thumbnails, + Page, duplicate, rename, reorder, delete, grid view | 5 | `Views/Panels/PageGridView*`, `Commands/PageCommands.cs` |
| [x] | Multi-select pages, copy / cut / paste pages (also into other projects), "+" between pages | 5 | `PageActions.cs`, `PageCommands.cs` |
| [x] | Crash protection: autosave every minute, recovery on next start | 5 | `Services/AutoSaveService.cs` |
| [x] | New picture templates (social sizes, A4, cards...) and "Start from" layouts (sizes and the 5 ready-made layouts) | 5 | `Services/DocumentTemplates.cs`, `NewImageDialog*` |
| [x] | Save as Template (user templates with a star) | 5 | `Services/UserTemplates.cs` |
| [x] | Window size / position remembered (`settings\window.json`) | 2 | `Services/WindowPlacement.cs` |
| [x] | Tutorial project (14 pages) created on first start; View > Tutorial | 10 | `Services/DemoProject.cs` |

## AI tools
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Model store: download once with permission, SHA-256 check, `models\` folder | 13 | `AI/Models/*` |
| [x] | GPU (DirectML) with CPU fallback | 13 | `AI/Runtime/OnnxSessionFactory.cs` |
| [x] | Remove Background (AI): sliders, live preview, editable cut-out keeps original (the colour-based "Remove by Color" is not ported yet) | 13 | `AI/Effects/*`, `AI/Segmentation/SubjectSegmenter.cs`, `Core/Effects/Background/*` |
| [x] | Refine brush (R): restore / remove, works after transform | 13 | `Tools/Painting/RefineTool.cs` |
| [x] | Select Subject | 13 | `SubjectSegmenter.cs` |
| [x] | Remove / Fill Selection (LaMa) on a new "AI Fill" layer | 13 | `AI/Inpainting/*`, `Commands/InpaintCommands.cs` |
| [x] | Remove Brush (AI) | 13 | `Tools/Retouch/RemoveBrushTool.cs` |
| [x] | AI Upscale / Clean Up (1x–4x, tiled, progress, cancel) | 13 | `AI/Upscaling/AiUpscaler.cs`, `Commands/UpscaleCommands.cs` |
| [x] | Click Select (MobileSAM, click or box, Shift/Alt) | 13 | `AI/Segmentation/ClickSegmenter.cs`, `Tools/Selecting/ClickSelectTool.cs` |
| [x] | AI Models dialog (list, size, download, delete, show folder) | 13 | `Views/Dialogs/AiModelsWindow*` |

## Objects
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | 36 shapes with special settings (points, hole, angle...) and shape menu with pictures | 8 | `Core/Objects/ShapeObject.cs`, `ShapeVertices.cs`, `Rendering/ShapeGeometryFactory.cs` |
| [x] | Fill, border, corners, solid / dashed / dotted, dash and gap, arrow heads | 8 | `ObjectStyle.cs` |
| [x] | Lines: bend, arc / S-wave, Edit as curve (double-click a curve with Select / Move) | 8 | `LineCurves.cs` |
| [x] | Curve tool (P): Bezier shapes and lines, point editing | 8 | `Tools/Shapes/CurveTool.cs`, `CurvePath.cs` |
| [x] | Placed pictures resizable without quality loss | 8 | `ImageObject.cs` |
| [x] | Select, multi-select, handles, rotate (Shift 15°), Keep proportions | 8 | `Tools/Selecting/MoveTool.cs`, `TransformHandles.cs` |
| [x] | Group / ungroup, edit inside a group, groups listed in Layers panel (ungrouping a group with text waits for Phase 9) | 8 | `GroupObject.cs`, `GroupChildViewModel.cs` |
| [~] | Lock (click-through), duplicate, order, flip, convert to pixels (done); crop one object (comes with the Crop tool, Phase 11) | 8 | `ObjectTransforms.cs`, `LayerCommands.cs` |
| [x] | Reshape corners (perspective / warp) | 8 | `LayerObject.cs` (Warp), `Core/Imaging/QuadWarp.cs` |
| [x] | Snapping, smart guides, equal spacing, Alt to disable | 8 | `Core/Snapping/SnapSession.cs`, `Tools/SnapDrag.cs` |
| [x] | Align / distribute, Align 3 x 3 grid in Properties | 8 | `Core/Layout/Alignment.cs`, `AlignCommands.cs` |
| [x] | Text: fonts, size, bold, italic, align, color, outline, letter / word / line gap | 9 | `TextObject.cs`, `Rendering/SpacedText.cs` |
| [x] | Text on a curve | 9 | `Rendering/ArcText.cs` |
| [x] | 16 text designs with previews | 9 | `TextPreset.cs`, `TextPresetPreviews.cs` |
| [x] | Gradients (2–8 colors, angle, radial, 11 presets, border, picture overlay, gradient background) | 9 | `GradientStyle.cs` |
| [x] | Drop shadow (10 presets, size), glow, inner shadow, outline | 9 | `ShadowFx.cs`, `ShadowPreset.cs`, `SurfaceFx.cs`, `EffectStyles.cs` |
| [x] | 3D look (5 presets, depth, angle, edges, side color) | 9 | `ThreeDFx.cs`, `ThreeDStyle.cs` |
| [~] | Properties > Adjust per object: blur (done); brightness ... tint come with the Adjust tab (Phase 11) | 9 | `LayerObject.cs` (Tone, BlurAmount) |
| [x] | Saved Styles ("Save this look...") | 9 | `Services/StyleLibrary.cs` |
| [x] | Frames, Photo Grid (10 layouts), Fill Frames, move / zoom picture in frame | 9 | `PictureFill.cs`, `Core/Layout/CollageLayouts.cs`, `PhotoGridDialog*` |
| [x] | Multi-selection Properties (common sections, one undo step) | 9 | `ObjectPropertiesViewModel.cs` |
| [x] | Copy / paste objects as editable objects between projects | 6 | `ClipboardService.cs` |

## Painting, selecting, retouching
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Rectangle, ellipse, lasso, polygon lasso, pen, magic wand, quick select, select layer content | 6 | `Core/Selections/*`, `Tools/Selecting/*` |
| [x] | Move pixels (nothing cut off), copy, cut, paste, delete, fill, invert, Reveal All, Split to New Layer | 6 | `PixelMover.cs`, `EditCommands.cs`, `MaskCommands.cs` |
| [x] | Brush types: round, square, calligraphy, pencil, chalk, spray, splatter | 7 | `Core/Painting/BrushStroke.cs`, `BrushType.cs` |
| [x] | Eraser, bucket, eyedropper, FG / BG colors | 7 | `Tools/Painting/*`, `Core/Painting/FloodFill.cs` |
| [x] | HSV color picker, recent colors, swatches | 7 | `ColorPickerDialog*`, `ColorsViewModel.cs` |
| [x] | Layer masks, Mask Brush (K), Clip to Layer Below | 7 | `LayerMask.cs`, `Tools/Painting/MaskBrushTool.cs` |
| [x] | Clone Stamp, Spot Healing, Healing Brush, Patch, Dodge / Burn, Blur / Sharpen | 12 | `Core/Retouch/*`, `Tools/Retouch/*` |
| [x] | Liquify | 12 | `Core/Painting/Liquify.cs`, `LiquifyTool.cs` |
| [x] | Tool groups in the toolbox (triangle, right-click list) | 2 | `ViewModels/ToolGroup.cs`, `ToolboxPanel*` |

## Layers and history
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Add, duplicate, delete, reorder (drag), show / hide, lock, opacity, rename | 4 | `LayerCommands.cs`, `LayersPanel*` |
| [x] | Merge down, merge selected (Ctrl+E), flatten, select / lock / unlock all | 4 | `LayerCommands.cs` |
| [x] | Blend modes (8) live | 3, 4 | `Core/Imaging/BlendMath*.cs` |
| [x] | Adjustment layers incl. Levels and Curves | 11 | `AdjustmentStyle.cs`, `CurveMath.cs`, `CurveEditor.cs` |
| [x] | Every layer has the same Properties (pixel layer → picture automatically) | 8 | `ObjectPropertiesViewModel.cs` |
| [x] | Undo / redo everything, History panel, memory cap | 4 | `Core/History/*` |
| [x] | Layer thumbnails redrawn after a change, not during drag | 4 | `LayerViewModel.cs` |

## Image, adjustments, filters
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Image Size (Lanczos, Bicubic, Bilinear, Nearest, % buttons) | 11 | `Core/Transforms/Resampler.cs`, `ImageSizeDialog*` |
| [x] | Canvas Size with anchor, Trim, rotate 90/180/any, flip | 11 | `ImageTransforms.cs`, `CanvasSizeDialog*` |
| [x] | Crop tool (ratios, new layer, original hidden), Perspective Crop, Straighten | 11 | `Tools/Transform/*`, `PerspectiveTools.cs` |
| [x] | Adjust: Recolor, Auto Levels, Brightness/Contrast, Exposure/Gamma, Hue/Saturation, Vibrance, Temperature/Tint, Grayscale, Invert, Sepia, Posterize, Threshold | 11 | `Core/Effects/Adjustments/*`, `RecolorDialog*` |
| [x] | Filter: Enhance, Gaussian Blur, Motion Blur, Sharpen, Reduce Noise, Add Noise, Pixelate, Vignette, Emboss, Find Edges | 11 | `Core/Effects/Filters/*` |
| [x] | Live preview; only the selection is changed | 11 | `Services/EffectPreviewSession.cs`, `EffectApplier.cs` |
| [x] | Split Asset Sheet | 14 | `Core/Sheets/SheetSplitter.cs`, `SplitSheetDialog*` |
| [x] | Magic Resize (optionally every page) | 14 | `Core/Layout/MagicResize.cs` |

## Insert
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | QR code (editable, repair level, styles) | 10 | `Core/Qr/QrEncoder.cs`, `QrRenderer.cs` |
| [x] | Barcodes: Code 128, EAN-13, UPC-A, EAN-8, Code 39 | 10 | `Core/Barcodes/BarcodeEncoder.cs`, `BarcodeRenderer.cs` |
| [x] | Charts: columns, bars, line, pie, donut, area, lollipop, half donut, radial bars, progress bars, funnel | 10 | `ChartObject.cs`, `ChartRenderer.cs` |
| [x] | Icons (~150, search, categories) | 10 | `Services/IconCatalog.cs`, `IconPickerDialog*` |
| [x] | Badges (52), Graphics (51), SmartArt (15 layouts) | 10 | `Rendering/Badges.cs`, `Graphics.cs`, `SmartArt.cs` |
| [x] | Heading / Subheading / Body text buttons | 10 | `InsertCommands.cs` |
| [x] | Brand Kit (12 colors, 3 fonts, colors from picture) | 10 | `BrandKitDialog*`, `Core/Imaging/PaletteExtractor.cs` |

## Export
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Export dialog: format, quality, size, crop, real file size (also HEIC / AVIF / RAW opening through Windows) | 14 | `ExportDialog*`, `ExportCropBox.cs`, `Core/IO/ExportSettings.cs` |
| [x] | PNG, JPG, WEBP, ICO, BMP, TIFF, GIF | 14 | `Core/IO/*Encoder.cs` |
| [x] | Export All Pages, Export For presets | 14 | `PagesExportCommands.cs`, `ExportPresets.cs`, `ExportForDialog*` |
| [x] | PDF, animated GIF | 14 | `IO/Pdf/PdfWriter.cs`, `IO/GifEncoder.cs` |
| [x] | SVG export and import | 14 | `IO/Svg/SvgFile.cs` |
| [x] | PSD import and export | 14 | `IO/Psd/PsdFile.cs` |
| [x] | PowerPoint export (editable / pictures) | 14 | `IO/Pptx/*` |
| [x] | Batch Convert Folder | 14 | `IO/BatchConverter.cs`, `BatchDialog*` |

## UI and settings
| Done | Feature | Ph | C# source |
|---|---|---|---|
| [x] | Ribbon with tabs, collapsible, File drawer | 2, 15 | `RibbonBar*`, `FileDrawer*`, `Themes/Ribbon.xaml` |
| [x] | Online templates in the Project Manager (GitHub catalog, search, categories, checked + cached downloads, works offline) | 15 | `Services/Templates/*`, `Controls/TemplatesView*`, `ProjectManagerWindow*` |
| [x] | Command search box | 15 | `RibbonBar.cs` |
| [x] | Right-click menus (canvas and layers; text only, no icons yet) | 15 | `CanvasView.ContextMenu.cs`, `LayersPanel.ContextMenu.cs` |
| [x] | Rulers, grid, guides, view aids bar | 3 | `RulerControl.cs`, `AidsOverlay.cs`, `ViewAidsBar*` |
| [x] | Options bar per tool + hint text | 2+ | `OptionsBar*`, `Tools/Options/*` |
| [x] | Keyboard shortcuts editor (F1) | 15 | `Services/Shortcuts.cs`, `ShortcutsDialog*` |
| [x] | Icon style Plain / Colorful | 15 | `Services/IconTheme.cs` |
| [x] | Quick Tour, About, Settings | 15 | `MainWindow.cs`, `SettingsDialog*` |
| [x] | Status bar (tool info, size, zoom) | 2 | `MainWindow.xaml` |
| [x] | Installer (one edition, Flash Editor; models download on demand) | 16 | `installer/*` |
