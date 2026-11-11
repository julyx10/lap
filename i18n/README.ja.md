<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - プライベート・ローカル写真管理ツール</h1>
  <h3>macOS、Windows、Linux向けのオープンソース・デスクトップ写真管理ツール。</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | 日本語 | [한국어](README.ko.md)

Lapは、オープンソースでローカルファーストな写真管理ツールです。家族のアルバムを閲覧したり、古い写真を素早く見つけたり、膨大な個人メディアライブラリをオフラインで管理したりするために設計されています。
クラウド写真サービスのプライバシーに配慮した代替案として、強制アップロードなし、ローカルAI検索、フォルダーファーストのワークフローを提供し、完全に無料で使用できます。

## Lapをダウンロード

[最新のリリースページ](https://github.com/julyx10/lap/releases/latest)を開き、お使いのシステムに合ったファイルをダウンロードしてください：

| プラットフォーム | パッケージ | 備考 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Appleによる公証済み |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未署名 — SmartScreenがダウンロードをブロックした場合は、**維持する**をクリックしてください |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Debian系ディストリビューション向け（Ubuntu、Debian、Linux Mintなど） |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | 実行権限を付与し、ダブルクリックで起動 |

### macOS with Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## スクリーンショット

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Lap のローカルフォトライブラリ管理" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Lap のマップビュー" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Lap の写真コラージュ" width="900">
</p>

## Lapを選ぶ理由

- **ローカルファースト設計**: 写真は自分のディスクに保存され、クラウドアカウントやアップロードは不要です。
- **ライブラリのロックインなし**: すべてを閉じたデータベースに取り込むのではなく、既存のフォルダーを直接扱えます。
- **プライベートなAIツール**: 検索、類似画像、スマートタグ、顔関連機能はすべてローカルで実行されます。
- **大規模コレクション向け**: 10万ファイル以上のライブラリでもスムーズに閲覧・整理できるよう最適化されています。
- **オープンソースで無料**: サブスクリプションや強制的なエコシステムはなく、コードを確認できます。

## 主な機能

- **柔軟なライブラリ閲覧**：日付、フォルダー、場所、カメラ、レンズ、タグ、評価、顔で絞り込み、並び替えや検索にも対応。
- **フォルダー中心のワークフロー**：複数ライブラリ、ドラッグ＆ドロップやコピー＆ペーストによる読み込み／書き出し、同期、安全なファイル操作に対応。
- **ローカル AI 検索**：テキスト、見た目の類似性、被写体、顔のグループ化に対応し、50 以上の言語での検索も選択可能。
- **スマートアルバム**：ルールに基づく表示を保存し、グループ分けや並び順を設定。
- **日付別の読み込み**：日・月・年別または単一フォルダーに整理し、元のファイル名を保持して重複をスキップ。
- **コレクションとタグ**：元ファイルを移動・複製せず、選択したファイルを一括整理。
- **重複整理**：解放できる容量を確認し、複数の重複グループを一括整理。
- **選別・比較ツール**：4 分割の画像比較ビューアーを搭載。
- **内蔵編集機能**：切り抜き、任意の角度での水平調整、回転、反転、サイズ変更、基本的な画像調整。
- **写真コラージュ**：選択した写真をグリッド、モザイク、写真の山のコラージュに構成。
- **Apple Live Photos と Google Motion Photos**：動きの再生と共通のスマートアルバムフィルターに対応。
- **RAW + JPEG/HEIC ペア**：1 つの項目として表示し、ファイル操作時は関連ファイルをまとめて処理。
- **RAW サムネイルとプレビューの選択**：RAW レンダリングまたはカメラの埋め込みプレビューを使用し、明るさの調整や RAW+JPEG ペアの表示にも対応。
- **インタラクティブな地図表示**：現在のフィルターに沿って、位置情報付きの写真や動画をグループで探索。
- **オフラインでのアルバム閲覧**：切断されたストレージやネットワークストレージ上のアルバムも、キャッシュ済みサムネイルで表示。
- **幅広い形式に対応**：60 以上の写真・RAW・動画形式をサポート。

## メタデータ、コレクション、ファイルの移動

Lapはフォルダーを中心に扱いますが、Lapに表示されるすべての情報が元のファイルに埋め込まれているわけではありません。同じフォルダーをFinder、エクスプローラー、または別の写真アプリでも管理する場合は、この違いが重要です。

### ファイルとともに残るもの

- 元の写真と動画は、既存のフォルダー内の通常のファイルのままです。
- EXIFの撮影日時、カメラ、レンズ、GPS、向きなど、ファイルにすでに埋め込まれているメタデータは、Lapがインデックスを作成する際にそのファイルから読み取られます。
- 内蔵の画像編集を保存すると、編集後の画像が選択した保存先に書き込まれます。
- ファイルを**Lap内で**名前変更、移動、コピー、削除すると、Lapは同時にローカルカタログを更新します。Apple Live Photoの構成ファイル、AAEサイドカーファイル、有効にしたRAW + JPEG/HEICペアなど、対応する関連ファイルもまとめて扱います。

### Lapがローカルに保存するもの

次の情報はLapのライブラリデータです。Lapのローカルデータベースまたはライブラリ設定に保存され、EXIF、IPTC、XMPサイドカーファイルには書き込まれません。

- コレクション、タグ、コメント、お気に入り、評価、選別状態（採用および除外）
- スマートアルバムと、そのルール、グループ化、並べ替え、順序
- AI検索データ、顔データ、サムネイル、その他のインデックスまたはキャッシュデータ

これらのデータは、Lapの外でファイルをコピー、書き出し、移動してもファイルとともには移動せず、他のアプリで自動的に利用できるものでもありません。

### Lapの外でファイルを扱う場合

Lapはフォルダーを再スキャンし、多くのファイルシステム上の変更を検出できます。ただし、Lapの外でファイルの名前変更、移動、置き換え、コピーを行うと、Lap内だけに保存されている整理情報に影響する場合があります。

コレクション、タグ、コメント、お気に入り、評価、選別状態を利用する場合は、最も確実な方法としてLapでファイルを名前変更・移動してください。Lapの外でもファイルを管理する場合は、写真とあわせてLapのデータベースと設定をバックアップしてください。データベースの場所の管理とバックアップの作成は、**設定 → ストレージ**から行えます。

Lapのデータベースまたは設定を削除すると、このローカルの整理情報とインデックスデータは削除されますが、元のメディアファイルは削除されません。

## Lapのアンインストール

Lapは既存の写真フォルダーを直接使用します。Lapをアンインストールしたり、データベースやキャッシュファイルを削除したりしても、元の写真は削除されません。

通常のアンインストールではアプリケーションのみが削除されます。Lapを完全に削除するには、まずLapを終了してアプリケーションをアンインストールし、その後、お使いのプラットフォームに対応するコマンドでローカルデータベース、サムネイルキャッシュ、設定ファイルを削除してください。

### macOS

HomebrewでLapをインストールした場合：

```bash
brew uninstall --cask lap
```

手動でインストールした場合は、Lapを終了し、`Applications`フォルダーの`Lap.app`をゴミ箱に移動してください。

Lapのデータベース、キャッシュ、設定ファイルをすべて削除するには：

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

**設定 > アプリ > インストールされているアプリ**を開き、**Lap**を探して**アンインストール**を選択します。

次にPowerShellを開き、Lapのデータベース、キャッシュ、設定ファイルをすべて削除します：

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

DEB でインストールした場合は、パッケージを削除します：

```bash
sudo apt remove lap
```

AppImage の場合は、Lap を終了し、ダウンロードした `.AppImage` ファイルを削除します。

次に、Lapのデータベース、キャッシュ、設定ファイルをすべて削除します：

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Lapの設定でカスタムデータベース保存先を選択している場合は、そのフォルダーにLapのデータベースファイルのみが含まれていることを確認してから、別途削除してください。

## ソースからのビルド

要件: Node.js 20+、pnpm、Rust stable.

```bash
# macOS システム依存関係
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Linux システム依存関係
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# クローンとビルド
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

配布パッケージでは、同梱の libheif と libde265 の代わりに、システムの libheif（1.17 以降）にリンクできます。ビルド時に `LAP_SYSTEM_LIBHEIF=1` を設定してください。この場合、サブモジュール `third_party/libheif` と `third_party/libde265` は不要です。HEVC のデコードは、システム libheif のコーデックプラグインに依存します。

## 対応形式

Lapは60以上の写真、RAW、動画形式に対応しています。

| タイプ | 形式 |
| :--- | :--- |
| 画像 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW写真 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| 動画 | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX など。H.264再生は全プラットフォームでサポートされており、ネイティブ再生が利用できない場合は自動的に互換性処理が行われます。HEVC/H.265およびVP9はmacOSでネイティブサポートされています。 |

### Linuxでの動画再生

Lap は AppImage を含め、動画再生にシステムの GStreamer プラグインを使用します。Ubuntu、Debian、Linux Mint で動画が再生できない場合は、以下をインストールしてください：

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## アーキテクチャ

- コア: Tauri + Rust
- フロントエンド: Vue + Vite + Tailwind CSS
- データ: SQLite

### 主要ライブラリ

| ライブラリ | 用途 |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | RAW画像のデコードとサムネイル抽出 |
| [libheif](https://github.com/strukturag/libheif) | HEIC/HEIF/HIF画像のデコードとプレビュー生成 |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | 高速なJPEGデコードとサムネイル生成 |
| [FFmpeg](https://ffmpeg.org/) | 動画処理とサムネイル生成 |
| [Video.js](https://videojs.com/) | クロスプラットフォームの動画再生UI |
| [ONNX Runtime](https://onnxruntime.ai/) | ローカルAIモデル推論エンジン |
| [CLIP](https://github.com/openai/CLIP) | 画像・テキストの類似度検索 |
| [InsightFace](https://github.com/deepinsight/insightface) | 顔検出と認識 |
| [Leaflet](https://leafletjs.com/) | ジオタグ付き写真用のインタラクティブマップ |
| [daisyUI](https://daisyui.com/) | UIコンポーネントライブラリ |

## ライセンス

GPL-3.0-or-later。詳細は [LICENSE](../LICENSE) をご覧ください。

## プライバシー

データの取り扱いと任意のオンラインサービスについては、[プライバシーポリシー](../PRIVACY.md)をご覧ください。
