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

[English](../README.md) | Česky | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap je open-source správce fotografií se zaměřením na lokální provoz. Umožňuje procházet rodinná alba, rychle vyhledávat staré fotografie a spravovat offline rozsáhlé osobní knihovny médií.
Je to alternativa ke cloudovým fotografickým službám zaměřená na soukromí: bez povinného nahrávání, s lokálním vyhledáváním pomocí AI, pracovním postupem založeným na složkách a bezplatným používáním.

## Stažení Lap

Otevřete [stránku nejnovějšího vydání](https://github.com/julyx10/lap/releases/latest) a stáhněte soubor určený pro váš operační systém:

| Platforma | Balíček | Poznámka |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notářsky ověřeno společností Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Nepodepsáno – pokud SmartScreen zablokuje stažení, klikněte na **Přesto pokračovat** |
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
</p>

## Proč Lap

- **Navržen pro lokální provoz**: vaše fotografie zůstávají na vlastním disku, bez nutnosti cloudového účtu nebo nahrávání.
- **Bez závislosti na jediné knihovně**: pracujte přímo se svými existujícími složkami, místo abyste vše importovali do uzavřené databáze.
- **Nástroje AI s důrazem na soukromí**: vyhledávání, podobnost, chytré štítky a funkce rozpoznávání tváří běží lokálně na vašem počítači.
- **Pro rozsáhlé sbírky**: optimalizováno pro plynulé procházení a organizaci knihoven se 100 000 a více fotografií.
- **Open source a zdarma**: bez předplatného, bez nuceného používání jednoho ekosystému a s veřejně dostupným zdrojovým kódem.

## Funkce

- **Flexibilní procházení knihovny** podle data, složky, místa, fotoaparátu, objektivu, štítků, hodnocení a tváří, včetně řazení a vyhledávání.
- **Pracovní postup založený na složkách** s více knihovnami, importem/exportem přetažením, importem/exportem pomocí kopírování a vložení, synchronizací se souborovým systémem a bezpečnými operacemi přesunu, kopírování a mazání.
- **Lokální vyhledávání pomocí AI** podle textových zadání, vizuální podobnosti, objektů, seskupování tváří a volitelné vícejazyčné vyhledávání ve více než 50 jazycích.
- **Chytrá alba** ukládají pohledy založené na pravidlech s vlastním seskupováním a řazením.
- **Import s uspořádáním podle data** do složek podle dne, měsíce, roku nebo do jediné složky, se zachováním původních názvů souborů a vynecháváním duplicit.
- **Kolekce a štítky** umožňují hromadně organizovat vybrané soubory, aniž by se originály přesouvaly nebo duplikovaly.
- **Odstraňování duplicit** s přehledem o velikosti místa, které lze uvolnit, a možností hromadného odstranění v rámci skupin duplicit.
- **Nástroje pro výběr a porovnávání** včetně prohlížeče pro porovnání obrázků ve čtyřech panelech.
- **Vestavěné úpravy** ořezu, narovnání pod libovolným úhlem, otočení, převrácení, změny velikosti a základních úprav obrazu.
- **Koláže fotografií** umožňují sestavit vybrané fotografie do mřížky, mozaiky se zarovnanými okraji nebo hromádky snímků.
- **Apple Live Photos a Google Motion Photos** s přehráváním pohybu a jednotným filtrem pro chytrá alba.
- **Dvojice RAW + JPEG/HEIC** se zobrazují jako jedna položka; propojené soubory zůstávají při operacích se soubory pohromadě.
- **Nastavitelné náhledy RAW** a miniatury s vykreslením RAW nebo s použitím náhledu vytvořeného fotoaparátem, s nastavitelným jasem a zobrazením dvojic RAW+JPEG.
- **Interaktivní zobrazení mapy** umožňuje procházet fotografie a videa s geotagy ve skupinách, které odpovídají aktuálním filtrům.
- **Procházení alb offline** zachovává viditelnost alb na odpojených nebo síťových úložištích díky uloženým miniaturám.
- **Široká podpora formátů** pro více než 60 formátů fotografií, RAW a videa.

## Metadata, kolekce a přesouvání souborů

Lap pracuje především se složkami, ale ne všechny informace zobrazované v Lap jsou vloženy do původního souboru. Tento rozdíl je důležitý, pokud stejné složky spravujete také ve Vyhledávači, Průzkumníku souborů nebo jiné fotografické aplikaci.

### Co zůstává u souboru

- Vaše původní fotografie a videa zůstávají vždy běžnými soubory ve svých stávajících složkách.
- Metadata již vložená do souboru, například datum pořízení EXIF, fotoaparát, objektiv, GPS a orientace, Lap je načítá při indexování z tohoto souboru.
- Uložení úprav obrázku pomocí vestavěných nástrojů zapíše výsledný obrázek do vybraného umístění.
- Když v Lap přejmenujete, přesunete, zkopírujete nebo odstraníte soubory, Lap současně aktualizuje svůj místní katalog. Zároveň zachovává pohromadě podporované skupiny souborů, například součásti Apple Live Photo, doprovodné soubory AAE a zapnuté dvojice RAW + JPEG/HEIC.

### Co Lap ukládá lokálně

Následující údaje patří ke knihovně Lap. Jsou uloženy v místní databázi Lap nebo v konfiguraci knihovny, nikoli v doprovodných souborech EXIF, IPTC nebo XMP:

- Kolekce, štítky, komentáře, oblíbené položky, hodnocení a stavy výběru (včetně položek Vybrané a Odmítnuté)
- Chytrá alba a jejich pravidla, seskupování, řazení a pořadí
- Data vyhledávání AI, data o tvářích, miniatury a další data indexu a mezipaměti

Tato data se nepřenášejí spolu se souborem při kopírování, exportu nebo přesunu mimo Lap a nejsou automaticky dostupná jiným aplikacím.

### Práce se soubory mimo Lap

Lap může znovu prohledat složky a zjistit mnoho změn v souborovém systému. Změny provedené mimo Lap – například přejmenování, přesun, nahrazení nebo kopírování souborů – však mohou narušit organizaci, která je uložena pouze v Lap.

Nejspolehlivější je přejmenovávat a přesouvat soubory přímo v Lap, pokud spoléháte na kolekce, štítky, komentáře, oblíbené položky, hodnocení nebo stavy výběru. Pokud soubory spravujete také mimo Lap, zálohujte databázi a konfiguraci Lap společně s fotografiemi. Umístění databáze můžete spravovat a zálohu vytvořit v **Nastavení → Úložiště**.

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

## Podporované formáty

Lap podporuje více než 60 formátů fotografií, RAW a videa.

| Typ | Formáty |
| :--- | :--- |
| Obrázky | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotografie RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Videa | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX a další. Přehrávání H.264 je podporováno na všech platformách; pokud nativní přehrávání není dostupné, automaticky se použije zpracování zajišťující kompatibilitu. HEVC/H.265 a VP9 jsou nativně podporovány v macOS. |

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
