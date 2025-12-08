#!/bin/bash
# Download AI models for jc-photo

echo "Creating resources directory..."
mkdir -p src-tauri/resources/models
cd src-tauri/resources/models

echo "Downloading Tokenizer..."
curl -L -o tokenizer.json https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json

echo "Downloading Text Encoder (ONNX - Quantized)..."
curl -L -o text_model.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx

echo "Downloading Vision Encoder (ONNX - Quantized)..."
curl -L -o vision_model.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx

echo "Download complete!"
