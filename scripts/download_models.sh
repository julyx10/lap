#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_DIR="$ROOT_DIR/src-tauri/resources/models"
mkdir -p "$TARGET_DIR"

MODELS=(
  "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json|tokenizer.json"
  "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx|text_model.onnx"
  "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx|vision_model.onnx"
  "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/det_500m.onnx?download=true|det_500m.onnx"
  "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/w600k_mbf.onnx?download=true|w600k_mbf.onnx"
)

echo "Downloading ${#MODELS[@]} models into $TARGET_DIR..."

for entry in "${MODELS[@]}"; do
  URL="${entry%%|*}"
  FILENAME="${entry##*|}"
  FILEPATH="$TARGET_DIR/$FILENAME"

  if [ -f "$FILEPATH" ]; then
    echo "✓ $FILENAME already exists, skipping."
  else
    echo "⬇ Downloading $FILENAME..."
    curl -L -o "$FILEPATH" "$URL"
    echo "✓ $FILENAME done."
  fi
done

echo "All downloads complete!"
