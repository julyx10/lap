<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 本地私有照片管理器</h1>
  <h3>适用于 macOS、Windows 和 Linux 的开源桌面照片管理工具。</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | 中文简体 | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap 是一款开源、本地优先的照片管理工具，帮助您轻松浏览家庭相册、快速查找旧照片，并离线管理大型个人资料库。
无需预先导入、订阅或上传。直接在 Lap 中打开现有文件夹。文件保留在原位置，格式不变。将评分和标签保存到 XMP 附属文件，可在兼容的照片管理软件中读取。AI 搜索在您的设备上运行。

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">官方网站</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">演示</a></strong></p>
</div>

## 下载 Lap

打开 [最新版本发布页面](https://github.com/julyx10/lap/releases/latest)，下载匹配您系统的文件：

| 平台 | 安装包 | 备注 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | 已通过 Apple 公证 |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未签名。若浏览器拦截下载，请选择 **仍然保留**。若 SmartScreen 拦截安装程序，请选择 **更多信息** → **仍要运行**。 |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | 适用于 Debian 系发行版（Ubuntu、Debian、Linux Mint 等） |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | 赋予文件执行权限后，双击运行 |

### 使用 Homebrew 安装 macOS 版

```bash
brew tap julyx10/lap
brew install --cask lap
```

## 屏幕截图

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lap 本地照片资料库" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Lap 地图视图" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Lap 照片拼图" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Lap 照片预览" width="900">
</p>

## 为什么选择 Lap

- **基于文件夹管理媒体库。** 直接在 Lap 中打开现有文件夹。文件保留在原位置，格式不变。
- **本地 AI 搜索。** 用文字描述查找照片、寻找相似照片，或按人脸浏览。所有计算都在本机进行。
- **评分与标签可跨应用使用。** 将评分和标签保存到 XMP 附属文件，可在兼容的照片管理软件中读取。
- **摄影选片工具。** 空格键快速预览、评分后自动切换、显示自动对焦区域、四窗格对比，以及重复文件保留规则。
- **面向包含 10 万张以上照片和视频的媒体库。** 原生 Rust 核心负责浏览、搜索和扫描。
- **免费开源。** 无需订阅或账号，支持 macOS、Windows 和 Linux，提供 17 种语言。

## 功能特性

- **灵活浏览资料库**：按日期、文件夹、地点、相机、镜头、标签、评分和人脸筛选，支持排序和搜索。
- **以文件夹为中心的工作流。** 支持多个媒体库、拖放或剪贴板导入导出、批量重命名和文件同步，支持安全地移动、复制和删除文件。
- **本地 AI 搜索**：支持文本搜索、视觉相似搜索、主题、人脸聚类，以及可选的 50 多种语言搜索。
- **元数据导入与 XMP 写回** · 读取内嵌的 EXIF、IPTC、XMP 和 XMP 附属文件中的评分、标签、收藏、选片标记、GPS 和日期等信息。可编辑文本字段并将整理信息写回 XMP 附属文件。
- **智能相册。** 支持自定义分组、排序、文件夹名称筛选和可搜索的值选择器。
- **按日期整理导入**：支持按日、月、年或单一文件夹导入，保留原文件名并跳过重复内容。
- **合集与标签**：批量整理所选文件，无需移动或复制原件。支持拖动和分支计数的标签树。
- **重复照片清理。** 按文件夹、格式、大小或日期保留文件，显示可释放空间并支持批量删除。
- **选片与对比工具。** 空格键快速预览、评分后自动切换、显示自动对焦区域、四窗格对比，以及重复文件保留规则。
- **内置编辑**：支持裁剪、任意角度拉直、旋转、翻转、缩放和基础图像调整。
- **照片拼图**：将所选照片排成网格、马赛克或照片堆，合成一张拼图。
- **Apple 实况照片与 Google 动态照片**：支持动态播放，并可通过统一的智能相册条件筛选。
- **RAW + JPEG/HEIC 配对**：显示为一个项目，文件操作时同步处理关联文件。删除 RAW 配对时可选择删除其中一个文件或同时删除两者。保留一个文件时，共用的 XMP 附属文件也会保留。
- **RAW 缩略图与预览来源**：可选择 RAW 渲染或相机内嵌预览，并支持调节亮度和 RAW+JPEG 配对显示。
- **可自定义的文件信息。** 可选择显示字段，按需读取详细信息，并在查看器中显示自动对焦区域。
- **交互式地图视图**：在地图上聚合浏览带地理位置的照片和视频，并遵循当前筛选条件。
- **离线相册浏览**：未连接或网络存储上的相册仍会显示，并保留缓存的缩略图。
- **仅云端文件。** 对 OneDrive 按需文件和 macOS 未下载的文件建立索引并显示，不会自动下载。
- **浅色和深色主题。** 可跟随系统外观。
- **广泛格式支持**：支持 60 多种图像、RAW 和视频格式。 Affinity 文档 (内嵌预览).

### 天地图 Token 申请与配置

如需使用中国地区底图，可在地图设置中选择天地图，并填写自己的应用密钥（Key，也称 Token 或 `tk`）。

1. 在[天地图官网](https://www.tianditu.gov.cn/)注册并登录账号，按页面提示完成账号认证。
2. 打开[天地图控制台](https://console.tianditu.gov.cn/api/key)，进入应用管理，选择创建新应用。
3. 填写应用名称（例如 `Lap`）等信息，应用类型选择 **浏览器端**，提交后复制生成的 **Key**。
4. 在 Lap 的 **设置 → 高级 → 地图** 中，将地图服务提供者设为 **中国（天地图）**，把 Key 粘贴到 **天地图 API Token**，按 Enter 或点击输入框外保存。
5. 返回地图视图，即可加载天地图底图。地图底图需要联网获取。

申请类型可参考 [QGIS 天地图插件的说明](https://github.com/liuxspro/qgis-plugin-tianditu#使用说明)；控制台页面和认证要求以天地图官方为准。

## 元数据、合集与文件移动

Lap 直接使用现有文件夹，显示的大多数信息都从文件中读取。您可以将整理信息写入标准 XMP 附属文件，供其他照片应用读取。

Lap 支持导入 Lightroom、digiKam、darktable 和 ACDSee 的兼容元数据。支持的字段因应用而异。

### 会随文件保留的信息

- 您的原始照片和视频始终是现有文件夹中的普通文件。
- 建立索引时，Lap 读取内嵌的 EXIF、IPTC 和 XMP，包括拍摄日期、相机、镜头、GPS、方向、评分、收藏、关键词、选片标记、描述、标题、短标题、创作者和版权。在 **设置 → 元数据 → 导入元数据** 中开启导入后，整理信息会进入本地数据库，已有的本地编辑会保留。
- 导入 XMP 附属文件时，受支持的字段优先使用附属文件中的值，包括 GPS 和拍摄日期。
- 标题、短标题、描述、创作者和版权可编辑，修改保存在本地数据库。
- 执行「保存元数据到 XMP」可将这些字段以及标签（包括层级关键词）、评分、收藏和选片状态写入 XMP 附属文件。其他 XMP 属性和原始文件会保留。
- 当您**在 Lap 中**重命名、移动、复制、删除或导入文件时，Lap 会同步更新本地资料库记录，并将 Apple 实况照片组件、XMP 和 AAE 附属文件和已启用的 RAW + JPEG/HEIC 配对等关联资源一并处理。删除 RAW 配对时可选择删除其中一个文件或同时删除两者。保留一个文件时，共用的 XMP 附属文件也会保留。
- 保存内置图片编辑时，生成的图片会写入所选目标位置。

### Lap 在本地存储的信息

以下资料库数据保存在 Lap 的本地数据库或资料库配置中。

- 合集、智能相册及其规则和视图设置保存在本地数据库或配置中。
- AI 搜索数据、人脸数据、缩略图以及其他索引或缓存数据
- 标签、评分、收藏、选片状态和可编辑文本字段保存在本地数据库，可按上述方式写入 XMP。

默认情况下，在 Lap 外复制、导出或移动文件时，这些本地信息不会随文件转移，也不会自动提供给其他应用。

### 在 Lap 外管理文件

Lap 可以重新扫描文件夹并发现许多文件系统变化。但在 Lap 外重命名、移动、替换或复制文件，可能影响仅保存在 Lap 中的整理信息。

启用元数据导入后，Lap 也会检查 XMP 附属文件是否有变更。在 Lap 内移动或重命名文件可保留合集归属。请将 XMP 附属文件与原始文件放在一起。如果也在 Lap 外管理文件，请将 Lap 的数据库和配置与照片一同备份。您可以在 **设置 → 存储** 中管理数据库位置并创建备份。

删除 Lap 的数据库或配置会移除这些本地整理和索引数据，但不会删除您的原始媒体文件。

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

使用 DEB 安装的用户，请卸载软件包：

```bash
sudo apt remove lap
```

卸载本地安装的 Arch 软件包请使用 `sudo pacman -R lap`.

使用 AppImage 的用户，请退出 Lap 并删除下载的 `.AppImage` 文件。

然后删除所有 Lap 数据库、缓存和配置文件：

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

如果您在 Lap 设置中选择了自定义数据库存储目录，请在确认其中仅包含 Lap 数据库文件后，单独删除该目录。

## 从源码构建

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

发行版软件包可以链接系统 libheif（1.17 或更新版本），而不使用内置的 libheif 和 libde265。构建时设置 `LAP_SYSTEM_LIBHEIF=1`。这样就无需 `third_party/libheif` 和 `third_party/libde265` 这两个子模块。HEVC 解码取决于系统 libheif 的编解码插件。

### Arch Linux (x86_64)

Arch 的实验性构建配方尚未发布到 AUR。可在本地构建和安装: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

配方固定上游提交，并用校验和验证随附的 AI 模型和 FFmpeg。更新由 pacman 管理。

## 支持格式

Lap 支持 60+ 种图像、RAW 和视频格式。

| 类型 | 格式清单 |
| :--- | :--- |
| 常规图片 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW 照片 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity 文档 | AF, AFPHOTO, AFDESIGN, AFPUB (内嵌预览) |
| 视频 | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. 所有平台均支持 H.264 播放；在不支持原生播放时，系统会自动进行兼容性处理。macOS 原生支持 HEVC/H.265 和 VP9。 |

### Linux 视频播放

Lap 使用系统 GStreamer 插件播放视频，AppImage 版本也不例外。如果在 Ubuntu、Debian 或 Linux Mint 上无法播放视频，请安装：

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | ExifTool 的 Rust 移植版 |
| [Video.js](https://videojs.com/) | 跨平台视频播放界面 |
| [ONNX Runtime](https://onnxruntime.ai/) | 本地 AI 模型推理引擎 |
| [CLIP](https://github.com/openai/CLIP) | 图文相似度搜索 |
| [InsightFace](https://github.com/deepinsight/insightface) | 人脸检测与识别 |
| [Leaflet](https://leafletjs.com/) | 用于地理位置照片的交互式地图 |
| [daisyUI](https://daisyui.com/) | UI 组件库 |

## 开源许可证

GPL-3.0-or-later。详情请参阅 [LICENSE](../LICENSE)。

## 隐私

有关数据处理和可选在线服务的详情，请参阅[隐私政策](../PRIVACY.md)。
