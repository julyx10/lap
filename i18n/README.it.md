<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Gestore fotografico locale e privato</h1>
  <h3>Gestore fotografico desktop open source per macOS, Windows e Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | Italiano | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap è un gestore di foto gratuito, open source e basato sui dati locali per macOS, Windows e Linux. Organizza le cartelle esistenti, cerca con IA locale e gestisci grandi librerie senza account cloud o abbonamenti.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">SITO UFFICIALE</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Scaricare Lap

Aprire la pagina dell'[ultima release](https://github.com/julyx10/lap/releases/latest) e scaricare il file corrispondente al proprio sistema:

| Piattaforma | Pacchetto | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizzato da Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Non firmato. Se il browser blocca il download, scegliere **Mantieni comunque**; se SmartScreen blocca il programma di installazione, scegliere **Ulteriori informazioni** → **Esegui comunque**. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Per distribuzioni basate su Debian (Ubuntu, Debian, Linux Mint, ecc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Rendere il file eseguibile, quindi fare doppio clic per avviarlo |

### macOS con Homebrew

```bash
brew install --cask lap
```

Installato tramite il tap `julyx10/lap`? Consultare le [istruzioni di migrazione](https://github.com/julyx10/lap/discussions/567).

## Screenshot

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gestore locale di raccolte fotografiche Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Vista mappa di Lap" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Anteprima di una foto in Lap" width="900">
</p>

## Perché Lap

- **IA privata sul dispositivo.** Descrivi una foto, trova scatti simili o sfoglia per volto. Tutto viene elaborato sul dispositivo.
- **Le tue cartelle, sotto il tuo controllo.** Apri le cartelle esistenti direttamente in Lap. I file conservano posizione e formato originali.
- **Metadati utilizzabili tra applicazioni.** Lap legge EXIF, IPTC e XMP incorporati e i sidecar (valutazioni, tag, preferiti, stati di selezione, GPS e date). È possibile salvare i metadati modificati nei sidecar XMP, lasciando invariati i file originali.
- **Selezione fotografica.** Anteprima con la barra spaziatrice, avanzamento dopo la valutazione, aree AF, confronto a quattro riquadri e regole per conservare i duplicati.
- **Per librerie con oltre 100.000 foto e video.** Un nucleo nativo in Rust gestisce navigazione, ricerca e scansione.
- **Gratuito e open source.** Senza abbonamento o account, in 17 lingue per macOS, Windows e Linux.

## Funzionalità

- **Navigazione flessibile della raccolta** per data, cartella, posizione, fotocamera, obiettivo, tag, valutazioni e volti, con ordinamento e ricerca.
- **Flusso di lavoro incentrato sulle cartelle.** Più librerie, importazione ed esportazione tramite trascinamento o appunti, rinomina in blocco e sincronizzazione, spostamento, copia ed eliminazione sicuri.
- **Ricerca IA locale** con prompt di testo, similarità visiva, soggetti, raggruppamento dei volti e ricerca multilingue facoltativa in oltre 50 lingue.
- **Importazione dei metadati e scrittura in XMP** · Lap legge EXIF, IPTC e XMP incorporati e i sidecar (valutazioni, tag, preferiti, stati di selezione, GPS e date). È possibile modificare i campi di testo e salvare i dati di organizzazione nei sidecar XMP.
- **Album intelligenti.** Raggruppamento e ordinamento personalizzati, filtri per nome cartella e selettori con ricerca.
- **Importazione organizzata per data** con struttura per giorno, mese, anno o cartella singola, nomi file originali e salto dei duplicati.
- **Raccolte e tag gerarchici** per organizzare in blocco i file selezionati senza spostare o duplicare gli originali. Albero di tag con trascinamento e conteggi per ramo.
- **Pulizia dei duplicati.** Regole per cartella, formato, dimensione e data, spazio recuperabile ed eliminazione in blocco.
- **Strumenti di selezione e confronto.** Anteprima con la barra spaziatrice, avanzamento dopo la valutazione, aree AF, confronto a quattro riquadri e regole per conservare i duplicati.
- **Modifica integrata** per ritaglio, raddrizzamento ad angolo arbitrario, rotazione, riflesso, ridimensionamento e regolazioni di base delle immagini.
- **Collage fotografico** per comporre le foto selezionate in un collage a griglia, a mosaico giustificato o a pila di foto.
- **Apple Live Photos e Google Motion Photos** con riproduzione del movimento e un filtro unificato per gli album intelligenti.
- **Coppie RAW + JPEG/HEIC** mostrate come un unico elemento, con i file collegati mantenuti insieme durante le operazioni sui file. È possibile eliminare uno o entrambi i file di una coppia RAW. Se ne mantiene uno, rimane anche il sidecar XMP condiviso.
- **Miniature e anteprime RAW configurabili**, generate dal rendering RAW o dall'anteprima integrata della fotocamera, con luminosità regolabile e visualizzazione delle coppie RAW+JPEG.
- **Informazioni sui file personalizzabili.** Campi selezionabili, dettagli su richiesta e aree AF nel visualizzatore.
- **Vista mappa interattiva** per esplorare foto e video geolocalizzati raggruppati in cluster che seguono i filtri attivi.
- **Navigazione degli album offline** che mantiene visibili gli album su archivi disconnessi o di rete con miniature memorizzate nella cache.
- **File disponibili solo online.** I file su richiesta di OneDrive e i segnaposto macOS vengono indicizzati e mostrati senza scaricarli automaticamente. Lap non li scarica mai automaticamente.
- **Temi chiaro e scuro.** Possono seguire l’aspetto del sistema.
- **Ampio supporto ai formati** per oltre 60 formati immagine, RAW e video. Documenti Affinity (anteprima incorporata).

## Metadati e controllo dei dati

Lap lavora direttamente con le cartelle e legge dai file la maggior parte delle informazioni visualizzate. È possibile scrivere i dati di organizzazione in sidecar XMP standard leggibili da altre app fotografiche.

Lap legge i metadati compatibili di Lightroom, digiKam, darktable e ACDSee. I campi supportati variano in base all’applicazione.

### Leggere i metadati esistenti

- Le foto e i video originali rimangono sempre file comuni nelle rispettive cartelle esistenti.
- Durante l’indicizzazione Lap legge i dati EXIF, IPTC e XMP incorporati, compresi i dati di scatto e di organizzazione. Attivare **Impostazioni → Metadati → Importa metadati** per importare i dati di organizzazione nel database locale. Le modifiche locali vengono conservate.
- I valori supportati nei sidecar XMP hanno precedenza sui valori incorporati, compresi GPS e data di scatto.

### Salvare le modifiche supportate in XMP

- Titolo, intestazione, descrizione, autore e copyright sono modificabili e salvati nel database locale.
- Il salvataggio in XMP scrive questi campi, tag, valutazioni, preferiti e stati di selezione nei sidecar, preservando le altre proprietà XMP e gli originali.
- Quando si rinominano, spostano, copiano, eliminano o importano file **in Lap**, Lap aggiorna contemporaneamente il proprio catalogo locale. Mantiene inoltre insieme i sidecar XMP e gli asset raggruppati supportati, come i componenti delle Apple Live Photo, i sidecar AAE e le coppie RAW + JPEG/HEIC abilitate. È possibile eliminare uno o entrambi i file di una coppia RAW. Se ne mantiene uno, rimane anche il sidecar XMP condiviso.
- Il salvataggio di una modifica immagine integrata scrive l'immagine risultante nella destinazione selezionata.

### Cosa rimane nel database di Lap

Lap conserva questi dati nel database locale o nella configurazione della libreria.

- Raccolte, album intelligenti, regole e impostazioni della vista restano nel database locale o nella configurazione.
- Dati di ricerca IA, dati dei volti, miniature e altri dati di indice e cache
- Tag, valutazioni, preferiti, stati di selezione e campi di testo modificabili sono salvati in locale e possono essere scritti in XMP.

Le modifiche restano nel database finché i campi supportati non vengono esplicitamente salvati in XMP. Raccolte, album intelligenti e dati IA e volti non sono inclusi in XMP. Eseguire il backup del database: copiare solo gli originali non conserva questa organizzazione.

### Lavorare con i file al di fuori di Lap

Lap può rieseguire la scansione delle cartelle e rilevare molte modifiche del file system. Tuttavia, le modifiche apportate al di fuori di Lap — come rinominare, spostare, sostituire o copiare file — possono compromettere l'organizzazione salvata solo in Lap.

Quando l’importazione dei metadati è attiva, Lap controlla anche le modifiche ai sidecar XMP. Rinominare e spostare i file in Lap per preservarne i dati nel database locale, inclusa l'appartenenza alle raccolte. Tenere i sidecar XMP insieme ai file. Se i file vengono gestiti anche al di fuori di Lap, eseguire il backup del database e della configurazione di Lap insieme alle foto. È possibile gestire la posizione del database e creare un backup in **Impostazioni → Archiviazione**.

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

Per disinstallare il pacchetto Arch locale, utilizzare `sudo pacman -R lap`.

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

### Arch Linux (x86_64)

Ricetta sperimentale per Arch, non ancora pubblicata su AUR, quindi `yay -S lap` non è ancora disponibile. Per compilarla e installarla localmente su Arch Linux: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

La ricetta fissa un commit upstream e verifica con checksum i modelli IA e gli strumenti FFmpeg inclusi. La build Arch disabilita gli aggiornamenti automatici in-app, in modo che gli aggiornamenti restino responsabilità di pacman; le build delle altre piattaforme conservano il loro updater esistente. Vedere la guida della ricetta per i dettagli di compilazione e le verifiche di rilascio ancora pendenti.

## Formati supportati

Lap supporta oltre 60 formati immagine, RAW e video.

| Tipo | Formati |
| :--- | :--- |
| Immagini | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Foto RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Documenti Affinity | AF, AFPHOTO, AFDESIGN, AFPUB (anteprima incorporata) |
| Video | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. La riproduzione H.264 è supportata su tutte le piattaforme, con elaborazione automatica di compatibilità quando la riproduzione nativa non è disponibile. HEVC/H.265 e VP9 sono supportati nativamente su macOS. |

### Riproduzione video su Linux

Se i video non vengono riprodotti, installare i plugin GStreamer:

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Port di ExifTool in Rust |
| [Video.js](https://videojs.com/) | Interfaccia di riproduzione video multipiattaforma |
| [ONNX Runtime](https://onnxruntime.ai/) | Motore di inferenza locale per i modelli IA |
| [CLIP](https://github.com/openai/CLIP) | Ricerca di similarità immagine-testo |
| [InsightFace](https://github.com/deepinsight/insightface) | Rilevamento e riconoscimento dei volti |
| [Leaflet](https://leafletjs.com/) | Mappa interattiva per le foto geolocalizzate |
| [daisyUI](https://daisyui.com/) | Libreria di componenti UI |

## Licenza

GPL-3.0-or-later. Vedere [LICENSE](../LICENSE).

## Privacy

Per dettagli sul trattamento dei dati e sui servizi online facoltativi, leggere l'[Informativa sulla privacy](../PRIVACY.md).
