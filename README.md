<div align="center">
  <img src="docs/public/logo.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap</h1>
  <h3>Your Photos, Your Rules. The Local-First, AI-Powered Photo Manager.</h3>
</div>

**Lap** is a lightning-fast, cross-platform photo manager built for people who love their files. It combines the privacy of folder-based organization with the power of modern AI—all running 100% locally on your device.

Browse 100,000+ photos smoothly, finding "cat in the grass" instantly, and keep your memories safe without a cloud subscription.

🌐 **Website**: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)

## Download

| Platform | Download | File Size | Notes |
|:--|:--|:--|:--|
| **macOS** | [Download Latest .dmg](https://github.com/julyx10/lap/releases/latest) | ~150 MB | Includes embedded AI models |
| **Windows** | - | - | Coming soon |

<table>
  <tr>
    <td align="center">
      <strong>Library - Dark Theme</strong><br>
      <img src="docs/public/screenshots/01%20lap-main-dark.jpg" alt="Library" width="520">
      <br>
      <em>Handle 100k+ photos with smooth scrolling and instant thumbnails.</em>
    </td>
    <td align="center">
      <strong>AI Search - Light Theme</strong><br>
      <img src="docs/public/screenshots/02%20lap-ai-search.jpg" alt="AI Search" width="520">
      <br>
      <em>"Sunset over the ocean" — Find exact moments with offline AI.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Calendar</strong><br>
      <img src="docs/public/screenshots/03%20lap-calendar.jpg" alt="Calendar" width="520">
      <br>
      <em>Time-travel through your memories by year, month, or day.</em>
    </td>
    <td align="center">
      <strong>Tags</strong><br>
      <img src="docs/public/screenshots/04%20lap-tag.jpg" alt="Tags" width="520">
      <br>
      <em>Organize your way: Bulk tagging and powerful filtering.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Location</strong><br>
      <img src="docs/public/screenshots/05%20lap-location.jpg" alt="Location" width="520">
      <br>
      <em>Interactive Map View: See where your story happened.</em>
    </td>
    <td align="center">
      <strong>Camera</strong><br>
      <img src="docs/public/screenshots/06%20lap-camera.jpg" alt="Camera" width="520">
      <br>
      <em>Group by Gear: Rediscover shots by Camera Make & Model.</em>
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Edit Image</strong><br>
      <img src="docs/public/screenshots/07%20lap-edit-image.jpg" alt="Edit Image" width="520">
      <br>
      <em>Quick essential edits without leaving your flow.</em>
    </td>
    <td align="center">
      <strong>Video</strong><br>
      <img src="docs/public/screenshots/08%20lap-video.jpg" alt="Video" width="520">
      <br>
      <em>Integrated Player: Watch videos seamlessly alongside photos.</em>
    </td>
  </tr>
</table>

> Notes: The sample images in above screenshots are sourced from [Wikimedia Commons](https://commons.wikimedia.org/).  

## Why LAP?

**LAP** stands for **Local-first, AI-powered Photo management**.

We built Lap because we were tired of choosing between **privacy** (dumb folder viewers) and **convenience** (cloud services that scan your life). Lap gives you both.

### 🌟 Key Highlights

#### 🤖 AI That Lives on Your Device
- **Natural Language Search**: Type "red car in rain" or "sunset at beach". The AI understands content, not just keywords.
- **Smart Face Recognition**: Automatically groups people. Rename "Person 1" to "Dad" and find all his photos instantly.
- **Similar Image Search**: Find the best shot in a burst sequence or locate duplicates.
- **Zero Privacy Risk**: All AI models run **locally**. Your photos never leave your computer.

#### 📂 Your Files, Your Control
- **No Import Required**: Lap reads your existing folders. No "library files", no duplication, no vendor lock-in.
- **File System Sync**: Move a file in Finder/Explorer? Lap updates instantly. Move it in Lap? It moves on disk.
- **Multi-Library**: Manage work assets separate from family archives. Switch in milliseconds.

#### ⚡ Built for Performance
- **Rust Core**: Powered by Tauri 2 and Rust for blazing speed and tiny memory footprint.
- **Lazy Loading**: Designed to handle libraries with **hundreds of thousands** of assets without stuttering.

#### 🎨 Delightful Experience
- **Beautiful Design**: A modern, fluid interface that feels at home on macOS and Windows.
- **Customizable**: Light/Dark modes with 10+ accent colors.
- **Multi-Language**: Speaks your language (English, Chinese, German, French, Japanese, and more).

## Comparison

| Feature | Cloud Albums (Google/iCloud) | Classic Viewers (XnView/IrfanView) | **Lap** |
| :--- | :---: | :---: | :---: |
| **Privacy** | ❌ (Data mining) | ✅ | ✅ (100% Offline) |
| **AI Search** | ✅ | ❌ | ✅ (Local AI) |
| **Organization** | Restricted (Album based) | Folder based | **Folder + AI/Smart View** |
| **Performance** | Network dependent | **Excellent** | **Excellent** |
| **Lock-in** | High | None | **None** |

## Who Is LAP For?

- **Photographers**: Manage huge collections by folder.
- **Privacy Advocates**: Enjoy AI features without Big Tech surveillance.
- **Archivists**: Search terabytes of family history instantly.
- **Everyone**: Who wants a fast, beautiful way to enjoy their memories.

## What LAP Is Not

- ❌ A Cloud Service: We don't host your photos. You own them.
- ❌ A Photoshop Replacement: We focus on management and browsing, not heavy retouching.
- ❌ A Sync Tool: We don't touch your cloud settings. We manage local files.

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

<div align="center">
  <img src="docs/public/logo2.png" alt="Lap Logo" width="200" style="border-radius: 20px">
</div>