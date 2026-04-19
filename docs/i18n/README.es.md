<div align="center">
  <img src="../../docs/public/icon.png" alt="Logo de Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Gestor de fotos privadas locales</h1>
  <h3>Gestor de fotos de escritorio de código abierto para macOS, Windows y Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="Lanzamiento en GitHub"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="Descargas en GitHub"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="Estrellas en GitHub"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="Licencia en GitHub"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | Español | [Português](README.pt.md) | [Русский](README.ru.md)

Lap es un gestor de fotos de código abierto y local-first, diseñado para explorar álbumes familiares, encontrar fotos antiguas rápidamente y gestionar grandes bibliotecas multimedia personales sin conexión.
Es una alternativa centrada en la privacidad frente a los servicios de fotos en la nube: sin cargas forzadas, con búsqueda local mediante IA, un flujo de trabajo basado en carpetas y de uso gratuito.

- Sitio web: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Vídeo de demostración: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacidad: [PRIVACY.md](../../PRIVACY.md)

## Descargar Lap

Abra la [página de las últimas versiones](https://github.com/julyx10/lap/releases/latest) y descargue el archivo que corresponda a su sistema:

| Plataforma | Paquete | Estado |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Notarizado por Apple |
| **macOS (Intel)** | `x64.dmg` | Notarizado por Apple |
| **Windows 10/11 (x64)** | `.msi` | Probado en Windows 11. Actualmente sin firmar (puede aparecer la advertencia de SmartScreen) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Probado en Linux Mint. Para un mejor soporte de reproducción de vídeo, consulte la nota de Linux a continuación. |

### Notas sobre la reproducción de vídeo en Linux

En Ubuntu/Debian/Linux Mint, instale estos paquetes para obtener un mejor soporte en la reproducción de vídeo:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Capturas de pantalla

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Captura de pantalla del gestor de biblioteca de fotos local Lap" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Captura de pantalla de la búsqueda de fotos con IA local en Lap" width="900">
</p>

> Las imágenes de ejemplo de las capturas de pantalla proceden de [Wikimedia Commons](https://commons.wikimedia.org/).

## Por qué elegir Lap

- **Sin necesidad de la nube**: mantenga su biblioteca en su propio disco duro en lugar de subirla a un servicio externo.
- **Privacidad por defecto**: el procesamiento se realiza localmente, por lo que sus fotos permanecen bajo su control.
- **Uso gratuito**: sin planes de suscripción ni cuotas recurrentes.
- **Centrado en carpetas**: trabaje directamente con sus carpetas actuales, sin necesidad de pasos de importación.
- **Alto rendimiento para grandes bibliotecas**: optimizado para una navegación y organización fluida de grandes colecciones de medios (más de 100 000 archivos por biblioteca).

## Características

- **Explorar y filtrar** por fecha, ubicación, cámara, lente, etiquetas, favoritos, valoraciones y rostros (BETA).
- **Gestionar múltiples bibliotecas** y cambiar entre ellas rápidamente.
- **Encontrar duplicados** y mover copias no deseadas a la papelera por lotes.
- **Edición en el lugar** con recorte, rotación, volteo, cambio de tamaño y ajustes básicos.
- **Mantener carpetas sincronizadas** con operaciones conscientes del sistema de archivos y soporte de actualización.
- **Herramientas de búsqueda local** como búsqueda de texto/imagen, búsqueda de imágenes similares, agrupación de rostros y etiquetas inteligentes.
- **Abrir formatos de imagen modernos** incluyendo WebP, HEIC/HEIF, AVIF y JXL (JPEG XL).
- **Ver fotos RAW** con decodificación integrada para más de 20 formatos RAW de cámaras (CR2, NEF, ARW, DNG, etc.).
- **Amplia compatibilidad de video**: soporte para MP4, MOV, AVI, MKV y más de 20 otros formatos con optimización multiplataforma.

## Características planeadas

- **Soporte para Live Photos y Motion Photos** para flujos de trabajo mixtos de foto y vídeo.
- **Ampliar el soporte de metadatos** para flujos de trabajo EXIF, XMP e IPTC.

## Compilar desde el código fuente

Requisitos: Node.js 20+, pnpm, Rust estable.

Para la reproducción de vídeo en Linux, consulte la nota sobre paquetes en la sección de descargas anterior.

```bash
# Dependencias del sistema macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dependencias del sistema Linux (Ubuntu/Debian)
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Clonar y compilar
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Formatos compatibles

| Tipo | Formatos |
| :--- | :--- |
| Imágenes | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX y más. La reproducción H.264 es compatible en todas las plataformas, con procesamiento automático de compatibilidad cuando la reproducción nativa no está disponible. HEVC/H.265 y VP9 son compatibles de forma nativa en macOS. |

## Arquitectura

- Núcleo: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Datos: SQLite

### Bibliotecas clave

| Biblioteca | Propósito |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Decodificación de imágenes RAW y extracción de miniaturas |
| [FFmpeg](https://ffmpeg.org/) | Procesamiento de vídeo y generación de miniaturas |
| [ONNX Runtime](https://onnxruntime.ai/) | Motor de inferencia de modelos de IA local |
| [CLIP](https://github.com/openai/CLIP) | Búsqueda de similitud entre imagen y texto |
| [InsightFace](https://github.com/deepinsight/insightface) | Detección y reconocimiento facial |
| [Leaflet](https://leafletjs.com/) | Mapa interactivo para fotos geolocalizadas |
| [daisyUI](https://daisyui.com/) | Biblioteca de componentes de interfaz de usuario |

## Licencia

GPL-3.0-o-posterior. Consulte [LICENSE](../../LICENSE).
