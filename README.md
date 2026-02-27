<div align="center">
  <img src="docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap</h1>
  <h3>Offline-first photo manager for large local libraries.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

Lap is a desktop photo manager for people who keep their original files in local folders and want modern search plus full control.
It is designed for everyday family albums and large personal archives (10k+ photos/videos), without cloud lock-in.

- Website: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Demo video: [https://youtu.be/niMD1tTzS24](https://youtu.be/niMD1tTzS24)
- Releases: [https://github.com/julyx10/lap/releases](https://github.com/julyx10/lap/releases)

## Why Lap

- **Folder-native workflow**: use real disk folders directly, no import library required.
- **Offline-first AI**: text/image search, similar-image search, and face clustering run locally.
- **Built for scale**: smooth browsing and organization across large libraries.
- **Privacy by design**: no cloud upload, no remote inference, no vendor lock-in.

## Features

- **Organization and filtering**: date, location, camera, tags, favorites, and faces.
- **Smart tags (family-focused)**: one-click semantic categories like people, kids, pets, food, and travel.
- **Multi-library management**: separate work/family libraries and switch quickly.
- **Duplicate cleanup**: hash-based duplicate detection and batch move to trash.
- **Built-in editor**: crop, rotate, flip, resize, and basic adjustments.
- **Disk sync**: operations in Lap map to your filesystem; external file changes can be refreshed.

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

## Comparison

| Feature | Cloud Albums (Google/iCloud) | Classic Viewers | **Lap** |
| :--- | :---: | :---: | :---: |
| Privacy | ❌ | ✅ | ✅ |
| AI Search | ✅ | ❌ | ✅ |
| Folder-native file control | ❌ | ✅ | ✅ |
| Offline-first | ❌ | ✅ | ✅ |
| Vendor lock-in | High | None | None |

## Supported Formats

| Type | Formats |
| :--- | :--- |
| Images | JPG, PNG, GIF, BMP, TIFF, WebP, HEIC |
| Videos | MP4, MOV, MKV, WebM, AVI |

## Architecture

- Core: [Tauri 2](https://tauri.app/) + Rust
- Frontend: Vue 3 + Vite
- Data: SQLite
- AI/media: CLIP, InsightFace, FFmpeg

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
