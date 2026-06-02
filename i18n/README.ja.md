<div align="center">
  <img src="../docs/public/icon.png" alt="Lap Logo" width="120" style="border-radius: 20px">
  <h1>Lap - プライベート・ローカルフォトマネージャー</h1>
  <h3>macOS、Windows、Linux向けのオープンソース・デスクトップ写真管理ツール。</h3>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="GitHub release"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="GitHub all releases"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="GitHub stars"></a>
  </p>
</div>

[English](../README.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [简体中文](README.zh-CN.md) | 日本語 | [한국어](README.ko.md)

Lapは、オープンソースでローカルファーストな写真管理ツールです。家族のアルバムを閲覧したり、古い写真を素早く見つけたり、膨大な個人メディアライブラリをオフラインで管理したりするために設計されています。
クラウド写真サービスのプライバシーに配慮した代替案として、強制アップロードなし、ローカルAI検索、フォルダーベースのワークフローを提供し、完全に無料で使用できます。

- ウェブサイト: [https://julyx10.github.io/lap/](https://julyx10.github.io/lap/)
- デモビデオ: [https://youtu.be/RbKqNKhbVUs](https://youtu.be/RbKqNKhbVUs)
- プライバシーポリシー: [PRIVACY.md](../PRIVACY.md)

## Lapをダウンロード

[最新のリリースページ](https://github.com/julyx10/lap/releases/latest)を開き、お使いのシステムに合ったファイルをダウンロードしてください：

| プラットフォーム | パッケージ | 備考 |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Appleによる公証済み |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | 未署名 — SmartScreenがダウンロードをブロックした場合は、**維持する**をクリックしてください |
| **Linux (amd64 / arm64)** | `_amd64.deb` / `_arm64.deb` | Debian系ディストリビューション向け（Ubuntu、Debian、Linux Mintなど） |

## スクリーンショット

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_1.png" alt="Lap ローカルフォトライブラリ管理 スクリーンショット" width="900">
</p>

<p align="center">
  <img src="../docs/public/screenshots/lap-home-0.1.10_2.png" alt="Lap ローカルAI写真検索 スクリーンショット" width="900">
</p>

## Lapを選ぶ理由

- **クラウド不要**: ライブラリをホスト型サービスにアップロードするのではなく、自身のディスクに保存できます。
- **デフォルトでプライベート**: 処理はローカルで行われるため、写真は常にユーザーの管理下にあります。
- **無料で使用可能**: サブスクリプションプランや継続的な費用は一切かかりません。
- **フォルダーベース**: 既存のフォルダーを直接操作でき、面倒なインポート操作は不要です。
- **大規模ライブラリ向けの高パフォーマンス**: 大規模なメディアコレクション（1ライブラリあたり10万ファイル以上）でもスムーズな閲覧と整理ができるよう最適化されています。

## 主な機能

- **閲覧とフィルタ**: 日付、場所、カメラ、レンズ、タグ、お気に入り、評価、人物（ベータ版）で絞り込めます。
- **複数ライブラリの管理**: 複数のライブラリを作成し、素早く切り替えることができます。
- **重複項目の検索**: 重複したファイルを特定し、不要なコピーを一括でゴミ箱へ移動できます。
- **その場で編集**: 切り抜き、回転、反転、リサイズ、および基本的な画質調整が可能です。
- **フォルダーの同期**: ファイルシステムの変更を感知する操作や手動更新をサポートしています。
- **ローカルAI検索ツール**: テキスト/画像検索、類似画像検索、顔のクラスタリング、スマートタグを利用できます。
- **50以上の言語で検索**: 必要に応じて追加ダウンロードできるオプションの多言語モデルを利用できます。
- **最新の画像形式に対応**: WebP、HEIC/HEIF/HIF、AVIF、JXL (JPEG XL)に対応しています。
- **RAW写真の表示**: 20以上のカメラメーカーのRAW形式（CR2、NEF、ARW、DNGなど）を内蔵デコーダーで表示できます。
- **幅広いビデオ互換性**: MP4、MOV、AVI、MKV、および20以上の形式をサポートし、クロスプラットフォーム向けに最適化されています。

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

Debianベースのディストリビューションでは、パッケージをアンインストールします：

```bash
sudo apt remove lap
```

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

## 対応形式

| タイプ | 形式 |
| :--- | :--- |
| 画像 | JPG/JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL |
| RAW写真 | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, DCR, KDC, ERF, MEF, RAW, MDC |
| 動画 | MP4, MOV, M4V, MKV, AVI, FLV, TS/M2TS, WMV, WebM, 3GP/3G2, F4V, VOB, MPG/MPEG, ASF, DIVX など。H.264再生は全プラットフォームでサポートされており、ネイティブ再生が利用できない場合は自動的に互換性処理が行われます。HEVC/H.265およびVP9はmacOSでネイティブサポートされています。 |

### Linuxでの動画再生に関する備考

Linux Mint/Ubuntu/Debianでは、動画再生のサポートを向上させるために以下のパッケージをインストールしてください：

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
