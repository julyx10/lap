<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 私人本地照片管理器</h1>
  <h3>适用于 macOS、Windows 和 Linux 的开源桌面照片管理工具。</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md) | 简体中文 | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap 是一款开源、本地优先的照片管理器，旨在帮助您轻松浏览家庭相册、快速找回旧照片，并离线管理庞大的个人多媒体库。
它是云端照片服务的隐私替代方案：无强制上传、内置本地 AI 搜索、以文件夹为中心的工作流，且完全免费使用。

- 官方网站: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- 演示视频: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- 隐私策略: [PRIVACY.md](../PRIVACY.md)

## 下载 Lap

打开 [最新版本发布页面](https://github.com/julyx10/lap/releases/latest)，下载匹配您系统的文件：

| 平台 | 安装包 | 备注 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | 已通过 Apple 公证 |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未签名 — 如果 SmartScreen 阻止下载，请点击**仍要保留** |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | 适用于 Debian 系发行版（Ubuntu、Debian、Linux Mint 等） |

## 界面预览

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap 本地照片管理界面" width="900">
</p>

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap 本地 AI 搜索界面" width="900">
</p>

## 为什么选择 Lap

- **无需云端**：将您的媒体库保存在自己的硬盘上，无需上传到托管服务。
- **隐私优先**：所有处理均在本地完成，确保您的照片始终处于您的控制之下。
- **完全免费**：没有订阅计划或周期性费用。
- **以文件夹为中心**：直接操作您现有的文件夹，无需冗长的导入步骤。
- **针对大库的高性能表现**：针对超大规模媒体库（单库 100k+ 文件）进行了流畅浏览和管理优化。

## 功能特性

- **浏览与筛选**：支持按日期、地点、相机、镜头、标签、收藏、评分以及人脸（测试版）进行筛选。
- **管理多个库**：支持创建多个媒体库并快速切换。
- **查找重复项**：自动发现重复文件并批量移动到废纸篓。
- **原地编辑**：支持裁剪、旋转、翻转、缩放及基础图像调整。
- **同步文件夹**：支持文件系统感知的同步操作和手动刷新。
- **本地 AI 搜索**：提供以文搜图、以图搜图、人脸聚类及智能标签。
- **支持 50+ 语言搜索**：可按需额外下载可选的多语言模型。
- **现代格式支持**：支持 WebP、HEIC/HEIF/HIF、AVIF 和 JXL (JPEG XL)。
- **查看 RAW 照片**：内置解码支持 20 多种主流相机的 RAW 格式（CR2, NEF, ARW, DNG 等）。
- **广泛的视频兼容性**：支持 MP4, MOV, AVI, MKV 等 20 多种格式，并针对跨平台进行了优化。

## 卸载 Lap

Lap 直接使用您现有的照片文件夹。卸载 Lap 或删除其数据库和缓存文件，**不会**删除您的原始照片。

常规卸载只会移除应用程序。如需彻底删除 Lap，请先退出 Lap，卸载应用程序，然后按照对应平台的命令删除本地数据库、缩略图缓存和配置文件。

### macOS

如果您通过 Homebrew 安装了 Lap：

```bash
brew uninstall --cask lap
```

如果您手动安装了 Lap，请退出 Lap，并将 `Applications` 文件夹中的 `Lap.app` 移到废纸篓。

删除所有 Lap 数据库、缓存和配置文件：

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

打开 **设置 > 应用 > 已安装的应用**，找到 **Lap** 并选择 **卸载**。

然后打开 PowerShell，删除所有 Lap 数据库、缓存和配置文件：

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

对于基于 Debian 的发行版，请卸载软件包：

```bash
sudo apt remove lap
```

然后删除所有 Lap 数据库、缓存和配置文件：

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

如果您在 Lap 设置中选择了自定义数据库存储目录，请在确认其中仅包含 Lap 数据库文件后，单独删除该目录。

## 源码编译

编译要求: Node.js 20+, pnpm, Rust stable.

```bash
# macOS 系统依赖
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux 系统依赖
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# 克隆并编译
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## 支持格式

| 类型 | 格式清单 |
| :--- | :--- |
| 常规图片 | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL |
| RAW 照片 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| 视频 | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX 等。所有平台均支持 H.264 播放；在不支持原生播放时，系统会自动进行兼容性处理。macOS 原生支持 HEVC/H.265 和 VP9。 |

### Linux 视频播放备注

在 Linux Mint/Ubuntu/Debian 上，请安装以下软件包以获得更好的视频播放支持：

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## 技术架构

- 核心: Tauri + Rust
- 前端: Vue + Vite + Tailwind CSS
- 数据: SQLite

### 关键库

| 库 | 用途 |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW 图像解码与缩略图提取 |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF 图像解码与预览生成 |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | 快速 JPEG 解码与缩略图生成 |
| [FFmpeg](https://ffmpeg.org/) | 视频处理与缩略图生成 |
| [Video.js](https://videojs.com/) | 跨平台视频播放界面 |
| [ONNX Runtime](https://onnxruntime.ai/) | 本地 AI 模型推理引擎 |
| [CLIP](https://github.com/openai/CLIP) | 图文相似度搜索 |
| [InsightFace](https://github.com/deepinsight/insightface) | 人脸检测与识别 |
| [Leaflet](https://leafletjs.com/) | 用于地理位置照片的交互式地图 |
| [daisyUI](https://daisyui.com/) | 界面 UI 组件库 |

## 开源协议

GPL-3.0-or-later。详情请参阅 [LICENSE](../LICENSE)。
