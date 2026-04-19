<div align="center">
  <img src="../../docs/public/icon.png" alt="Логотип Lap" width="120" style="border-radius: 20px">
  <h1>Lap — Приватный локальный менеджер фотографий</h1>
  <h3>Менеджер фотографий с открытым исходным кодом для macOS, Windows и Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | Русский

Lap — это локальный менеджер фотографий с открытым исходным кодом, созданный для просмотра семейных альбомов, быстрого поиска старых снимков и управления огромными медиабиблиотеками без доступа к интернету.
Это приватная альтернатива облачным сервисам: никакой принудительной загрузки, локальный поиск на базе ИИ, рабочий процесс на основе папок и бесплатное использование.

- Сайт: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Демо-видео: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Конфиденциальность: [PRIVACY.md](../../PRIVACY.md)

## Скачать Lap

Откройте [страницу последних релизов](https://github.com/julyx10/lap/releases/latest) и скачайте файл, подходящий для вашей системы:

| Платформа | Пакет | Статус |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Нотариально заверено Apple |
| **macOS (Intel)** | `x64.dmg` | Нотариально заверено Apple |
| **Windows 10/11 (x64)** | `.msi` | Протестировано в Windows 11. На данный момент без подписи (может появиться предупреждение SmartScreen) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Протестировано в Linux Mint. Для лучшей поддержки воспроизведения видео см. примечание для Linux ниже. |

### Примечания по воспроизведению видео в Linux

В Ubuntu/Debian/Linux Mint установите эти пакеты для лучшей поддержки воспроизведения видео:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Скриншоты

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Скриншот локального менеджера фото Lap" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Скриншот локального ИИ-поиска фото Lap" width="900">
</p>

> Примеры изображений на скриншотах взяты из [Wikimedia Commons](https://commons.wikimedia.org/).

## Почему Lap

- **Облако не требуется**: храните свою библиотеку на собственном диске, а не на стороннем сервисе.
- **Приватность по умолчанию**: вся обработка происходит локально, ваши фотографии остаются под вашим контролем.
- **Бесплатно**: никаких подписок или периодических платежей.
- **Работа с папками**: работайте напрямую с существующими папками, этап импорта не требуется.
- **Высокая производительность для больших библиотек**: оптимизировано для плавного просмотра и организации огромных медиаколекций (100 000+ файлов в одной библиотеке).

## Возможности

- **Просмотр и фильтрация** по дате, месту, камере, объективу, тегам, избранному, рейтингу и лицам (BETA).
- **Управление несколькими библиотеками** и быстрое переключение между ними.
- **Поиск дубликатов** и пакетное перемещение ненужных копий в корзину.
- **Редактирование на месте**: обрезка, поворот, отражение, изменение размера и базовые настройки изображения.
- **Синхронизация папок**: операции, учитывающие изменения в файловой системе, и ручное обновление.
- **Локальный ИИ-поиск**: поиск по тексту/изображению, поиск похожих изображений, кластеризация лиц и умные теги.
- **Поддержка современных форматов**: WebP, HEIC/HEIF, AVIF и JXL (JPEG XL).
- **Просмотр RAW**: встроенное декодирование более чем 20 форматов RAW (CR2, NEF, ARW, DNG и др.).
- **Широкая совместимость видео**: поддержка MP4, MOV, AVI, MKV и более 20 других форматов с кроссплатформенной оптимизацией.

## Планируемые функции

- **Поддержка Live Photos и Motion Photos**: поддержка смешанных рабочих процессов фото/видео.
- **Расширенная поддержка метаданных**: поддержка EXIF, XMP и IPTC.

## Сборка из исходников

Требования: Node.js 20+, pnpm, Rust stable.

Для воспроизведения видео в Linux см. примечание по пакетам в разделе загрузки выше.

```bash
# Системные зависимости macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Системные зависимости Linux (Ubuntu/Debian)
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Клонирование и сборка
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Поддерживаемые форматы

| Тип | Форматы |
| :--- | :--- |
| Изображения | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| RAW-фото | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Видео | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX и другие. Воспроизведение H.264 поддерживается на всех платформах с автоматической обработкой совместимости, если нативное воспроизведение недоступно. HEVC/H.265 и VP9 поддерживаются нативно в macOS. |

## Архитектура

- Ядро: Tauri + Rust
- Фронтенд: Vue + Vite + Tailwind CSS
- Данные: SQLite

### Ключевые библиотеки

| Библиотека | Назначение |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Декодирование RAW-изображений и извлечение миниатюр |
| [FFmpeg](https://ffmpeg.org/) | Обработка видео и генерация миниатюр |
| [ONNX Runtime](https://onnxruntime.ai/) | Локальный движок вывода ИИ-моделей |
| [CLIP](https://github.com/openai/CLIP) | Поиск сходства изображений и текста |
| [InsightFace](https://github.com/deepinsight/insightface) | Обнаружение и распознавание лиц |
| [Leaflet](https://leafletjs.com/) | Интерактивная карта для фото с геометками |
| [daisyUI](https://daisyui.com/) | Библиотека UI-компонентов |

## Лицензия

GPL-3.0-or-later. См. [LICENSE](../../LICENSE).
