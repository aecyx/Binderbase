# App icons

> **These files are placeholders — the default Tauri logo from `npm create
tauri-app`. Replace them with real Binderbase branding before 1.0.**

## What each file is for

Every filename here is referenced by either `src-tauri/tauri.conf.json`
(`bundle.icon` array) or the Windows Store / Microsoft Store MSIX manifest
that Tauri's bundler generates. The names are convention — do **not** rename
them unless you also update the corresponding references.

### Cross-platform (referenced in `tauri.conf.json` → `bundle.icon`)

| File             | Use                                        | Required dimensions |
| ---------------- | ------------------------------------------ | ------------------- |
| `32x32.png`      | Small tray / taskbar icon                  | 32 × 32 px          |
| `128x128.png`    | Standard app icon (Linux, fallback)        | 128 × 128 px        |
| `128x128@2x.png` | HiDPI app icon (macOS Retina, Linux HiDPI) | 256 × 256 px        |
| `icon.icns`      | macOS application bundle icon              | Multi-resolution    |
| `icon.ico`       | Windows executable icon                    | Multi-resolution    |
| `icon.png`       | Source master (used by `tauri icon` tool)  | 1024 × 1024 px      |

### Microsoft Store / MSIX (referenced by Tauri's AppxManifest template)

These specific filenames are required by MSIX packaging. Renaming them breaks
Windows Store builds.

| File                    | Use                          | Required dimensions |
| ----------------------- | ---------------------------- | ------------------- |
| `Square30x30Logo.png`   | App list small icon          | 30 × 30 px          |
| `Square44x44Logo.png`   | Start-menu logo (100% scale) | 44 × 44 px          |
| `Square71x71Logo.png`   | Small tile                   | 71 × 71 px          |
| `Square89x89Logo.png`   | App list medium icon         | 89 × 89 px          |
| `Square107x107Logo.png` | App list large (HiDPI)       | 107 × 107 px        |
| `Square142x142Logo.png` | Medium tile                  | 142 × 142 px        |
| `Square150x150Logo.png` | Medium tile (100% scale)     | 150 × 150 px        |
| `Square284x284Logo.png` | Medium tile (HiDPI)          | 284 × 284 px        |
| `Square310x310Logo.png` | Large tile                   | 310 × 310 px        |
| `StoreLogo.png`         | Store listing thumbnail      | 50 × 50 px          |

## Easiest way to regenerate: one command

Tauri ships a CLI that generates every file above from a single source PNG:

```bash
# From the repo root. Source should be a 1024×1024 (or larger), square,
# transparent-background PNG of the Binderbase logo.
npm run tauri -- icon path/to/binderbase-logo-1024.png
```

That writes the full set into this directory, preserving filenames so
`tauri.conf.json` keeps working. Commit the result in one changeset.

## Design notes for the real logo

- Square, transparent background.
- Legible down to 16–32 px (avoid thin strokes or fine text).
- Keep padding inside the canvas so the logo doesn't touch the edges — some
  platforms crop or round-mask icons.
- macOS expects a ~10–15% transparent margin by convention; Windows does
  not, but a small margin still looks better.
- Supply the source at **1024 × 1024 px** so every downscale has room.
