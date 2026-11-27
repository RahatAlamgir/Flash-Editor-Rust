# Fonts shipped inside the program

## Material Icons (Google)

- `MaterialIcons-Regular.ttf` and `MaterialIcons-Regular.codepoints` (the icon names and their codes).
- Source: https://github.com/google/material-design-icons (folder `font`).
- License: Apache License 2.0 (https://www.apache.org/licenses/LICENSE-2.0). Free to use, change and ship, also in commercial
  work. The icons are Copyright Google LLC.
- Used by Insert > Icons > "Google icons". The font is built into the program (`fe-core/src/library/material.rs`), so projects
  that use these icons look the same on a PC where the font is not installed.
