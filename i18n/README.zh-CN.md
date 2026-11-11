<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 本地私有照片管理器</h1>
  <h3>适用于 macOS、Windows 和 Linux 的开源桌面照片管理工具。</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | 中文简体 | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap 是一款开源、本地优先的照片管理工具，帮助您轻松浏览家庭相册、快速查找旧照片，并离线管理大型个人资料库。
它是云端照片服务的隐私替代方案：无强制上传、内置本地 AI 搜索、以文件夹为中心的工作流，且完全免费使用。

## 下载 Lap

打开 [最新版本发布页面](https://github.com/julyx10/lap/releases/latest)，下载匹配您系统的文件：

| 平台 | 安装包 | 备注 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | 已通过 Apple 公证 |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未签名 — 如果 SmartScreen 阻止下载，请点击**仍要保留** |
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
</p>

## 为什么选择 Lap

- **本地优先设计**：照片保存在您自己的硬盘上，无需云账号或强制上传。
- **不锁定资料库**：直接使用现有文件夹，无需将所有内容导入封闭数据库。
- **本地 AI 工具**：搜索、相似照片、智能标签和人脸识别都在本机运行。
- **为大型资料库优化**：即使资料库包含超过 10 万个文件，浏览和整理依然流畅。
- **开源且免费**：无订阅、无强制生态绑定，代码可自行审查。

## 功能特性

- **灵活浏览资料库**：按日期、文件夹、地点、相机、镜头、标签、评分和人脸筛选，支持排序和搜索。
- **以文件夹为中心的工作流**：支持多个资料库、拖放导入/导出、复制粘贴导入/导出、文件系统同步，以及安全的移动、复制和删除。
- **本地 AI 搜索**：支持文本搜索、视觉相似搜索、主题、人脸聚类，以及可选的 50 多种语言搜索。
- **智能相册**：保存基于规则的视图，自定义分组和排序。
- **按日期整理导入**：支持按日、月、年或单一文件夹导入，保留原文件名并跳过重复内容。
- **合集与标签**：批量整理所选文件，无需移动或复制原件。
- **重复照片清理**：查看可释放空间，跨重复组批量清理。
- **选片与对比工具**：包含四窗格图片对比查看器。
- **内置编辑**：支持裁剪、任意角度拉直、旋转、翻转、缩放和基础图像调整。
- **照片拼图**：将所选照片排成网格、马赛克或照片堆，合成一张拼图。
- **Apple 实况照片与 Google 动态照片**：支持动态播放，并可通过统一的智能相册条件筛选。
- **RAW + JPEG/HEIC 配对**：显示为一个项目，文件操作时同步处理关联文件。
- **RAW 缩略图与预览来源**：可选择 RAW 渲染或相机内嵌预览，并支持调节亮度和 RAW+JPEG 配对显示。
- **交互式地图视图**：在地图上聚合浏览带地理位置的照片和视频，并遵循当前筛选条件。
- **离线相册浏览**：未连接或网络存储上的相册仍会显示，并保留缓存的缩略图。
- **广泛格式支持**：支持 60 多种照片、RAW 和视频格式。

### 天地图 Token 申请与配置

如需使用中国地区底图，可在地图设置中选择天地图，并填写自己的应用密钥（Key，也称 Token 或 `tk`）。

1. 在[天地图官网](https://www.tianditu.gov.cn/)注册并登录账号，按页面提示完成账号认证。
2. 打开[天地图控制台](https://console.tianditu.gov.cn/api/key)，进入应用管理，选择创建新应用。
3. 填写应用名称（例如 `Lap`）等信息，应用类型选择 **浏览器端**，提交后复制生成的 **Key**。
4. 在 Lap 的 **设置 → 高级 → 地图** 中，将地图服务提供者设为 **中国（天地图）**，把 Key 粘贴到 **天地图 API Token**，按 Enter 或点击输入框外保存。
5. 返回地图视图，即可加载天地图底图。地图底图需要联网获取。

申请类型可参考 [QGIS 天地图插件的说明](https://github.com/liuxspro/qgis-plugin-tianditu#使用说明)；控制台页面和认证要求以天地图官方为准。

## 元数据、合集与文件移动

Lap 以文件夹为中心，但 Lap 中显示的所有信息并非都嵌入在原始文件中。如果您也会在 Finder、资源管理器或其他照片应用中管理同一批文件夹，理解这一区别尤为重要。

### 会随文件保留的信息

- 您的原始照片和视频始终是现有文件夹中的普通文件。
- EXIF 拍摄日期、相机、镜头、GPS 和方向等已嵌入文件的元数据，会在 Lap 索引时从文件中读取。
- 保存内置图片编辑时，生成的图片会写入所选目标位置。
- 当您**在 Lap 中**重命名、移动、复制或删除文件时，Lap 会同步更新本地资料库记录，并将 Apple 实况照片组件、AAE 附属文件和已启用的 RAW + JPEG/HEIC 配对等关联资源一并处理。

### Lap 在本地存储的信息

以下是 Lap 的资料库数据，保存在 Lap 的本地数据库或资料库配置中，不会写入 EXIF、IPTC 或 XMP 附属文件：

- 合集、标签、注释、收藏、评分和选片状态（包括已选与已排除）
- 智能相册及其规则、分组、排序和顺序
- AI 搜索数据、人脸数据、缩略图以及其他索引或缓存数据

当文件在 Lap 外被复制、导出或移动时，这些数据不会随文件一起传递，也不会自动提供给其他应用。

### 在 Lap 外管理文件

Lap 可以重新扫描文件夹并发现许多文件系统变化。但在 Lap 外重命名、移动、替换或复制文件，可能影响仅保存在 Lap 中的整理信息。

若您依赖合集、标签、注释、收藏、评分或选片状态，建议在 Lap 中重命名和移动文件，以获得最可靠的结果。如果也在 Lap 外管理文件，请将 Lap 的数据库和配置与照片一同备份。您可以在 **设置 → 存储** 中管理数据库位置并创建备份。

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

## 支持格式

Lap 支持 60+ 种照片、RAW 和视频格式。

| 类型 | 格式清单 |
| :--- | :--- |
| 常规图片 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW 照片 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| 视频 | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX 等。所有平台均支持 H.264 播放；在不支持原生播放时，系统会自动进行兼容性处理。macOS 原生支持 HEVC/H.265 和 VP9。 |

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
