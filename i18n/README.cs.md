<div align="center">
  <img src="../docs/public/icon.png" alt="Logo Lap" width="120" style="border-radius: 20px">
  <h1>Lap – soukromý lokální správce fotografií</h1>
  <h3>Open-source desktopový správce fotografií pro macOS, Windows a Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="Vydání na GitHubu"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="Celkový počet stažení všech vydání na GitHubu"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="Hvězdičky na GitHubu"></a>
  </p>
</div>

[English](../README.md) | Česky | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap je open-source správce fotografií se zaměřením na lokální provoz. Umožňuje procházet rodinná alba, rychle vyhledávat staré fotografie a spravovat offline rozsáhlé osobní knihovny médií.
Bez povinného importu, předplatného nebo nahrávání. Otevřete stávající složky přímo v Lap. Soubory si zachovají umístění i formát. Uložte hodnocení a štítky do standardních doprovodných souborů XMP pro kompatibilní aplikace. Vyhledávání AI běží na vašem zařízení.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">OFICIÁLNÍ WEB</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Stažení Lap

Otevřete [stránku nejnovějšího vydání](https://github.com/julyx10/lap/releases/latest) a stáhněte soubor určený pro váš operační systém:

| Platforma | Balíček | Poznámka |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notářsky ověřeno společností Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Nepodepsáno. V prohlížeči zvolte **Přesto ponechat** nebo v SmartScreenu **Další informace** → **Přesto spustit**. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Pro distribuce založené na Debianu (Ubuntu, Debian, Linux Mint atd.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Nastavte soubor jako spustitelný a potom jej spusťte dvojitým kliknutím |

### macOS s Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Snímky obrazovek

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Správce místní knihovny fotografií Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Zobrazení mapy v Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Koláž fotografií v Lap" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Náhled fotografie v Lap" width="900">
</p>

## Proč Lap

- **Správa knihovny podle složek.** Otevřete stávající složky přímo v Lap. Soubory si zachovají umístění i formát.
- **Místní vyhledávání s AI.** Popište fotografii, vyhledejte podobné snímky nebo procházejte podle tváří. Vše se zpracovává místně.
- **Přenosné hodnocení a štítky.** Uložte je do doprovodných souborů XMP pro použití v kompatibilních aplikacích.
- **Výběr fotografií.** Náhled mezerníkem, posun po ohodnocení, zobrazení oblastí AF, porovnání ve čtyřech panelech a pravidla pro ponechání duplikátů.
- **Pro knihovny s více než 100 000 fotografiemi a videi.** Nativní jádro v Rustu zajišťuje procházení, vyhledávání i skenování.
- **Zdarma a s otevřeným kódem.** Bez předplatného a účtu, v 17 jazycích pro macOS, Windows a Linux.

## Funkce

- **Flexibilní procházení knihovny** podle data, složky, místa, fotoaparátu, objektivu, štítků, hodnocení a tváří, včetně řazení a vyhledávání.
- **Pracovní postup založený na složkách.** Více knihoven, import a export přetažením či přes schránku, dávkové přejmenování a synchronizace, bezpečné přesouvání, kopírování a mazání.
- **Lokální vyhledávání pomocí AI** podle textových zadání, vizuální podobnosti, objektů, seskupování tváří a volitelné vícejazyčné vyhledávání ve více než 50 jazycích.
- **Import metadat a zápis do XMP** · Lap čte vložené EXIF, IPTC a XMP i doprovodné soubory (hodnocení, štítky, oblíbené položky, stav výběru, GPS a data). Textová pole lze upravovat a organizační údaje uložit do XMP.
- **Chytrá alba.** Vlastní seskupování a řazení, filtry podle názvu složky a prohledávatelný výběr hodnot.
- **Import s uspořádáním podle data** do složek podle dne, měsíce, roku nebo do jediné složky, se zachováním původních názvů souborů a vynecháváním duplicit.
- **Kolekce a štítky** umožňují hromadně organizovat vybrané soubory, aniž by se originály přesouvaly nebo duplikovaly. Strom štítků s přetahováním a počty v jednotlivých větvích.
- **Odstraňování duplicit.** Pravidla podle složky, formátu, velikosti a data, uvolnitelné místo a hromadné odstranění.
- **Nástroje pro výběr a porovnávání.** Náhled mezerníkem, posun po ohodnocení, zobrazení oblastí AF, porovnání ve čtyřech panelech a pravidla pro ponechání duplikátů.
- **Vestavěné úpravy** ořezu, narovnání pod libovolným úhlem, otočení, převrácení, změny velikosti a základních úprav obrazu.
- **Koláže fotografií** umožňují sestavit vybrané fotografie do mřížky, mozaiky se zarovnanými okraji nebo hromádky snímků.
- **Apple Live Photos a Google Motion Photos** s přehráváním pohybu a jednotným filtrem pro chytrá alba.
- **Dvojice RAW + JPEG/HEIC** se zobrazují jako jedna položka; propojené soubory zůstávají při operacích se soubory pohromadě. Z dvojice RAW lze odstranit jeden nebo oba soubory. Pokud jeden zachováte, zachová se i sdílený soubor XMP.
- **Nastavitelné náhledy RAW** a miniatury s vykreslením RAW nebo s použitím náhledu vytvořeného fotoaparátem, s nastavitelným jasem a zobrazením dvojic RAW+JPEG.
- **Přizpůsobitelné informace o souboru.** Volitelná pole, podrobnosti na vyžádání a oblasti AF v prohlížeči.
- **Interaktivní zobrazení mapy** umožňuje procházet fotografie a videa s geotagy ve skupinách, které odpovídají aktuálním filtrům.
- **Procházení alb offline** zachovává viditelnost alb na odpojených nebo síťových úložištích díky uloženým miniaturám.
- **Soubory dostupné pouze online.** Soubory OneDrive na vyžádání a zástupné soubory macOS se indexují a zobrazují bez automatického stažení.
- **Světlý a tmavý motiv.** Mohou se řídit vzhledem systému.
- **Široká podpora formátů** pro více než 60 formátů obrázků, RAW a videa. Dokumenty Affinity (vložený náhled).

## Metadata, kolekce a přesouvání souborů

Lap pracuje přímo se složkami. Většinu zobrazených údajů čte ze souborů. Organizační údaje lze zapsat do standardních doprovodných souborů XMP a používat je v jiných aplikacích.

Lap čte kompatibilní metadata z Lightroomu, digiKamu, darktable a ACDSee. Rozsah podporovaných polí závisí na aplikaci.

### Co zůstává u souboru

- Vaše původní fotografie a videa zůstávají vždy běžnými soubory ve svých stávajících složkách.
- Při indexování Lap čte vložené EXIF, IPTC a XMP, včetně údajů o snímku a organizačních polí. Organizační údaje importujete do místní databáze přes **Nastavení → Metadata → Import metadat**. Místní úpravy zůstanou zachovány.
- Platné hodnoty z doprovodného souboru XMP mají přednost před vloženými hodnotami, včetně GPS a data pořízení.
- Název, titulek, popis, autora a autorská práva lze upravit a uložit do místní databáze.
- Uložení do XMP zapíše tato pole, štítky (včetně hierarchických klíčových slov), hodnocení, oblíbené položky a stav výběru do doprovodných souborů. Ostatní vlastnosti XMP a originály zůstanou zachovány.
- Když v Lap přejmenujete, přesunete, zkopírujete, odstraníte nebo importujete soubory, Lap současně aktualizuje svůj místní katalog. Zároveň zachovává pohromadě podporované skupiny souborů, například součásti Apple Live Photo, doprovodné soubory XMP a AAE a zapnuté dvojice RAW + JPEG/HEIC. Z dvojice RAW lze odstranit jeden nebo oba soubory. Pokud jeden zachováte, zachová se i sdílený soubor XMP.
- Uložení úprav obrázku pomocí vestavěných nástrojů zapíše výsledný obrázek do vybraného umístění.

### Co Lap ukládá lokálně

Tyto údaje Lap ukládá do místní databáze nebo konfigurace knihovny.

- Kolekce, chytrá alba, pravidla a nastavení zobrazení zůstávají v místní databázi nebo konfiguraci.
- Data vyhledávání AI, data o tvářích, miniatury a další data indexu a mezipaměti
- Štítky, hodnocení, oblíbené položky, stav výběru a upravitelná textová pole jsou uloženy lokálně a lze je zapsat do XMP.

Při práci se soubory mimo Lap se tato data automaticky nepřenášejí.

### Práce se soubory mimo Lap

Lap může znovu prohledat složky a zjistit mnoho změn v souborovém systému. Změny provedené mimo Lap – například přejmenování, přesun, nahrazení nebo kopírování souborů – však mohou narušit organizaci, která je uložena pouze v Lap.

Při povoleném importu metadat Lap kontroluje také změny v doprovodných souborech XMP. Soubory přejmenovávejte a přesouvejte v Lap, aby zůstaly v kolekcích. Doprovodné soubory XMP ponechte u originálů. Pokud soubory spravujete také mimo Lap, zálohujte databázi a konfiguraci Lap společně s fotografiemi. Umístění databáze můžete spravovat a zálohu vytvořit v **Nastavení → Úložiště**.

Odstranění databáze nebo konfigurace Lap smaže tato místní organizační a indexová data, nikoli však původní mediální soubory.

## Odinstalování Lap

Lap pracuje přímo s vašimi stávajícími složkami s fotografiemi. Odinstalování Lap ani odstranění jeho databázových a mezipaměťových souborů **nesmaže vaše původní fotografie**.

Standardní postup odinstalování odstraní aplikaci. Chcete-li Lap odstranit úplně, nejprve jej ukončete, odinstalujte aplikaci a poté pomocí příkazu pro příslušnou platformu odstraňte místní databázi, mezipaměť miniatur a konfigurační soubory.

### macOS

Pokud jste Lap nainstalovali pomocí Homebrew:

```bash
brew uninstall --cask lap
```

Při ruční odinstalaci ukončete Lap a přesuňte `Lap.app` ze složky `Applications` do Koše.

Chcete-li odstranit veškerou databázi Lap, mezipaměť a konfigurační soubory:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Otevřete **Nastavení > Aplikace > Nainstalované aplikace**, vyhledejte **Lap** a vyberte **Odinstalovat**.

Poté otevřete PowerShell a odstraňte veškerou databázi Lap, mezipaměť a konfigurační soubory:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Při odinstalaci balíčku DEB odinstalujte balíček:

```bash
sudo apt remove lap
```

Místní balíček Arch odinstalujete příkazem `sudo pacman -R lap`.

Při použití AppImage ukončete Lap a odstraňte stažený soubor `.AppImage`.

Poté odstraňte veškerou databázi Lap, mezipaměť a konfigurační soubory:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Pokud jste v nastavení Lap vybrali vlastní adresář pro ukládání databáze, odstraňte tento adresář samostatně až poté, co ověříte, že obsahuje pouze databázové soubory Lap.

## Sestavení ze zdrojového kódu

Požadavky: Node.js 20+, pnpm, Rust stable.

```bash
# Systémové závislosti pro macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Systémové závislosti pro Linux
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Klonování a sestavení
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

Distribuční balíčky mohou místo přibalených knihoven libheif a libde265 odkazovat na systémovou knihovnu libheif (verze 1.17 nebo novější). Pro sestavení nastavte `LAP_SYSTEM_LIBHEIF=1`. V takovém případě nejsou potřeba submoduly `third_party/libheif` a `third_party/libde265`. Dekódování HEVC závisí na kodekových zásuvných modulech systémové knihovny libheif.

### Arch Linux (x86_64)

Experimentální recept pro Arch dosud není v AUR. Místní instalace: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

Recept určuje konkrétní revizi zdrojů a kontroluje součty modelů AI a FFmpeg. Aktualizace spravuje pacman.

## Podporované formáty

Lap podporuje více než 60 formátů obrázků, RAW a videa.

| Typ | Formáty |
| :--- | :--- |
| Obrázky | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotografie RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Dokumenty Affinity | AF, AFPHOTO, AFDESIGN, AFPUB (vložený náhled) |
| Videa | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. Přehrávání H.264 je podporováno na všech platformách; pokud nativní přehrávání není dostupné, automaticky se použije zpracování zajišťující kompatibilitu. HEVC/H.265 a VP9 jsou nativně podporovány v macOS. |

### Přehrávání videa v Linuxu

Lap používá pro přehrávání videa systémové zásuvné moduly GStreamer, a to i v AppImage. Pokud se videa v Ubuntu, Debianu nebo Linux Mint nepřehrávají, nainstalujte:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architektura

- Jádro: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Data: SQLite

### Klíčové knihovny

| Knihovna | Účel |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Dekódování obrázků RAW a získávání miniatur |
| [libheif](https://github.com/strukturag/libheif) | Dekódování obrázků HEIC/HEIF/HIF a vytváření náhledů |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Rychlé dekódování JPEG a vytváření miniatur |
| [FFmpeg](https://ffmpeg.org/) | Zpracování videa a vytváření miniatur |
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Rust port nástroje ExifTool |
| [Video.js](https://videojs.com/) | Uživatelské rozhraní pro přehrávání videa napříč platformami |
| [ONNX Runtime](https://onnxruntime.ai/) | Lokální modul pro spouštění modelů AI |
| [CLIP](https://github.com/openai/CLIP) | Vyhledávání podobnosti mezi obrázkem a textem |
| [InsightFace](https://github.com/deepinsight/insightface) | Detekce a rozpoznávání tváří |
| [Leaflet](https://leafletjs.com/) | Interaktivní mapa pro fotografie s geotagy |
| [daisyUI](https://daisyui.com/) | Knihovna komponent uživatelského rozhraní |

## Licence

GPL-3.0-nebo-novější. Viz [LICENSE](../LICENSE).

## Ochrana soukromí

Podrobnosti o nakládání s daty a volitelných online službách najdete v [Zásadách ochrany soukromí](../PRIVACY.md).
