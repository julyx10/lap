<div align="center">
  <img src="../docs/public/icon.png" alt="Logo de Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Gestor de fotos privadas locales</h1>
  <h3>Gestor de fotos de escritorio de código abierto para macOS, Windows y Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="Lanzamiento en GitHub"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="Descargas en GitHub"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="Estrellas en GitHub"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | Español | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap es un gestor de fotos gratuito, de código abierto y centrado en el almacenamiento local para macOS, Windows y Linux. Organice carpetas existentes, busque con IA local y gestione grandes bibliotecas sin cuenta en la nube ni suscripción.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">SITIO WEB OFICIAL</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

> [!IMPORTANT]
> Actualmente, el único sitio web oficial de Lap es [julyx10.github.io/lap](https://julyx10.github.io/lap/).
> `lapphotos.com` no es un sitio oficial, no tiene relación con el proyecto Lap y no está autorizado ni respaldado por el proyecto.

## Descargar Lap

Siempre que sea posible, descargue Lap directamente desde [GitHub Releases](https://github.com/julyx10/lap/releases), en lugar de sitios web de terceros.

| Plataforma | Paquete | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizado por Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Sin firma. Si el navegador bloquea la descarga, seleccione **Conservar de todos modos**. Si SmartScreen bloquea el instalador, seleccione **Más información** → **Ejecutar de todos modos**. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Para distribuciones basadas en Debian (Ubuntu, Debian, Linux Mint, etc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Marque el archivo como ejecutable y haga doble clic para iniciarlo |

### macOS con Homebrew

```bash
brew install --cask lap
```

¿Lo instaló mediante el tap `julyx10/lap`? Consulte las [instrucciones de migración](https://github.com/julyx10/lap/discussions/567).

## Capturas de pantalla

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gestor local de biblioteca de fotos Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Vista de mapa de Lap" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Vista previa de una foto en Lap" width="900">
</p>

## Por qué elegir Lap

- **IA local y privada.** Describa una foto, busque imágenes similares o explore por rostros. Todo se procesa en su equipo.
- **Sus carpetas, bajo su control.** Abra sus carpetas directamente en Lap. Los archivos conservan su ubicación y formato.
- **Metadatos compatibles entre aplicaciones.** Lee EXIF, IPTC y XMP integrados y archivos auxiliares (valoraciones, etiquetas, favoritos, estados de selección, GPS y fechas). Permite guardar los metadatos modificados en archivos XMP auxiliares, sin alterar los archivos originales.
- **Selección fotográfica.** Vista previa con la barra espaciadora, avance tras puntuar, áreas de enfoque, comparación en cuatro paneles y reglas de conservación de duplicados.
- **Para bibliotecas de más de 100.000 fotos y vídeos.** Un núcleo nativo en Rust se encarga de la navegación, la búsqueda y el escaneo.
- **Gratis y de código abierto.** Sin suscripción ni cuenta, en 17 idiomas para macOS, Windows y Linux.

## Características

- **Exploración flexible de la biblioteca** por fecha, carpeta, ubicación, cámara, objetivo, etiquetas, valoraciones y rostros, con orden y búsqueda.
- **Flujo basado en carpetas.** Varias bibliotecas, importación y exportación por arrastre o portapapeles, renombrado por lotes y sincronización, operaciones seguras de movimiento, copia y eliminación.
- **Búsqueda con IA local** mediante texto, similitud visual, temas, agrupación de rostros y búsqueda opcional en más de 50 idiomas.
- **Importación de metadatos y escritura en XMP** · Lee EXIF, IPTC y XMP integrados y archivos auxiliares (valoraciones, etiquetas, favoritos, estados de selección, GPS y fechas). Permite editar campos de texto y guardar los datos de organización en XMP.
- **Álbumes inteligentes.** Agrupación y orden personalizados, filtros por nombre de carpeta y selectores con búsqueda.
- **Importación organizada por fecha** en carpetas diarias, mensuales, anuales o una sola carpeta, conservando los nombres originales y omitiendo duplicados.
- **Colecciones y etiquetas** para organizar archivos seleccionados en bloque sin mover ni duplicar los originales. Árbol de etiquetas con arrastre y recuento por rama.
- **Limpieza de duplicados.** Reglas por carpeta, formato, tamaño y fecha, espacio recuperable y eliminación masiva.
- **Herramientas de selección y comparación.** Vista previa con la barra espaciadora, avance tras puntuar, áreas de enfoque, comparación en cuatro paneles y reglas de conservación de duplicados.
- **Edición integrada** para recortar, enderezar en cualquier ángulo, girar, voltear, redimensionar y realizar ajustes básicos.
- **Collage de fotos** para componer collages en cuadrícula, mosaico justificado o pila de fotos con las fotos seleccionadas.
- **Apple Live Photos y Google Motion Photos** con reproducción del movimiento y un filtro común en los álbumes inteligentes.
- **Pares RAW + JPEG/HEIC** mostrados como un solo elemento, manteniendo juntos los archivos vinculados durante las operaciones. Puede eliminar uno o ambos archivos de un par RAW. Si conserva uno, también se mantiene su archivo XMP compartido.
- **Miniaturas y vistas previas RAW configurables** mediante revelado RAW o la vista previa integrada de la cámara, con brillo ajustable y visualización de pares RAW+JPEG.
- **Información de archivo personalizable.** Campos elegibles, detalles bajo demanda y áreas de enfoque en el visor.
- **Vista de mapa interactiva** para explorar fotos y vídeos geolocalizados en grupos que respetan los filtros actuales.
- **Exploración de álbumes sin conexión** que mantiene visibles los álbumes en almacenamiento desconectado o de red con miniaturas en caché.
- **Archivos solo en la nube.** Los archivos a petición de OneDrive y los archivos sin datos locales de macOS se indexan y muestran sin descarga automática.
- **Temas claro y oscuro.** Pueden seguir la apariencia del sistema.
- **Amplia compatibilidad** con más de 60 formatos de imágenes, RAW y vídeo. Documentos de Affinity (vista previa incrustada).

## Metadatos y control de sus datos

Lap trabaja directamente con sus carpetas y lee de los archivos la mayoría de los datos que muestra. Puede guardar los datos de organización en archivos auxiliares XMP estándar y abrirlos en otras aplicaciones de fotografía.

Lap lee metadatos compatibles de Lightroom, digiKam, darktable y ACDSee. Los campos compatibles varían según la aplicación.

### Leer metadatos existentes

- Sus fotos y vídeos originales siempre siguen siendo archivos normales en sus carpetas existentes.
- Durante la indexación, Lap lee EXIF, IPTC y XMP integrados, con datos de captura y organización. Active **Ajustes → Metadatos → Importar metadatos** para importar los datos de organización a la base de datos local. Se conservan las ediciones locales.
- Los valores compatibles de los archivos auxiliares XMP tienen prioridad sobre los integrados, incluidos el GPS y la fecha de captura.

### Guardar los cambios compatibles en XMP

- El título, titular, descripción, autor y copyright se pueden editar y se guardan en la base de datos local.
- Guardar metadatos en XMP escribe estos campos, las etiquetas (incluidas las palabras clave jerárquicas), valoraciones, favoritos y estados de selección en archivos auxiliares, sin alterar las demás propiedades XMP ni los originales.
- Cuando renombra, mueve, copia, elimina o importa archivos **en Lap**, Lap actualiza al mismo tiempo su catálogo local. También mantiene juntos los recursos agrupados compatibles, como los componentes de Apple Live Photo, los archivos auxiliares XMP y AAE y los pares RAW + JPEG/HEIC activados. Puede eliminar uno o ambos archivos de un par RAW. Si conserva uno, también se mantiene su archivo XMP compartido.
- Al guardar una edición de imagen integrada se escribe la imagen resultante en el destino elegido.

### Qué permanece en la base de datos de Lap

Lap almacena estos datos en la base de datos local o en la configuración de la biblioteca.

- Las colecciones, los álbumes inteligentes, sus reglas, la agrupación, el orden y la disposición
- Los datos de búsqueda con IA, datos de rostros, miniaturas y otros datos de índice o caché
- Las etiquetas, valoraciones, favoritos, estados de selección y campos de texto editables se guardan localmente y pueden escribirse en XMP.

Los cambios permanecen en la base de datos hasta que guarde explícitamente los campos compatibles en XMP. Las colecciones, álbumes inteligentes y datos de IA y rostros no se incluyen en XMP. Haga una copia de seguridad de la base de datos; copiar solo los originales no conserva esta organización.

### Trabajar con archivos fuera de Lap

Lap puede volver a escanear carpetas y detectar muchos cambios del sistema de archivos. Con la importación de metadatos activada, Lap también comprueba los cambios en los archivos auxiliares XMP. Sin embargo, los cambios realizados fuera de Lap —como renombrar, mover, sustituir o copiar archivos— pueden afectar a la organización que solo se guarda en Lap.

Renombre y mueva los archivos en Lap para conservar sus colecciones. Mantenga los archivos auxiliares XMP junto a los originales. Si también administra archivos fuera de Lap, haga una copia de seguridad de la base de datos y la configuración de Lap junto con sus fotos. Puede gestionar la ubicación de la base de datos y crear una copia de seguridad en **Ajustes → Almacenamiento**.

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

Para desinstalar el paquete local de Arch, use `sudo pacman -R lap`.

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

Los paquetes de distribución pueden enlazar con el libheif del sistema (1.17 o posterior) en lugar del libheif y libde265 incluidos. Establezca `LAP_SYSTEM_LIBHEIF=1` para la compilación. Entonces no se necesitan los submódulos `third_party/libheif` y `third_party/libde265`. La decodificación HEVC depende de los complementos de códec del libheif del sistema.

### Arch Linux (x86_64)

Hay una receta experimental de compilación para Arch en [`contrib/aur`](../contrib/aur/README.md). Todavía no se ha publicado en el AUR, por lo que `yay -S lap` no está disponible. Para compilarla e instalarla localmente en Arch Linux:

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

La receta fija un commit del código fuente original y verifica mediante sumas de comprobación los modelos de IA y las herramientas FFmpeg incluidos. La compilación de Arch desactiva las autoactualizaciones de la aplicación para que pacman siga a cargo de las actualizaciones; las compilaciones de otras plataformas conservan su actualizador. Consulte la guía de la receta para más detalles de compilación y comprobaciones de publicación pendientes.

## Formatos compatibles

Lap admite más de 60 formatos de imágenes, RAW y vídeo.

| Tipo | Formatos |
| :--- | :--- |
| Imágenes | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Documentos de Affinity | AF, AFPHOTO, AFDESIGN, AFPUB (vista previa incrustada) |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. La reproducción H.264 es compatible en todas las plataformas, con procesamiento automático de compatibilidad cuando la reproducción nativa no está disponible. HEVC/H.265 y VP9 son compatibles de forma nativa en macOS. |

### Reproducción de vídeo en Linux

Si los vídeos no se reproducen, instale los complementos de GStreamer:

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Port de ExifTool a Rust |
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
