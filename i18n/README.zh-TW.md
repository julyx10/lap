<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 私人本機相片管理器</h1>
  <h3>適用於 macOS、Windows 與 Linux 的開放原始碼桌面相片管理軟體。</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | 中文繁體 | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap 是一款免費、開源、本機優先的相片管理工具，支援 macOS、Windows 和 Linux。直接瀏覽和整理現有相片資料夾，使用本機 AI 搜尋，無需雲端帳號或訂閱即可管理大型媒體庫。

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">官方網站</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">演示</a></strong></p>
</div>

> [!IMPORTANT]
> Lap 目前唯一的官方網站是 [julyx10.github.io/lap](https://julyx10.github.io/lap/)。
> `lapphotos.com` 並非官方網站，與 Lap 專案無關，也未獲得本專案的授權或認可。

## 下載 Lap

請盡可能直接從 [GitHub Releases](https://github.com/julyx10/lap/releases) 下載 Lap，而非第三方網站。

| 平台 | 安裝包 | 備註 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | 已通過 Apple 公證 |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未簽署。若瀏覽器阻擋下載，請選擇 **仍要保留**。若 SmartScreen 阻擋安裝程式，請選擇 **其他資訊** → **仍要執行**。 |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | 適用於 Debian 系列發行版（Ubuntu、Debian、Linux Mint 等） |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | 授予檔案可執行權限後，雙擊執行 |

### 使用 Homebrew 安裝 macOS 版

```bash
brew install --cask lap
```

先前透過 `julyx10/lap` tap 安裝？請參閱[遷移說明](https://github.com/julyx10/lap/discussions/567)。

## 螢幕截圖

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lap 本機相片資料庫管理器" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Lap 地圖檢視" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Lap 相片預覽" width="900">
</p>

## 為什麼選擇 Lap

- **私密的本機 AI。** 依描述搜尋、尋找相似相片或依人臉瀏覽，處理均在本機進行。
- **你的資料夾，由你掌控。** 直接開啟現有資料夾，原始檔案保留原位，無需上傳或匯入媒體庫。
- **跨應用程式使用中繼資料。** 讀取內嵌 EXIF/IPTC/XMP 和現有附屬檔案，支援將修改的中繼資料儲存至 XMP 附屬檔案，原始檔案保持不變。
- **相片篩選工具。** 空白鍵快速預覽、評分後自動切換、顯示自動對焦區域、四窗格比較，以及重複檔案保留規則。
- **適用於有 10 萬張以上相片與影片的媒體庫。** 原生 Rust 核心負責瀏覽、搜尋與掃描。
- **免費開放原始碼。** 無須訂閱或帳號，支援 macOS、Windows 和 Linux，提供 17 種語言。

## 功能特色

- **彈性瀏覽資料庫**：依日期、資料夾、地點、相機、鏡頭、標籤、評分與人臉篩選，支援排序與搜尋。
- **以資料夾為中心的工作流程。** 支援多個媒體庫、拖曳或剪貼簿匯入匯出、批次重新命名和檔案同步，支援安全地移動、複製及刪除檔案。
- **本機 AI 搜尋**：支援文字搜尋、視覺相似搜尋、主題、人臉分群，以及可選的 50 多種語言搜尋。
- **中繼資料匯入與 XMP 寫回**：讀取內嵌的 EXIF、IPTC、XMP 及 XMP 附屬檔案中的評分、標籤、我的最愛、選片標記、GPS 和日期等資料，可編輯文字欄位並將整理資訊寫回 XMP 附屬檔案。
- **智慧相簿。** 支援自訂分組、排序、資料夾名稱篩選和可搜尋的值選擇器。
- **依日期整理的匯入方式**：支援按日、月、年或單一資料夾匯入，保留原檔名並跳過重複項目。
- **收藏與階層式標籤**：批次整理所選檔案，無需移動或複製原始檔案。支援可拖曳排序和分支計數的標籤樹。
- **重複照片清理。** 依資料夾、格式、大小或日期保留檔案，顯示可釋放空間並支援批次刪除。
- **選片與比較工具。** 空白鍵快速預覽、評分後自動切換、顯示自動對焦區域、四窗格比較，以及重複檔案保留規則。
- **內建編輯**：支援裁剪、任意角度拉直、旋轉、翻轉、縮放與基本影像調整。
- **相片拼貼**：將所選相片排成格狀、馬賽克或相片堆疊，合成一張拼貼。
- **Apple 原況相片與 Google 動態相片**：支援動態播放，並可透過統一的智慧相簿條件篩選。
- **RAW + JPEG/HEIC 配對**：以單一項目顯示，進行檔案操作時同步處理相關聯檔案。刪除 RAW 配對時可選擇刪除其中一個或兩個檔案。保留一個檔案時，也會保留共用的 XMP 附屬檔案。
- **可設定的 RAW 縮圖與預覽**：可使用 RAW 算繪或相機內嵌預覽，並支援調整亮度與 RAW+JPEG 配對顯示。
- **可自訂的檔案資訊。** 可選擇顯示欄位、按需載入詳細資訊，並在檢視器中顯示自動對焦區域。
- **互動式地圖檢視**：以叢集方式瀏覽含地理資訊的相片與影片，並遵循目前的篩選條件。
- **離線相簿瀏覽**：未連接或網路儲存上的相簿仍會顯示，並保留快取的縮圖。
- **僅存在雲端的檔案。** 為 OneDrive 隨選檔案和 macOS 尚未下載的檔案建立索引並顯示，不會自動下載。
- **淺色與深色主題。** 可跟隨系統外觀。
- **廣泛的格式支援**：支援 60 多種影像、RAW 與影片格式，以及 Affinity 文件（內嵌預覽）。

## 中繼資料與資料自主權

Lap 直接使用現有資料夾，顯示的大部分資訊都從檔案讀取。你可以將整理資訊寫入標準 XMP 附屬檔案，供其他相片應用程式讀取。

Lap 可匯入 Lightroom、digiKam、darktable 和 ACDSee 的相容中繼資料。支援的欄位因應用程式而異。

### 讀取現有中繼資料

- 你的原始相片與影片始終是現有資料夾中的普通檔案。
- 建立索引時，Lap 讀取內嵌的 EXIF、IPTC 與 XMP，包括拍攝日期、相機、鏡頭、GPS、方向、評分、我的最愛、關鍵字、選片標記、描述、標題、短標題、創作者及版權。在 **設定 → 中繼資料 → 匯入中繼資料** 開啟匯入後，整理資訊會存入本機資料庫，並保留本機編輯。
- 匯入 XMP 附屬檔案時，受支援的欄位優先採用附屬檔案中的值，包括 GPS 與拍攝日期。

### 將支援的修改儲存至 XMP

- 標題、短標題、描述、創作者和版權皆可編輯，變更會儲存在本機資料庫。
- 執行「將中繼資料儲存至 XMP」即可將這些欄位和標籤（包括階層式關鍵字）、評分、我的最愛與選片狀態寫入 XMP 附屬檔案。其他 XMP 屬性和原始檔會保留。
- 當你**在 Lap 中**重新命名、移動、複製、刪除或匯入檔案時，Lap 會同時更新其本機目錄。支援的群組資源，例如 Apple 原況相片元件、XMP 與 AAE 附屬檔案與已啟用的 RAW + JPEG/HEIC 配對，都會一併維持在一起。刪除 RAW 配對時可選擇刪除其中一個或兩個檔案。保留一個檔案時，也會保留共用的 XMP 附屬檔案。
- 儲存內建圖片編輯時，產生的圖片會寫入所選的目標位置。

### 保留在 Lap 資料庫中的資訊

下列資料庫資訊儲存在 Lap 的本機資料庫或資料庫設定中。

- 收藏、智慧相簿及其規則、分組、排序與順序
- AI 搜尋資料、人臉資料、縮圖，以及其他索引或快取資料
- 標籤、評分、我的最愛、選片狀態和可編輯的文字欄位儲存在本機資料庫，可依上述方式寫入 XMP。

在你選擇將支援的欄位儲存至 XMP 之前，修改保留在 Lap 資料庫中。收藏、智慧相簿、AI 搜尋資料和人臉資料僅保存在 Lap 中，不包含在 XMP 附屬檔案中。請備份資料庫以保留這些整理成果；僅複製原始檔案無法保留它們。

### 在 Lap 之外管理檔案

Lap 可以重新掃描資料夾並偵測許多檔案系統變更。但若在 Lap 之外進行重新命名、移動、取代或複製檔案等操作，可能會影響僅儲存於 Lap 中的整理資訊。

開啟中繼資料匯入後，Lap 也會檢查 XMP 附屬檔案是否有變更。在 Lap 中移動或重新命名檔案可保留收藏歸屬。請將 XMP 附屬檔案與原始檔一起存放。若你也在 Lap 之外管理檔案，請將 Lap 的資料庫與設定連同相片一起備份。你可以在 **設定 → 儲存空間** 中管理資料庫位置並建立備份。

刪除 Lap 的資料庫或設定檔會移除這些本機整理與索引資料，但不會刪除你的原始媒體檔案。

## 解除安裝 Lap

Lap 直接使用你現有的相片資料夾。解除安裝 Lap，或刪除其資料庫與快取檔案，**不會**刪除你的原始相片。

標準的解除安裝步驟會移除應用程式。若要完全移除 Lap，請先結束 Lap，解除安裝應用程式，然後依照你平台的清理指令，刪除其本機資料庫、縮圖快取與設定檔。

### macOS

若你使用 Homebrew 安裝 Lap：

```bash
brew uninstall --cask lap
```

若你手動安裝，請結束 Lap，並將 `Applications` 資料夾中的 `Lap.app` 移到垃圾桶。

若要移除所有 Lap 資料庫、快取與設定檔：

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

開啟 **設定 > 應用程式 > 已安裝的應用程式**，找到 **Lap**，然後選擇 **解除安裝**。

接著開啟 PowerShell，並移除所有 Lap 資料庫、快取與設定檔：

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

對於 DEB 安裝方式，請解除安裝套件：

```bash
sudo apt remove lap
```

解除安裝在本機安裝的 Arch 套件請使用 `sudo pacman -R lap`。

對於 AppImage 安裝方式，請結束 Lap，並刪除下載的 `.AppImage` 檔案。

接著移除所有 Lap 資料庫、快取與設定檔：

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

若你曾在 Lap 設定中選擇自訂的資料庫儲存目錄，請先確認其中僅包含 Lap 資料庫檔案，再單獨刪除該目錄。

## 從原始碼建置

需求：Node.js 20+、pnpm、Rust stable。

```bash
# macOS 系統相依套件
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux 系統相依套件
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# 複製並建置
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

發行版套件可以連結系統的 libheif（1.17 或更新版本），取代內建的 libheif 與 libde265。建置時請設定 `LAP_SYSTEM_LIBHEIF=1`。此時不再需要 `third_party/libheif` 與 `third_party/libde265` 這兩個子模組。HEVC 解碼取決於系統 libheif 的編解碼器外掛程式。

### Arch Linux (x86_64)

Arch 的實驗性建置配方尚未發佈到 AUR，因此暫不可透過 `yay -S lap` 安裝。在 Arch Linux 上於本機建置和安裝：[`contrib/aur`](../contrib/aur/README.md)。

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

配方固定上游提交，並以校驗和驗證隨附的 AI 模型和 FFmpeg 工具。Arch 建置版本停用應用程式內自動更新，由 pacman 負責升級；其他平台的建置版本保留原有更新程式。建置細節與待完成的發布檢查請參閱配方說明。

## 支援的格式

Lap 支援 60 多種影像、RAW 與影片格式。

| 類型 | 格式 |
| :--- | :--- |
| 圖片 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW 相片 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity 文件 | AF, AFPHOTO, AFDESIGN, AFPUB (內嵌預覽) |
| 影片 | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. 所有平台皆支援 H.264 播放；當原生播放不可用時，會自動進行相容性處理。HEVC/H.265 與 VP9 在 macOS 上原生支援。 |

### Linux 影片播放

若影片無法播放，請安裝 GStreamer 外掛程式：

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## 架構

- 核心：Tauri + Rust
- 前端：Vue + Vite + Tailwind CSS
- 資料：SQLite

### 主要函式庫

| 函式庫 | 用途 |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW 影像解碼與縮圖擷取 |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF 影像解碼與預覽產生 |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | 快速 JPEG 解碼與縮圖產生 |
| [FFmpeg](https://ffmpeg.org/) | 影片處理與縮圖產生 |
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | ExifTool 的 Rust 移植版 |
| [Video.js](https://videojs.com/) | 跨平台影片播放使用者介面 |
| [ONNX Runtime](https://onnxruntime.ai/) | 本機 AI 模型推論引擎 |
| [CLIP](https://github.com/openai/CLIP) | 影像與文字相似性搜尋 |
| [InsightFace](https://github.com/deepinsight/insightface) | 人臉偵測與辨識 |
| [Leaflet](https://leafletjs.com/) | 用於含地理資訊相片的互動式地圖 |
| [daisyUI](https://daisyui.com/) | UI 元件函式庫 |

## 授權條款

GPL-3.0-or-later。請參閱 [LICENSE](../LICENSE)。

## 隱私權

有關資料處理與可選線上服務的詳細資訊，請參閱[隱私權政策](../PRIVACY.md)。
