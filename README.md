<div align="center">
  <img src="docs/screenshots/logo.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap</h1>
</div>

Local-first, fast, cross‑platform photo and video manager built with Tauri 2 (Rust) and Vue 3. Lap helps you browse large folders, curate albums, tag and favorite items, and view images/videos in a fluid desktop experience.

## Download

| Platform | Download | File Size | Notes |
|:--|:--|:--|:--|
| **macOS (Apple Silicon)** | [Lap_0.1.0_aarch64.dmg](https://github.com/julyx10/lap/releases/download/v0.1.0/Lap_0.1.0_aarch64.dmg) | 307 MB | Includes embedded AI models for offline capability |
| **Windows (Intel)** | [ - ](-) | ` - ` | Coming soon |

> Notes: Visit the [Releases page](https://github.com/julyx10/lap/releases) for older versions and changelogs.

<table>
  <tr>
    <td align="center">
      <strong>Library - Dark Theme</strong><br>
      <img src="docs/screenshots/01%20lap-main-dark.png" alt="Library" width="520">
      <br>
      <em>Quickly browse all thumbnails. Display in comfortable layout.</em>
    </td>
    <td align="center">
      <strong>AI Search - Light Theme</strong><br>
      <img src="docs/screenshots/02%20lap-ai-search.png" alt="AI Search" width="520">
      <br>
      <em>Search by image content using AI. Display in filmstrip view.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Calendar</strong><br>
      <img src="docs/screenshots/03%20lap-calendar.png" alt="Calendar" width="520">
      <br>
      <em>Find items by year, month, or day. Display in compact layout.</em>
    </td>
    <td align="center">
      <strong>Tags</strong><br>
      <img src="docs/screenshots/04%20lap-tag.png" alt="Tags" width="520">
      <br>
      <em>Browse by tag; add/remove tags on selected items.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Location</strong><br>
      <img src="docs/screenshots/05%20lap-location.png" alt="Location" width="520">
      <br>
      <em>Find items by location. Display in map view based on GPS metadata.</em>
    </td>
    <td align="center">
      <strong>Camera</strong><br>
      <img src="docs/screenshots/06%20lap-camera.png" alt="Camera" width="520">
      <br>
      <em>Find items by camera make and model.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Edit Image</strong><br>
      <img src="docs/screenshots/07%20lap-edit-image.png" alt="Edit Image" width="520">
      <br>
      <em>Perform the basic edits on images.</em>
    </td>
    <td align="center">
      <strong>Video</strong><br>
      <img src="docs/screenshots/08%20lap-video.png" alt="Video" width="520">
      <br>
      <em>Play videos with built-in player.</em>
    </td>
  </tr>
</table>

> Notes: The sample images in above screenshots are sourced from [Wikimedia Commons](https://commons.wikimedia.org/).  

## Why LAP?

**LAP** stands for **Local-first, AI-powered Photo management**. The name also represents the three hierarchical layers of organization: **Library**, **Album**, and **Photo**.

LAP was created because I cannot find a satisfying photo management app on macOS for users who own **large and long-term photo collections**.

Many existing apps either feel too limited, or require importing photos into closed systems that duplicate files and restrict workflows.

LAP focuses on **organizing, browsing, and finding photos**, not replacing professional editing tools.

## Who Is LAP For?

LAP is designed for users who manage **a large number of photos**, such as:

- Photography enthusiasts and professionals
- Families maintaining long-term photo archives
- Anyone who prefers folder-based photo organization

These users need **fast browsing, flexible navigation, and powerful search**, rather than heavy editing features.

## What LAP Is Not

- LAP is **not** a cloud-based photo service. The only scenario it access network is to display map.
- LAP is **not** a photo syncing or backup solution.
- LAP is **not** a professional image editing tool.

Its goal is to manage, browse, and search photos efficiently, while keeping files fully under the user’s control.

## Features

### 📂 Organization & Libraries
- **Multi-Library Support**: Create and manage separate photo libraries; switch between them instantly.
- **Folder-Based Structure**: Reflects your actual file system. Add any folder as an album.
- **Lazy Loading**: Handles giant photo collections with ease using virtual scrolling and concurrent thumbnail generation.

### 🤖 AI Search & Smart Catalog
- **AI Search**: Search by image content or find similar images using the local AI.
- **Face Recognition**: Private, offline face detection and clustering to group photos by person.
- **Metadata Grouping**: Auto-organize by **Location** (GPS/Reverse Geocoding), **Camera** (Make/Model), and **Time** (Year/Month/Day).

### 🔒 Privacy & Control
- **Local-First**: All AI processing (face recognition) and indexing happens on your device. No data leaves your machine.
- **Safe File Operations**: Rename, move, copy, and delete (Trash support) directly from the app.
- **No Lock-In**: Your photos stay in your folders.

### 🎨 UX & Customization
- **Theme Support**: Seamless switching between **Light** and **Dark** modes to suit your environment.
- **Multi-Language**: Fully localized in English, Simplified Chinese, German, French, Japanese, Korean, Russian, Spanish, and Portuguese.

## Supported Formats

| Type | Formats | Notes |
| :--- | :--- | :--- |
| **Images** | JPG, PNG, GIF, BMP, TIFF, WebP, HEIC | HEIC depends on OS support. |
| **Videos** | MP4, MOV, MKV, WebM, AVI | Playback depends on OS WebView codecs. |

## Development

### Tech Stack
- **Core**: [Tauri 2](https://tauri.app) (Rust), cross-platform desktop app framework
- **Frontend**: Vue 3, Vite, Tailwind CSS, daisyUI
- **Data**: SQLite, locally embedded for indexing and metadata storage
- **AI/Media**: Based on the latest open-source LLM for image understanding

### Build & Run

**Prerequisites**: Node.js 18+, Rust (stable), and platform build tools (Xcode for macOS, MSVC for Windows).

```bash
# 1. Install frontend dependencies
cd src-vite
pnpm install

# 2. Run in development mode
cd ../src-tauri
cargo tauri dev

# 3. Build for production
cargo tauri build
```

## License

Licensed under **GPL-3.0-or-later**. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built on the shoulders of giants: Tauri, Vue, Rusqlite, FFmpeg, and many other open-source crates and libraries.
