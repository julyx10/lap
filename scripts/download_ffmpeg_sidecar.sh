#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/src-tauri/resources/ffmpeg"
mkdir -p "$TARGET_DIR"

RELEASE_BASE="https://github.com/julyx10/lap-binaries/releases/download/ffmpeg-8.1"

# Support overriding OS and ARCH for cross-compilation or CI
OS_TYPE="${1:-$(uname -s)}"
ARCH_TYPE="${2:-$(uname -m)}"

case "$OS_TYPE" in
  Darwin|macos|macos-*)
    case "$ARCH_TYPE" in
      x86_64|x64)
        FFMPEG_FILE="ffmpeg-x86_64-apple-darwin"
        FFPROBE_FILE="ffprobe-x86_64-apple-darwin"
        ;;
      arm64|aarch64)
        FFMPEG_FILE="ffmpeg-aarch64-apple-darwin"
        FFPROBE_FILE="ffprobe-aarch64-apple-darwin"
        ;;
      *)
        echo "Unsupported macOS architecture: $ARCH_TYPE"
        exit 1
        ;;
    esac
    ;;
  Linux|linux|linux-*|ubuntu-*)
    case "$ARCH_TYPE" in
      x86_64|x64)
        FFMPEG_FILE="ffmpeg-x86_64-unknown-linux-gnu"
        FFPROBE_FILE="ffprobe-x86_64-unknown-linux-gnu"
        FFMPEG_ASSET="405019648"
        FFPROBE_ASSET="405019643"
        FFMPEG_SHA256="caf3206f8858cfb3e87dcc02f1bf5d514790ad7f64b6c49a6da511b3e3033628"
        FFPROBE_SHA256="931f0eabf33462953978eea5d0cb92edc60ec4e33eaa1f0b5845537636ab190d"
        ;;
      aarch64|arm64)
        FFMPEG_FILE="ffmpeg-aarch64-unknown-linux-gnu"
        FFPROBE_FILE="ffprobe-aarch64-unknown-linux-gnu"
        FFMPEG_ASSET="405019654"
        FFPROBE_ASSET="405019651"
        FFMPEG_SHA256="21ba8322e97370ecd081e6b128a99b1323f73cec7fa631829a7e2e7e67bcbe0f"
        FFPROBE_SHA256="1856b1c9cedc0be91cca5c49148d2faa319afd09837c91e36ea163baab5cd24e"
        ;;
      *)
        echo "Unsupported Linux architecture: $ARCH_TYPE"
        exit 1
        ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT|windows|windows-*)
    FFMPEG_FILE="ffmpeg-x86_64-pc-windows-msvc.exe"
    FFPROBE_FILE="ffprobe-x86_64-pc-windows-msvc.exe"
    ;;
  *)
    echo "Unsupported platform: $OS_TYPE"
    exit 1
    ;;
esac

DOWNLOADS=(
  "$FFMPEG_FILE|${FFMPEG_ASSET:-}|${FFMPEG_SHA256:-}"
  "$FFPROBE_FILE|${FFPROBE_ASSET:-}|${FFPROBE_SHA256:-}"
)

# Linux asset identities and hashes are documented in contrib/aur/README.md.
# Non-Linux downloads retain their existing release URLs and integrity scope.
if command -v sha256sum >/dev/null 2>&1; then
  SHA256=(sha256sum)
else
  SHA256=(shasum -a 256)
fi

verify_file() {
  if ! printf '%s  %s\n' "$2" "$1" | "${SHA256[@]}" -c >/dev/null; then
    echo "SHA-256 mismatch: $1; remove the rejected file before downloading again." >&2
    return 1
  fi
}

TEMP_FILE=""
trap 'if [ -n "$TEMP_FILE" ]; then rm -f -- "$TEMP_FILE"; fi' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "Downloading FFmpeg sidecar files into $TARGET_DIR..."
for entry in "${DOWNLOADS[@]}"; do
  IFS='|' read -r FILENAME ASSET EXPECTED_SHA256 <<< "$entry"
  URL="$RELEASE_BASE/$FILENAME"
  CURL_ARGS=(-fL)
  if [ -n "$ASSET" ]; then
    URL="https://api.github.com/repos/julyx10/lap-binaries/releases/assets/$ASSET"
    CURL_ARGS+=(-H "Accept: application/octet-stream")
  fi
  DEST="$TARGET_DIR/$FILENAME"
  if [ -f "$DEST" ]; then
    if [ -n "$EXPECTED_SHA256" ]; then
      verify_file "$DEST" "$EXPECTED_SHA256"
    fi
    echo "✓ $FILENAME already exists, skipping download."
  else
    echo "⬇ Downloading $FILENAME..."
    TEMP_FILE="$(mktemp "$DEST.part.XXXXXX")"
    curl "${CURL_ARGS[@]}" -o "$TEMP_FILE" "$URL"
    if [ -n "$EXPECTED_SHA256" ]; then
      verify_file "$TEMP_FILE" "$EXPECTED_SHA256"
    fi
    chmod 755 "$TEMP_FILE"
    mv -- "$TEMP_FILE" "$DEST"
    TEMP_FILE=""
    echo "✓ $FILENAME downloaded."
  fi
  chmod +x "$DEST"
done

echo "All FFmpeg sidecar files downloaded into $TARGET_DIR."