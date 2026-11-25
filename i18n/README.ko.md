<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 프라이빗 로컬 사진 관리자</h1>
  <h3>macOS, Windows, Linux를 위한 오픈 소스 데스크톱 사진 관리 도구.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | 한국어

Lap은 macOS, Windows, Linux용 무료 오픈 소스 로컬 우선 사진 관리 앱입니다. 기존 폴더를 정리하고 기기 내 AI로 검색하며 클라우드 계정이나 구독 없이 대규모 라이브러리를 관리하세요.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">공식 웹사이트</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">데모</a></strong></p>
</div>

> [!IMPORTANT]
> 현재 Lap의 유일한 공식 웹사이트는 [julyx10.github.io/lap](https://julyx10.github.io/lap/)입니다.
> `lapphotos.com`은 공식 웹사이트가 아니며, Lap 프로젝트와 관련이 없고 프로젝트의 승인이나 지지를 받지 않았습니다.

## Lap 다운로드

가능하면 타사 웹사이트 대신 [GitHub Releases](https://github.com/julyx10/lap/releases)에서 직접 Lap을 다운로드하세요.

| 플랫폼 | 패키지 | 비고 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Apple 공증 완료 |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 서명되지 않았습니다. 브라우저에서 **계속 유지**를 선택하거나 SmartScreen에서 **추가 정보** → **실행**을 선택하세요. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Debian 기반 배포판용(Ubuntu, Debian, Linux Mint 등) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | 파일에 실행 권한을 부여한 다음 더블 클릭하여 실행 |

### Homebrew로 macOS에 설치

```bash
brew install --cask lap
```

이전에 `julyx10/lap` tap으로 설치했다면 [이전 안내](https://github.com/julyx10/lap/discussions/567)를 참고하세요.

## 스크린샷

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lap 로컬 사진 라이브러리 관리" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Lap 지도 보기" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Lap 사진 미리보기" width="900">
</p>

## 왜 Lap인가요?

- **개인정보를 지키는 로컬 AI.** 사진을 문장으로 검색하고, 비슷한 사진을 찾거나 얼굴별로 탐색할 수 있습니다. 모든 처리는 기기에서 이루어집니다.
- **직접 관리하는 내 폴더.** 기존 폴더를 Lap에서 바로 엽니다. 파일의 위치와 형식은 그대로 유지됩니다.
- **앱 간에 활용하는 메타데이터.** 내장 EXIF, IPTC, XMP와 사이드카(평점, 태그, 즐겨찾기, 선별 상태, GPS, 날짜 등)를 읽습니다. 수정한 메타데이터를 XMP 사이드카에 저장할 수 있으며, 원본 파일은 변경되지 않습니다.
- **사진 선별 도구.** 스페이스바로 즉시 미리 보기, 평점 지정 후 자동 이동, AF 영역 표시, 4분할 비교 및 중복 파일 보존 규칙을 지원합니다.
- **사진과 동영상 10만 개 이상의 라이브러리 지원.** Rust 네이티브 코어가 탐색, 검색, 스캔을 처리합니다.
- **무료 오픈 소스.** 구독이나 계정 없이 macOS, Windows, Linux에서 17개 언어를 지원합니다.

## 주요 기능

- **유연한 라이브러리 탐색**: 날짜, 폴더, 위치, 카메라, 렌즈, 태그, 평점, 얼굴별 필터와 정렬, 검색을 지원합니다.
- **폴더 중심 작업 흐름.** 여러 라이브러리, 끌어서 놓기 또는 클립보드를 통한 가져오기 및 내보내기, 일괄 이름 변경과 동기화, 안전한 이동, 복사 및 삭제.
- **로컬 AI 검색**: 텍스트, 시각적 유사성, 피사체, 얼굴 그룹화를 지원하며 50개 이상의 언어로 검색할 수 있습니다.
- **메타데이터 가져오기 및 XMP 쓰기** · 내장 EXIF, IPTC, XMP와 사이드카(평점, 태그, 즐겨찾기, 선별 상태, GPS, 날짜 등)를 읽습니다. 텍스트 필드를 편집하고 정리 정보를 XMP 사이드카에 저장할 수 있습니다.
- **스마트 앨범.** 사용자 지정 그룹화와 정렬, 폴더 이름 필터, 검색 가능한 값 선택기.
- **날짜별 가져오기**: 일별, 월별, 연도별 또는 단일 폴더로 정리하고 원래 파일명을 유지하며 중복을 건너뜁니다.
- **컬렉션과 계층형 태그**: 원본을 이동하거나 복제하지 않고 선택한 파일을 일괄 정리합니다. 드래그와 하위 분기별 개수 표시를 지원하는 태그 트리.
- **중복 정리.** 폴더, 형식, 크기, 날짜별 보존 규칙, 확보 가능한 공간 및 일괄 삭제.
- **사진 선별 및 비교 도구.** 스페이스바로 즉시 미리 보기, 평점 지정 후 자동 이동, AF 영역 표시, 4분할 비교 및 중복 파일 보존 규칙을 지원합니다.
- **내장 편집**: 자르기, 임의 각도 기울기 보정, 회전, 뒤집기, 크기 변경과 기본 이미지 조정을 지원합니다.
- **사진 콜라주**: 선택한 사진을 격자, 모자이크, 사진 더미 콜라주로 구성합니다.
- **Apple Live Photos와 Google Motion Photos**: 움직임 재생과 통합 스마트 앨범 필터를 지원합니다.
- **RAW + JPEG/HEIC 쌍**: 하나의 항목으로 표시하고 파일 작업 시 연결된 파일을 함께 처리합니다. RAW 쌍에서 한 파일 또는 두 파일을 모두 삭제할 수 있습니다. 한 파일을 남기면 공유 XMP 사이드카도 유지됩니다.
- **RAW 썸네일과 미리보기 선택**: RAW 렌더링 또는 카메라에 내장된 미리보기를 사용하고, 밝기 조정과 RAW+JPEG 쌍 표시를 지원합니다.
- **사용자 지정 파일 정보.** 표시할 필드 선택, 필요한 상세 정보만 불러오기 및 뷰어의 AF 영역 표시.
- **대화형 지도 보기**: 현재 필터에 맞는 위치 정보가 있는 사진과 동영상을 그룹으로 탐색합니다.
- **오프라인 앨범 탐색**: 연결이 끊긴 저장소나 네트워크 저장소의 앨범도 캐시된 썸네일로 계속 볼 수 있습니다.
- **클라우드에만 있는 파일.** OneDrive 주문형 파일 및 macOS 자리 표시자를 자동 다운로드 없이 인덱싱하고 표시합니다. Lap은 자동으로 다운로드하지 않습니다.
- **밝은 테마와 어두운 테마.** 시스템 테마를 따를 수 있습니다.
- **폭넓은 형식 지원**: 60개 이상의 이미지, RAW, 동영상 형식을 지원합니다. Affinity 문서 (내장 미리 보기).

## 메타데이터와 데이터 소유권

Lap은 기존 폴더를 직접 사용하며 표시되는 정보 대부분을 파일에서 읽습니다. 정리 정보를 다른 사진 앱에서도 읽을 수 있는 표준 XMP 사이드카에 저장할 수 있습니다.

Lap은 Lightroom, digiKam, darktable, ACDSee의 호환 메타데이터를 읽습니다. 지원하는 필드는 앱마다 다릅니다.

### 기존 메타데이터 읽기

- 원본 사진과 동영상은 기존 폴더에 있는 일반 파일로 항상 유지됩니다.
- 인덱싱할 때 촬영 및 정리 정보가 포함된 내장 EXIF, IPTC, XMP를 읽습니다. 로컬 데이터베이스로 정리 정보를 가져오려면 **설정 → 메타데이터 → 메타데이터 가져오기**를 켜세요. 로컬 편집 내용은 유지됩니다.
- GPS와 촬영 날짜를 포함하여 지원되는 XMP 사이드카 값이 내장 값보다 우선합니다.

### 지원되는 편집을 XMP에 저장

- 제목, 헤드라인, 설명, 제작자, 저작권을 편집할 수 있으며 변경 내용은 로컬 데이터베이스에 저장됩니다.
- XMP로 저장하면 이 필드와 태그 (계층형 키워드 포함), 평점, 즐겨찾기, 선별 상태가 사이드카에 기록됩니다. 다른 XMP 속성과 원본 파일은 유지됩니다.
- 파일을 **Lap에서** 이름 변경, 이동, 복사, 삭제 또는 가져오기하면 Lap은 동시에 로컬 카탈로그를 업데이트합니다. Apple Live Photo 구성 요소, XMP 및 AAE 사이드카 파일 및 활성화한 RAW + JPEG/HEIC 쌍처럼 지원되는 연결 자산도 함께 처리합니다. RAW 쌍에서 한 파일 또는 두 파일을 모두 삭제할 수 있습니다. 한 파일을 남기면 공유 XMP 사이드카도 유지됩니다.
- 내장 이미지 편집을 저장하면 결과 이미지가 선택한 대상 위치에 기록됩니다.

### Lap 데이터베이스에 남는 정보

이 라이브러리 데이터는 로컬 데이터베이스 또는 라이브러리 구성에 저장됩니다.

- 컬렉션, 스마트 앨범의 규칙 및 보기 설정은 로컬 데이터베이스 또는 구성에 저장됩니다.
- AI 검색 데이터, 얼굴 데이터, 썸네일 및 기타 인덱스 또는 캐시 데이터
- 태그, 평점, 즐겨찾기, 선별 상태 및 편집 가능한 텍스트 필드는 로컬에 저장되며 XMP에 쓸 수 있습니다.

편집은 지원되는 필드를 명시적으로 XMP에 저장하기 전까지 데이터베이스에 남습니다. 컬렉션, 스마트 앨범, AI 검색 및 얼굴 데이터는 XMP에 포함되지 않습니다. 원본 파일 복사만으로는 보존되지 않으므로 데이터베이스를 백업하세요.

### Lap 외부에서 파일 작업하기

Lap은 폴더를 다시 스캔하고 많은 파일 시스템 변경 사항을 감지할 수 있습니다. 하지만 파일을 Lap 외부에서 이름 변경, 이동, 교체 또는 복사하면 Lap에만 저장된 정리 정보에 영향을 줄 수 있습니다.

메타데이터 가져오기를 켜면 XMP 사이드카의 변경도 확인합니다. 컬렉션을 유지하려면 Lap에서 파일 이름을 바꾸거나 이동하세요. XMP 사이드카는 원본과 함께 보관하세요. Lap 외부에서도 파일을 관리한다면 사진과 함께 Lap의 데이터베이스와 구성도 백업하세요. 데이터베이스 위치 관리와 백업 생성은 **설정 → 저장소**에서 할 수 있습니다.

Lap의 데이터베이스 또는 구성을 삭제하면 이 로컬 정리 정보와 인덱스 데이터는 제거되지만 원본 미디어 파일은 삭제되지 않습니다.

## Lap 제거

Lap은 기존 사진 폴더를 직접 사용합니다. Lap을 제거하거나 데이터베이스 및 캐시 파일을 삭제해도 원본 사진은 삭제되지 않습니다.

일반 제거 절차는 애플리케이션만 제거합니다. Lap을 완전히 제거하려면 먼저 Lap을 종료하고 애플리케이션을 제거한 다음, 사용 중인 플랫폼에 맞는 명령으로 로컬 데이터베이스, 썸네일 캐시 및 설정 파일을 삭제하세요.

### macOS

Homebrew로 Lap을 설치한 경우:

```bash
brew uninstall --cask lap
```

수동으로 설치한 경우 Lap을 종료하고 `Applications` 폴더의 `Lap.app`을 휴지통으로 이동하세요.

Lap 데이터베이스, 캐시 및 설정 파일을 모두 삭제하려면:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

**설정 > 앱 > 설치된 앱**을 열고 **Lap**을 찾아 **제거**를 선택하세요.

그런 다음 PowerShell을 열고 Lap 데이터베이스, 캐시 및 설정 파일을 모두 삭제하세요:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

DEB로 설치한 경우 패키지를 제거합니다:

```bash
sudo apt remove lap
```

로컬에 설치한 Arch 패키지는 `sudo pacman -R lap`을 사용하세요.

AppImage로 설치한 경우 Lap을 종료하고 다운로드한 `.AppImage` 파일을 삭제합니다.

그런 다음 Lap 데이터베이스, 캐시 및 설정 파일을 모두 삭제하세요:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Lap 설정에서 사용자 지정 데이터베이스 저장 폴더를 선택했다면, 해당 폴더에 Lap 데이터베이스 파일만 포함되어 있는지 확인한 후 별도로 삭제하세요.

## 소스에서 빌드하기

요구 사항: Node.js 20+, pnpm, Rust stable

```bash
# macOS 시스템 의존성
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux 시스템 의존성
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# 복제 및 빌드
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

배포 패키지는 번들된 libheif와 libde265 대신 시스템 libheif(1.17 이상)에 링크할 수 있습니다. 빌드 시 `LAP_SYSTEM_LIBHEIF=1`을(를) 설정하세요. 그러면 `third_party/libheif` 및 `third_party/libde265` 하위 모듈이 필요하지 않습니다. HEVC 디코딩은 시스템 libheif의 코덱 플러그인에 따라 달라집니다.

### Arch Linux (x86_64)

Arch용 실험적 소스 빌드 레시피는 [`contrib/aur`](../contrib/aur/README.md)에 있습니다. 아직 AUR에 게시되지 않아 `yay -S lap`은 사용할 수 없습니다. Arch Linux에서 로컬로 빌드하고 설치하려면:

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

레시피는 업스트림 소스 커밋을 고정하고 번들된 AI 모델과 FFmpeg 도구를 체크섬으로 검증합니다. Arch 빌드는 앱 내 자체 업데이트를 비활성화하여 업그레이드를 pacman이 담당하도록 하며, 다른 플랫폼 빌드는 기존 업데이터를 유지합니다. 빌드 세부 사항과 남은 릴리스 확인 사항은 레시피 가이드를 참조하세요.

## 지원 포맷

Lap은 60개 이상의 이미지, RAW, 비디오 포맷을 지원합니다.

| 유형 | 포맷 목록 |
| :--- | :--- |
| 이미지 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW 사진 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity 문서 | AF, AFPHOTO, AFDESIGN, AFPUB (내장 미리 보기) |
| 비디오 | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. H.264 재생은 모든 플랫폼에서 지원되며, 네이티브 재생이 불가능한 경우 자동으로 호환성 처리가 진행됩니다. HEVC/H.265 및 VP9은 macOS에서 네이티브 지원됩니다. |

### Linux 동영상 재생

동영상이 재생되지 않으면 GStreamer 플러그인을 설치하세요:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## 아키텍처

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Data: SQLite

### 주요 라이브러리

| 라이브러리 | 용도 |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW 이미지 디코딩 및 썸네일 추출 |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF 이미지 디코딩 및 미리보기 생성 |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | 빠른 JPEG 디코딩 및 썸네일 생성 |
| [FFmpeg](https://ffmpeg.org/) | 비디오 처리 및 썸네일 생성 |
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | ExifTool의 Rust 포팅 |
| [Video.js](https://videojs.com/) | 교차 플랫폼 비디오 재생 UI |
| [ONNX Runtime](https://onnxruntime.ai/) | 로컬 AI 모델 추론 엔진 |
| [CLIP](https://github.com/openai/CLIP) | 이미지-텍스트 유사도 검색 |
| [InsightFace](https://github.com/deepinsight/insightface) | 얼굴 감지 및 인식 |
| [Leaflet](https://leafletjs.com/) | 위치 정보가 포함된 사진을 위한 대화형 지도 |
| [daisyUI](https://daisyui.com/) | UI 컴포넌트 라이브러리 |

## 라이선스

GPL-3.0-or-later. 자세한 내용은 [LICENSE](../LICENSE)를 참조하세요.

## 개인정보 보호

데이터 처리와 선택적 온라인 서비스에 대한 자세한 내용은 [개인정보 처리방침](../PRIVACY.md)을 참조하세요.
