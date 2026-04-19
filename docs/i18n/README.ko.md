<div align="center">
  <img src="../../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - 프라이빗 로컬 사진 관리자</h1>
  <h3>macOS, Windows, Linux를 위한 오픈 소스 데스크톱 사진 관리 도구.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | 한국어 | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md)

Lap은 오픈 소스 기반의 '로컬 우선(local-first)' 사진 관리 도구입니다. 가족 앨범을 둘러보고, 오래된 사진을 빠르게 찾으며, 대규모 개인 미디어 라이브러리를 오프라인에서 직접 관리할 수 있도록 설계되었습니다.
클라우드 사진 서비스의 개인정보 보호 대안으로서, 강제 업로드 없음, 로컬 AI 검색, 폴더 중심의 워크플로우를 제공하며 완전히 무료로 사용할 수 있습니다.

- 웹사이트: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- 데모 비디오: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- 개인정보 처리방침: [PRIVACY.md](../../PRIVACY.md)

## Lap 다운로드

[최신 릴리스 페이지](https://github.com/julyx10/lap/releases/latest)를 열고, 시스템에 맞는 파일을 다운로드하세요.

| 플랫폼 | 패키지 | 상태 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Apple 공증 완료 |
| **macOS (Intel)** | `x64.dmg` | Apple 공증 완료 |
| **Windows 10/11 (x64)** | `.msi` | Windows 11에서 테스트 완료. 현재 서명되지 않음 (SmartScreen 경고가 나타날 수 있음) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Linux Mint에서 테스트 완료. 비디오 재생 지원에 대해서는 아래 Linux 참고 사항을 확인하세요. |

### Linux 비디오 재생 참고 사항

Ubuntu/Debian/Linux Mint 사용자는 더 원활한 비디오 재생을 위해 아래 패키지를 설치해야 합니다.

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## 스크린샷

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap 로컬 사진 라이브러리 관리 스크린샷" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap 로컬 AI 사진 검색 스크린샷" width="900">
</p>

> 스크린샷에 사용된 샘플 이미지는 [Wikimedia Commons](https://commons.wikimedia.org/)에서 가져왔습니다.

## 왜 Lap인가요?

- **클라우드 불필요**: 호스팅 서비스에 업로드하는 대신 본인의 디스크에 라이브러리를 보관합니다.
- **기본적으로 프라이빗**: 모든 처리가 로컬에서 이루어지므로 사진이 항상 사용자의 제어 하에 있습니다.
- **무료 사용**: 구독 플랜이나 정기 결제가 없습니다.
- **폴더 중심**: 기존 폴더 구조를 직접 활용하므로 번거로운 '가져오기' 단계가 필요 없습니다.
- **대규모 라이브러리를 위한 고성능**: 대규모 미디어 컬렉션(라이브러리당 10만 개 이상의 파일)에서도 부드러운 탐색과 정리가 가능하도록 최적화되었습니다.

## 주요 기능

- **탐색 및 필터링**: 날짜, 위치, 카메라, 렌즈, 태그, 즐겨찾기, 별점 및 얼굴(BETA)별로 필터링할 수 있습니다.
- **다중 라이브러리 관리**: 여러 라이브러리를 생성하고 빠르게 전환할 수 있습니다.
- **중복 항목 찾기**: 중복 파일을 찾고 불필요한 복사본을 휴지통으로 일괄 이동할 수 있습니다.
- **즉석 편집**: 자르기, 회전, 뒤집기, 크기 조절 및 기본 보정 기능을 제공합니다.
- **폴더 동기화**: 파일 시스템 감지 기반의 동기화 및 수동 새로고침을 지원합니다.
- **로컬 AI 검색 도구**: 텍스트/이미지 검색, 유사 이미지 검색, 얼굴 클러스터링, 스마트 태그 기능을 사용할 수 있습니다.
- **현대적 이미지 포맷 지원**: WebP, HEIC/HEIF, AVIF 및 JXL (JPEG XL) 형식을 지원합니다.
- **RAW 사진 보기**: 20개 이상의 카메라 제조사 RAW 포맷(CR2, NEF, ARW, DNG 등)에 대한 내장 디코딩을 지원합니다.
- **광범위한 비디오 호환성**: MP4, MOV, AVI, MKV 및 20개 이상의 다양한 형식을 지원하며 교차 플랫폼 최적화를 제공합니다.

## 향후 계획

- **Live Photos 및 Motion Photos 지원**: 사진/비디오 혼합 워크플로우 지원.
- **메타데이터 지원 확장**: EXIF, XMP, IPTC 워크플로우 강화.

## 소스에서 빌드하기

요구 사양: Node.js 20+, pnpm, Rust stable.

Linux 사용자는 위의 다운로드 섹션에 있는 비디오 재생 패키지 참고 사항을 확인하세요.

```bash
# macOS 시스템 의존성
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux 시스템 의존성 (Ubuntu/Debian)
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

## 지원 포맷

| 유형 | 포맷 목록 |
| :--- | :--- |
| 이미지 | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| RAW 사진 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| 비디오 | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX 등. H.264 재생은 모든 플랫폼에서 지원되며, 네이티브 재생이 불가한 경우 자동으로 호환성 처리가 진행됩니다. HEVC/H.265 및 VP9은 macOS에서 네이티브 지원됩니다. |

## 아키텍처

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Data: SQLite

### 주요 라이브러리

| 라이브러리 | 용도 |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW 이미지 디코딩 및 썸네일 추출 |
| [FFmpeg](https://ffmpeg.org/) | 비디오 처리 및 썸네일 생성 |
| [ONNX Runtime](https://onnxruntime.ai/) | 로컬 AI 모델 추론 엔진 |
| [CLIP](https://github.com/openai/CLIP) | 이미지-텍스트 유사도 검색 |
| [InsightFace](https://github.com/deepinsight/insightface) | 얼굴 감지 및 인식 |
| [Leaflet](https://leafletjs.com/) | 위치 정보가 포함된 사진을 위한 대화형 지도 |
| [daisyUI](https://daisyui.com/) | UI 컴포넌트 라이브러리 |

## 라이선스

GPL-3.0-or-later. 자세한 내용은 [LICENSE](../../LICENSE)를 참조하세요.
