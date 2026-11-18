<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap – Privát, helyi fotókezelő</h1>
  <h3>Nyílt forráskódú asztali fotókezelő macOS, Windows és Linux rendszerekhez.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | Magyar | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

A Lap egy nyílt forráskódú, helyi alapú fotókezelő családi albumok böngészéséhez, régi fényképek gyors megtalálásához és nagy személyes médiakönyvtárak offline kezeléséhez.
Nincs kötelező importálás, előfizetés vagy feltöltés. Nyissa meg meglévő mappáit közvetlenül a Lapban. A fájlok helye és formátuma megmarad. Mentse őket XMP-kísérőfájlokba kompatibilis alkalmazásokhoz. A mesterséges intelligenciával végzett keresés a saját gépeden fut.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">HIVATALOS WEBOLDAL</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMÓ</a></strong></p>
</div>

## A Lap letöltése

Nyissa meg a [legújabb kiadás oldalát](https://github.com/julyx10/lap/releases/latest), majd töltse le a rendszerének megfelelő fájlt:

| Platform | Csomag | Megjegyzés |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Az Apple által hitelesítve (notarizált) |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Nincs aláírva. A böngészőben válassza a **Megtartás mindenképpen**, illetve a SmartScreenben a **További információ** → **Futtatás mindenképpen** lehetőséget. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Debian-alapú disztribúciókhoz (Ubuntu, Debian, Linux Mint stb.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Tegye a fájlt futtathatóvá, majd indítsa dupla kattintással |

### macOS Homebrew-val

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Képernyőképek

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="A Lap helyi fotókönyvtár-kezelője" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="A Lap térképnézete" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Lap fotókollázs" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Fénykép előnézete a Lapban" width="900">
</p>

## Miért a Lap?

- **Mappaalapú könyvtárkezelés.** Nyissa meg meglévő mappáit közvetlenül a Lapban. A fájlok helye és formátuma megmarad.
- **Helyi AI-keresés.** Írjon le egy fotót, keressen hasonló képeket vagy böngésszen arcok szerint. Minden feldolgozás helyben történik.
- **Hordozható értékelések és címkék.** Mentse őket XMP-kísérőfájlokba kompatibilis alkalmazásokhoz.
- **Fotóválogatás.** Előnézet szóközzel, automatikus továbblépés értékelés után, AF-területek, négypaneles összehasonlítás és duplikátum-megtartási szabályok.
- **Több mint 100 000 fotót és videót tartalmazó könyvtárakhoz.** Natív Rust-mag végzi a böngészést, keresést és beolvasást.
- **Ingyenes és nyílt forráskódú.** Előfizetés és fiók nélkül, 17 nyelven, macOS, Windows és Linux rendszeren.

## Funkciók

- **Rugalmas könyvtárböngészés** dátum, mappa, helyszín, fényképezőgép, objektív, címkék, értékelések és arcok szerint, rendezéssel és kereséssel.
- **Mappaközpontú munkafolyamat.** Több könyvtár, importálás és exportálás húzással vagy vágólapon, kötegelt átnevezés és szinkronizálás, biztonságos áthelyezés, másolás és törlés.
- **Helyi AI-keresés** szöveges lekérdezésekhez, vizuális hasonlósághoz, témákhoz, arccsoportosításhoz, valamint opcionális többnyelvű kereséshez több mint 50 nyelven.
- **Metaadat-importálás és XMP-írás** · A Lap beágyazott EXIF-, IPTC- és XMP-adatokat és kísérőfájlokat olvas (értékelések, címkék, kedvencek, válogatási állapotok, GPS és dátumok). A szövegmezők szerkeszthetők, a rendszerezési adatok XMP-be menthetők.
- **Intelligens albumok.** Egyéni csoportosítás és rendezés, mappanévszűrők és kereshető értékválasztók.
- **Dátum szerinti importálás** napi, havi, éves vagy egyetlen mappás elrendezéssel, eredeti fájlnevekkel és a duplikátumok kihagyásával.
- **Gyűjtemények és címkék** a kiválasztott fájlok tömeges rendszerezéséhez, az eredeti fájlok áthelyezése vagy duplikálása nélkül. Címkefa húzással és ágankénti elemszámmal.
- **Duplikátum-tisztítás.** Szabályok mappa, formátum, méret és dátum szerint, felszabadítható hely és tömeges törlés.
- **Válogatási és összehasonlítási eszközök.** Előnézet szóközzel, automatikus továbblépés értékelés után, AF-területek, négypaneles összehasonlítás és duplikátum-megtartási szabályok.
- **Beépített szerkesztés** vágáshoz, tetszőleges szögű kiegyenesítéshez, forgatáshoz, tükrözéshez, átméretezéshez és alapvető képmódosításokhoz.
- **Fotókollázs** a kiválasztott fényképek rácsos, sorkizárt mozaik vagy képhalom elrendezésű összeállításához.
- **Apple Live Photos és Google Motion Photos** mozgáslejátszással és egységes intelligensalbum-szűrővel.
- **RAW + JPEG/HEIC párok** egyetlen elemként megjelenítve, a kapcsolódó fájlok a fájlműveletek során együtt maradnak. RAW-pár esetén az egyik vagy mindkét fájl törölhető. Ha az egyik megmarad, a közös XMP-fájl is megmarad.
- **Konfigurálható RAW-bélyegképek és előnézetek** RAW-rendereléssel vagy a fényképezőgép beágyazott előnézetének használatával, állítható fényerővel és RAW+JPEG páros megjelenítéssel.
- **Testreszabható fájlinformációk.** Választható mezők, igény szerint betöltött részletek és AF-területek a nézőben.
- **Interaktív térképnézet** a földrajzi címkével ellátott fényképek és videók felfedezéséhez, az aktuális szűrők szerinti csoportosításban.
- **Offline albumböngészés**, amely a gyorsítótárazott bélyegképeknek köszönhetően láthatóan tartja a leválasztott vagy hálózati tárolókon lévő albumokat.
- **Csak a felhőben elérhető fájlok.** A OneDrive igény szerinti fájljai és a macOS helyőrzői automatikus letöltés nélkül jelennek meg és kerülnek az indexbe.
- **Világos és sötét téma.** Követhetik a rendszer megjelenését.
- **Széles körű formátumtámogatás** több mint 60 kép-, RAW- és videoformátumhoz. Affinity-dokumentumok (beágyazott előnézet).

## Metaadatok, gyűjtemények és fájlok áthelyezése

A Lap közvetlenül a mappákkal dolgozik. A megjelenített adatok többségét a fájlokból olvassa. A rendszerezési adatok szabványos XMP-kísérőfájlba írhatók, amelyet más fotóalkalmazások is olvashatnak.

A Lap beolvassa a Lightroom, a digiKam, a darktable és az ACDSee kompatibilis metaadatait. A támogatott mezők alkalmazásonként eltérnek.

### Mi marad a fájllal

- Eredeti fényképei és videói mindig szokásos fájlok maradnak a meglévő mappáikban.
- Indexeléskor a Lap beolvassa a beágyazott EXIF-, IPTC- és XMP-adatokat, beleértve a felvételi és rendszerezési adatokat. A rendszerezési adatok helyi importálásához kapcsold be a **Beállítások → Metaadatok → Metaadatok importálása** lehetőséget. A helyi módosítások megmaradnak.
- Az XMP-kísérőfájl támogatott értékei elsőbbséget élveznek a beágyazott értékekkel szemben, ideértve a GPS-t és a felvétel dátumát.
- A cím, címsor, leírás, készítő és szerzői jog szerkeszthető és a helyi adatbázisban tárolódik.
- Az XMP-mentés ezeket a mezőket, a címkéket (a hierarchikus kulcsszavakkal együtt), az értékeléseket, a kedvenceket és a válogatási állapotokat kísérőfájlba írja. A többi XMP-adat és az eredeti megmarad.
- Amikor fájlokat nevez át, helyez át, másol, töröl vagy importál **a Lapban**, a Lap ezzel egyidejűleg frissíti a helyi katalógusát. A támogatott csoportosított elemeket – például az Apple Live Photo összetevőit, az XMP- és AAE-kísérőfájlokat és az engedélyezett RAW + JPEG/HEIC párokat – együtt tartja. RAW-pár esetén az egyik vagy mindkét fájl törölhető. Ha az egyik megmarad, a közös XMP-fájl is megmarad.
- Egy beépített képszerkesztés mentése az eredményül kapott képet a kiválasztott célhelyre írja.

### Mit tárol a Lap helyileg

A Lap ezeket a helyi adatbázisban vagy a könyvtár beállításaiban tárolja.

- A gyűjtemények, intelligens albumok, szabályok és nézetbeállítások a helyi adatbázisban vagy konfigurációban maradnak.
- AI-keresési adatok, arcadatok, bélyegképek és egyéb index- vagy gyorsítótár-adatok
- A címkék, értékelések, kedvencek, válogatási állapotok és szerkeszthető szövegmezők helyben tárolódnak és XMP-be írhatók.

A Lap alkalmazáson kívül ezek az adatok alapértelmezés szerint nem követik a fájlokat.

### Munka a Lapon kívüli fájlokkal

A Lap képes újra beolvasni a mappákat, és sok fájlrendszer-változást érzékel. A Lapon kívül végzett módosítások – például az átnevezés, áthelyezés, csere vagy másolás – azonban megzavarhatják az olyan rendszerezést, amely csak a Lapban tárolódik.

A metaadat-importálás bekapcsolásakor a Lap az XMP-kísérőfájlok változásait is ellenőrzi. A gyűjtemények megőrzéséhez a Lapban nevezze át és helyezze át a fájlokat. Tartsa az XMP-kísérőfájlokat az eredetiek mellett. Ha a fájlokat a Lapon kívül is kezeli, készítsen biztonsági mentést a Lap adatbázisáról és konfigurációjáról a fényképeivel együtt. Az adatbázis helyét a **Beállítások → Tárolás** menüpontban kezelheti, és ott biztonsági mentést is készíthet.

A Lap adatbázisának vagy konfigurációjának törlése eltávolítja ezt a helyi rendszerezést és indexadatot, de nem törli az eredeti médiafájljait.

## A Lap eltávolítása

A Lap közvetlenül a meglévő fotómappáival működik. A Lap eltávolítása vagy az adatbázis- és gyorsítótár-fájlok törlése **nem** törli az eredeti fényképeit.

A szabványos eltávolítási lépések törlik az alkalmazást. A Lap teljes eltávolításához először lépjen ki a Lapból, távolítsa el az alkalmazást, majd törölje a helyi adatbázist, a bélyegkép-gyorsítótárat és a konfigurációs fájlokat a platformjának megfelelő tisztítási paranccsal.

### macOS

Ha a Lapot Homebrew-val telepítette:

```bash
brew uninstall --cask lap
```

Kézi telepítés esetén lépjen ki a Lapból, és helyezze át a `Lap.app` fájlt az `Applications` mappából a Kukába.

A Lap összes adatbázis-, gyorsítótár- és konfigurációs fájljának eltávolításához:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Nyissa meg a **Gépház > Alkalmazások > Telepített alkalmazások** menüpontot, keresse meg a **Lap** alkalmazást, majd válassza az **Eltávolítás** lehetőséget.

Ezután nyissa meg a PowerShellt, és távolítsa el a Lap összes adatbázis-, gyorsítótár- és konfigurációs fájlját:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

DEB-telepítés esetén távolítsa el a csomagot:

```bash
sudo apt remove lap
```

A helyi Arch csomag eltávolítása: `sudo pacman -R lap`.

AppImage-telepítés esetén lépjen ki a Lapból, és törölje a letöltött `.AppImage` fájlt.

Ezután távolítsa el a Lap összes adatbázis-, gyorsítótár- és konfigurációs fájlját:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Ha a Lap beállításaiban egyéni adatbázis-tárolási könyvtárat választott, törölje ezt a könyvtárat külön, miután megbizonyosodott arról, hogy csak Lap adatbázisfájlokat tartalmaz.

## Build forrásból

Követelmények: Node.js 20+, pnpm, Rust stabil.

```bash
# macOS rendszerfüggőségek
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux rendszerfüggőségek
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Klónozás és build
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

A terjesztési csomagok a rendszer libheif könyvtárát (1.17 vagy újabb) használhatják a csomagolt libheif és libde265 helyett. Állítsa be a `LAP_SYSTEM_LIBHEIF=1` értéket a buildhez. Ekkor a `third_party/libheif` és a `third_party/libde265` almodulokra nincs szükség. A HEVC-dekódolás a rendszer libheif kodek-bővítményeitől függ.

### Arch Linux (x86_64)

A kísérleti Arch recept még nem érhető el az AUR-ban. Helyi telepítés: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

A recept rögzíti a forrásverziót, és ellenőrzőösszeggel vizsgálja az AI-modelleket és az FFmpeg-et. A frissítést a pacman kezeli.

## Támogatott formátumok

A Lap több mint 60 kép-, RAW- és videoformátumot támogat.

| Típus | Formátumok |
| :--- | :--- |
| Képek | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW-fotók | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity-dokumentumok | AF, AFPHOTO, AFDESIGN, AFPUB (beágyazott előnézet) |
| Videók | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. A H.264 lejátszás minden platformon támogatott, automatikus kompatibilitási feldolgozással, ha a natív lejátszás nem érhető el. A HEVC/H.265 és a VP9 natívan támogatott macOS-en. |

### Videolejátszás Linuxon

A Lap a rendszer GStreamer-bővítményeit használja a videolejátszáshoz, az AppImage-ban is. Ha a videók nem játszódnak le Ubuntun, Debianon vagy Linux Minten, telepítse a következőt:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architektúra

- Mag: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Adat: SQLite

### Fő könyvtárak

| Könyvtár | Cél |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW-képek dekódolása és bélyegképek kinyerése |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF képek dekódolása és előnézet-generálás |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Gyors JPEG-dekódolás és bélyegkép-generálás |
| [FFmpeg](https://ffmpeg.org/) | Videofeldolgozás és bélyegkép-generálás |
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Az ExifTool Rust-portja |
| [Video.js](https://videojs.com/) | Platformokon átívelő videolejátszó felhasználói felület |
| [ONNX Runtime](https://onnxruntime.ai/) | Helyi AI-modell következtető motor |
| [CLIP](https://github.com/openai/CLIP) | Kép-szöveg hasonlósági keresés |
| [InsightFace](https://github.com/deepinsight/insightface) | Arcészlelés és arcfelismerés |
| [Leaflet](https://leafletjs.com/) | Interaktív térkép földrajzi címkével ellátott fényképekhez |
| [daisyUI](https://daisyui.com/) | UI komponenskönyvtár |

## Licenc

GPL-3.0-or-later. Lásd a [LICENSE](../LICENSE) fájlt.

## Adatvédelem

Az adatkezeléssel és az opcionális online szolgáltatásokkal kapcsolatos részletekért olvassa el az [Adatvédelmi irányelvek](../PRIVACY.md) dokumentumot.
