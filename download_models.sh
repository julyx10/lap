#!/bin/bash
# Download AI models

echo "Creating resources directory..."
mkdir -p src-tauri/resources/models
cd src-tauri/resources/models

echo "Downloading Tokenizer..."
curl -L -o tokenizer.json https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json

echo "Downloading Text Encoder (ONNX - Quantized)..."
curl -L -o text_model.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx

echo "Downloading Vision Encoder (ONNX - Quantized)..."
curl -L -o vision_model.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx


echo "Downloading Face Recognition Models..."
BASE_URL="https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s"

echo "Downloading Face Detection Model (det_500m.onnx)..."
curl -L -o det_500m.onnx "${BASE_URL}/det_500m.onnx?download=true"

echo "Downloading Face Embedding Model (w600k_mbf.onnx)..."
curl -L -o w600k_mbf.onnx "${BASE_URL}/w600k_mbf.onnx?download=true"

echo "Download complete!"
