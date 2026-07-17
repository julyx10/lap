<div align="center">
  <img src="../docs/public/icon.png" alt="Logo de Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Gestor de fotos privadas locales</h1>
  <h3>Gestor de fotos de escritorio de código abierto para macOS, Windows y Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="Lanzamiento en GitHub"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="Descargas en GitHub"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="Estrellas en GitHub"></a>
  </p>
</div>

[English](../README.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | Español | [Português](README.pt.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap es un gestor de fotos de código abierto y local-first, diseñado para explorar álbumes familiares, encontrar fotos antiguas rápidamente y gestionar grandes bibliotecas multimedia personales sin conexión.
Es una alternativa centrada en la privacidad frente a los servicios de fotos en la nube: sin cargas forzadas, con búsqueda local mediante IA, un flujo de trabajo basado en carpetas y de uso gratuito.

- Sitio web: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Vídeo de demostración: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacidad: [PRIVACY.md](../PRIVACY.md)

## Descargar Lap

Abra la [página de las últimas versiones](https://github.com/julyx10/lap/releases/latest) y descargue el archivo que corresponda a su sistema:

| Plataforma | Paquete | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizado por Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Sin firmar — si SmartScreen bloquea la descarga, haga clic en **Conservar de todos modos** |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | Para distribuciones basadas en Debian (Ubuntu, Debian, Linux Mint, etc.) |

### macOS con Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Capturas de pantalla

<p align="center">
  <img src="../docs/public/screenshots/Lap_0.3.0_main_1.png" alt="Captura de pantalla del gestor de biblioteca de fotos local Lap" width="900">
</p>

## Por qué elegir Lap

- **Diseñado local-first**: sus fotos permanecen en su propio disco, sin cuenta en la nube ni subida obligatoria.
- **Sin bloqueo de biblioteca**: trabaje directamente con sus carpetas existentes en lugar de importar todo a una base de datos cerrada.
- **Herramientas de IA privadas**: búsqueda, similitud, etiquetas inteligentes y funciones de rostros se ejecutan localmente en su equipo.
- **Creado para grandes colecciones**: optimizado para explorar y organizar bibliotecas con más de 100 000 archivos.
- **Código abierto y gratuito**: sin suscripción, sin ecosistema obligatorio y con código que puede inspeccionar.

## Características

- **Exploración flexible de bibliotecas** con filtros por línea de tiempo, carpeta, ubicación, cámara, lente, etiqueta, favorito, valoración, tema y rostro.
- **Álbumes inteligentes** para guardar vistas basadas en reglas con agrupación, ordenación y orden personalizados.
- **Bandeja de colecciones** para mantener colecciones de archivos puntuales sin moverlos de sus carpetas originales.
- **Búsqueda local con IA** para indicaciones de texto, similitud visual, temas, agrupación de rostros y búsqueda multilingüe opcional en más de 50 idiomas.
- **Live Photos de Apple** reconoce pares HEIC/MOV, los reproduce en el visor y conserva juntos los archivos auxiliares MOV y AAE al renombrar, mover, copiar y eliminar.
- **Flujo de trabajo basado en carpetas** con múltiples bibliotecas, importación por arrastrar y soltar, importación por copiar y pegar, sincronización del sistema de archivos y operaciones seguras de mover/copiar/eliminar.
- **Herramientas de revisión y comparación**, incluido un visor de comparación de imágenes en cuatro paneles.
- **Herramientas de limpieza** para encontrar duplicados y mover archivos no deseados a la papelera por lotes.
- **Edición integrada** para recortar, rotar, voltear, redimensionar y realizar ajustes básicos de imagen.
- **Amplio soporte de formatos** para más de 60 formatos de foto, RAW y vídeo.

## Desinstalar Lap

Lap trabaja directamente con sus carpetas de fotos existentes. Desinstalar Lap o eliminar sus archivos de base de datos y caché **no** elimina sus fotos originales.

La desinstalación estándar elimina la aplicación. Para eliminar Lap por completo, cierre Lap primero, desinstale la aplicación y después elimine la base de datos local, la caché de miniaturas y los archivos de configuración con los comandos correspondientes a su plataforma.

### macOS

Si instaló Lap con Homebrew:

```bash
brew uninstall --cask lap
```

Para una instalación manual, cierre Lap y mueva `Lap.app` de la carpeta `Applications` a la Papelera.

Para eliminar todos los archivos de base de datos, caché y configuración de Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Abra **Configuración > Aplicaciones > Aplicaciones instaladas**, busque **Lap** y seleccione **Desinstalar**.

Después abra PowerShell y elimine todos los archivos de base de datos, caché y configuración de Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

En distribuciones basadas en Debian, desinstale el paquete:

```bash
sudo apt remove lap
```

Después elimine todos los archivos de base de datos, caché y configuración de Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Si seleccionó un directorio personalizado para la base de datos en la configuración de Lap, elimínelo por separado después de confirmar que contiene únicamente archivos de base de datos de Lap.

## Compilar desde el código fuente

Requisitos: Node.js 20+, pnpm, Rust estable.

```bash
# Dependencias del sistema macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dependencias del sistema Linux
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

Lap admite más de 60 formatos de foto, RAW y vídeo.

| Tipo | Formatos |
| :--- | :--- |
| Imágenes | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX y más. La reproducción H.264 es compatible en todas las plataformas, con procesamiento automático de compatibilidad cuando la reproducción nativa no está disponible. HEVC/H.265 y VP9 son compatibles de forma nativa en macOS. |

### Notas sobre la reproducción de vídeo en Linux

En Linux Mint/Ubuntu/Debian, instale estos paquetes para obtener un mejor soporte en la reproducción de vídeo:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Arquitectura

- Núcleo: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Datos: SQLite

### Bibliotecas clave

| Biblioteca | Propósito |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Decodificación de imágenes RAW y extracción de miniaturas |
| [libheif](https://github.com/strukturag/libheif) | Decodificación de imágenes HEIC/HEIF/HIF y generación de vistas previas |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Decodificación rápida de JPEG y generación de miniaturas |
| [FFmpeg](https://ffmpeg.org/) | Procesamiento de vídeo y generación de miniaturas |
| [Video.js](https://videojs.com/) | Interfaz de reproducción de vídeo multiplataforma |
| [ONNX Runtime](https://onnxruntime.ai/) | Motor de inferencia de modelos de IA local |
| [CLIP](https://github.com/openai/CLIP) | Búsqueda de similitud entre imagen y texto |
| [InsightFace](https://github.com/deepinsight/insightface) | Detección y reconocimiento facial |
| [Leaflet](https://leafletjs.com/) | Mapa interactivo para fotos geolocalizadas |
| [daisyUI](https://daisyui.com/) | Biblioteca de componentes de interfaz de usuario |

## Licencia

GPL-3.0-o-posterior. Consulte [LICENSE](../LICENSE).
