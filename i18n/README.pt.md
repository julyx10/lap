<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Gerenciador de fotos privadas locais</h1>
  <h3>Gerenciador de fotos de desktop de código aberto para macOS, Windows e Linux.</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | Português | [Русский](README.ru.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap é um gerenciador de fotos de código aberto e local-first, projetado para navegar em álbuns de família, encontrar fotos antigas rapidamente e gerenciar grandes bibliotecas de mídia pessoal offline.
É uma alternativa focada na privacidade aos serviços de fotos na nuvem: sem upload forçado, busca por IA local, fluxo de trabalho centrado em pastas e gratuito para usar.

- Site: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- Vídeo de demonstração: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- Privacidade: [PRIVACY.md](../PRIVACY.md)

## Baixar Lap

Abra a [página de lançamentos recentes](https://github.com/julyx10/lap/releases/latest) e baixe o arquivo que corresponde ao seu sistema:

| Plataforma | Pacote | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizado pela Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Não assinado — se o SmartScreen bloquear o download, clique em **Manter mesmo assim** |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | Para distribuições baseadas em Debian (Ubuntu, Debian, Linux Mint, etc.) |

## Capturas de tela

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Captura de tela do gerenciador de biblioteca de fotos local Lap" width="900">
</p>

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Captura de tela da busca de fotos por IA local do Lap" width="900">
</p>

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
- **Buscar em mais de 50 idiomas** com modelos multilíngues opcionais, disponíveis como download adicional quando necessário.
- **Abrir formatos de imagem modernos** incluindo WebP, HEIC/HEIF/HIF, AVIF e JXL (JPEG XL).
- **Visualizar fotos RAW** com decodificação integrada para mais de 20 formatos RAW de câmeras (CR2, NEF, ARW, DNG, etc.).
- **Ampla compatibilidade de vídeo**: suporte para MP4, MOV, AVI, MKV e mais de 20 outros formatos com otimização multiplataforma.

## Desinstalar Lap

O Lap trabalha diretamente com suas pastas de fotos existentes. Desinstalar o Lap ou excluir seus arquivos de banco de dados e cache **não** exclui suas fotos originais.

A desinstalação padrão remove o aplicativo. Para remover completamente o Lap, feche o Lap primeiro, desinstale o aplicativo e depois exclua o banco de dados local, o cache de miniaturas e os arquivos de configuração usando os comandos da sua plataforma.

### macOS

Se você instalou o Lap com o Homebrew:

```bash
brew uninstall --cask lap
```

Para uma instalação manual, feche o Lap e mova `Lap.app` da pasta `Applications` para a Lixeira.

Para remover todos os arquivos de banco de dados, cache e configuração do Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Abra **Configurações > Aplicativos > Aplicativos instalados**, encontre **Lap** e selecione **Desinstalar**.

Depois abra o PowerShell e remova todos os arquivos de banco de dados, cache e configuração do Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Em distribuições baseadas em Debian, desinstale o pacote:

```bash
sudo apt remove lap
```

Depois remova todos os arquivos de banco de dados, cache e configuração do Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Se você selecionou um diretório personalizado para o banco de dados nas configurações do Lap, exclua esse diretório separadamente após confirmar que ele contém apenas arquivos de banco de dados do Lap.

## Compilar a partir do código fonte

Requisitos: Node.js 20+, pnpm, Rust estável.

```bash
# Dependências do sistema macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Dependências do sistema Linux
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
| Imagens | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX e mais. A reprodução H.264 é suportada em todas as plataformas, com processamento de compatibilidade automático quando a reprodução nativa não estiver disponível. HEVC/H.265 e VP9 são suportados nativamente no macOS. |

### Notas sobre reprodução de vídeo no Linux

No Linux Mint/Ubuntu/Debian, instale estes pacotes para melhor suporte à reprodução de vídeo:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Arquitetura

- Core: Tauri + Rust
- Frontend: Vue + Vite + Tailwind CSS
- Dados: SQLite

### Bibliotecas Principais

| Biblioteca | Finalidade |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Decodificação de imagem RAW e extração de miniaturas |
| [libheif](https://github.com/strukturag/libheif) | Decodificação de imagem HEIC/HEIF/HIF e geração de pré-visualização |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Decodificação rápida de JPEG e geração de miniaturas |
| [FFmpeg](https://ffmpeg.org/) | Processamento de vídeo e geração de miniaturas |
| [Video.js](https://videojs.com/) | Interface de reprodução de vídeo multiplataforma |
| [ONNX Runtime](https://onnxruntime.ai/) | Mecanismo de inferência de modelo de IA local |
| [CLIP](https://github.com/openai/CLIP) | Busca de similaridade imagem-texto |
| [InsightFace](https://github.com/deepinsight/insightface) | Detecção e reconhecimento facial |
| [Leaflet](https://leafletjs.com/) | Mapa interativo para fotos com geo-tags |
| [daisyUI](https://daisyui.com/) | Biblioteca de componentes de interface do usuário |

## Licença

GPL-3.0-ou-posterior. Veja [LICENSE](../LICENSE).
