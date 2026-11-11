<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Privé lokale fotobeheerder</h1>
  <h3>Open-source fotobeheerder voor desktop op macOS, Windows en Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | Nederlands | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap is een open-source, local-first fotobeheerder om door familiealbums te bladeren, oude foto's snel terug te vinden en grote persoonlijke mediabibliotheken offline te beheren.
Het is een privacyvriendelijk alternatief voor clouddiensten voor foto's: geen verplichte upload, lokaal zoeken met AI, werken vanuit je eigen mappen, en gratis te gebruiken.

## Lap downloaden

Open de [pagina met de nieuwste release](https://github.com/julyx10/lap/releases/latest) en download het bestand dat bij je systeem past:

| Platform | Pakket | Opmerking |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Genotariseerd door Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Niet ondertekend — als SmartScreen de download blokkeert, klik dan op **Toch behouden** |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Voor op Debian gebaseerde distributies (Ubuntu, Debian, Linux Mint, enz.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Maak het bestand uitvoerbaar en dubbelklik om het te starten |

### macOS met Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Schermafbeeldingen

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lap, lokale beheerder voor fotobibliotheken" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Kaartweergave in Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Fotocollage in Lap" width="900">
</p>

## Waarom Lap

- **Local-first als uitgangspunt**: je foto's blijven op je eigen schijf, zonder verplicht cloudaccount of upload.
- **Geen lock-in**: werk rechtstreeks met je bestaande mappen in plaats van alles te importeren in een gesloten database.
- **Privé AI-functies**: zoeken, vergelijkbare foto's, slimme tags en gezichtsherkenning draaien lokaal op je eigen computer.
- **Gemaakt voor grote verzamelingen**: geoptimaliseerd voor vlot bladeren en ordenen in bibliotheken met meer dan 100.000 bestanden.
- **Open source en gratis**: geen abonnement, geen gedwongen ecosysteem, en code die je zelf kunt inzien.

## Functies

- **Flexibel bladeren door je bibliotheek** op datum, map, locatie, camera, objectief, tags, beoordelingen en gezichten, met sortering en zoeken.
- **Werken vanuit mappen** met meerdere bibliotheken, importeren/exporteren via slepen of kopiëren en plakken, synchronisatie met het bestandssysteem en veilig verplaatsen, kopiëren en verwijderen.
- **Lokaal zoeken met AI** op tekst, visuele overeenkomst, onderwerpen, gezichtsgroepering en optioneel meertalig zoeken in meer dan 50 talen.
- **Slimme albums** bewaren weergaven op basis van regels, met eigen groepering en sortering.
- **Importeren op datum** met een indeling per dag, maand, jaar of in één map, met behoud van originele bestandsnamen en het overslaan van duplicaten.
- **Collecties en tags** om geselecteerde bestanden in bulk te ordenen zonder de originelen te verplaatsen of te dupliceren.
- **Duplicaten opruimen** met een overzicht van vrij te maken ruimte en bulkverwijdering over duplicaatsets heen.
- **Hulpmiddelen om te schiften en te vergelijken**, waaronder een vergelijkingsviewer met vier vensters.
- **Ingebouwde bewerking** voor bijsnijden, rechttrekken met willekeurige hoek, draaien, spiegelen, formaat wijzigen en eenvoudige beeldaanpassingen.
- **Fotocollage** om geselecteerde foto's samen te stellen in collages als raster, uitgelijnd mozaïek of fotostapel.
- **Apple Live Photos en Google Motion Photos** met afspelen van beweging en één gezamenlijk filter voor slimme albums.
- **RAW + JPEG/HEIC-paren** weergegeven als één item, waarbij gekoppelde bestanden bij bestandsbewerkingen bij elkaar blijven.
- **Instelbare RAW-miniaturen en -voorbeelden** via RAW-rendering of het ingesloten voorbeeld van de camera, met instelbare helderheid en weergave van RAW+JPEG-paren.
- **Interactieve kaartweergave** om foto's en video's met locatiegegevens te verkennen in clusters die je huidige filters volgen.
- **Offline door albums bladeren** waardoor albums op ontkoppelde of netwerkopslag zichtbaar blijven met miniaturen in de cache.
- **Brede formaatondersteuning** voor meer dan 60 foto-, RAW- en videoformaten.

## Metadata, collecties en het verplaatsen van bestanden

Lap werkt vanuit je mappen, maar niet alle informatie die Lap toont, is in het originele bestand opgeslagen. Dat onderscheid is belangrijk als je dezelfde mappen ook beheert in Finder, Verkenner of een andere foto-app.

### Wat bij het bestand blijft

- Je originele foto's en video's blijven altijd gewone bestanden in hun bestaande mappen.
- Metadata die al in een bestand zit, zoals de EXIF-opnamedatum, camera, objectief, GPS en oriëntatie, wordt uit dat bestand gelezen wanneer Lap het indexeert.
- Als je een bewerking in de ingebouwde editor opslaat, wordt de bewerkte afbeelding naar de gekozen bestemming geschreven.
- Als je bestanden **in Lap** hernoemt, verplaatst, kopieert of verwijdert, werkt Lap tegelijk zijn lokale catalogus bij. Ook ondersteunde gegroepeerde bestanden blijven bij elkaar, zoals de onderdelen van een Apple Live Photo, AAE-sidecars en ingeschakelde RAW + JPEG/HEIC-paren.

### Wat Lap lokaal opslaat

Het volgende zijn bibliotheekgegevens van Lap. Ze worden opgeslagen in de lokale database of bibliotheekconfiguratie van Lap, en niet weggeschreven naar EXIF, IPTC of XMP-sidecars:

- Collecties, tags, opmerkingen, favorieten, beoordelingen en schiftstatus (inclusief gekozen en afgewezen)
- Slimme albums met hun regels, groepering, sortering en volgorde
- AI-zoekgegevens, gezichtsgegevens, miniaturen en andere index- en cachegegevens

Deze gegevens gaan niet mee met een bestand dat buiten Lap wordt gekopieerd, geëxporteerd of verplaatst, en zijn niet automatisch beschikbaar voor andere programma's.

### Werken met bestanden buiten Lap

Lap kan mappen opnieuw scannen en veel wijzigingen in het bestandssysteem herkennen. Maar wijzigingen buiten Lap — zoals bestanden hernoemen, verplaatsen, vervangen of kopiëren — kunnen de ordening verstoren die alleen in Lap is opgeslagen.

Voor het meest betrouwbare resultaat hernoem en verplaats je bestanden in Lap zodra je collecties, tags, opmerkingen, favorieten, beoordelingen of schiftstatus gebruikt. Beheer je bestanden ook buiten Lap, maak dan samen met je foto's een back-up van de database en configuratie van Lap. De databaselocatie beheren en een back-up maken doe je via **Instellingen → Opslag**.

Als je de database of configuratie van Lap verwijdert, verdwijnen deze lokale ordening en indexgegevens, maar je originele mediabestanden worden niet verwijderd.

## Lap verwijderen

Lap werkt rechtstreeks met je bestaande fotomappen. Als je Lap verwijdert, of de database- en cachebestanden ervan wist, worden je originele foto's **niet** verwijderd.

Met de standaardstappen verwijder je alleen het programma. Om Lap volledig te verwijderen sluit je Lap eerst af, verwijder je het programma en wis je daarna de lokale database, miniaturencache en configuratiebestanden met de opschoonopdracht voor jouw platform.

### macOS

Als je Lap met Homebrew hebt geïnstalleerd:

```bash
brew uninstall --cask lap
```

Bij een handmatige installatie sluit je Lap af en sleep je `Lap.app` uit de map `Apps` naar de prullenmand.

Om alle database-, cache- en configuratiebestanden van Lap te verwijderen:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Open **Instellingen > Apps > Geïnstalleerde apps**, zoek **Lap** en kies **Verwijderen**.

Open daarna PowerShell en verwijder alle database-, cache- en configuratiebestanden van Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Bij een DEB-installatie verwijder je het pakket:

```bash
sudo apt remove lap
```

Bij een AppImage-installatie sluit je Lap af en verwijder je het gedownloade `.AppImage`-bestand.

Verwijder daarna alle database-, cache- en configuratiebestanden van Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Heb je in de instellingen van Lap een eigen opslagmap voor de database gekozen, verwijder die map dan apart, nadat je hebt gecontroleerd dat er alleen databasebestanden van Lap in staan.

## Zelf bouwen vanuit de broncode

Vereisten: Node.js 20+, pnpm, Rust stable.

```bash
# macOS-systeemafhankelijkheden
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux-systeemafhankelijkheden
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Klonen en bouwen
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

Distributiepakketten kunnen de libheif van het systeem (1.17 of nieuwer) gebruiken in plaats van de meegeleverde libheif en libde265. Zet hiervoor `LAP_SYSTEM_LIBHEIF=1` tijdens het bouwen. De submodules `third_party/libheif` en `third_party/libde265` zijn dan niet nodig. Het decoderen van HEVC hangt af van de codec-plug-ins van de libheif op het systeem.

## Ondersteunde formaten

Lap ondersteunt meer dan 60 foto-, RAW- en videoformaten.

| Type | Formaten |
| :--- | :--- |
| Afbeeldingen | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-foto's | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Video's | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX en meer. Afspelen van H.264 wordt op alle platforms ondersteund, met automatische compatibiliteitsverwerking als afspelen zonder omzetting niet lukt. HEVC/H.265 en VP9 worden op macOS rechtstreeks ondersteund. |

### Video afspelen op Linux

Lap gebruikt de GStreamer-plug-ins van het systeem om video's af te spelen, ook in de AppImage. Als video's niet afspelen op Ubuntu, Debian of Linux Mint, installeer dan:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architectuur

- Kern: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Gegevens: SQLite

### Belangrijkste bibliotheken

| Bibliotheek | Doel |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW-afbeeldingen decoderen en miniaturen extraheren |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF-afbeeldingen decoderen en voorbeelden genereren |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Snel JPEG decoderen en miniaturen genereren |
| [FFmpeg](https://ffmpeg.org/) | Videoverwerking en miniaturen genereren |
| [Video.js](https://videojs.com/) | Interface voor video afspelen op alle platforms |
| [ONNX Runtime](https://onnxruntime.ai/) | Engine voor lokale uitvoering van AI-modellen |
| [CLIP](https://github.com/openai/CLIP) | Zoeken op overeenkomst tussen afbeelding en tekst |
| [InsightFace](https://github.com/deepinsight/insightface) | Gezichtsdetectie en -herkenning |
| [Leaflet](https://leafletjs.com/) | Interactieve kaart voor foto's met locatiegegevens |
| [daisyUI](https://daisyui.com/) | Bibliotheek met UI-componenten |

## Licentie

GPL-3.0-or-later. Zie [LICENSE](../LICENSE).

## Privacy

Lees het [privacybeleid](../PRIVACY.md) voor details over de omgang met gegevens en optionele onlinediensten.
