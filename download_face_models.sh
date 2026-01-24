#!/bin/bash
# Download Face Recognition models
# Models: InsightFace buffalo_s bundle
#   - det_500m.onnx (face detection - 2.52 MB)
#   - w600k_mbf.onnx (MobileFaceNet embedding - 13.6 MB)
# Total: ~16 MB

set -e

echo "Creating resources directory..."
mkdir -p src-tauri/resources/models
cd src-tauri/resources/models

BASE_URL="https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s"

echo ""
echo "Downloading Face Detection Model (det_500m.onnx - 2.52 MB)..."
curl -L -o det_500m.onnx "${BASE_URL}/det_500m.onnx?download=true"

echo ""
echo "Downloading Face Embedding Model (w600k_mbf.onnx - 13.6 MB)..."
curl -L -o w600k_mbf.onnx "${BASE_URL}/w600k_mbf.onnx?download=true"

echo ""
echo "Verifying downloads..."
ls -lh det_500m.onnx w600k_mbf.onnx

echo ""
echo "Download complete!"
echo "Models saved to src-tauri/resources/models/"
echo "  - det_500m.onnx (face detection)"
echo "  - w600k_mbf.onnx (face embedding - MobileFaceNet)"
