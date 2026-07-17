<div align="center">
  <img src="../docs/public/icon.png" alt="Логотип Lap" width="120" style="border-radius: 20px">
  <h1>Lap — Приватный локальный менеджер фотографий</h1>
  <h3>Менеджер фотографий с открытым исходным кодом для macOS, Windows и Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | Русский | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap — это локальный менеджер фотографий с открытым исходным кодом, созданный для просмотра семейных альбомов, быстрого поиска старых снимков и управления огромными медиабиблиотеками без доступа к интернету.
Это приватная альтернатива облачным сервисам: никакой принудительной загрузки, локальный поиск на базе ИИ, рабочий процесс на основе папок и бесплатное использование.

- Сайт: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Демо-видео: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Конфиденциальность: [PRIVACY.md](../PRIVACY.md)

## Скачать Lap

Откройте [страницу последних релизов](https://github.com/julyx10/lap/releases/latest) и скачайте файл, подходящий для вашей системы:

| Платформа | Пакет | Примечание |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Нотариально заверено Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Без подписи — если SmartScreen блокирует загрузку, нажмите **Все равно сохранить** |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | Для дистрибутивов на базе Debian (Ubuntu, Debian, Linux Mint и т.д.) |

### macOS с Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Скриншоты

<p align="center">
  <img src="../docs/public/screenshots/Lap_0.3.0_main_1.png" alt="Скриншот локального менеджера фото Lap" width="900">
</p>

## Почему Lap

- **Локальный подход по умолчанию**: ваши фотографии остаются на вашем диске, без обязательного облачного аккаунта или загрузки.
- **Без привязки к библиотеке**: работайте напрямую с существующими папками, вместо того чтобы импортировать всё в закрытую базу данных.
- **Приватные ИИ-инструменты**: поиск, похожие изображения, умные теги и функции лиц работают локально на вашем компьютере.
- **Для больших коллекций**: оптимизировано для плавного просмотра и организации библиотек с 100 000+ файлов.
- **Открытый исходный код и бесплатно**: без подписки, без навязанной экосистемы и с кодом, который можно проверить.

## Возможности

- **Гибкий просмотр библиотеки** с фильтрами по времени, папкам, месту, камере, объективу, тегам, избранному, рейтингу, сюжетам и лицам.
- **Умные альбомы** сохраняют представления на основе правил с собственной группировкой, сортировкой и порядком.
- **Панель коллекций** хранит временные подборки файлов без перемещения из исходных папок.
- **Локальный ИИ-поиск** для текстовых запросов, визуального сходства, сюжетов, кластеризации лиц и опционального многоязычного поиска на 50+ языках.
- **Apple Live Photos** распознаёт пары HEIC/MOV, воспроизводит их в просмотрщике и сохраняет связанные файлы MOV и AAE при переименовании, перемещении, копировании и удалении.
- **Рабочий процесс на основе папок** с несколькими библиотеками, импортом перетаскиванием, импортом через копирование и вставку, синхронизацией файловой системы и безопасными операциями перемещения/копирования/удаления.
- **Инструменты просмотра и сравнения**, включая просмотрщик для сравнения до четырёх изображений.
- **Инструменты очистки** для поиска дубликатов и пакетного перемещения ненужных файлов в корзину.
- **Встроенное редактирование** для обрезки, поворота, отражения, изменения размера и базовых настроек изображения.
- **Широкая поддержка форматов**: более 60 форматов фото, RAW и видео.

## Удаление Lap

Lap работает напрямую с существующими папками фотографий. Удаление Lap или файлов базы данных и кэша **не** удаляет исходные фотографии.

Стандартное удаление убирает приложение. Чтобы полностью удалить Lap, сначала закройте Lap, удалите приложение, а затем удалите локальную базу данных, кэш миниатюр и файлы конфигурации с помощью команд для вашей платформы.

### macOS

Если вы установили Lap с помощью Homebrew:

```bash
brew uninstall --cask lap
```

При ручной установке закройте Lap и переместите `Lap.app` из папки `Applications` в Корзину.

Чтобы удалить все файлы базы данных, кэша и конфигурации Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Откройте **Параметры > Приложения > Установленные приложения**, найдите **Lap** и выберите **Удалить**.

Затем откройте PowerShell и удалите все файлы базы данных, кэша и конфигурации Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

В дистрибутивах на основе Debian удалите пакет:

```bash
sudo apt remove lap
```

Затем удалите все файлы базы данных, кэша и конфигурации Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Если в настройках Lap вы выбрали пользовательский каталог для базы данных, удалите его отдельно, предварительно убедившись, что он содержит только файлы базы данных Lap.

## Сборка из исходников

Требования: Node.js 20+, pnpm, Rust stable.

```bash
# Системные зависимости macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Системные зависимости Linux
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

Lap поддерживает более 60 форматов фото, RAW и видео.

| Тип | Форматы |
| :--- | :--- |
| Изображения | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-фото | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Видео | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX и другие. Воспроизведение H.264 поддерживается на всех платформах с автоматической обработкой совместимости, если нативное воспроизведение недоступно. HEVC/H.265 и VP9 поддерживаются нативно в macOS. |

### Примечания по воспроизведению видео в Linux

В Linux Mint/Ubuntu/Debian установите эти пакеты для лучшей поддержки воспроизведения видео:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Архитектура

- Ядро: Tauri + Rust
- Фронтенд: Vue + Vite + Tailwind CSS
- Данные: SQLite

### Ключевые библиотеки

| Библиотека | Назначение |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Декодирование RAW-изображений и извлечение миниатюр |
| [libheif](https://github.com/strukturag/libheif) | Декодирование HEIC/HEIF/HIF изображений и генерация превью |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Быстрое декодирование JPEG и генерация миниатюр |
| [FFmpeg](https://ffmpeg.org/) | Обработка видео и генерация миниатюр |
| [Video.js](https://videojs.com/) | Кроссплатформенный интерфейс воспроизведения видео |
| [ONNX Runtime](https://onnxruntime.ai/) | Локальный движок вывода ИИ-моделей |
| [CLIP](https://github.com/openai/CLIP) | Поиск сходства изображений и текста |
| [InsightFace](https://github.com/deepinsight/insightface) | Обнаружение и распознавание лиц |
| [Leaflet](https://leafletjs.com/) | Интерактивная карта для фото с геометками |
| [daisyUI](https://daisyui.com/) | Библиотека UI-компонентов |

## Лицензия

GPL-3.0-or-later. См. [LICENSE](../LICENSE).
