<div align="center">
  <img src="docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Private Local Photo Manager</h1>
  <h3>Open-source desktop photo manager for macOS, Windows, and Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>


English | [Deutsch](i18n/README.de.md) | [Français](i18n/README.fr.md) | [Español](i18n/README.es.md) | [Português](i18n/README.pt.md) | [Русский](i18n/README.ru.md) | [简体中文](i18n/README.zh-CN.md) | [日本語](i18n/README.ja.md) | [한국어](i18n/README.ko.md)

Lap is an open-source, local-first photo manager for browsing family albums, finding old photos quickly, and managing large personal media libraries offline.
It is a privacy-focused alternative to cloud photo services: no forced upload, local AI search, folder-first workflow, and free to use.

- Website: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Demo: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacy: [PRIVACY.md](PRIVACY.md)

## Download Lap

### macOS with Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

### Manual Download

Open the [latest release page](https://github.com/julyx10/lap/releases/latest), then download the file that matches your system:

| Platform | Package | Note |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarized by Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Unsigned — if SmartScreen blocks the download, click **Keep anyway** |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | For Debian-based distros (Ubuntu, Debian, Linux Mint, etc.) |

## Screenshots

<p align="center">
  <img src="docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap local photo library manager screenshot" width="900">
</p>

<p align="center">
  <img src="docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap local AI photo search screenshot" width="900">
</p>

## Press
- **Windows Central** — [After testing LAP Photo Manager on Windows 11, I'm convinced it outperforms the built-in Photos app in all the ways that matter](https://www.windowscentral.com/microsoft/windows-11/lap-photo-manager-for-windows-11-is-it-better-than-photos-i-think-so)

## Why Lap

- **No cloud required**: keep your library on your own disk instead of uploading it to a hosted service.
- **Private by default**: processing happens locally, so your photos stay under your control.
- **Free to use**: no subscription plan or recurring fee.
- **Folder-first**: work directly with your existing folders, no import step required.
- **High performance for large libraries**: optimized for smooth browsing and organization across huge media collections (100k+ files per library).

## Features

- **Browse and filter** by date, location, camera, lens, tags, favorites, ratings, and faces (BETA).
- **Manage multiple libraries** and switch between them quickly.
- **Find duplicates** and batch move unwanted copies to trash.
- **Edit in place** with crop, rotate, flip, resize, and basic adjustments.
- **Keep folders in sync** with filesystem-aware operations and refresh support.
- **Use local search tools** such as text/image search, similar-image search, face clustering, and smart tags.
- **Search in 50+ languages** with optional multilingual models, available as an additional download when needed.
- **Open modern image formats** including WebP, HEIC/HEIF/HIF, AVIF, and JXL (JPEG XL).
- **View RAW photos** with built-in decoding for 20+ camera RAW formats (CR2, NEF, ARW, DNG, etc.).
- **Broad video compatibility** supporting MP4, MOV, AVI, MKV, and 20+ other formats with cross-platform optimization.

## Uninstall Lap

Lap works directly with your existing photo folders. Uninstalling Lap, or deleting its database and cache files, does **not** delete your original photos.

The standard uninstall steps remove the application. To remove Lap completely, quit Lap first, uninstall the application, then delete its local database, thumbnail cache, and configuration files using the cleanup command for your platform.

### macOS

If you installed Lap with Homebrew:

```bash
brew uninstall --cask lap
```

For a manual installation, quit Lap and move `Lap.app` from the `Applications` folder to the Trash.

To remove all Lap database, cache, and configuration files:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Open **Settings > Apps > Installed apps**, find **Lap**, and select **Uninstall**.

Then open PowerShell and remove all Lap database, cache, and configuration files:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

For Debian-based distributions, uninstall the package:

```bash
sudo apt remove lap
```

Then remove all Lap database, cache, and configuration files:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

If you selected a custom database storage directory in Lap settings, delete that directory separately after confirming that it contains only Lap database files.

## Build from Source

Requirements: Node.js 20+, pnpm, Rust stable.

```bash
# macOS system deps
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux system deps
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Clone and build
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Supported Formats

| Type | Formats |
| :--- | :--- |
| Images | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL |
| RAW photos | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Videos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX and more. H.264 playback is supported on all platforms, with automatic compatibility processing when native playback is unavailable. HEVC/H.265 and VP9 are natively supported on macOS. |

### Linux Video Playback Notes

On Ubuntu/Debian/Linux Mint, install these packages for better video playback support:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architecture

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Data: SQLite

### Key Libraries

| Library | Purpose |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW image decoding and thumbnail extraction |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF image decoding and preview generation |
| [FFmpeg](https://ffmpeg.org/) | Video processing and thumbnail generation |
| [ONNX Runtime](https://onnxruntime.ai/) | Local AI model inference engine |
| [CLIP](https://github.com/openai/CLIP) | Image-text similarity search |
| [InsightFace](https://github.com/deepinsight/insightface) | Face detection and recognition |
| [Leaflet](https://leafletjs.com/) | Interactive map for geotagged photos |
| [daisyUI](https://daisyui.com/) | UI component library |

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
