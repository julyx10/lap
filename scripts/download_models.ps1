$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$TargetDir = Join-Path $RootDir "src-tauri\resources\models"
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$Models = @(
    @{ Url = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json"; File = "tokenizer.json" },
    @{ Url = "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx"; File = "text_model.onnx" },
    @{ Url = "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx"; File = "vision_model.onnx" },
    @{ Url = "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/det_500m.onnx?download=true"; File = "det_500m.onnx" },
    @{ Url = "https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/w600k_mbf.onnx?download=true"; File = "w600k_mbf.onnx" }
)

Write-Host "Downloading $($Models.Count) models into $TargetDir..."

foreach ($model in $Models) {
    $FilePath = Join-Path $TargetDir $model.File
    if (Test-Path $FilePath) {
        Write-Host "# $($model.File) already exists, skipping."
    } else {
        Write-Host "Downloading $($model.File)..."
        Invoke-WebRequest -Uri $model.Url -OutFile $FilePath -UseBasicParsing
        Write-Host "# $($model.File) done."
    }
}

Write-Host "All downloads complete!"
