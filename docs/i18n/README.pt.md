<div align="center">
  <img src="../../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Gerenciador de fotos privadas locais</h1>
  <h3>Gerenciador de fotos de desktop de código aberto para macOS, Windows e Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
    <a href="https://github.com/julyx10/lap/blob/main/LICENSE"><img src="https://img.shields.io/github/license/julyx10/lap" alt="GitHub license"></a>
  </p>
</div>

[English](../../README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | Português | [Русский](README.ru.md)

Lap é um gerenciador de fotos de código aberto e local-first, projetado para navegar em álbuns de família, encontrar fotos antigas rapidamente e gerenciar grandes bibliotecas de mídia pessoal offline.
É uma alternativa focada na privacidade aos serviços de fotos na nuvem: sem upload forçado, busca por IA local, fluxo de trabalho centrado em pastas e gratuito para usar.

- Site: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Vídeo de demonstração: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacidade: [PRIVACY.md](../../PRIVACY.md)

## Baixar Lap

Abra a [página de lançamentos recentes](https://github.com/julyx10/lap/releases/latest) e baixe o arquivo que corresponde ao seu sistema:

| Plataforma | Pacote | Status |
| :-- | :-- | :-- |
| **macOS (Apple Silicon)** | `aarch64.dmg` | Notarizado pela Apple |
| **macOS (Intel)** | `x64.dmg` | Notarizado pela Apple |
| **Windows 10/11 (x64)** | `.msi` | Testado no Windows 11. Atualmente sem assinatura (um aviso do SmartScreen pode aparecer) |
| **Ubuntu/Debian (amd64)** | `amd64.deb` | Testado no Linux Mint. Para melhor suporte a reprodução de vídeo, veja a nota sobre Linux abaixo. |

### Notas sobre reprodução de vídeo no Linux

No Ubuntu/Debian/Linux Mint, instale estes pacotes para melhor suporte à reprodução de vídeo:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Capturas de tela

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Captura de tela do gerenciador de biblioteca de fotos local Lap" width="900">
</p>

<p align="center">
  <img src="../../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Captura de tela da busca de fotos por IA local do Lap" width="900">
</p>

> As imagens de exemplo nas capturas de tela vêm do [Wikimedia Commons](https://commons.wikimedia.org/).

## Por que Lap

- **Sem necessidade de nuvem**: mantenha sua biblioteca em seu próprio disco em vez de enviá-la para um serviço hospedado.
- **Privado por padrão**: o processamento ocorre localmente, portanto, suas fotos permanecem sob seu controle.
- **Gratuito para usar**: sem planos de assinatura ou taxas recorrentes.
- **Pastas primeiro**: trabalhe diretamente com suas pastas existentes, sem etapa de importação necessária.
- **Alto desempenho para grandes bibliotecas**: otimizado para navegação e organização fluidas de grandes coleções de mídia (mais de 100 mil arquivos por biblioteca).

## Recursos

- **Navegar e filtrar** por data, local, câmera, lente, tags, favoritos, classificações e rostos (BETA).
- **Gerenciar várias bibliotecas** e alternar entre elas rapidamente.
- **Encontrar duplicados** e mover cópias indesejadas para a lixeira em lote.
- **Editar no local** com cortar, girar, inverter, redimensionar e ajustes básicos.
- **Manter pastas sincronizadas** com operações sensíveis ao sistema de arquivos e suporte a atualização.
- **Usar ferramentas de busca local** como busca por texto/imagem, busca por imagens semelhantes, agrupamento de rostos e tags inteligentes.
- **Abrir formatos de imagem modernos** incluindo WebP, HEIC/HEIF, AVIF e JXL (JPEG XL).
- **Visualizar fotos RAW** com decodificação integrada para mais de 20 formatos RAW de câmeras (CR2, NEF, ARW, DNG, etc.).
- **Ampla compatibilidade de vídeo**: suporte para MP4, MOV, AVI, MKV e mais de 20 outros formatos com otimização multiplataforma.

## Recursos planejados

- **Suporte para Live Photos e Motion Photos** para flujos de trabalho mistos de foto/vídeo.
- **Expandir o suporte a metadados** para fluxos de trabalho EXIF, XMP e IPTC.

## Compilar a partir do código fonte

Requisitos: Node.js 20+, pnpm, Rust estável.

Para reprodução de vídeo no Linux, veja a nota sobre pacotes na seção de download acima.

```bash
# Dependências do sistema macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dependências do sistema Linux (Ubuntu/Debian)
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Clonar e compilar
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

## Formatos Suportados

| Tipo | Formatos |
| :--- | :--- |
| Imagens | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF, AVIF, JXL |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX e mais. A reprodução H.264 é suportada em todas as plataformas, com processamento de compatibilidade automático quando a reprodução nativa não estiver disponível. HEVC/H.265 e VP9 são suportados nativamente no macOS. |

## Arquitetura

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Dados: SQLite

### Bibliotecas Principais

| Biblioteca | Finalidade |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Decodificação de imagem RAW e extração de miniaturas |
| [FFmpeg](https://ffmpeg.org/) | Processamento de vídeo e geração de miniaturas |
| [ONNX Runtime](https://onnxruntime.ai/) | Mecanismo de inferência de modelo de IA local |
| [CLIP](https://github.com/openai/CLIP) | Busca de similaridade imagem-texto |
| [InsightFace](https://github.com/deepinsight/insightface) | Detecção e reconhecimento facial |
| [Leaflet](https://leafletjs.com/) | Mapa interativo para fotos com geo-tags |
| [daisyUI](https://daisyui.com/) | Biblioteca de componentes de interface do usuário |

## Licença

GPL-3.0-ou-posterior. Veja [LICENSE](../../LICENSE).
