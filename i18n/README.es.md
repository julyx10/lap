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

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | Español | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap es un gestor de fotos de código abierto y local-first, diseñado para explorar álbumes familiares, encontrar fotos antiguas rápidamente y gestionar grandes bibliotecas multimedia personales sin conexión.
Es una alternativa centrada en la privacidad frente a los servicios de fotos en la nube: sin cargas forzadas, con búsqueda local mediante IA, un flujo de trabajo basado en carpetas y de uso gratuito.

## Descargar Lap

Abra la [página de las últimas versiones](https://github.com/julyx10/lap/releases/latest) y descargue el archivo que corresponda a su sistema:

| Plataforma | Paquete | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizado por Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Sin firmar — si SmartScreen bloquea la descarga, haga clic en **Conservar de todos modos** |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Para distribuciones basadas en Debian (Ubuntu, Debian, Linux Mint, etc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Marque el archivo como ejecutable y haga doble clic para iniciarlo |

### macOS con Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Capturas de pantalla

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gestor local de biblioteca de fotos Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Vista de mapa de Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Collage de fotos de Lap" width="900">
</p>

## Por qué elegir Lap

- **Diseñado local-first**: sus fotos permanecen en su propio disco, sin cuenta en la nube ni subida obligatoria.
- **Sin bloqueo de biblioteca**: trabaje directamente con sus carpetas existentes en lugar de importar todo a una base de datos cerrada.
- **Herramientas de IA privadas**: búsqueda, similitud, etiquetas inteligentes y funciones de rostros se ejecutan localmente en su equipo.
- **Creado para grandes colecciones**: optimizado para explorar y organizar bibliotecas con más de 100 000 archivos.
- **Código abierto y gratuito**: sin suscripción, sin ecosistema obligatorio y con código que puede inspeccionar.

## Características

- **Exploración flexible** por fecha, carpeta, ubicación, cámara, objetivo, etiquetas, valoraciones y rostros, con orden y búsqueda.
- **Flujo basado en carpetas** con varias bibliotecas, importación y exportación mediante arrastrar y soltar o copiar y pegar, sincronización y operaciones de archivos seguras.
- **Búsqueda con IA local** mediante texto, similitud visual, temas, agrupación de rostros y búsqueda opcional en más de 50 idiomas.
- **Álbumes inteligentes** para guardar vistas basadas en reglas con agrupación y orden personalizados.
- **Importación organizada por fecha** en carpetas diarias, mensuales, anuales o una sola carpeta, conservando los nombres originales y omitiendo duplicados.
- **Colecciones y etiquetas** para organizar archivos seleccionados en bloque sin mover ni duplicar los originales.
- **Limpieza de duplicados** con resumen del espacio recuperable y eliminación en bloque entre grupos.
- **Herramientas de selección y comparación** con un visor de imágenes de cuatro paneles.
- **Edición integrada** para recortar, enderezar en cualquier ángulo, girar, voltear, redimensionar y realizar ajustes básicos.
- **Collage de fotos** para componer collages en cuadrícula, mosaico justificado o pila de fotos con las fotos seleccionadas.
- **Apple Live Photos y Google Motion Photos** con reproducción del movimiento y un filtro común en los álbumes inteligentes.
- **Pares RAW + JPEG/HEIC** mostrados como un solo elemento, manteniendo juntos los archivos vinculados durante las operaciones.
- **Miniaturas y vistas previas RAW configurables** mediante revelado RAW o la vista previa integrada de la cámara, con brillo ajustable y visualización de pares RAW+JPEG.
- **Vista de mapa interactiva** para explorar fotos y vídeos geolocalizados en grupos que respetan los filtros actuales.
- **Exploración de álbumes sin conexión** que mantiene visibles los álbumes en almacenamiento desconectado o de red con miniaturas en caché.
- **Amplia compatibilidad** con más de 60 formatos de fotos, RAW y vídeo.

## Metadatos, colecciones y movimiento de archivos

Lap está centrado en las carpetas, pero no toda la información que muestra se integra en el archivo original. Esta distinción es importante si también administra las mismas carpetas en Finder, Explorador u otra aplicación de fotos.

### Lo que permanece con el archivo

- Sus fotos y vídeos originales siempre siguen siendo archivos normales en sus carpetas existentes.
- Los metadatos ya integrados en un archivo, como la fecha EXIF, la cámara, el objetivo, el GPS y la orientación, se leen de ese archivo cuando Lap lo indexa.
- Al guardar una edición de imagen integrada se escribe la imagen resultante en el destino elegido.
- Cuando renombra, mueve, copia o elimina archivos **en Lap**, Lap actualiza al mismo tiempo su catálogo local. También mantiene juntos los recursos agrupados compatibles, como los componentes de Apple Live Photo, los archivos auxiliares AAE y los pares RAW + JPEG/HEIC activados.

### Lo que Lap almacena localmente

La siguiente información son datos de biblioteca de Lap. Se almacenan en la base de datos local o en la configuración de biblioteca de Lap, y no se escriben en EXIF, IPTC ni en archivos auxiliares XMP:

- Colecciones, etiquetas, comentarios, favoritos, valoraciones y estados de selección (Seleccionadas y Rechazadas)
- Los álbumes inteligentes y sus reglas, agrupación, ordenación y orden
- Los datos de búsqueda con IA, datos de rostros, miniaturas y otros datos de índice o caché

Estos datos no viajan con un archivo cuando se copia, exporta o mueve fuera de Lap, ni están disponibles automáticamente para otras aplicaciones.

### Trabajar con archivos fuera de Lap

Lap puede volver a escanear carpetas y detectar muchos cambios del sistema de archivos. Sin embargo, los cambios realizados fuera de Lap —como renombrar, mover, sustituir o copiar archivos— pueden afectar a la organización que solo se guarda en Lap.

Para obtener los resultados más fiables, renombre y mueva archivos en Lap cuando use colecciones, etiquetas, comentarios, favoritos, valoraciones o estados de selección. Si también administra archivos fuera de Lap, haga una copia de seguridad de la base de datos y la configuración de Lap junto con sus fotos. Puede gestionar la ubicación de la base de datos y crear una copia de seguridad en **Ajustes → Almacenamiento**.

Eliminar la base de datos o la configuración de Lap elimina esta organización local y los datos de índice, pero no elimina sus archivos multimedia originales.

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

Para instalaciones DEB, desinstale el paquete:

```bash
sudo apt remove lap
```

Para instalaciones AppImage, cierre Lap y elimine el archivo `.AppImage` descargado.

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

Los paquetes de distribución pueden enlazar con el libheif del sistema (1.17 o posterior) en lugar del libheif y libde265 incluidos. Establece `LAP_SYSTEM_LIBHEIF=1` para la compilación. Entonces no se necesitan los submódulos `third_party/libheif` y `third_party/libde265`. La decodificación HEVC depende de los complementos de códec del libheif del sistema.

## Formatos compatibles

Lap admite más de 60 formatos de foto, RAW y vídeo.

| Tipo | Formatos |
| :--- | :--- |
| Imágenes | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX y más. La reproducción H.264 es compatible en todas las plataformas, con procesamiento automático de compatibilidad cuando la reproducción nativa no está disponible. HEVC/H.265 y VP9 son compatibles de forma nativa en macOS. |

### Reproducción de vídeo en Linux

Lap utiliza los complementos GStreamer del sistema para reproducir vídeos, también en el AppImage. Si los vídeos no se reproducen en Ubuntu, Debian o Linux Mint, instale:

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

## Privacidad

Consulte la [Política de privacidad](../PRIVACY.md) para conocer el tratamiento de datos y los servicios en línea opcionales.
