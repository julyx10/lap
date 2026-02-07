
import os
import urllib.request
import sys

def download_file(url, filepath):
    print(f"Downloading {os.path.basename(filepath)}...")
    try:
        urllib.request.urlretrieve(url, filepath)
        print("Done.")
    except Exception as e:
        print(f"Error downloading {url}: {e}")
        sys.exit(1)

def main():
    # Define target directory
    target_dir = os.path.join("src-tauri", "resources", "models")
    
    # Create directory if it doesn't exist
    if not os.path.exists(target_dir):
        print(f"Creating directory: {target_dir}")
        os.makedirs(target_dir)
    
    # Change to target directory to keep file paths simple if desired, 
    # but using absolute paths is safer. Lets just stick to full paths.
    
    # Define models to download
    models = [
        {
            "url": "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json",
            "filename": "tokenizer.json"
        },
        {
            "url": "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx",
            "filename": "text_model.onnx"
        },
        {
            "url": "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx",
            "filename": "vision_model.onnx"
        },
        {
            "url": "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/det_500m.onnx?download=true",
            "filename": "det_500m.onnx"
        },
        {
            "url": "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/w600k_mbf.onnx?download=true",
            "filename": "w600k_mbf.onnx"
        }
    ]
    
    print(f"Starting download of {len(models)} models into {target_dir}...")
    
    for model in models:
        filepath = os.path.join(target_dir, model["filename"])
        download_file(model["url"], filepath)
        
    print("All downloads complete!")

if __name__ == "__main__":
    main()
