<div align="center">
  <img src="../docs/public/icon.png" alt="Logo Lap" width="120" style="border-radius: 20px">
  <h1>Lap - Trình quản lý ảnh cục bộ riêng tư</h1>
  <h3>Trình quản lý ảnh để bàn mã nguồn mở cho macOS, Windows và Linux.</h3>
  <p>
    <a href="https://trendshift.io/repositories/24497?utm_source=repository-badge&amp;utm_medium=badge&amp;utm_campaign=badge-repository-24497" target="_blank" rel="noopener noreferrer"><img src="https://trendshift.io/api/badge/repositories/24497" alt="julyx10%2Flap | Trendshift" width="250" height="55"/></a>
  </p>
  <p>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/v/release/julyx10/lap" alt="Bản phát hành GitHub"></a>
    <a href="https://github.com/julyx10/lap/releases"><img src="https://img.shields.io/github/downloads/julyx10/lap/total" alt="Tổng lượt tải xuống trên GitHub"></a>
    <a href="https://github.com/julyx10/lap/stargazers"><img src="https://img.shields.io/github/stars/julyx10/lap" alt="Số sao GitHub"></a>
  </p>
</div>

[English](../README.md) | [Česky](README.cs.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Français](README.fr.md) | [Italiano](README.it.md) | [Magyar](README.hu.md) | [Nederlands](README.nl.md) | [Polski](README.pl.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | Tiếng Việt | [中文简体](README.zh-CN.md) | [中文繁體](README.zh-TW.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

Lap là trình quản lý ảnh mã nguồn mở, ưu tiên cục bộ, dùng để duyệt album gia đình, tìm nhanh ảnh cũ và quản lý thư viện phương tiện cá nhân lớn khi ngoại tuyến.
Không cần nhập trước, đăng ký trả phí hay tải lên. Mở thư mục hiện có trực tiếp trong Lap. Tệp giữ nguyên vị trí và định dạng. Lưu xếp hạng và thẻ vào tệp phụ XMP chuẩn để dùng trong ứng dụng tương thích. Tìm kiếm AI chạy trên thiết bị của bạn.

<div align="center">
  <p><strong><a href="https://julyx10.github.io/lap/">TRANG WEB CHÍNH THỨC</a> | <a href="https://julyx10.github.io/lap/demos/lap/index.html">DEMO</a></strong></p>
</div>

## Tải Lap

Mở [trang phát hành mới nhất](https://github.com/julyx10/lap/releases/latest), sau đó tải tệp phù hợp với hệ thống của bạn:

| Nền tảng | Gói | Ghi chú |
| :-- | :-- | :-- |
| **macOS (Apple Silicon / Intel)** | `_aarch64.dmg` / `_x64.dmg` | Được Apple công chứng |
| **Windows 10/11 (x64 / ARM64)** | `_x64_en-US.msi` / `_arm64_en-US.msi` | Chưa ký. Chọn **Vẫn giữ** trong trình duyệt hoặc **Thông tin thêm** → **Vẫn chạy** trong SmartScreen. |
| **Linux (x64 / ARM64)** | `_amd64.deb` / `_arm64.deb` | Dành cho các bản phân phối dựa trên Debian (Ubuntu, Debian, Linux Mint, v.v.) |
| **Linux (x64 / ARM64)** | `_amd64.AppImage` / `_aarch64.AppImage` | Cấp quyền thực thi cho tệp, sau đó nhấp đúp để chạy |

### macOS với Homebrew

```bash
brew tap julyx10/lap
brew install --cask lap
```

## Ảnh chụp màn hình

<p align="center">
  <img src="../docs/public/screenshots/lap_library.png" alt="Trình quản lý thư viện ảnh cục bộ Lap" width="900">
  <img src="../docs/public/screenshots/lap_map_view.png" alt="Chế độ xem bản đồ Lap" width="900">
  <img src="../docs/public/screenshots/lap_montage.png" alt="Ảnh ghép Lap" width="900">
  <img src="../docs/public/screenshots/lap_preview.png" alt="Xem trước ảnh trong Lap" width="900">
</p>

## Vì sao chọn Lap

- **Quản lý thư viện theo thư mục.** Mở thư mục hiện có trực tiếp trong Lap. Tệp giữ nguyên vị trí và định dạng.
- **Tìm kiếm AI cục bộ.** Mô tả ảnh, tìm ảnh tương tự hoặc duyệt theo khuôn mặt. Mọi xử lý diễn ra trên thiết bị.
- **Xếp hạng và thẻ có thể chuyển đổi.** Lưu vào tệp phụ XMP để dùng trong ứng dụng tương thích.
- **Chọn lọc ảnh.** Xem nhanh bằng phím cách, tự chuyển sau khi xếp hạng, hiển thị vùng lấy nét, so sánh bốn khung và quy tắc giữ bản trùng.
- **Cho thư viện hơn 100.000 ảnh và video.** Lõi Rust gốc xử lý việc duyệt, tìm kiếm và quét.
- **Miễn phí và mã nguồn mở.** Không cần đăng ký trả phí hay tài khoản, có 17 ngôn ngữ trên macOS, Windows và Linux.

## Tính năng

- **Duyệt thư viện linh hoạt** theo ngày, thư mục, vị trí, máy ảnh, ống kính, thẻ, xếp hạng và khuôn mặt, kèm sắp xếp và tìm kiếm.
- **Quy trình ưu tiên thư mục.** Nhiều thư viện, nhập và xuất qua kéo thả hoặc bảng nhớ tạm, đổi tên hàng loạt và đồng bộ, di chuyển, sao chép và xóa an toàn.
- **Tìm kiếm AI cục bộ** theo câu lệnh văn bản, độ tương đồng hình ảnh, chủ thể, phân nhóm khuôn mặt và tìm kiếm đa ngôn ngữ tùy chọn trong hơn 50 ngôn ngữ.
- **Nhập siêu dữ liệu và ghi vào XMP** · Lap đọc EXIF, IPTC và XMP nhúng cùng các tệp phụ (xếp hạng, thẻ, mục yêu thích, trạng thái chọn lọc, GPS và ngày chụp). Có thể sửa các trường văn bản và lưu dữ liệu tổ chức vào XMP.
- **Album thông minh.** Nhóm và sắp xếp tùy chỉnh, lọc theo tên thư mục và bộ chọn có tìm kiếm.
- **Nhập ảnh theo ngày** với bố cục theo ngày, tháng, năm hoặc một thư mục, giữ nguyên tên tệp gốc và bỏ qua tệp trùng.
- **Bộ sưu tập và thẻ** để sắp xếp hàng loạt tệp đã chọn mà không di chuyển hay nhân bản bản gốc. Cây thẻ kéo thả với số lượng theo nhánh.
- **Dọn ảnh trùng lặp.** Quy tắc theo thư mục, định dạng, kích thước và ngày, dung lượng có thể thu hồi và xóa hàng loạt.
- **Công cụ chọn và so sánh.** Xem nhanh bằng phím cách, tự chuyển sau khi xếp hạng, hiển thị vùng lấy nét, so sánh bốn khung và quy tắc giữ bản trùng.
- **Chỉnh sửa tích hợp** cắt, làm thẳng góc tùy ý, xoay, lật, đổi kích thước và điều chỉnh ảnh cơ bản.
- **Ảnh ghép** để kết hợp ảnh đã chọn thành bố cục lưới, mosaic căn đều hoặc chồng ảnh.
- **Apple Live Photos và Google Motion Photos** với phát lại chuyển động và bộ lọc Album thông minh thống nhất.
- **Cặp RAW + JPEG/HEIC** hiển thị như một mục, giữ các tệp liên kết bên nhau khi thao tác tệp. Có thể xóa một hoặc cả hai tệp trong cặp RAW. Nếu giữ một tệp, tệp phụ XMP dùng chung cũng được giữ lại.
- **Ảnh thu nhỏ và bản xem trước RAW có thể cấu hình** bằng cách hiển thị RAW hoặc bản xem trước nhúng của máy ảnh, với độ sáng điều chỉnh được và hiển thị cặp RAW+JPEG.
- **Thông tin tệp tùy chỉnh.** Chọn trường hiển thị, tải chi tiết theo yêu cầu và hiển thị vùng lấy nét trong trình xem.
- **Chế độ xem bản đồ tương tác** để khám phá ảnh và video có gắn thẻ địa lý theo cụm tuân theo bộ lọc hiện tại.
- **Duyệt album ngoại tuyến** giữ album trên thiết bị lưu trữ bị ngắt kết nối hoặc mạng vẫn hiển thị với ảnh thu nhỏ được lưu đệm.
- **Tệp chỉ có trên đám mây.** Tệp theo yêu cầu của OneDrive và tệp giữ chỗ macOS được lập chỉ mục và hiển thị mà không tự động tải xuống.
- **Giao diện sáng và tối.** Có thể theo giao diện hệ thống.
- **Hỗ trợ nhiều định dạng** cho hơn 60 định dạng ảnh, RAW và video. Tài liệu Affinity (bản xem trước nhúng).

## Siêu dữ liệu, bộ sưu tập và di chuyển tệp

Lap làm việc trực tiếp với các thư mục và đọc phần lớn thông tin hiển thị từ tệp. Bạn có thể lưu dữ liệu tổ chức vào tệp phụ XMP chuẩn để các ứng dụng ảnh khác đọc được.

Lap đọc siêu dữ liệu tương thích từ Lightroom, digiKam, darktable và ACDSee. Các trường được hỗ trợ tùy theo ứng dụng.

### Thông tin nào đi cùng tệp

- Ảnh và video gốc của bạn luôn là tệp thông thường trong các thư mục hiện có.
- Khi lập chỉ mục, Lap đọc EXIF, IPTC và XMP nhúng, gồm dữ liệu chụp và tổ chức. Bật **Cài đặt → Siêu dữ liệu → Nhập siêu dữ liệu** để nhập dữ liệu tổ chức vào cơ sở dữ liệu cục bộ. Các chỉnh sửa cục bộ được giữ lại.
- Các giá trị được hỗ trợ trong tệp phụ XMP được ưu tiên hơn giá trị nhúng, bao gồm GPS và ngày chụp.
- Có thể sửa tiêu đề, tiêu đề ngắn, mô tả, tác giả và bản quyền. Thay đổi được lưu trong cơ sở dữ liệu cục bộ.
- Lưu vào XMP ghi các trường này cùng thẻ (bao gồm từ khóa phân cấp), xếp hạng, mục yêu thích và trạng thái chọn lọc vào tệp phụ, đồng thời giữ nguyên các thuộc tính XMP khác và tệp gốc.
- Khi bạn đổi tên, di chuyển, sao chép, xóa hoặc nhập tệp **trong Lap**, Lap cập nhật danh mục cục bộ cùng lúc. Lap cũng giữ các thành phần được hỗ trợ đi cùng nhau, chẳng hạn như các thành phần Apple Live Photo, tệp phụ XMP và AAE và cặp RAW + JPEG/HEIC đã bật. Có thể xóa một hoặc cả hai tệp trong cặp RAW. Nếu giữ một tệp, tệp phụ XMP dùng chung cũng được giữ lại.
- Lưu chỉnh sửa ảnh tích hợp sẽ ghi ảnh kết quả vào đích đã chọn.

### Những gì Lap lưu cục bộ

Lap lưu các dữ liệu thư viện này trong cơ sở dữ liệu cục bộ hoặc cấu hình thư viện.

- Bộ sưu tập, album thông minh, quy tắc và tùy chọn hiển thị nằm trong cơ sở dữ liệu cục bộ hoặc cấu hình.
- Dữ liệu tìm kiếm AI, dữ liệu khuôn mặt, ảnh thu nhỏ và dữ liệu chỉ mục/bộ nhớ đệm khác
- Thẻ, xếp hạng, mục yêu thích, trạng thái chọn lọc và trường văn bản có thể sửa được lưu cục bộ và có thể ghi vào XMP.

Theo mặc định, dữ liệu này không đi theo tệp khi thao tác bên ngoài Lap.

### Làm việc với tệp ngoài Lap

Lap có thể quét lại thư mục và phát hiện nhiều thay đổi hệ thống tệp. Tuy nhiên, các thay đổi thực hiện ngoài Lap—như đổi tên, di chuyển, thay thế hoặc sao chép tệp—có thể làm gián đoạn tổ chức chỉ được lưu trong Lap.

Khi bật nhập siêu dữ liệu, Lap cũng kiểm tra thay đổi trong các tệp phụ XMP. Đổi tên và di chuyển tệp trong Lap để giữ nguyên bộ sưu tập. Đặt tệp phụ XMP cạnh tệp gốc. Nếu bạn cũng quản lý tệp ngoài Lap, hãy sao lưu cơ sở dữ liệu và cấu hình của Lap cùng với ảnh. Bạn có thể quản lý vị trí cơ sở dữ liệu và tạo bản sao lưu trong **Cài đặt → Lưu trữ**.

Xóa cơ sở dữ liệu hoặc cấu hình của Lap sẽ xóa dữ liệu tổ chức và chỉ mục cục bộ này, nhưng không xóa tệp phương tiện gốc.

## Gỡ cài đặt Lap

Lap làm việc trực tiếp với các thư mục ảnh hiện có. Gỡ cài đặt Lap hoặc xóa cơ sở dữ liệu và tệp bộ nhớ đệm **không** xóa ảnh gốc.

Các bước gỡ cài đặt tiêu chuẩn chỉ xóa ứng dụng. Để xóa hoàn toàn Lap, hãy thoát Lap trước, gỡ cài đặt ứng dụng, sau đó xóa cơ sở dữ liệu, bộ nhớ đệm ảnh thu nhỏ và tệp cấu hình cục bộ bằng lệnh dọn dẹp cho nền tảng của bạn.

### macOS

Nếu bạn cài Lap bằng Homebrew:

```bash
brew uninstall --cask lap
```

Đối với cài đặt thủ công, hãy thoát Lap và chuyển `Lap.app` từ thư mục `Applications` vào Thùng rác.

Để xóa tất cả cơ sở dữ liệu, bộ nhớ đệm và tệp cấu hình của Lap:

```bash
rm -rf "$HOME/Library/Application Support/com.julyx10.lap" \
       "$HOME/Library/Caches/com.julyx10.lap" \
       "$HOME/Library/WebKit/com.julyx10.lap"
rm -f "$HOME/Library/Preferences/com.julyx10.lap.plist"
```

### Windows

Mở **Cài đặt > Ứng dụng > Ứng dụng đã cài đặt**, tìm **Lap** và chọn **Gỡ cài đặt**.

Sau đó mở PowerShell và xóa tất cả cơ sở dữ liệu, bộ nhớ đệm và tệp cấu hình của Lap:

```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:LOCALAPPDATA\com.julyx10.lap"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$env:APPDATA\com.julyx10.lap"
```

### Linux

Đối với cài đặt DEB, gỡ gói:

```bash
sudo apt remove lap
```

Để gỡ gói Arch cài cục bộ, dùng `sudo pacman -R lap`.

Đối với gói Arch cài cục bộ, dùng `sudo pacman -R lap`.

Đối với cài đặt AppImage, thoát Lap và xóa tệp `.AppImage` đã tải xuống.

Sau đó xóa tất cả cơ sở dữ liệu, bộ nhớ đệm và tệp cấu hình của Lap:

```bash
rm -rf "$HOME/.local/share/com.julyx10.lap" \
       "$HOME/.cache/com.julyx10.lap" \
       "$HOME/.config/com.julyx10.lap"
```

Nếu bạn đã chọn thư mục lưu cơ sở dữ liệu tùy chỉnh trong cài đặt Lap, hãy xóa thư mục đó riêng sau khi xác nhận rằng nó chỉ chứa tệp cơ sở dữ liệu Lap.

## Biên dịch từ mã nguồn

Yêu cầu: Node.js 20+, pnpm, Rust stable.

```bash
# Phụ thuộc hệ thống macOS
xcode-select --install
brew install nasm pkg-config autoconf automake libtool cmake

# Phụ thuộc hệ thống Linux
# sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
#   patchelf nasm clang pkg-config autoconf automake libtool cmake

# Sao chép và biên dịch
git clone --recursive https://github.com/julyx10/lap.git
cd lap
git submodule update --init --recursive
cargo install tauri-cli --version "^2.0.0" --locked
./scripts/download_models.sh            # Windows: .\scripts\download_models.ps1
./scripts/download_ffmpeg_sidecar.sh    # Windows: .\scripts\download_ffmpeg_sidecar.ps1
cd src-vite && pnpm install && cd ..
cargo tauri dev
```

Các gói phân phối có thể liên kết với libheif hệ thống (1.17 trở lên) thay vì libheif và libde265 đi kèm. Đặt `LAP_SYSTEM_LIBHEIF=1` khi biên dịch. Khi đó không cần các mô-đun con `third_party/libheif` và `third_party/libde265`. Giải mã HEVC phụ thuộc vào plugin codec của libheif hệ thống.

### Arch Linux (x86_64)

Công thức biên dịch thử nghiệm cho Arch chưa được đưa lên AUR. Cài đặt cục bộ: [`contrib/aur`](../contrib/aur/README.md).

```bash
cd contrib/aur
makepkg -s
sudo pacman -U lap-<version>-x86_64.pkg.tar.zst
```

Công thức cố định commit và xác minh tổng kiểm tra của mô hình AI và FFmpeg. Pacman quản lý cập nhật.

## Định dạng hỗ trợ

Lap hỗ trợ hơn 60 định dạng ảnh, RAW và video.

| Loại | Định dạng |
| :--- | :--- |
| Hình ảnh | JPG/JPEG/JFIF, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF/HIF, AVIF, JXL, PSD, EXR, HDR/RGBE, TGA, JPEG 2000 (JP2/J2K/J2C/JPC/JPF/JPX), DDS, DPX, QOI |
| Ảnh RAW | CR2, CR3, CRW, NEF, NRW, ARW, SRF, SR2, RAF, RW2, ORF, PEF, DNG, SRW, RWL, MRW, 3FR, MOS, IIQ, DCR, KDC, ERF, MEF, RAW, MDC |
| Tài liệu Affinity | AF, AFPHOTO, AFDESIGN, AFPUB (bản xem trước nhúng) |
| Video | MP4, MOV, M4V, MKV, AVI, FLV, TS/MTS/M2TS, WMV, WebM, 3GP, MPG/MPEG, VOB, MOD/TOD, HEVC, ASF. Phát lại H.264 được hỗ trợ trên mọi nền tảng, với xử lý tương thích tự động khi không thể phát native. HEVC/H.265 và VP9 được hỗ trợ native trên macOS. |

### Phát video trên Linux

Lap sử dụng plugin GStreamer của hệ thống để phát video, kể cả trong AppImage. Nếu video không phát được trên Ubuntu, Debian hoặc Linux Mint, hãy cài:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

## Kiến trúc

- Lõi: Tauri + Rust
- Giao diện: Vue + Vite + Tailwind CSS
- Dữ liệu: SQLite

### Thư viện chính

| Thư viện | Mục đích |
| :-- | :-- |
| [LibRaw](https://github.com/LibRaw/LibRaw) | Giải mã ảnh RAW và trích xuất ảnh thu nhỏ |
| [libheif](https://github.com/strukturag/libheif) | Giải mã ảnh HEIC/HEIF/HIF và tạo bản xem trước |
| [libjpeg-turbo](https://libjpeg-turbo.org/) | Giải mã JPEG nhanh và tạo ảnh thu nhỏ |
| [FFmpeg](https://ffmpeg.org/) | Xử lý video và tạo ảnh thu nhỏ |
| [exiftool-rs](https://github.com/Le-Syl21/exiftool-rs) | Bản chuyển ExifTool sang Rust |
| [Video.js](https://videojs.com/) | Giao diện phát video đa nền tảng |
| [ONNX Runtime](https://onnxruntime.ai/) | Công cụ suy luận mô hình AI cục bộ |
| [CLIP](https://github.com/openai/CLIP) | Tìm kiếm tương đồng hình ảnh-văn bản |
| [InsightFace](https://github.com/deepinsight/insightface) | Phát hiện và nhận diện khuôn mặt |
| [Leaflet](https://leafletjs.com/) | Bản đồ tương tác cho ảnh gắn thẻ địa lý |
| [daisyUI](https://daisyui.com/) | Thư viện thành phần giao diện |

## Giấy phép

GPL-3.0-or-later. Xem [LICENSE](../LICENSE).

## Quyền riêng tư

Đọc [Chính sách quyền riêng tư](../PRIVACY.md) để biết chi tiết về xử lý dữ liệu và các dịch vụ trực tuyến tùy chọn.
