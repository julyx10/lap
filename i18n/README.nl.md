<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Privé lokale fotobeheerder</h1>
  <h3>Open-source fotobeheerder voor desktop op macOS, Windows en Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | Nederlands | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap is een open-source, local-first fotobeheerder om door familiealbums te bladeren, oude foto's snel terug te vinden en grote persoonlijke mediabibliotheken offline te beheren.
Geen verplichte import, abonnement of upload. Open bestaande mappen rechtstreeks in Lap. Bestanden behouden hun locatie en formaat. Sla ze op in XMP-nevenbestanden voor gebruik in compatibele foto-apps. AI-zoeken draait op je eigen apparaat.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">OFFICIËLE WEBSITE</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Lap downloaden

Open de [pagina met de nieuwste release](https://github.com/julyx10/lap/releases/latest) en download het bestand dat bij je systeem past:

| Platform | Pakket | Opmerking |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Genotariseerd door Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Niet ondertekend. Kies **Toch behouden** in de browser of **Meer informatie** → **Toch uitvoeren** in SmartScreen. |
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
  <img src="../docs/public/screenshots/lap_preview.png" alt="Fotovoorbeeld in Lap" width="900">
</p>

## Waarom Lap

- **Bibliotheekbeheer op basis van mappen.** Open bestaande mappen rechtstreeks in Lap. Bestanden behouden hun locatie en formaat.
- **Lokale AI-zoekfunctie.** Beschrijf een foto, vind vergelijkbare beelden of blader op gezicht. Alles wordt lokaal verwerkt.
- **Overdraagbare beoordelingen en tags.** Sla ze op in XMP-nevenbestanden voor gebruik in compatibele foto-apps.
- **Fotoselectie.** Voorvertoning met de spatiebalk, automatisch doorgaan na beoordeling, AF-gebieden, vergelijking in vier panelen en bewaarregels voor duplicaten.
- **Voor bibliotheken met meer dan 100.000 foto's en video's.** Een native Rust-kern verzorgt bladeren, zoeken en scannen.
- **Gratis en opensource.** Zonder abonnement of account, in 17 talen voor macOS, Windows en Linux.

## Functies

- **Flexibel bladeren door je bibliotheek** op datum, map, locatie, camera, objectief, tags, beoordelingen en gezichten, met sortering en zoeken.
- **Werken vanuit mappen.** Meerdere bibliotheken, import en export via slepen of klembord, batchgewijs hernoemen en synchronisatie, veilig verplaatsen, kopiëren en verwijderen.
- **Lokaal zoeken met AI** op tekst, visuele overeenkomst, onderwerpen, gezichtsgroepering en optioneel meertalig zoeken in meer dan 50 talen.
- **Metadata importeren en naar XMP schrijven** · Lap leest ingebedde EXIF-, IPTC- en XMP-gegevens en sidecars (beoordelingen, tags, favorieten, selectiestatus, GPS en datums). Je kunt tekstvelden bewerken en organisatiegegevens naar XMP schrijven.
- **Slimme albums.** Aangepaste groepering en sortering, filters op mapnaam en doorzoekbare keuzelijsten.
- **Importeren op datum** met een indeling per dag, maand, jaar of in één map, met behoud van originele bestandsnamen en het overslaan van duplicaten.
- **Collecties en tags** om geselecteerde bestanden in bulk te ordenen zonder de originelen te verplaatsen of te dupliceren. Tagboom met slepen en aantallen per tak.
- **Duplicaten opruimen.** Regels voor map, formaat, grootte en datum, vrij te maken ruimte en bulkverwijdering.
- **Hulpmiddelen om te schiften en te vergelijken.** Voorvertoning met de spatiebalk, automatisch doorgaan na beoordeling, AF-gebieden, vergelijking in vier panelen en bewaarregels voor duplicaten.
- **Ingebouwde bewerking** voor bijsnijden, rechttrekken met willekeurige hoek, draaien, spiegelen, formaat wijzigen en eenvoudige beeldaanpassingen.
- **Fotocollage** om geselecteerde foto's samen te stellen in collages als raster, uitgelijnd mozaïek of fotostapel.
- **Apple Live Photos en Google Motion Photos** met afspelen van beweging en één gezamenlijk filter voor slimme albums.
- **RAW + JPEG/HEIC-paren** weergegeven als één item, waarbij gekoppelde bestanden bij bestandsbewerkingen bij elkaar blijven. Bij een RAW-paar kunt u één bestand of beide verwijderen. Als één bestand blijft, blijft ook het gedeelde XMP-bestand behouden.
- **Instelbare RAW-miniaturen en -voorbeelden** via RAW-rendering of het ingesloten voorbeeld van de camera, met instelbare helderheid en weergave van RAW+JPEG-paren.
- **Aanpasbare bestandsinformatie.** Kiesbare velden, details op aanvraag en AF-gebieden in de viewer.
- **Interactieve kaartweergave** om foto's en video's met locatiegegevens te verkennen in clusters die je huidige filters volgen.
- **Offline door albums bladeren** waardoor albums op ontkoppelde of netwerkopslag zichtbaar blijven met miniaturen in de cache.
- **Bestanden die alleen online staan.** OneDrive-bestanden op aanvraag en macOS-placeholders worden zonder automatische download geïndexeerd en weergegeven.
- **Licht en donker thema.** Kunnen de systeemweergave volgen.
- **Brede formaatondersteuning** voor meer dan 60 afbeeldings-, RAW- en videoformaten. Affinity-documenten (ingesloten voorvertoning).

## Metadata, collecties en het verplaatsen van bestanden

Lap werkt rechtstreeks met je mappen en leest de meeste getoonde informatie uit de bestanden. Je kunt organisatiegegevens bewaren in standaard XMP-sidecars die andere foto-apps kunnen lezen.

Lap leest compatibele metadata van Lightroom, digiKam, darktable en ACDSee. Welke velden worden ondersteund, verschilt per app.

### Wat bij het bestand blijft

- Je originele foto's en video's blijven altijd gewone bestanden in hun bestaande mappen.
- Bij het indexeren leest Lap ingebedde EXIF-, IPTC- en XMP-gegevens, waaronder opname- en organisatiegegevens. Schakel **Instellingen → Metadata → Metadata importeren** in om organisatiegegevens in de lokale database op te nemen. Lokale bewerkingen blijven behouden.
- Ondersteunde waarden uit XMP-sidecars gaan vóór ingebedde waarden, ook voor GPS en opnamedatum.
- Titel, kopregel, beschrijving, maker en copyright zijn bewerkbaar en worden lokaal in de database opgeslagen.
- Opslaan naar XMP schrijft deze velden, tags (inclusief hiërarchische trefwoorden), beoordelingen, favorieten en selectiestatussen naar nevenbestanden. Andere XMP-eigenschappen en originelen blijven behouden.
- Als je bestanden **in Lap** hernoemt, verplaatst, kopieert, verwijdert of importeert, werkt Lap tegelijk zijn lokale catalogus bij. Ook ondersteunde gegroepeerde bestanden blijven bij elkaar, zoals de onderdelen van een Apple Live Photo, XMP- en AAE-sidecars en ingeschakelde RAW + JPEG/HEIC-paren. Bij een RAW-paar kunt u één bestand of beide verwijderen. Als één bestand blijft, blijft ook het gedeelde XMP-bestand behouden.
- Als je een bewerking in de ingebouwde editor opslaat, wordt de bewerkte afbeelding naar de gekozen bestemming geschreven.

### Wat Lap lokaal opslaat

Lap bewaart deze bibliotheekgegevens in de lokale database of bibliotheekconfiguratie.

- Collecties, slimme albums, regels en weergave-instellingen blijven in de lokale database of configuratie.
- AI-zoekgegevens, gezichtsgegevens, miniaturen en andere index- en cachegegevens
- Tags, beoordelingen, favorieten, selectiestatus en bewerkbare tekstvelden worden lokaal opgeslagen en kunnen naar XMP worden geschreven.

Standaard gaan deze gegevens niet mee met bestanden buiten Lap.

### Werken met bestanden buiten Lap

Lap kan mappen opnieuw scannen en veel wijzigingen in het bestandssysteem herkennen. Maar wijzigingen buiten Lap — zoals bestanden hernoemen, verplaatsen, vervangen of kopiëren — kunnen de ordening verstoren die alleen in Lap is opgeslagen.

Als metadata-import is ingeschakeld, controleert Lap ook op wijzigingen in XMP-nevenbestanden. Hernoem en verplaats bestanden in Lap om collecties te behouden. Houd XMP-nevenbestanden bij de originelen. Beheer je bestanden ook buiten Lap, maak dan samen met je foto's een back-up van de database en configuratie van Lap. De databaselocatie beheren en een back-up maken doe je via **Instellingen → Opslag**.

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

Verwijder een lokaal Arch-pakket met `sudo pacman -R lap`.

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

### Arch Linux (x86_64)

Experimenteel bouwrecept voor Arch, nog niet in de AUR. Lokale installatie: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

Het recept legt de commit vast en controleert AI-modellen en FFmpeg met checksums. Pacman beheert updates.

## Ondersteunde formaten

Lap ondersteunt meer dan 60 afbeeldings-, RAW- en videoformaten.

| Type | Formaten |
| :--- | :--- |
| Afbeeldingen | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-foto's | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity-documenten | AF, AFPHOTO, AFDESIGN, AFPUB (ingesloten voorvertoning) |
| Video's | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. Afspelen van H.264 wordt op alle platforms ondersteund, met automatische compatibiliteitsverwerking als afspelen zonder omzetting niet lukt. HEVC/H.265 en VP9 worden op macOS rechtstreeks ondersteund. |

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Rust-port van ExifTool |
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
