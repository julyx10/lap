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
  - File name search; filter by file type; multiple sort types with asc/desc
- Preview pane
  - Inline image/video preview with rotate and comments
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
  - Theme switching (35 built-in themes per your preference)
  - Multi-language UI (switch language at runtime)

## Screenshots

<table>
  <tr>
    <td align="center">
      <strong>Home</strong><br>
      <img src="docs/screenshots/02%20screenshot-home.png" alt="Home" width="520">
      <br>
      <em>Quickly browse all thumbnails with live preview and inline actions.</em>
    </td>
    <td align="center">
      <strong>Image Viewer</strong><br>
      <img src="docs/screenshots/10%20screenshot-imageviewer.png" alt="Image Viewer" width="520">
      <br>
      <em>Zoom (fit/actual), rotate, favorite/tag, comments, slideshow, file info.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Home — Dark Theme</strong><br>
      <img src="docs/screenshots/01%20screenshot-home-dark.png" alt="Home — Dark Theme" width="520">
      <br>
      <em>Dark mode for low‑light environments; context menu to manage selections.</em>
    </td>
    <td align="center">
      <strong>Home — Light Theme</strong><br>
      <img src="docs/screenshots/03%20screenshot-home-light.png" alt="Home — Light Theme" width="520">
      <br>
      <em>Light mode; supports all 35 built‑in DaisyUI themes.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Albums</strong><br>
      <img src="docs/screenshots/04%20screenshot-album-video.png" alt="Albums" width="520">
      <br>
      <em>Album with videos; preview pane plays clips inline.</em>
    </td>
    <td align="center">
      <strong>Favorites</strong><br>
      <img src="docs/screenshots/05%20screenshot-favorites.png" alt="Favorites" width="520">
      <br>
      <em>Star files and folders you use most for quick access.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Tags</strong><br>
      <img src="docs/screenshots/06%20screenshot-tags.png" alt="Tags" width="520">
      <br>
      <em>Browse by tag; add/remove tags on selected items.</em>
    </td>
    <td align="center">
      <strong>Calendar</strong><br>
      <img src="docs/screenshots/07%20screenshot-calendar.png" alt="Calendar" width="520">
      <br>
      <em>Find items by year, month, or day.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Locations</strong><br>
      <img src="docs/screenshots/08%20screenshot-locations.png" alt="Locations" width="520">
      <br>
      <em>Group by place/admin area using EXIF GPS and reverse geocoding.</em>
    </td>
    <td align="center">
      <strong>Camera View</strong><br>
      <img src="docs/screenshots/09%20screenshot-camera.png" alt="Camera View" width="520">
      <br>
      <em>Explore by camera make/model and compare counts per device.</em>
    </td>
  </tr>
</table>

Notes
- The sample images in above screenshots are sourced from [Wikimedia Commons](https://commons.wikimedia.org/).  

## Supported file types

Images
- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- BMP (.bmp)
- TIFF (.tif, .tiff)
- WebP (.webp)
- HEIC (.heic) - depends on your OS

Videos
- Thumbnail generation: MP4, MOV, MKV, WebM, AVI, M4V and many others supported by FFmpeg
- Playback in app: depends on your OS WebView codec support. Tested working:
  - MP4 (H.264/AAC)
  - MOV (H.264/AAC)
  - WebM/MKV support varies by platform; on macOS, WebM/VP9 may not play in the system WebView.

Notes
- Camera RAW formats are not currently supported for inline viewing.

## Common keyboard shortcuts

Main Window and Viewer
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

## Tech stack

- Desktop: Tauri 2 (Rust), custom asset protocol and Tauri plugins (fs, os, shell, dialog, window-state)
- Backend: Rust, rusqlite (SQLite), exif parsing, reverse geocoding, FFmpeg (via ffmpeg-next), image processing
- Frontend: Vue 3, Vite 6, TypeScript, Pinia, Tailwind CSS v4 + DaisyUI, Video.js, vue-i18n

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

## Configuration & security

- Asset protocol enabled; access scopes limited to safe locations (e.g., $HOME, $PICTURES, $VIDEOS, $DOCUMENT, $DESKTOP, $DOWNLOAD)
- All data and processing remain local; no network services are required

## Notes on data

- jc-photo maintains a local SQLite database to index albums, folders, files, tags, and metadata
- Thumbnails are generated locally; progress is shown while they build

## Acknowledgments

Thanks to the open-source libraries and tools that make jc-photo possible:

- Desktop (Rust/Tauri)
  - Tauri 2 (tauri, tauri-build, plugins: window-state, shell, os, fs, dialog)
  - serde, serde_json
  - rusqlite (SQLite)
  - chrono
  - kamadak-exif
  - reverse_geocoder
  - ffmpeg-next (FFmpeg)
  - image, imagesize
  - tokio
  - dirs, walkdir
  - trash
  - base64
  - opener
  - arboard
  - pinyin
  - once_cell

- Frontend (Vue)
  - Vue 3, vue-router, vue-i18n, @vue/compiler-sfc
  - Vite, @vitejs/plugin-vue, vite-plugin-vue-devtools, vite-svg-loader
  - Tailwind CSS v4, DaisyUI, @tailwindcss/vite, @tailwindcss/postcss, PostCSS, Autoprefixer
  - TypeScript, @types/node
  - Pinia, pinia-plugin-persistedstate
  - Video.js
  - date-fns, lodash
  - vue-draggable-plus, vue-lazyload

## License

This project is dual-licensed under MIT OR Apache-2.0.

See LICENSE, LICENSE-MIT, and LICENSE-APACHE for details.

