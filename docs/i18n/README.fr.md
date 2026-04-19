<div align="center">
  <img src="../../docs/public/icon.png" alt="Logo Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Gestionnaire de photos privées locales</h1>
  <h3>Gestionnaire de photos de bureau open source pour macOS, Windows et Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Deutsch](README.de.md) | Français | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md)

Lap est un gestionnaire de photos open source et local-first conçu pour parcourir les albums familiaux, retrouver rapidement d'anciennes photos et gérer de grandes bibliothèques multimédias personnelles hors ligne.
C'est une alternative respectueuse de la vie privée aux services de photos en ligne : pas de téléchargement forcé, recherche IA locale, flux de travail centré sur les dossiers, et gratuit à utiliser.

- Site web : [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Vidéo de démonstration : [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Confidentialité : [PRIVACY.md](../../PRIVACY.md)

## Télécharger Lap

Ouvrez la [page des dernières versions](https://github.com/julyx10/lap/releases/latest), puis téléchargez le fichier correspondant à votre système :

| Plateforme | Paquet | Statut |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Notarié par Apple |
| **macOS (Intel)** | `x64.dmg` | Notarié par Apple |
| **Windows 10/11 (x64)** | `.msi` | Testé sur Windows 11. Actuellement non signé (un avertissement SmartScreen peut apparaître) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Testé sur Linux Mint. Pour un meilleur support de la lecture vidéo, voir la note Linux ci-dessous. |

### Note sur la lecture vidéo sous Linux

Sur Ubuntu/Debian/Linux Mint, installez ces paquets pour un meilleur support de la lecture vidéo :

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Captures d'écran

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Capture d'écran du gestionnaire de photos locales Lap" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Capture d'écran de la recherche de photos par IA locale Lap" width="900">
</p>

> Les images d'exemple des captures d'écran proviennent de [Wikimedia Commons](https://commons.wikimedia.org/).

## Pourquoi Lap

- **Pas de cloud requis** : conservez votre bibliothèque sur votre propre disque au lieu de la télécharger sur un service hébergé.
- **Privé par défaut** : le traitement se fait localement, vos photos restent donc sous votre contrôle.
- **Gratuit** : pas d'abonnement ni de frais récurrents.
- **Dossiers d'abord** : travaillez directement avec vos dossiers existants, aucune étape d'importation n'est requise.
- **Haute performance pour les grandes bibliothèques** : optimisé pour une navigation et une organisation fluides de vastes collections de médias (plus de 100 000 fichiers par bibliothèque).

## Fonctionnalités

- **Parcourir et filtrer** par date, lieu, appareil photo, objectif, tags, favoris, notes et visages (BETA).
- **Gérer plusieurs bibliothèques** et passer de l'une à l'autre rapidement.
- **Trouver les doublons** et déplacer par lots les copies indésirables vers la corbeille.
- **Édition sur place** avec recadrage, rotation, retournement, redimensionnement et ajustements de base.
- **Garder les dossiers synchronisés** avec des opérations sensibles au système de fichiers et un support de rafraîchissement.
- **Utiliser les outils de recherche locaux** tels que la recherche texte/image, la recherche d'images similaires, le regroupement de visages et les tags intelligents.
- **Ouvrir les formats d'image modernes** y compris WebP, HEIC/HEIF, AVIF et JXL (JPEG XL).
- **Consulter les photos RAW** avec un décodage intégré pour plus de 20 formats RAW d'appareils photo (CR2, NEF, ARW, DNG, etc.).
- **Large compatibilité vidéo** : prise en charge des formats MP4, MOV, AVI, MKV et de plus de 20 autres formats avec optimisation multiplateforme.

## Fonctionnalités prévues

- **Support des Live Photos et Motion Photos** pour les flux de travail mixtes photo/vidéo.
- **Étendre le support des métadonnées** pour les flux de travail EXIF, XMP et IPTC.

## Compiler à partir des sources

Configuration requise : Node.js 20+, pnpm, Rust stable.

Pour la lecture vidéo sous Linux, voir la note sur les paquets dans la section téléchargement ci-dessus.

```bash
# Dépendances système macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dépendances système Linux (Ubuntu/Debian)
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

## Formats supportés

| Type | Formats |
| :--- | :--- |
| Images | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| Photos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vidéos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX et plus. La lecture H.264 est supportée sur toutes les plateformes, avec un traitement automatique de compatibilité lorsque la lecture native n'est pas disponible. HEVC/H.265 et VP9 sont supportés nativement sur macOS. |

## Architecture

- Cœur : Tauri + Rust
- Frontend : Vue + Vite + Tailwind CSS
- Données : SQLite

### Bibliothèques clés

| Bibliothèque | Usage |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Décodage d'images RAW et extraction de miniatures |
| [FFmpeg](https://ffmpeg.org/) | Traitement vidéo et génération de miniatures |
| [ONNX Runtime](https://onnxruntime.ai/) | Moteur d'inférence de modèles d'IA local |
| [CLIP](https://github.com/openai/CLIP) | Recherche de similitude image-texte |
| [InsightFace](https://github.com/deepinsight/insightface) | Détection et reconnaissance faciale |
| [Leaflet](https://leafletjs.com/) | Carte interactive pour les photos géotaguées |
| [daisyUI](https://daisyui.com/) | Bibliothèque de composants UI |

## Licence

GPL-3.0-or-later. Voir [LICENSE](../../LICENSE).
