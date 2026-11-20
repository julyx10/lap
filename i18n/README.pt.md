<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - Gerenciador de fotos privadas locais</h1>
  <h3>Gerenciador de fotos de desktop de código aberto para macOS, Windows e Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | Português | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap é um gestor de fotos gratuito, de código aberto e com prioridade ao armazenamento local para macOS, Windows e Linux. Organize pastas existentes, pesquise com IA local e gerencie grandes bibliotecas sem conta na nuvem nem assinatura.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">SITE OFICIAL</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Baixar Lap

Abra a [página de lançamentos recentes](https://github.com/julyx10/lap/releases/latest) e baixe o arquivo que corresponde ao seu sistema:

| Plataforma | Pacote | Nota |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Notarizado pela Apple |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Sem assinatura. Selecione **Manter mesmo assim** no navegador ou **Mais informações** → **Executar assim mesmo** no SmartScreen. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Para distribuições baseadas em Debian (Ubuntu, Debian, Linux Mint, etc.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Torne o arquivo executável e clique duas vezes para iniciar |

### macOS com Homebrew

```bash
brew install --cask lap
```

Instalou pelo tap `julyx10/lap`? Consulte as [instruções de migração](https://github.com/julyx10/lap/discussions/567).

## Capturas de tela

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Gerenciador local de biblioteca de fotos Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Visualização de mapa do Lap" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Prévia de uma foto no Lap" width="900">
</p>

## Por que Lap

- **IA privada no dispositivo.** Descreva uma foto, encontre imagens parecidas ou navegue por rosto. Tudo é processado no seu dispositivo.
- **Suas pastas, sob seu controle.** Abra suas pastas diretamente no Lap. Os arquivos mantêm o local e o formato original.
- **Metadados que funcionam entre aplicativos.** O Lap lê EXIF, IPTC e XMP incorporados e sidecars XMP existentes (avaliações, etiquetas, favoritos, estados de seleção, GPS e datas). Há suporte para salvar os metadatos modificados em sidecars XMP, mantendo os arquivos originais intactos.
- **Seleção de fotos.** Prévia com a barra de espaço, avanço após avaliar, áreas de foco, comparação em quatro painéis e regras para manter duplicatas.
- **Para bibliotecas com mais de 100.000 fotos e vídeos.** Um núcleo nativo em Rust cuida da navegação, busca e varredura.
- **Gratuito e de código aberto.** Sem assinatura ou conta, em 17 idiomas no macOS, Windows e Linux.

## Recursos

- **Navegação flexível** por data, pasta, local, câmera, lente, etiquetas, avaliações e rostos, com ordenação e busca.
- **Fluxo baseado em pastas.** Várias bibliotecas, importação e exportação por arrastar ou pela área de transferência, renomeação em lote e sincronização, movimentação, cópia e exclusão seguras.
- **Busca com IA local** por texto, similaridade visual, temas e agrupamento de rostos, com busca opcional em mais de 50 idiomas.
- **Importação de metadados e gravação em XMP**: o Lap lê EXIF, IPTC e XMP incorporados e sidecars XMP existentes (avaliações, etiquetas, favoritos, estados de seleção, GPS e datas), permite editar campos de texto e gravar sua organização explicitamente em sidecars XMP, mantendo os arquivos originais intactos.
- **Álbuns inteligentes.** Agrupamento e ordenação personalizados, filtros por nome de pasta e seletores com busca.
- **Importação organizada por data** em pastas por dia, mês, ano ou uma única pasta, preservando os nomes originais e ignorando duplicatas.
- **Coleções e tags** para organizar arquivos selecionados em lote sem mover ou duplicar os originais. Árvore de etiquetas com arraste e contagem por ramo.
- **Limpeza de duplicatas.** Regras por pasta, formato, tamanho e data, espaço recuperável e exclusão em lote.
- **Ferramentas de seleção e comparação.** Prévia com a barra de espaço, avanço após avaliar, áreas de foco, comparação em quatro painéis e regras para manter duplicatas.
- **Edição integrada** para cortar, endireitar em qualquer ângulo, girar, inverter, redimensionar e fazer ajustes básicos.
- **Colagem de fotos** para compor colagens em grade, mosaico justificado ou pilha de fotos com as fotos selecionadas.
- **Apple Live Photos e Google Motion Photos** com reprodução do movimento e um filtro comum nos álbuns inteligentes.
- **Pares RAW + JPEG/HEIC** exibidos como um único item, mantendo os arquivos associados juntos durante as operações. Você pode excluir um ou ambos os arquivos de um par RAW. Se mantiver um, o sidecar XMP compartilhado permanece com o arquivo restante.
- **Miniaturas e prévias RAW configuráveis** usando renderização RAW ou a prévia incorporada da câmera, com brilho ajustável e exibição de pares RAW+JPEG.
- **Informações de arquivo personalizáveis.** Campos selecionáveis, detalhes sob demanda e áreas AF no visualizador.
- **Mapa interativo** para explorar fotos e vídeos geolocalizados em grupos que seguem os filtros atuais.
- **Navegação offline de álbuns** que mantém os álbuns em armazenamento desconectado ou de rede visíveis com miniaturas em cache.
- **Arquivos disponíveis apenas na nuvem.** Arquivos sob demanda do OneDrive e arquivos sem dados locais do macOS são indexados e exibidos sem download automático.
- **Temas claro e escuro.** Podem, opcionalmente, seguir a aparência do sistema.
- **Amplo suporte a formatos** para mais de 60 formatos de imagens, RAW e vídeo. Documentos Affinity (prévia incorporada).

## Metadados e controle dos dados

O Lap trabalha diretamente com suas pastas e lê dos arquivos a maioria das informações exibidas. Você pode gravar sua organização em sidecars XMP padrão, para que ela acompanhe os arquivos e permaneça legível por outros aplicativos de fotos.

O Lap lê metadados compatíveis do Lightroom, digiKam, darktable e ACDSee. Os campos compatíveis variam conforme o aplicativo.

### Ler metadados existentes

- Suas fotos e vídeos originais permanecem sempre como arquivos comuns em suas pastas existentes.
- Ao indexar os arquivos, o Lap lê EXIF, IPTC e XMP incorporados: data de captura, câmera, lente, GPS, orientação, avaliações, favoritos, palavras-chave, estados de seleção, descrições, título, manchete, autor e direitos autorais. Ative **Configurações → Metadados → Importar metadados** para importar os campos de organização para o banco local. As edições locais são preservadas.
- Os valores compatíveis em sidecars XMP têm prioridade sobre os incorporados, incluindo GPS e data de captura.

### Salvar alterações compatíveis em XMP

- Título, manchete, descrição, autor e direitos autorais podem ser editados e ficam no banco de dados local.
- **Salvar metadados em XMP** grava em sidecars o título, a manchete, a descrição, o autor e os direitos autorais, além de etiquetas (incluindo palavras-chave hierárquicas), avaliações, favoritos e estados de seleção. Ele preserva outras propriedades XMP e mantém os arquivos originais intactos.
- Quando você renomeia, move, copia, exclui ou importa arquivos **no Lap**, ele atualiza simultaneamente seu banco de dados local. Ele também mantém juntos os recursos agrupados compatíveis, como componentes de Apple Live Photo, sidecars XMP e AAE e pares RAW + JPEG/HEIC ativados. Você pode excluir um ou ambos os arquivos de um par RAW. Se mantiver um, o sidecar XMP compartilhado permanece com o arquivo restante.
- Salvar uma edição de imagem integrada grava a imagem resultante no destino escolhido.

### O que permanece no banco de dados do Lap

O Lap armazena esses dados no banco local ou na configuração da biblioteca.

- Coleções, álbuns inteligentes e suas regras, agrupamento, ordenação e ordem
- Dados de pesquisa por IA, dados faciais, miniaturas e outros dados de índice ou cache
- Etiquetas, avaliações, favoritos, estados de seleção e campos de texto editáveis são armazenados localmente e podem ser gravados em XMP.

As alterações ficam no banco até você salvar explicitamente os campos compatíveis em XMP. Coleções, álbuns inteligentes, dados de IA e dados faciais permanecem locais no Lap e não são incluídos em sidecars XMP. Faça backup do banco para preservar essa organização; copiar apenas os originais não a preserva.

### Trabalhar com arquivos fora do Lap

O Lap pode reexaminar pastas e detectar muitas alterações no sistema de arquivos. No entanto, alterações feitas fora do Lap — como renomear, mover, substituir ou copiar arquivos — podem afetar a organização armazenada apenas no Lap.

Com a importação de metadados ativada, o Lap também verifica alterações nos sidecars XMP. Renomeie e mova arquivos no Lap para preservar seus dados no banco local, incluindo a associação a coleções. Para compartilhar metadados compatíveis com outros aplicativos, use **Salvar metadados em XMP** e mantenha o sidecar junto do arquivo. Se também gerencia arquivos fora do Lap, mantenha um backup do banco de dados e da configuração do Lap junto com suas fotos. Você pode gerenciar o local do banco de dados e criar um backup em **Configurações → Armazenamento**.

Excluir o banco de dados ou a configuração do Lap remove essa organização local e os dados de índice, mas não exclui seus arquivos de mídia originais.

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

Para instalações DEB, desinstale o pacote:

```bash
sudo apt remove lap
```

Para desinstalar o pacote local do Arch, use `sudo pacman -R lap`.

Para instalações AppImage, feche o Lap e exclua o arquivo `.AppImage` baixado.

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

Os pacotes de distribuição podem vincular ao libheif do sistema (1.17 ou mais recente) em vez do libheif e libde265 incluídos. Defina `LAP_SYSTEM_LIBHEIF=1` para a compilação. Os submódulos `third_party/libheif` e `third_party/libde265` então não são necessários. A decodificação HEVC depende dos plugins de codec do libheif do sistema.

### Arch Linux (x86_64)

Uma receita experimental de build a partir do código-fonte para Arch está em [`contrib/aur`](../contrib/aur/README.md). Ela ainda não foi publicada no AUR, portanto `yay -S lap` ainda não está disponível. Para compilar e instalar localmente no Arch Linux:

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

A receita fixa um commit do código-fonte original e verifica com somas de controle os modelos de IA e as ferramentas FFmpeg incluídos. O build para Arch desativa as atualizações automáticas no aplicativo para que o pacman continue responsável pelas atualizações; os builds das outras plataformas mantêm o atualizador existente. Consulte o guia da receita para detalhes de compilação e verificações de lançamento pendentes.

## Formatos Suportados

O Lap suporta mais de 60 formatos de imagens, RAW e vídeo.

| Tipo | Formatos |
| :--- | :--- |
| Imagens | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Fotos RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Documentos Affinity | AF, AFPHOTO, AFDESIGN, AFPUB (prévia incorporada) |
| Vídeos | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. A reprodução H.264 é suportada em todas as plataformas, com processamento de compatibilidade automático quando a reprodução nativa não estiver disponível. HEVC/H.265 e VP9 são suportados nativamente no macOS. |

### Reprodução de vídeo no Linux

Se os vídeos não forem reproduzidos, instale os plugins GStreamer:

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Port do ExifTool para Rust |
| [Video.js](https://videojs.com/) | Interface de reprodução de vídeo multiplataforma |
| [ONNX Runtime](https://onnxruntime.ai/) | Mecanismo de inferência de modelo de IA local |
| [CLIP](https://github.com/openai/CLIP) | Busca de similaridade imagem-texto |
| [InsightFace](https://github.com/deepinsight/insightface) | Detecção e reconhecimento facial |
| [Leaflet](https://leafletjs.com/) | Mapa interativo para fotos com geo-tags |
| [daisyUI](https://daisyui.com/) | Biblioteca de componentes de interface do usuário |

## Licença

GPL-3.0-ou-posterior. Veja [LICENSE](../LICENSE).

## Privacidade

Consulte a [Política de Privacidade](../PRIVACY.md) para saber mais sobre o tratamento de dados e os serviços online opcionais.
