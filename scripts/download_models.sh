#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/src-tauri/resources/models"
mkdir -p "$TARGET_DIR"

MODELS=(
  "https://huggingface.co/openai/clip-vit-base-patch32/resolve/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268/tokenizer.json|tokenizer.json|b556ac8c99757ffb677208af34bc8c6721572114111a6e0aaf5fa69ff0b8d842"
  "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/text_model_quantized.onnx|text_model.onnx|73baab855d406190da9faa498cfedf65f15cf309f4cc7385b7b032e6d08e5c3a"
  "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/vision_model_quantized.onnx|vision_model.onnx|583fd1110a514667812fee7d684952aaf82a99b959760c8d7dca7e0ab9839299"
  "https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/det_500m.onnx?download=true|det_500m.onnx|5e4447f50245bbd7966bd6c0fa52938c61474a04ec7def48753668a9d8b4ea3a"
  "https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/w600k_mbf.onnx?download=true|w600k_mbf.onnx|9cc6e4a75f0e2bf0b1aed94578f144d15175f357bdc05e815e5c4a02b319eb4f"
)

# Keep this contract in sync with contrib/aur/README.md.
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

echo "Downloading ${#MODELS[@]} models into $TARGET_DIR..."

for entry in "${MODELS[@]}"; do
  IFS='|' read -r URL FILENAME EXPECTED_SHA256 <<< "$entry"
  FILEPATH="$TARGET_DIR/$FILENAME"

  if [ -f "$FILEPATH" ]; then
    verify_file "$FILEPATH" "$EXPECTED_SHA256"
    echo "✓ $FILENAME verified, skipping download."
  else
    echo "⬇ Downloading $FILENAME..."
    TEMP_FILE="$(mktemp "$FILEPATH.part.XXXXXX")"
    CURL_ARGS=(-fL)
    if [ -n "${HF_TOKEN:-}" ]; then
      CURL_ARGS+=(-H "Authorization: Bearer $HF_TOKEN")
    fi
    curl "${CURL_ARGS[@]}" -o "$TEMP_FILE" "$URL"
    verify_file "$TEMP_FILE" "$EXPECTED_SHA256"
    chmod 644 "$TEMP_FILE"
    mv -- "$TEMP_FILE" "$FILEPATH"
    TEMP_FILE=""
    echo "✓ $FILENAME done."
  fi
done

echo "All downloads complete!"
