# jc-photo

Local-first, fast, cross‑platform photo and video manager built with Tauri 2 (Rust) and Vue 3. jc-photo helps you browse large folders, curate albums, tag and favorite items, and view images/videos in a fluid desktop experience.


## Highlights

- Albums and folders
  - Add folders as albums; browse nested folders; optionally hide albums
  - Favorite folders for quick access; order albums; reveal folders in Finder/Explorer
- Fast thumbnail grid
  - Lazy, concurrent thumbnail generation; progress indicator while thumbs build
  - Configurable thumbnail size and label styles (name/size/dimensions/dates/duration)
- Search, filter and sort
  - Full‑text search; filter by file type; multiple sort types with asc/desc
- Preview pane and status bar
  - Inline image/video preview with rotate and comments; status bar with totals and selection info
- Powerful image viewer
  - Keyboard driven navigation (prev/next/home/end), slideshow with configurable interval
  - Zoom in/out/fit/actual; rotate; favorite; tag; comment overlay; copy image to clipboard; print
  - Full screen mode (with pin/unpin UI) and file info panel
- Metadata views
  - Favorites (files/folders), Tags, Calendar (year/month/day), Camera (make/model), Location (admin area/place)
- File operations (local, safe)
  - Rename, move, copy, delete (uses Trash on supported OS), go to folder, reveal in system file manager
- Tagging and comments
  - Add/remove tags; edit per‑file comments; see tag state directly in grid and viewer
- Local, private, and fast
  - All processing is on your machine; SQLite indexing, EXIF parsing, and thumbnails via Rust crates (EXIF, image, FFmpeg)
- Appearance and localization
  - Theme switching (light/dark per your preference)
  - Multi-language UI via vue-i18n (switch language at runtime)


## Tech stack

- Desktop: Tauri 2 (Rust), custom asset protocol and Tauri plugins (fs, os, shell, dialog, window-state)
- Backend: Rust, rusqlite (SQLite), exif parsing, reverse geocoding, FFmpeg (via ffmpeg-next), image processing
- Frontend: Vue 3, Vite 6, TypeScript, Pinia, Tailwind CSS v4 + DaisyUI, Video.js, vue-i18n

## Supported file types

Images (decoded locally via Rust image crate)
- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- BMP (.bmp)
- TIFF (.tif, .tiff)
- WebP (.webp)

Videos
- Thumbnail generation: MP4, MOV, MKV, WebM, AVI, M4V and many others supported by FFmpeg
- Playback in app: depends on your OS WebView codec support. Tested working:
  - MP4 (H.264/AAC)
  - MOV (H.264/AAC)
  - WebM/MKV support varies by platform; on macOS, WebM/VP9 may not play in the system WebView.

Notes
- Camera RAW formats and HEIC/HEIF are not currently supported for inline viewing.


## Screenshots

<table>
  <tr>
    <td align="center">
      <strong>Home Dark</strong><br>
      <img src="docs/screenshots/01%20screenshot-home-dark.png" alt="Home Dark">
    </td>
    <td align="center">
      <strong>Home</strong><br>
      <img src="docs/screenshots/02%20screenshot-home.png" alt="Home">
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Home Light</strong><br>
      <img src="docs/screenshots/03%20screenshot-home-light.png" alt="Home Light">
    </td>
    <td align="center">
      <strong>Album Video</strong><br>
      <img src="docs/screenshots/04%20screenshot-album-video.png" alt="Album Video">
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Favorites</strong><br>
      <img src="docs/screenshots/05%20screenshot-favorites.png" alt="Favorites">
    </td>
    <td align="center">
      <strong>Tags</strong><br>
      <img src="docs/screenshots/06%20screenshot-tags.png" alt="Tags">
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Calendar</strong><br>
      <img src="docs/screenshots/07%20screenshot-calendar.png" alt="Calendar">
    </td>
    <td align="center">
      <strong>Locations</strong><br>
      <img src="docs/screenshots/08%20screenshot-locations.png" alt="Locations">
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Camera</strong><br>
      <img src="docs/screenshots/09%20screenshot-camera.png" alt="Camera">
    </td>
    <td align="center">
      <strong>Imageviewer</strong><br>
      <img src="docs/screenshots/10%20screenshot-imageviewer.png" alt="Imageviewer">
    </td>
  </tr>
</table>

## Requirements

- macOS, Windows, or Linux
- Rust (stable toolchain) and Cargo
- Node.js >= 18 and pnpm (recommended)
- Platform build tooling
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools (MSVC)
  - Linux: system toolchain per Tauri 2 docs


## Getting started (development)

1) Install frontend dependencies

```
cd src-vite
pnpm install
```

2) Start the desktop app in dev mode (runs Vite dev server and opens Tauri)

```
cd ../src-tauri
cargo tauri dev
```

- The dev server URL is configured at http://localhost:3580 (see src-tauri/tauri.conf.json)
- The Tauri runner will call `pnpm dev` for the frontend using the config


## Production build

Build the optimized frontend and bundle the desktop app:

```
# From repo root (frontend build is handled by tauri beforeBuildCommand)
cd src-tauri
cargo tauri build
```

- Output artifacts (installers/binaries) are generated under `src-tauri/target` according to your OS


## Common keyboard shortcuts

Grid and viewer
- Arrow keys: Navigate items
- Home / End: Jump to first/last
- Cmd/Ctrl + Enter: Open
- Delete (or Cmd + Backspace on macOS): Delete / move to Trash
- Cmd/Ctrl + F: Toggle Favorite
- Cmd/Ctrl + T: Tag
- Cmd/Ctrl + R: Rotate
- Cmd/Ctrl + C: Copy image to clipboard (for images)

Viewer
- Space: Toggle Zoom Fit
- = / - : Zoom in / out
- 0: Zoom actual size


## Configuration & security

- Asset protocol enabled; access scopes limited to safe locations (e.g., $HOME, $PICTURES, $VIDEOS, $DOCUMENT, $DESKTOP, $DOWNLOAD)
- All data and processing remain local; no network services are required


## Notes on data

- jc-photo maintains a local SQLite database to index albums, folders, files, tags, and metadata
- Thumbnails are generated locally; progress is shown while they build
- The database file may be excluded from packaging for releases


## Contributing

Issues and PRs are welcome. Please run in dev, verify on at least one desktop OS, and keep changes local-first and privacy preserving.


## Sample Images

The sample images used in this project are sourced from [Wikimedia Commons](https://commons.wikimedia.org/).  
They are provided under various open licenses (e.g., CC0, CC-BY, CC-BY-SA, or Public Domain).  
Please refer to the [Wikimedia Commons licensing page](https://commons.wikimedia.org/wiki/Commons:Reusing_content_outside_Wikimedia) for details.


## License

- Frontend: MIT (see package.json)
- Desktop app (Rust crate): Dual-licensed under MIT OR Apache-2.0 (see Cargo.toml)

