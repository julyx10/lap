<div align="center">
  <img src="../docs/public/icon.png" alt="Logo Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Gestionnaire de photos privées locales</h1>
  <h3>Gestionnaire de photos de bureau open source pour macOS, Windows et Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | Français | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap est un gestionnaire de photos open source et local-first conçu pour parcourir les albums familiaux, retrouver rapidement d'anciennes photos et gérer de grandes bibliothèques multimédias personnelles hors ligne.
C'est une alternative respectueuse de la vie privée aux services de photos en ligne : pas de téléchargement forcé, recherche IA locale, flux de travail centré sur les dossiers, et gratuit à utiliser.

## Télécharger Lap

Ouvrez la [page des dernières versions](https://github.com/julyx10/lap/releases/latest), puis téléchargez le fichier correspondant à votre système :

| Plateforme | Paquet | Remarque |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarié par Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Non signé — si SmartScreen bloque le téléchargement, cliquez sur **Conserver quand même** |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Pour les distributions basées sur Debian (Ubuntu, Debian, Linux Mint, etc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Rendez le fichier exécutable, puis double-cliquez pour le lancer |

### macOS avec Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Captures d'écran

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gestionnaire de photothèque locale Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Vue carte de Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Collage de photos dans Lap" width="900">
</p>

## Pourquoi Lap

- **Conçu en local-first** : vos photos restent sur votre propre disque, sans compte cloud ni téléversement obligatoire.
- **Pas de verrouillage de bibliothèque** : travaillez directement avec vos dossiers existants au lieu de tout importer dans une base de données fermée.
- **Outils d'IA privés** : recherche, similarité, tags intelligents et fonctions de visages s'exécutent localement sur votre machine.
- **Conçu pour les grandes collections** : optimisé pour parcourir et organiser des bibliothèques de plus de 100 000 fichiers.
- **Open source et gratuit** : pas d'abonnement, pas d'écosystème imposé, et un code que vous pouvez inspecter.

## Fonctionnalités

- **Navigation flexible** par date, dossier, lieu, appareil, objectif, tags, notes et visages, avec tri et recherche.
- **Organisation par dossiers** avec plusieurs bibliothèques, import/export par glisser-déposer ou copier-coller, synchronisation et opérations de fichiers sécurisées.
- **Recherche par IA locale** par texte, similarité visuelle, sujets et regroupement de visages, avec recherche multilingue optionnelle dans plus de 50 langues.
- **Albums intelligents** pour enregistrer des vues basées sur des règles, avec regroupement et tri personnalisés.
- **Import organisé par date** dans des dossiers par jour, mois, année ou un dossier unique, en conservant les noms d’origine et en ignorant les doublons.
- **Collections et tags** pour organiser les fichiers sélectionnés par lot sans déplacer ni dupliquer les originaux.
- **Nettoyage des doublons** avec estimation de l’espace récupérable et suppression par lot dans plusieurs groupes.
- **Outils de sélection et de comparaison** avec une visionneuse à quatre volets.
- **Édition intégrée** pour recadrer, redresser à un angle quelconque, pivoter, retourner, redimensionner et effectuer des ajustements de base.
- **Collage de photos** pour assembler les photos sélectionnées en grille, en mosaïque justifiée ou en pile de photos.
- **Apple Live Photos et Google Motion Photos** avec lecture animée et filtre commun dans les albums intelligents.
- **Paires RAW + JPEG/HEIC** affichées comme un seul élément, dont les fichiers liés restent ensemble lors des opérations.
- **Miniatures et aperçus RAW configurables** à partir du rendu RAW ou de l’aperçu intégré de l’appareil, avec luminosité réglable et affichage des paires RAW+JPEG.
- **Carte interactive** pour explorer les photos et vidéos géolocalisées, regroupées selon les filtres actifs.
- **Navigation hors ligne des albums** qui garde les albums sur un stockage déconnecté ou réseau visibles avec des miniatures en cache.
- **Large compatibilité** avec plus de 60 formats photo, RAW et vidéo.

## Métadonnées, collections et déplacement de fichiers

Lap est centré sur les dossiers, mais toutes les informations affichées dans Lap ne sont pas intégrées au fichier d’origine. Cette distinction est importante si vous gérez aussi les mêmes dossiers dans Finder, l’Explorateur ou une autre application photo.

### Ce qui reste avec le fichier

- Vos photos et vidéos originales restent toujours des fichiers ordinaires dans leurs dossiers existants.
- Les métadonnées déjà intégrées à un fichier, telles que la date EXIF, l’appareil, l’objectif, le GPS et l’orientation, sont lues depuis ce fichier lorsque Lap l’indexe.
- L’enregistrement d’une modification d’image intégrée écrit l’image obtenue à l’emplacement choisi.
- Lorsque vous renommez, déplacez, copiez ou supprimez des fichiers **dans Lap**, Lap met simultanément à jour son catalogue local. Il conserve également ensemble les éléments groupés pris en charge, tels que les composants Apple Live Photo, les fichiers annexes AAE et les paires RAW + JPEG/HEIC activées.

### Ce que Lap stocke localement

Les informations suivantes sont des données de bibliothèque Lap. Elles sont stockées dans la base de données locale ou la configuration de bibliothèque de Lap, et ne sont pas écrites dans les données EXIF, IPTC ou les fichiers annexes XMP :

- Collections, tags, commentaires, favoris, notes et états de sélection (Sélectionnées et Rejetées)
- Les albums intelligents, ainsi que leurs règles, regroupements, tris et ordres
- Les données de recherche IA, données de visage, miniatures et autres données d’index ou de cache

Ces données ne voyagent pas avec le fichier lorsqu’il est copié, exporté ou déplacé hors de Lap, et elles ne sont pas automatiquement disponibles dans d’autres applications.

### Travailler avec des fichiers hors de Lap

Lap peut analyser à nouveau les dossiers et détecter de nombreux changements du système de fichiers. Toutefois, les modifications effectuées hors de Lap — comme le renommage, le déplacement, le remplacement ou la copie de fichiers — peuvent affecter l’organisation stockée uniquement dans Lap.

Pour des résultats plus fiables, renommez et déplacez les fichiers dans Lap lorsque vous utilisez des collections, tags, commentaires, favoris, notes ou états de sélection. Si vous gérez aussi des fichiers hors de Lap, sauvegardez la base de données et la configuration de Lap avec vos photos. Vous pouvez gérer l’emplacement de la base de données et créer une sauvegarde dans **Paramètres → Stockage**.

Supprimer la base de données ou la configuration de Lap supprime cette organisation locale et les données d’index, mais ne supprime pas vos fichiers média originaux.

## Désinstaller Lap

Lap utilise directement vos dossiers de photos existants. La désinstallation de Lap ou la suppression de ses fichiers de base de données et de cache ne supprime **pas** vos photos originales.

La désinstallation standard supprime l'application. Pour supprimer complètement Lap, quittez d'abord Lap, désinstallez l'application, puis supprimez la base de données locale, le cache des miniatures et les fichiers de configuration à l'aide des commandes correspondant à votre plateforme.

### macOS

Si vous avez installé Lap avec Homebrew :

```bash
brew uninstall --cask lap
```

Pour une installation manuelle, quittez Lap et déplacez `Lap.app` du dossier `Applications` vers la Corbeille.

Pour supprimer tous les fichiers de base de données, de cache et de configuration de Lap :

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Ouvrez **Paramètres > Applications > Applications installées**, recherchez **Lap** et sélectionnez **Désinstaller**.

Ouvrez ensuite PowerShell et supprimez tous les fichiers de base de données, de cache et de configuration de Lap :

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Pour une installation DEB, désinstallez le paquet :

```bash
sudo apt remove lap
```

Pour une installation AppImage, quittez Lap et supprimez le fichier `.AppImage` téléchargé.

Supprimez ensuite tous les fichiers de base de données, de cache et de configuration de Lap :

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Si vous avez sélectionné un dossier de stockage personnalisé pour la base de données dans les paramètres de Lap, supprimez-le séparément après avoir vérifié qu'il contient uniquement des fichiers de base de données Lap.

## Compiler à partir des sources

Configuration requise : Node.js 20+, pnpm, Rust stable.

```bash
# Dépendances système macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dépendances système Linux
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Cloner et compiler
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

Les paquets de distribution peuvent utiliser le libheif système (1.17 ou plus récent) au lieu du libheif et du libde265 intégrés. Définissez `LAP_SYSTEM_LIBHEIF=1` pour la compilation. Les sous-modules `third_party/libheif` et `third_party/libde265` ne sont alors pas nécessaires. Le décodage HEVC dépend des plugins de codec du libheif système.

## Formats supportés

Lap prend en charge plus de 60 formats photo, RAW et vidéo.

| Type | Formats |
| :--- | :--- |
| Images | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Photos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vidéos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX et plus. La lecture H.264 est supportée sur toutes les plateformes, avec un traitement automatique de compatibilité lorsque la lecture native n'est pas disponible. HEVC/H.265 et VP9 sont supportés nativement sur macOS. |

### Lecture vidéo sous Linux

Lap utilise les plugins GStreamer du système pour la lecture vidéo, y compris dans l’AppImage. Si les vidéos ne se lisent pas sous Ubuntu, Debian ou Linux Mint, installez :

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Architecture

- Cœur : Tauri + Rust
- Frontend : Vue + Vite + Tailwind CSS
- Données : SQLite

### Bibliothèques clés

| Bibliothèque | Usage |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Décodage d'images RAW et extraction de miniatures |
| [libheif](https://github.com/strukturag/libheif) | Décodage d'images HEIC/HEIF/HIF et génération d'aperçus |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Décodage JPEG rapide et génération de miniatures |
| [FFmpeg](https://ffmpeg.org/) | Traitement vidéo et génération de miniatures |
| [Video.js](https://videojs.com/) | Interface de lecture vidéo multiplateforme |
| [ONNX Runtime](https://onnxruntime.ai/) | Moteur d'inférence de modèles d'IA local |
| [CLIP](https://github.com/openai/CLIP) | Recherche de similitude image-texte |
| [InsightFace](https://github.com/deepinsight/insightface) | Détection et reconnaissance faciale |
| [Leaflet](https://leafletjs.com/) | Carte interactive pour les photos géotaguées |
| [daisyUI](https://daisyui.com/) | Bibliothèque de composants UI |

## Licence

GPL-3.0-or-later. Voir [LICENSE](../LICENSE).

## Confidentialité

Consultez la [Politique de confidentialité](../PRIVACY.md) pour en savoir plus sur le traitement des données et les services en ligne optionnels.
