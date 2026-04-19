<div align="center">
  <img src="../../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Privater lokaler Fotomanager</h1>
  <h3>Open-Source-Desktop-Fotomanager für macOS, Windows und Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | Deutsch | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md)

Lap ist ein quelloffener, lokal orientierter Fotomanager zum Durchsuchen von Familienalben, zum schnellen Finden alter Fotos und zum Offline-Verwalten großer persönlichen Medienbibliotheken.
Es ist eine datenschutzorientierte Alternative zu Cloud-Fotodiensten: kein erzwungener Upload, lokale KI-Suche, ordnerbasierter Workflow und kostenlos nutzbar.

- Website: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Demo-Video: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Datenschutz: [PRIVACY.md](../../PRIVACY.md)

## Lap herunterladen

Öffnen Sie die Seite der [neuesten Veröffentlichungen](https://github.com/julyx10/lap/releases/latest) und laden Sie die Datei herunter, die Ihrem System entspricht:

| Plattform | Paket | Status |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Von Apple notarisiert |
| **macOS (Intel)** | `x64.dmg` | Von Apple notarisiert |
| **Windows 10/11 (x64)** | `.msi` | Unter Windows 11 getestet. Derzeit nicht signiert (SmartScreen-Warnung kann erscheinen) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Unter Linux Mint getestet. Für bessere Video-Unterstützung siehe die Linux-Hinweise unten. |

### Linux Video-Wiedergabe Hinweise

Installieren Sie unter Ubuntu/Debian/Linux Mint diese Pakete für eine bessere Unterstützung der Videowiedergabe:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Screenshots

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap lokaler Fotobibliotheksmanager Screenshot" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap lokale KI-Fotosuche Screenshot" width="900">
</p>

> Die Beispielbilder in den Screenshots stammen von [Wikimedia Commons](https://commons.wikimedia.org/).

## Warum Lap

- **Keine Cloud erforderlich**: Speichern Sie Ihre Bibliothek auf Ihrer eigenen Festplatte, anstatt sie bei einem Dienst hochzuladen.
- **Standardmäßig privat**: Die Verarbeitung erfolgt lokal, sodass Ihre Fotos unter Ihrer Kontrolle bleiben.
- **Kostenlos nutzbar**: Kein Abonnement oder wiederkehrende Gebühren.
- **Ordner-fokussiert**: Arbeiten Sie direkt mit Ihren vorhandenen Ordnern, kein Importschritt erforderlich.
- **Hohe Performance für große Bibliotheken**: Optimiert für flüssiges Durchsuchen und Organisieren riesiger Mediensammlungen (100.000+ Dateien pro Bibliothek).

## Funktionen

- **Durchsuchen und Filtern** nach Datum, Ort, Kamera, Objektiv, Schlagworten, Favoriten, Bewertungen und Gesichtern (BETA).
- **Mehrere Bibliotheken verwalten** und schnell zwischen ihnen wechseln.
- **Duplikate finden** und unerwünschte Kopien stapelweise in den Papierkorb verschieben.
- **Direkt bearbeiten** mit Zuschneiden, Drehen, Spiegeln, Skalieren und grundlegenden Anpassungen.
- **Ordner synchron halten** mit dateisystembasierten Operationen und Aktualisierungsunterstützung.
- **Lokale KI-Werkzeuge nutzen** wie Text-/Bildsuche, Suche nach ähnlichen Bildern, Gesichts-Clustering und intelligente Schlagworte.
- **Moderne Bildformate öffnen** einschließlich WebP, HEIC/HEIF, AVIF and JXL (JPEG XL).
- **RAW-Fotos anzeigen** mit integrierter Dekodierung für über 20 Kamera-RAW-Formate (CR2, NEF, ARW, DNG usw.).
- **Breite Videokompatibilität**: Unterstützung für MP4, MOV, AVI, MKV und über 20 weitere Formate mit plattformübergreifender Optimierung.

## Geplante Funktionen

- **Unterstützung für Live Photos und Motion Photos** für gemischte Foto-/Video-Workflows.
- **Erweiterte Metadaten-Unterstützung** für EXIF-, XMP- und IPTC-Workflows.

## Aus dem Quellcode erstellen

Anforderungen: Node.js 20+, pnpm, Rust stabil.

Für die Videowiedergabe unter Linux siehe die Pakethinweise im Abschnitt Downloads oben.

```bash
# macOS System-Abhängigkeiten
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux System-Abhängigkeiten (Ubuntu/Debian)
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Klonen und Erstellen
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Unterstützte Formate

| Typ | Formate |
| :--- | :--- |
| Bilder | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| RAW-Fotos | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Videos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX und weitere. Die H.264-Wiedergabe wird auf allen Plattformen unterstützt, mit automatischer Kompatibilitätsverarbeitung, wenn die native Wiedergabe nicht verfügbar ist. HEVC/H.265 und VP9 werden nativ auf macOS unterstützt. |

## Architektur

- Kern: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Daten: SQLite

### Wichtige Bibliotheken

| Bibliothek | Zweck |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW-Bilddekodierung und Thumbnail-Extraktion |
| [FFmpeg](https://ffmpeg.org/) | Videoverarbeitung und Thumbnail-Generierung |
| [ONNX Runtime](https://onnxruntime.ai/) | Lokale KI-Modell-Inferenz-Engine |
| [CLIP](https://github.com/openai/CLIP) | Bild-Text-Ähnlichkeitssuche |
| [InsightFace](https://github.com/deepinsight/insightface) | Gesichtserkennung und -identifizierung |
| [Leaflet](https://leafletjs.com/) | Interaktive Karte für Fotos mit GPS-Daten |
| [daisyUI](https://daisyui.com/) | UI-Komponentenbibliothek |

## Lizenz

GPL-3.0-oder-später. Siehe [LICENSE](../../LICENSE).
