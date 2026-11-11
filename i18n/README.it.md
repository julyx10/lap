<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Gestore fotografico locale e privato</h1>
  <h3>Gestore fotografico desktop open source per macOS, Windows e Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | Italiano | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap è un gestore fotografico open source e local-first, pensato per sfogliare album di famiglia, ritrovare rapidamente vecchie foto e gestire offline ampie raccolte multimediali personali.
È un'alternativa attenta alla privacy ai servizi fotografici cloud: nessun caricamento forzato, ricerca IA locale, flusso di lavoro incentrato sulle cartelle e utilizzo gratuito.

## Scaricare Lap

Aprire la pagina dell'[ultima release](https://github.com/julyx10/lap/releases/latest) e scaricare il file corrispondente al proprio sistema:

| Piattaforma | Pacchetto | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizzato da Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Non firmato — se SmartScreen blocca il download, fare clic su **Mantieni comunque** |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Per distribuzioni basate su Debian (Ubuntu, Debian, Linux Mint, ecc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Rendere il file eseguibile, quindi fare doppio clic per avviarlo |

### macOS con Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Screenshot

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gestore locale di raccolte fotografiche Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Vista mappa di Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Collage fotografico di Lap" width="900">
</p>

## Perché Lap

- **Local-first per progettazione**: le foto restano sul proprio disco, senza account cloud o caricamenti obbligatori.
- **Nessun vincolo sulla raccolta**: si lavora direttamente sulle cartelle esistenti invece di importare tutto in un database chiuso.
- **Strumenti IA privati**: ricerca, similarità, tag intelligenti e funzioni per i volti vengono eseguiti localmente sul proprio dispositivo.
- **Pensato per ampie raccolte**: ottimizzato per una navigazione e un'organizzazione fluide anche su raccolte con oltre 100.000 file.
- **Open source e gratuito**: nessun abbonamento, nessun ecosistema forzato e codice liberamente verificabile.

## Funzionalità

- **Navigazione flessibile della raccolta** per data, cartella, posizione, fotocamera, obiettivo, tag, valutazioni e volti, con ordinamento e ricerca.
- **Flusso di lavoro incentrato sulle cartelle** con raccolte multiple, importazione/esportazione tramite trascinamento, importazione/esportazione con copia-incolla, sincronizzazione del file system e operazioni sicure di spostamento, copia ed eliminazione.
- **Ricerca IA locale** con prompt di testo, similarità visiva, soggetti, raggruppamento dei volti e ricerca multilingue facoltativa in oltre 50 lingue.
- **Album intelligenti** che salvano viste basate su regole, con raggruppamento e ordinamento personalizzati.
- **Importazione organizzata per data** con struttura per giorno, mese, anno o cartella singola, nomi file originali e salto dei duplicati.
- **Raccolte e tag** per organizzare in blocco i file selezionati senza spostare o duplicare gli originali.
- **Pulizia dei duplicati** con riepilogo dello spazio recuperabile ed eliminazione in blocco tra i gruppi di duplicati.
- **Strumenti di selezione e confronto**, incluso un visualizzatore per il confronto delle immagini a quattro riquadri.
- **Modifica integrata** per ritaglio, raddrizzamento ad angolo arbitrario, rotazione, riflesso, ridimensionamento e regolazioni di base delle immagini.
- **Collage fotografico** per comporre le foto selezionate in un collage a griglia, a mosaico giustificato o a pila di foto.
- **Apple Live Photos e Google Motion Photos** con riproduzione del movimento e un filtro unificato per gli album intelligenti.
- **Coppie RAW + JPEG/HEIC** mostrate come un unico elemento, con i file collegati mantenuti insieme durante le operazioni sui file.
- **Miniature e anteprime RAW configurabili**, generate dal rendering RAW o dall'anteprima integrata della fotocamera, con luminosità regolabile e visualizzazione delle coppie RAW+JPEG.
- **Vista mappa interattiva** per esplorare foto e video geolocalizzati raggruppati in cluster che seguono i filtri attivi.
- **Navigazione degli album offline** che mantiene visibili gli album su archivi disconnessi o di rete con miniature memorizzate nella cache.
- **Ampio supporto ai formati** per oltre 60 formati fotografici, RAW e video.

## Metadati, raccolte e spostamento dei file

Lap è incentrato sulle cartelle, ma non tutte le informazioni mostrate in Lap sono incorporate nel file originale. Questa distinzione è importante quando le stesse cartelle vengono gestite anche nel Finder, in Esplora risorse o in un'altra app per le foto.

### Cosa resta nel file

- Le foto e i video originali rimangono sempre file comuni nelle rispettive cartelle esistenti.
- I metadati già incorporati in un file, come data di scatto EXIF, fotocamera, obiettivo, GPS e orientamento, vengono letti da quel file quando Lap lo indicizza.
- Il salvataggio di una modifica immagine integrata scrive l'immagine risultante nella destinazione selezionata.
- Quando si rinominano, spostano, copiano o eliminano file **in Lap**, Lap aggiorna contemporaneamente il proprio catalogo locale. Mantiene inoltre insieme gli asset raggruppati supportati, come i componenti delle Apple Live Photo, i file sidecar AAE e le coppie RAW + JPEG/HEIC abilitate.

### Cosa viene salvato localmente da Lap

Le voci seguenti sono dati della raccolta di Lap. Vengono salvati nel database locale o nella configurazione della raccolta di Lap, non scritti nei sidecar EXIF, IPTC o XMP:

- Raccolte, tag, commenti, preferiti, valutazioni e stati di selezione (inclusi Scelti e Scartati)
- Album intelligenti con le relative regole, raggruppamento, ordinamento e disposizione
- Dati di ricerca IA, dati dei volti, miniature e altri dati di indice e cache

Questi dati non seguono il file quando viene copiato, esportato o spostato al di fuori di Lap e non sono disponibili automaticamente per le altre applicazioni.

### Lavorare con i file al di fuori di Lap

Lap può rieseguire la scansione delle cartelle e rilevare molte modifiche del file system. Tuttavia, le modifiche apportate al di fuori di Lap — come rinominare, spostare, sostituire o copiare file — possono compromettere l'organizzazione salvata solo in Lap.

Per risultati più affidabili, rinominare e spostare i file in Lap ogni volta che si utilizzano raccolte, tag, commenti, preferiti, valutazioni o stati di selezione. Se i file vengono gestiti anche al di fuori di Lap, eseguire il backup del database e della configurazione di Lap insieme alle foto. È possibile gestire la posizione del database e creare un backup in **Impostazioni → Archiviazione**.

L'eliminazione del database o della configurazione di Lap rimuove questi dati locali di organizzazione e indice, ma non elimina i file multimediali originali.

## Disinstallare Lap

Lap lavora direttamente sulle cartelle fotografiche esistenti. Disinstallare Lap, o eliminarne il database e i file di cache, **non** elimina le foto originali.

La procedura di disinstallazione standard rimuove l'applicazione. Per rimuovere Lap completamente, chiudere prima Lap, disinstallare l'applicazione e quindi eliminare il database locale, la cache delle miniature e i file di configurazione usando il comando di pulizia previsto per la propria piattaforma.

### macOS

Se Lap è stato installato con Homebrew:

```bash
brew uninstall --cask lap
```

Per un'installazione manuale, chiudere Lap e spostare `Lap.app` dalla cartella `Applications` nel Cestino.

Per rimuovere tutti i file di database, cache e configurazione di Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Aprire **Impostazioni > App > App installate**, individuare **Lap** e selezionare **Disinstalla**.

Quindi aprire PowerShell e rimuovere tutti i file di database, cache e configurazione di Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Per le installazioni DEB, disinstallare il pacchetto:

```bash
sudo apt remove lap
```

Per le installazioni AppImage, chiudere Lap ed eliminare il file `.AppImage` scaricato.

Quindi rimuovere tutti i file di database, cache e configurazione di Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Se nelle impostazioni di Lap è stata selezionata una directory personalizzata per l'archiviazione del database, eliminare quella directory separatamente dopo aver verificato che contenga solo file del database di Lap.

## Compilazione dai sorgenti

Requisiti: Node.js 20+, pnpm, Rust stabile.

```bash
# Dipendenze di sistema per macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dipendenze di sistema per Linux
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Clonazione e compilazione
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

I pacchetti di distribuzione possono essere collegati al libheif di sistema (1.17 o successivo) invece che ai libheif e libde265 inclusi. Impostare `LAP_SYSTEM_LIBHEIF=1` per la compilazione. In tal caso i sottomoduli `third_party/libheif` e `third_party/libde265` non sono necessari. La decodifica HEVC dipende dai plugin codec del libheif di sistema.

## Formati supportati

Lap supporta oltre 60 formati fotografici, RAW e video.

| Tipo | Formati |
| :--- | :--- |
| Immagini | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Foto RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Video | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX e altri. La riproduzione H.264 è supportata su tutte le piattaforme, con elaborazione automatica di compatibilità quando la riproduzione nativa non è disponibile. HEVC/H.265 e VP9 sono supportati nativamente su macOS. |

### Riproduzione video su Linux

Lap utilizza i plugin GStreamer di sistema per la riproduzione video, anche nell'AppImage. Se i video non vengono riprodotti su Ubuntu, Debian o Linux Mint, installare:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architettura

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Dati: SQLite

### Librerie principali

| Libreria | Scopo |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Decodifica delle immagini RAW ed estrazione delle miniature |
| [libheif](https://github.com/strukturag/libheif) | Decodifica delle immagini HEIC/HEIF/HIF e generazione delle anteprime |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Decodifica JPEG veloce e generazione delle miniature |
| [FFmpeg](https://ffmpeg.org/) | Elaborazione video e generazione delle miniature |
| [Video.js](https://videojs.com/) | Interfaccia di riproduzione video multipiattaforma |
| [ONNX Runtime](https://onnxruntime.ai/) | Motore di inferenza locale per i modelli IA |
| [CLIP](https://github.com/openai/CLIP) | Ricerca di similarità immagine-testo |
| [InsightFace](https://github.com/deepinsight/insightface) | Rilevamento e riconoscimento dei volti |
| [Leaflet](https://leafletjs.com/) | Mappa interattiva per le foto geolocalizzate |
| [daisyUI](https://daisyui.com/) | Libreria di componenti UI |

## Licenza

GPL-3.0-o-successive. Vedere [LICENSE](../LICENSE).

## Privacy

Per dettagli sul trattamento dei dati e sui servizi online facoltativi, leggere l'[Informativa sulla privacy](../PRIVACY.md).
