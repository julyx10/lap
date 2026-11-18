<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Privater lokaler Fotomanager</h1>
  <h3>Open-Source-Desktop-Fotomanager für macOS, Windows und Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | Deutsch | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap ist ein quelloffener, lokal orientierter Fotomanager zum Durchsuchen von Familienalben, zum schnellen Finden alter Fotos und zum Offline-Verwalten großer persönlichen Medienbibliotheken.
Kein Pflichtimport, kein Abo, kein Upload. Öffnen Sie vorhandene Ordner direkt in Lap. Ihre Dateien bleiben am ursprünglichen Ort und im ursprünglichen Format. Speichern Sie Bewertungen und Tags für kompatible Foto-Apps in XMP-Begleitdateien. Die KI-Suche läuft auf Ihrem Gerät.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">OFFIZIELLE WEBSITE</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Lap herunterladen

Öffnen Sie die Seite der [neuesten Veröffentlichungen](https://github.com/julyx10/lap/releases/latest) und laden Sie die Datei herunter, die Ihrem System entspricht:

| Plattform | Paket | Hinweis |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Von Apple notarisiert |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Nicht signiert. Wählen Sie im Browser **Trotzdem behalten** oder in SmartScreen **Weitere Informationen** → **Trotzdem ausführen**. |
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
  <img src="../docs/public/screenshots/lap_preview.png" alt="Foto-Vorschau in Lap" width="900">
</p>

## Warum Lap

- **Ordnerbasierte Bibliotheksverwaltung.** Öffnen Sie vorhandene Ordner direkt in Lap. Ihre Dateien bleiben am ursprünglichen Ort und im ursprünglichen Format.
- **Lokale KI-Suche.** Beschreiben Sie ein Foto, finden Sie ähnliche Bilder oder durchsuchen Sie Ihre Bibliothek nach Gesichtern. Alles läuft lokal.
- **Übertragbare Bewertungen und Tags.** Speichern Sie Bewertungen und Tags für kompatible Foto-Apps in XMP-Begleitdateien.
- **Fotoauswahl.** Sofortvorschau mit der Leertaste, automatisches Weiterschalten nach der Bewertung, AF-Feldanzeige, Vierfachvergleich und Regeln zum Behalten von Duplikaten.
- **Für Bibliotheken mit über 100.000 Fotos und Videos.** Ein nativer Rust-Kern übernimmt Durchsuchen, Suchen und Scannen.
- **Kostenlos und quelloffen.** Ohne Abo oder Konto, in 17 Sprachen für macOS, Windows und Linux.

## Funktionen

- **Flexible Bibliotheksansichten** nach Datum, Ordner, Ort, Kamera, Objektiv, Tags, Bewertungen und Gesichtern, mit Sortierung und Suche.
- **Ordnerbasierter Workflow.** Mehrere Bibliotheken, Import und Export per Drag-and-drop oder Zwischenablage, Stapelumbenennung und Dateisynchronisierung, sicheres Verschieben, Kopieren und Löschen.
- **Lokale KI-Suche** mit Textanfragen, visueller Ähnlichkeit, Motiven, Gesichtsgruppierung und optionaler Suche in über 50 Sprachen.
- **Metadatenimport und XMP-Export** · Lap liest eingebettetes EXIF, IPTC und XMP sowie Begleitdateien (Bewertungen, Tags, Favoriten, Auswahlstatus, GPS und Datumsangaben). Textfelder lassen sich bearbeiten und Organisationsdaten in XMP speichern.
- **Intelligente Alben.** Eigene Gruppierung und Sortierung, Ordnernamenfilter und durchsuchbare Werteauswahl.
- **Import nach Datum** in Tages-, Monats-, Jahres- oder einzelne Ordner, mit ursprünglichen Dateinamen und Überspringen von Duplikaten.
- **Sammlungen und Tags** zum gemeinsamen Organisieren ausgewählter Dateien, ohne Originale zu verschieben oder zu kopieren. Verschlagwortung mit Tag-Baum, Drag-and-drop und Zählung je Teilbaum.
- **Duplikatbereinigung.** Regeln nach Ordner, Format, Größe und Datum, freigebbarer Speicherplatz und Stapellöschung.
- **Auswahl- und Vergleichswerkzeuge.** Sofortvorschau mit der Leertaste, automatisches Weiterschalten nach der Bewertung, AF-Feldanzeige, Vierfachvergleich und Regeln zum Behalten von Duplikaten.
- **Integrierte Bearbeitung** zum Zuschneiden, Geradedrehen in beliebigen Winkeln, Drehen, Spiegeln, Skalieren und für grundlegende Bildanpassungen.
- **Fotocollage** zum Zusammenstellen ausgewählter Fotos als Raster, bündiges Mosaik oder Fotostapel.
- **Apple Live Photos und Google Motion Photos** mit Bewegungswiedergabe und gemeinsamem Filter für intelligente Alben.
- **RAW + JPEG/HEIC-Paare** als ein Eintrag, dessen zugehörige Dateien bei Dateiaktionen zusammenbleiben. Bei RAW-Paaren können Sie eine oder beide Dateien löschen. Bleibt eine Datei erhalten, bleibt auch die gemeinsame XMP-Begleitdatei erhalten.
- **Wählbare RAW-Miniaturen und Vorschauen** aus RAW-Rendering oder eingebetteter Kameravorschau, mit einstellbarer Helligkeit und Anzeige von RAW+JPEG-Paaren.
- **Anpassbare Dateiinfo.** Auswählbare Felder, bedarfsweise geladene Details und AF-Felder im Viewer.
- **Interaktive Kartenansicht** für Fotos und Videos mit Standortdaten, gruppiert nach den aktuellen Filtern.
- **Offline-Albenansicht** hält Alben auf getrennten oder Netzwerkspeichern mit zwischengespeicherten Miniaturen sichtbar.
- **Reine Cloud-Dateien.** OneDrive-Dateien bei Bedarf und macOS-Platzhalter werden ohne automatischen Download indiziert und angezeigt.
- **Helle und dunkle Designs.** Optional an das Systemdesign angepasst.
- **Breite Formatunterstützung** für über 60 Bild-, RAW- und Videoformate. Affinity-Dokumente (eingebettete Vorschau).

## Metadaten, Sammlungen und das Verschieben von Dateien

Lap arbeitet direkt mit Ordnern. Die meisten angezeigten Informationen stammen aus den Dateien. Organisationsdaten lassen sich in standardisierte XMP-Begleitdateien schreiben und in anderen Foto-Apps lesen.

Lap liest kompatible Metadaten aus Lightroom, digiKam, darktable und ACDSee. Die unterstützten Felder hängen von der Anwendung ab.

### Was bei der Datei bleibt

- Ihre Originalfotos und -videos bleiben stets normale Dateien in ihren vorhandenen Ordnern.
- Beim Indizieren liest Lap eingebettetes EXIF, IPTC und XMP mit Aufnahme- und Organisationsdaten. Unter **Einstellungen → Metadaten → Metadaten importieren** können Sie Organisationsdaten in die lokale Datenbank übernehmen. Lokale Änderungen bleiben erhalten.
- Unterstützte Werte aus XMP-Begleitdateien haben Vorrang vor eingebetteten Werten, auch bei GPS und Aufnahmedatum.
- Titel, Überschrift, Beschreibung, Urheber und Copyright lassen sich bearbeiten und werden in der lokalen Datenbank gespeichert.
- Der Befehl zum Speichern von Metadaten in XMP schreibt diese Felder sowie Tags (einschließlich hierarchischer Stichwörter), Bewertungen, Favoriten und Auswahlstatus in Begleitdateien. Andere XMP-Eigenschaften und die Originaldateien bleiben erhalten.
- Wenn Sie Dateien **in Lap** umbenennen, verschieben, kopieren, löschen oder importieren, aktualisiert Lap gleichzeitig seinen lokalen Katalog. Unterstützte zusammengehörige Dateien wie Apple-Live-Photo-Komponenten, XMP- und AAE-Begleitdateien und aktivierte RAW- + JPEG/HEIC-Paare werden dabei zusammengehalten. Bei RAW-Paaren können Sie eine oder beide Dateien löschen. Bleibt eine Datei erhalten, bleibt auch die gemeinsame XMP-Begleitdatei erhalten.
- Das Speichern einer integrierten Bildbearbeitung schreibt das resultierende Bild an den gewählten Speicherort.

### Was Lap lokal speichert

Diese Bibliotheksdaten speichert Lap in der lokalen Datenbank oder Bibliothekskonfiguration.

- Sammlungen, Smart-Alben, deren Regeln und Ansichtseinstellungen bleiben in der lokalen Datenbank oder Konfiguration.
- KI-Suchdaten, Gesichtsdaten, Thumbnails und weitere Index- bzw. Cache-Daten
- Tags, Bewertungen, Favoriten, Auswahlstatus und bearbeitbare Textfelder werden lokal gespeichert und können in XMP geschrieben werden.

Bei Dateioperationen außerhalb von Lap werden diese Daten standardmäßig nicht mitgeführt.

### Arbeiten mit Dateien außerhalb von Lap

Lap kann Ordner erneut scannen und viele Änderungen im Dateisystem erkennen. Werden Dateien jedoch außerhalb von Lap umbenannt, verschoben, ersetzt oder kopiert, können nur in Lap gespeicherte Organisationsdaten beeinträchtigt werden.

Bei aktiviertem Metadatenimport prüft Lap auch XMP-Begleitdateien auf Änderungen. Benennen und verschieben Sie Dateien in Lap, um die Sammlungszuordnung zu erhalten. Halten Sie XMP-Begleitdateien bei den jeweiligen Dateien. Wenn Sie Dateien auch außerhalb von Lap verwalten, sichern Sie die Datenbank und Konfiguration von Lap zusammen mit Ihren Fotos. Den Datenbankort können Sie in **Einstellungen → Speicher** verwalten und dort auch eine Sicherung erstellen.

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

Ein lokal installiertes Arch-Paket entfernen Sie mit `sudo pacman -R lap`.

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

### Arch Linux (x86_64)

Experimentelles Arch-Baurezept, noch nicht im AUR. Lokale Installation: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

Das Rezept fixiert den Quellcommit und prüft KI-Modelle und FFmpeg per Prüfsumme. Pacman übernimmt die Aktualisierung.

## Unterstützte Formate

Lap unterstützt über 60 Bild-, RAW- und Videoformate.

| Typ | Formate |
| :--- | :--- |
| Bilder | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-Fotos | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity-Dokumente | AF, AFPHOTO, AFDESIGN, AFPUB (eingebettete Vorschau) |
| Videos | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. Die H.264-Wiedergabe wird auf allen Plattformen unterstützt, mit automatischer Kompatibilitätsverarbeitung, wenn die native Wiedergabe nicht verfügbar ist. HEVC/H.265 und VP9 werden nativ auf macOS unterstützt. |

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Rust-Portierung von ExifTool |
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
