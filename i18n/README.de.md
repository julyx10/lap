<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Privater lokaler Fotomanager</h1>
  <h3>Open-Source-Desktop-Fotomanager für macOS, Windows und Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | Deutsch | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap ist ein quelloffener, lokal orientierter Fotomanager zum Durchsuchen von Familienalben, zum schnellen Finden alter Fotos und zum Offline-Verwalten großer persönlichen Medienbibliotheken.
Es ist eine datenschutzorientierte Alternative zu Cloud-Fotodiensten: kein erzwungener Upload, lokale KI-Suche, ordnerorientierter Workflow und kostenlos nutzbar.

## Lap herunterladen

Öffnen Sie die Seite der [neuesten Veröffentlichungen](https://github.com/julyx10/lap/releases/latest) und laden Sie die Datei herunter, die Ihrem System entspricht:

| Plattform | Paket | Hinweis |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Von Apple notarisiert |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Nicht signiert — falls SmartScreen den Download blockiert, klicken Sie auf **Trotzdem behalten** |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Für Debian-basierte Distributionen (Ubuntu, Debian, Linux Mint, etc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Datei ausführbar machen und per Doppelklick starten |

### macOS mit Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Screenshots

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lokaler Fotobibliotheksmanager Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Kartenansicht von Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Fotocollage in Lap" width="900">
</p>

## Warum Lap

- **Lokal zuerst konzipiert**: Ihre Fotos bleiben auf Ihrer eigenen Festplatte, ohne erforderliches Cloud-Konto oder Upload.
- **Kein Bibliotheks-Lock-in**: Arbeiten Sie direkt mit Ihren vorhandenen Ordnern, statt alles in eine geschlossene Datenbank zu importieren.
- **Lokale KI-Werkzeuge**: Suche, Ähnlichkeit, intelligente Tags und Gesichtserkennung laufen lokal auf Ihrem Gerät.
- **Für große Sammlungen gebaut**: Optimiert für flüssiges Durchsuchen und Organisieren von Bibliotheken mit 100.000+ Dateien.
- **Open Source und kostenlos**: Kein Abonnement, kein erzwungenes Ökosystem und Code, den Sie prüfen können.

## Funktionen

- **Flexible Bibliotheksansichten** nach Datum, Ordner, Ort, Kamera, Objektiv, Tags, Bewertungen und Gesichtern, mit Sortierung und Suche.
- **Ordnerbasierter Workflow** mit mehreren Bibliotheken, Import/Export per Drag-and-drop oder Zwischenablage, Dateisystemsynchronisierung und sicheren Dateiaktionen.
- **Lokale KI-Suche** mit Textanfragen, visueller Ähnlichkeit, Motiven, Gesichtsgruppierung und optionaler Suche in über 50 Sprachen.
- **Intelligente Alben** speichern regelbasierte Ansichten mit eigener Gruppierung und Sortierung.
- **Import nach Datum** in Tages-, Monats-, Jahres- oder einzelne Ordner, mit ursprünglichen Dateinamen und Überspringen von Duplikaten.
- **Sammlungen und Tags** zum gemeinsamen Organisieren ausgewählter Dateien, ohne Originale zu verschieben oder zu kopieren.
- **Duplikatbereinigung** mit Übersicht des freigebbaren Speicherplatzes und gruppenübergreifendem Löschen.
- **Auswahl- und Vergleichswerkzeuge** mit einem Bildvergleich in vier Bereichen.
- **Integrierte Bearbeitung** zum Zuschneiden, Geradedrehen in beliebigen Winkeln, Drehen, Spiegeln, Skalieren und für grundlegende Bildanpassungen.
- **Fotocollage** zum Zusammenstellen ausgewählter Fotos als Raster, bündiges Mosaik oder Fotostapel.
- **Apple Live Photos und Google Motion Photos** mit Bewegungswiedergabe und gemeinsamem Filter für intelligente Alben.
- **RAW + JPEG/HEIC-Paare** als ein Eintrag, dessen zugehörige Dateien bei Dateiaktionen zusammenbleiben.
- **Wählbare RAW-Miniaturen und Vorschauen** aus RAW-Rendering oder eingebetteter Kameravorschau, mit einstellbarer Helligkeit und Anzeige von RAW+JPEG-Paaren.
- **Interaktive Kartenansicht** für Fotos und Videos mit Standortdaten, gruppiert nach den aktuellen Filtern.
- **Offline-Albenansicht** hält Alben auf getrennten oder Netzwerkspeichern mit zwischengespeicherten Miniaturen sichtbar.
- **Breite Formatunterstützung** für über 60 Foto-, RAW- und Videoformate.

## Metadaten, Sammlungen und das Verschieben von Dateien

Lap arbeitet ordnerorientiert, aber nicht jede in Lap angezeigte Information ist in der Originaldatei eingebettet. Das ist wichtig, wenn Sie dieselben Ordner auch im Finder, Explorer oder einer anderen Foto-App verwalten.

### Was bei der Datei bleibt

- Ihre Originalfotos und -videos bleiben stets normale Dateien in ihren vorhandenen Ordnern.
- Bereits in einer Datei eingebettete Metadaten wie EXIF-Aufnahmedatum, Kamera, Objektiv, GPS und Ausrichtung werden beim Indizieren von Lap aus der Datei gelesen.
- Das Speichern einer integrierten Bildbearbeitung schreibt das resultierende Bild an den gewählten Speicherort.
- Wenn Sie Dateien **in Lap** umbenennen, verschieben, kopieren oder löschen, aktualisiert Lap gleichzeitig seinen lokalen Katalog. Unterstützte zusammengehörige Dateien wie Apple-Live-Photo-Komponenten, AAE-Seitendateien und aktivierte RAW- + JPEG/HEIC-Paare werden dabei zusammengehalten.

### Was Lap lokal speichert

Die folgenden Informationen sind Bibliotheksdaten von Lap. Sie werden in der lokalen Datenbank oder Bibliothekskonfiguration von Lap gespeichert und nicht in EXIF, IPTC oder XMP-Seitendateien geschrieben:

- Sammlungen, Tags, Kommentare, Favoriten, Bewertungen und Auswahl-Status (Ausgewählt und Abgelehnt)
- Smart-Alben sowie deren Regeln, Gruppierung, Sortierung und Reihenfolge
- KI-Suchdaten, Gesichtsdaten, Thumbnails und weitere Index- bzw. Cache-Daten

Diese Daten reisen nicht mit einer Datei mit, wenn sie außerhalb von Lap kopiert, exportiert oder verschoben wird, und stehen anderen Anwendungen nicht automatisch zur Verfügung.

### Arbeiten mit Dateien außerhalb von Lap

Lap kann Ordner erneut scannen und viele Änderungen im Dateisystem erkennen. Werden Dateien jedoch außerhalb von Lap umbenannt, verschoben, ersetzt oder kopiert, können nur in Lap gespeicherte Organisationsdaten beeinträchtigt werden.

Für die zuverlässigsten Ergebnisse sollten Sie Dateien in Lap umbenennen und verschieben, wenn Sie Sammlungen, Tags, Kommentare, Favoriten, Bewertungen oder Auswahl-Status verwenden. Wenn Sie Dateien auch außerhalb von Lap verwalten, sichern Sie die Datenbank und Konfiguration von Lap zusammen mit Ihren Fotos. Den Datenbankort können Sie in **Einstellungen → Speicher** verwalten und dort auch eine Sicherung erstellen.

Das Löschen der Datenbank oder Konfiguration von Lap entfernt diese lokale Organisation und Indexdaten, löscht jedoch nicht Ihre Originalmediendateien.

## Lap deinstallieren

Lap arbeitet direkt mit Ihren vorhandenen Fotoordnern. Die Deinstallation von Lap oder das Löschen der Datenbank- und Cache-Dateien löscht **nicht** Ihre Originalfotos.

Die normale Deinstallation entfernt die Anwendung. Um Lap vollständig zu entfernen, beenden Sie Lap zuerst, deinstallieren Sie die Anwendung und löschen Sie anschließend die lokale Datenbank, den Thumbnail-Cache und die Konfigurationsdateien mit den Befehlen für Ihre Plattform.

### macOS

Wenn Sie Lap mit Homebrew installiert haben:

```bash
brew uninstall --cask lap
```

Bei einer manuellen Installation beenden Sie Lap und verschieben Sie `Lap.app` aus dem Ordner `Applications` in den Papierkorb.

So entfernen Sie alle Datenbank-, Cache- und Konfigurationsdateien von Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Öffnen Sie **Einstellungen > Apps > Installierte Apps**, suchen Sie **Lap** und wählen Sie **Deinstallieren**.

Öffnen Sie anschließend PowerShell und entfernen Sie alle Datenbank-, Cache- und Konfigurationsdateien von Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Bei einer DEB-Installation deinstallieren Sie das Paket:

```bash
sudo apt remove lap
```

Bei einer AppImage-Installation beenden Sie Lap und löschen die heruntergeladene `.AppImage`-Datei.

Entfernen Sie anschließend alle Datenbank-, Cache- und Konfigurationsdateien von Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Wenn Sie in den Lap-Einstellungen ein benutzerdefiniertes Verzeichnis für die Datenbank ausgewählt haben, löschen Sie dieses Verzeichnis separat, nachdem Sie geprüft haben, dass es nur Lap-Datenbankdateien enthält.

## Aus dem Quellcode erstellen

Anforderungen: Node.js 20+, pnpm, Rust stabil.

```bash
# macOS System-Abhängigkeiten
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux System-Abhängigkeiten
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

Distributionspakete können das System-libheif (1.17 oder neuer) anstelle des gebündelten libheif und libde265 verwenden. Setze `LAP_SYSTEM_LIBHEIF=1` für den Build. Die Submodule `third_party/libheif` und `third_party/libde265` werden dann nicht benötigt. Die HEVC-Dekodierung hängt von den Codec-Plugins des System-libheif ab.

## Unterstützte Formate

Lap unterstützt über 60 Foto-, RAW- und Videoformate.

| Typ | Formate |
| :--- | :--- |
| Bilder | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-Fotos | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Videos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX und weitere. Die H.264-Wiedergabe wird auf allen Plattformen unterstützt, mit automatischer Kompatibilitätsverarbeitung, wenn die native Wiedergabe nicht verfügbar ist. HEVC/H.265 und VP9 werden nativ auf macOS unterstützt. |

### Videowiedergabe unter Linux

Lap verwendet die GStreamer-Plugins des Systems für die Videowiedergabe, auch im AppImage. Falls Videos unter Ubuntu, Debian oder Linux Mint nicht abgespielt werden, installieren Sie:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architektur

- Kern: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Daten: SQLite

### Wichtige Bibliotheken

| Bibliothek | Zweck |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW-Bilddekodierung und Thumbnail-Extraktion |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF-Bilddekodierung und Vorschaugenerierung |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Schnelle JPEG-Dekodierung und Thumbnail-Generierung |
| [FFmpeg](https://ffmpeg.org/) | Videoverarbeitung und Thumbnail-Generierung |
| [Video.js](https://videojs.com/) | Plattformübergreifende Benutzeroberfläche für die Videowiedergabe |
| [ONNX Runtime](https://onnxruntime.ai/) | Lokale KI-Modell-Inferenz-Engine |
| [CLIP](https://github.com/openai/CLIP) | Bild-Text-Ähnlichkeitssuche |
| [InsightFace](https://github.com/deepinsight/insightface) | Gesichtserkennung und -identifizierung |
| [Leaflet](https://leafletjs.com/) | Interaktive Karte für Fotos mit GPS-Daten |
| [daisyUI](https://daisyui.com/) | UI-Komponentenbibliothek |

## Lizenz

GPL-3.0-oder-später. Siehe [LICENSE](../LICENSE).

## Datenschutz

Details zum Umgang mit Daten und zu optionalen Onlinediensten finden Sie in der [Datenschutzerklärung](../PRIVACY.md).
