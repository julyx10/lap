<div align="center">
  <img src="docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Private Local Photo Manager</h1>
  <h3>Open-source desktop photo manager for macOS, Windows, and Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

Lap is an open-source desktop photo manager and local photo organizer for people who want to browse family albums, find old photos quickly, and manage large personal media libraries offline.
It is built as a privacy-focused alternative to cloud photo services: no forced upload, local AI photo search, folder-first workflow, and free to use.

- Website: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Demo: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacy: [PRIVACY.md](PRIVACY.md)

## Download Lap

Open the [latest release page](https://github.com/julyx10/lap/releases/latest), then download the file that matches your system:

| Platform | Package | Status |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Notarized by Apple |
| **macOS (Intel)** | `x64.dmg` | Notarized by Apple |
| **Windows 10/11 (x64)** | `.msi` | Currently unsigned (SmartScreen warning may appear) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Linux adaptation and testing are not completed yet |

## Screenshots

<p align="center">
  <img src="docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap local photo library manager screenshot" width="900">
</p>

<p align="center">
  <img src="docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap local AI photo search screenshot" width="900">
</p>

> Screenshot sample images come from [Wikimedia Commons](https://commons.wikimedia.org/).

## Why Lap

- **No cloud required**: keep your library on your own disk instead of uploading it to a hosted service.
- **Private by default**: processing happens locally, so your photos stay under your control.
- **Free to use**: no subscription plan or recurring fee.
- **Folder-first**: work directly with your existing folders, no import step required.
- **Built for large libraries**: smooth browsing and organization across thousands of photos and videos.

## Features

- **Browse and filter** by date, location, camera, lens, tags, favorites, ratings, and faces(BETA).
- **Manage multiple libraries** and switch between them quickly.
- **Find duplicates** and batch move unwanted copies to trash.
- **Edit in place** with crop, rotate, flip, resize, and basic adjustments.
- **Keep folders in sync** with filesystem-aware operations and refresh support.
- **Use local search tools** such as text/image search, similar-image search, face clustering, and smart tags.
- **View RAW photos** with built-in decoding for 20+ camera RAW formats (CR2, NEF, ARW, DNG, etc.).

## Planned Features

- **Support more image and video formats** such as Live Photos and Motion Photos.
- **Expand metadata support** for standards commonly used in photography workflows, including EXIF, XMP, and IPTC.

## Build from Source

Requirements: Node.js 20+, pnpm, Rust stable.

```bash
# macOS system deps
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux system deps (Ubuntu/Debian)
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Clone and build
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Supported Formats

| Type | Formats |
| :--- | :--- |
| Images | JPG, JPEG, PNG, GIF, BMP, TIFF, WebP, AVIF, HEIC, HEIF |
| RAW photos | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Videos | MP4, MOV, MKV, WebM |

## Architecture

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Data: SQLite

### Key Libraries

| Library | Purpose |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW image decoding and thumbnail extraction |
| [FFmpeg](https://ffmpeg.org/) | Video decoding and thumbnail generation |
| [ONNX Runtime](https://onnxruntime.ai/) | Local AI model inference engine |
| [CLIP](https://github.com/openai/CLIP) | Image-text similarity search |
| [InsightFace](https://github.com/deepinsight/insightface) | Face detection and recognition |
| [Leaflet](https://leafletjs.com/) | Interactive map for geotagged photos |
| [Video.js](https://videojs.com/) | Video playback |
| [daisyUI](https://daisyui.com/) | UI component library |

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
