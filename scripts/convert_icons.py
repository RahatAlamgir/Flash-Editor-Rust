"""Converts the C# line icons (Themes/Icons.xaml) into Rust source.

The C# icons are path geometries drawn on a 20 x 20 grid and stroked (1.4 px, round ends). Their path syntax is the
same as SVG, so each one becomes an SVG path string. Output: crates/fe-ui-kit/src/icons/paths.rs

Run again whenever the C# icons change:
    python scripts/convert_icons.py
"""

import pathlib
import re

CSHARP_ICONS = pathlib.Path(r"D:\AiBuild\Flash Editor\src\FlashEditor.App\Themes\Icons.xaml")
OUTPUT = pathlib.Path(__file__).resolve().parent.parent / "crates" / "fe-ui-kit" / "src" / "icons" / "paths.rs"

# Icons the C# app draws inline in its XAML (not in Icons.xaml).
EXTRA = {
    "Folder": "M3,6 L8,6 L10,8 L17,8 L17,16 L3,16 Z",
    "Undo": "M7,6 L3,10 L7,14 M3,10 L12,10 C15,10 17,12 17,14.5",
    "Redo": "M13,6 L17,10 L13,14 M17,10 L8,10 C5,10 3,12 3,14.5",
    "Chevron": "M4,6 L8,10 L12,6",
    "Play": "M6.5,4.5 L15.5,10 L6.5,15.5 Z",
}


def main() -> None:
    text = CSHARP_ICONS.read_text(encoding="utf-8")
    icons = dict(re.findall(r'<Geometry x:Key="Icon\.([^"]+)">([^<]+)</Geometry>', text))
    icons.update(EXTRA)
    lines = [
        "//! Line icons converted from the C# `Themes/Icons.xaml` by `scripts/convert_icons.py`. Do not edit by hand.",
        "//! Each path is drawn on a 20 x 20 grid and stroked (see `super::svg_for`).",
        "",
        "/// (name, SVG path data), sorted by name.",
        "pub const ICON_PATHS: &[(&str, &str)] = &[",
    ]
    for name in sorted(icons):
        data = " ".join(icons[name].split())
        lines.append(f'    ("{name}", "{data}"),')
    lines.append("];")
    OUTPUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(icons)} icons to {OUTPUT}")


if __name__ == "__main__":
    main()
