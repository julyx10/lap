<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - プライベート・ローカル写真管理ツール</h1>
  <h3>macOS、Windows、Linux向けのオープンソース・デスクトップ写真管理ツール。</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Tiếng Việt](README.vi.md) | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | 日本語 | [한국어](README.ko.md)

Lapは、オープンソースでローカルファーストな写真管理ツールです。家族のアルバムを閲覧したり、古い写真を素早く見つけたり、膨大な個人メディアライブラリをオフラインで管理したりするために設計されています。
読み込み作業やサブスクリプション、アップロードは不要です。既存のフォルダーを Lap で直接開けます。ファイルの場所と形式はそのままです。互換性のある写真アプリで使えるよう、評価とタグを XMP サイドカーに保存できます。AI 検索はお使いの端末上で実行されます。

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">公式サイト</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">デモ</a></strong></p>
</div>

## Lapをダウンロード

[最新のリリースページ](https://github.com/julyx10/lap/releases/latest)を開き、お使いのシステムに合ったファイルをダウンロードしてください：

| プラットフォーム | パッケージ | 備考 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Appleによる公証済み |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 署名なし。ブラウザーでは **保持する**、SmartScreen では **詳細情報** → **実行** を選びます。 |
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
  <img src="../docs/public/screenshots/lap_preview.png" alt="Lap の写真プレビュー" width="900">
</p>

## Lapを選ぶ理由

- **フォルダー単位のライブラリ管理。** 既存のフォルダーを Lap で直接開けます。ファイルの場所と形式はそのままです。
- **ローカル AI 検索。** 写真を文章で探したり、類似写真や顔で絞り込んだりできます。処理はすべて端末上で行われます。
- **評価とタグの持ち運び。** 互換性のある写真アプリで使えるよう、評価とタグを XMP サイドカーに保存できます。
- **写真の選別。** スペースキーで即時プレビュー、評価後の自動送り、AF エリア表示、4 分割比較、重複ファイルの保持ルールに対応します。
- **写真と動画が 10 万点以上のライブラリに対応。** Rust 製のネイティブコアが閲覧、検索、スキャンを処理します。
- **無料のオープンソース。** サブスクリプションやアカウントは不要です。macOS、Windows、Linux で 17 言語に対応します。

## 主な機能

- **柔軟なライブラリ閲覧**：日付、フォルダー、場所、カメラ、レンズ、タグ、評価、顔で絞り込み、並び替えや検索にも対応。
- **フォルダー中心のワークフロー。** 複数ライブラリ、ドラッグ＆ドロップやクリップボードでの読み込みと書き出し、一括名前変更、ファイル同期、安全な移動・コピー・削除。
- **ローカル AI 検索**：テキスト、見た目の類似性、被写体、顔のグループ化に対応し、50 以上の言語での検索も選択可能。
- **メタデータの読み込みと XMP への書き出し** · 埋め込み EXIF・IPTC・XMP とサイドカー（評価、タグ、お気に入り、選別状態、GPS、日付など）を読み取ります。テキスト項目を編集し、整理情報を XMP サイドカーに保存できます。
- **スマートアルバム。** 独自のグループ化と並べ替え、フォルダー名フィルター、検索可能な値の選択。
- **日付別の読み込み**：日・月・年別または単一フォルダーに整理し、元のファイル名を保持して重複をスキップ。
- **コレクションとタグ**：元ファイルを移動・複製せず、選択したファイルを一括整理。ドラッグ操作と枝ごとの件数表示に対応したタグツリー。
- **重複整理。** フォルダー、形式、サイズ、日付による保持ルール、解放可能な容量の表示、一括削除。
- **選別・比較ツール。** スペースキーで即時プレビュー、評価後の自動送り、AF エリア表示、4 分割比較、重複ファイルの保持ルールに対応します。
- **内蔵編集機能**：切り抜き、任意の角度での水平調整、回転、反転、サイズ変更、基本的な画像調整。
- **写真コラージュ**：選択した写真をグリッド、モザイク、写真の山のコラージュに構成。
- **Apple Live Photos と Google Motion Photos**：動きの再生と共通のスマートアルバムフィルターに対応。
- **RAW + JPEG/HEIC ペア**：1 つの項目として表示し、ファイル操作時は関連ファイルをまとめて処理。 RAW ペアは片方だけでも両方でも削除できます。片方を残す場合、共有 XMP サイドカーも保持されます。
- **RAW サムネイルとプレビューの選択**：RAW レンダリングまたはカメラの埋め込みプレビューを使用し、明るさの調整や RAW+JPEG ペアの表示にも対応。
- **カスタマイズ可能なファイル情報。** 表示項目の選択、詳細の遅延読み込み、ビューアーの AF エリア表示。
- **インタラクティブな地図表示**：現在のフィルターに沿って、位置情報付きの写真や動画をグループで探索。
- **オフラインでのアルバム閲覧**：切断されたストレージやネットワークストレージ上のアルバムも、キャッシュ済みサムネイルで表示。
- **クラウド上にのみ保存されたファイル。** OneDrive のオンデマンドファイルや macOS のプレースホルダーを自動ダウンロードせずにインデックス化して表示。
- **ライトテーマとダークテーマ。** システムの外観に合わせることもできます。
- **幅広い形式に対応**：60 以上の画像・RAW・動画形式をサポート。 Affinity ドキュメント (埋め込みプレビュー).

## メタデータ、コレクション、ファイルの移動

Lap は既存のフォルダーを直接扱い、表示する情報の大半をファイルから読み取ります。整理情報は、他の写真アプリでも読み取れる標準の XMP サイドカーに保存できます。

Lap は Lightroom、digiKam、darktable、ACDSee の互換メタデータを読み込みます。対応する項目はアプリによって異なります。

### ファイルとともに残るもの

- 元の写真と動画は、既存のフォルダー内の通常のファイルのままです。
- インデックス作成時に、撮影情報や整理情報を含む埋め込み EXIF・IPTC・XMP を読み取ります。整理情報をローカルデータベースに取り込むには、**設定 → メタデータ → メタデータを読み込む** を有効にします。ローカルでの編集内容は保持されます。
- GPS や撮影日時を含め、対応する XMP サイドカーの値が埋め込み値より優先されます。
- タイトル、見出し、説明、作成者、著作権を編集できます。変更はローカルデータベースに保存されます。
- XMP への保存では、これらの項目とタグ（階層キーワードを含む）、評価、お気に入り、選別状態をサイドカーに書き出します。その他の XMP プロパティや元ファイルは保持されます。
- ファイルを**Lap内で**名前変更、移動、コピー、削除、読み込みすると、Lapは同時にローカルカタログを更新します。Apple Live Photoの構成ファイル、XMP・AAE サイドカー、有効にしたRAW + JPEG/HEICペアなど、対応する関連ファイルもまとめて扱います。 RAW ペアは片方だけでも両方でも削除できます。片方を残す場合、共有 XMP サイドカーも保持されます。
- 内蔵の画像編集を保存すると、編集後の画像が選択した保存先に書き込まれます。

### Lapがローカルに保存するもの

これらのライブラリ情報はローカルデータベースまたはライブラリ設定に保存されます。

- コレクション、スマートアルバム、ルールや表示設定はローカルデータベースまたは設定に保存されます。
- AI検索データ、顔データ、サムネイル、その他のインデックスまたはキャッシュデータ
- タグ、評価、お気に入り、選別状態、編集可能なテキストはローカルに保存され、XMP に書き出せます。

既定では Lap の外でファイルを扱ってもこれらの情報は引き継がれません。

### Lapの外でファイルを扱う場合

Lapはフォルダーを再スキャンし、多くのファイルシステム上の変更を検出できます。ただし、Lapの外でファイルの名前変更、移動、置き換え、コピーを行うと、Lap内だけに保存されている整理情報に影響する場合があります。

メタデータの読み込みが有効な場合は、XMP サイドカーの変更も確認します。コレクションへの所属を維持するには、Lap で名前変更や移動を行ってください。XMP サイドカーは元ファイルと一緒に管理してください。 Lapの外でもファイルを管理する場合は、写真とあわせてLapのデータベースと設定をバックアップしてください。データベースの場所の管理とバックアップの作成は、**設定 → ストレージ**から行えます。

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

ローカルでインストールした Arch パッケージの削除には次を使います。 `sudo pacman -R lap`.

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

### Arch Linux (x86_64)

Arch 用の試験的なビルドレシピです。AUR では未公開です。ローカルでのインストール手順: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

ソースのコミットを固定し、AI モデルと FFmpeg をチェックサムで検証します。更新は pacman が管理します。

## 対応形式

Lapは60以上の画像、RAW、動画形式に対応しています。

| タイプ | 形式 |
| :--- | :--- |
| 画像 | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| RAW写真 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Affinity ドキュメント | AF, AFPHOTO, AFDESIGN, AFPUB (埋め込みプレビュー) |
| 動画 | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. H.264再生は全プラットフォームでサポートされており、ネイティブ再生が利用できない場合は自動的に互換性処理が行われます。HEVC/H.265およびVP9はmacOSでネイティブサポートされています。 |

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
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | ExifTool の Rust 移植版 |
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
