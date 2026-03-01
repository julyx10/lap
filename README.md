<div align="center">
  <img src="docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap</h1>
  <h3>Private photo manager for family albums and local memories.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

Lap is a desktop photo manager for people who want an easier way to browse family albums, find old photos, and revisit meaningful moments from a growing local library.
It helps you organize and explore your collection in practical ways, while keeping a simple promise: no cloud upload, private by default, and free to use.

- Website: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Demo video: [https://youtu.be/niMD1tTzS24](https://youtu.be/niMD1tTzS24)
- Releases: [https://github.com/julyx10/lap/releases](https://github.com/julyx10/lap/releases)

## Why Lap

- **No cloud required**: keep your library on your own disk instead of uploading it to a hosted service.
- **Private by default**: processing happens locally, so your photos stay under your control.
- **Free to use**: no subscription plan or recurring fee.
- **Folder-first**: work directly with your existing folders, no import step required.
- **Built for large libraries**: smooth browsing and organization across thousands of photos and videos.

## Features

- **Browse and filter** by date, location, camera, tags, favorites, ratings, and faces.
- **Manage multiple libraries** and switch between them quickly.
- **Find duplicates** and batch move unwanted copies to trash.
- **Edit in place** with crop, rotate, flip, resize, and basic adjustments.
- **Keep folders in sync** with filesystem-aware operations and refresh support.
- **Use local search tools** such as text/image search, similar-image search, face clustering, and smart tags.

## Download

| Platform | Download | Notes |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | [Download .dmg (aarch64)](https://github.com/julyx10/lap/releases/latest) | Notarized by Apple, fully tested |
| **macOS (Intel)** | [Download .dmg (x64)](https://github.com/julyx10/lap/releases/latest) | Notarized by Apple, not fully tested yet |
| **Linux (Ubuntu/Debian)** | [Download .deb (amd64)](https://github.com/julyx10/lap/releases/latest) | Community testing welcome |
| **Windows (x64)** | [Download .msi](https://github.com/julyx10/lap/releases/latest) | Tested on Windows 11 |

## Product Snapshot

<p align="center">
  <img src="docs/public/screenshots/lap-home3.png" alt="Lap Library" width="900">
</p>

<p align="center">
  <img src="docs/public/screenshots/lap-search.png" alt="Lap AI Search" width="900">
</p>

> Screenshot sample images come from [Wikimedia Commons](https://commons.wikimedia.org/).

## Build from Source

Requirements:
- Node.js 20+, pnpm 8+
- Rust stable
- macOS: `xcode-select --install`, `brew install nasm pkg-config`
- Linux: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf nasm clang pkg-config`

Run locally:

```bash
# Download AI models
./scripts/download_models.sh
# Windows (PowerShell): .\scripts\download_models.ps1

# Install frontend deps
cd src-vite && pnpm install && cd ..

# Run dev app
cargo tauri dev
```

## Supported Formats

| Type | Formats |
| :--- | :--- |
| Images | JPG, PNG, GIF, BMP, TIFF, WebP, HEIC |
| Videos | MP4, MOV, MKV, WebM, AVI |

## Architecture

- Core: [Tauri 2](https://tauri.app/) + Rust
- Frontend: Vue 3 + Vite
- Data: SQLite
- Media and local search: CLIP, InsightFace, FFmpeg

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
