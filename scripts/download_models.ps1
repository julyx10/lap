$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$TargetDir = Join-Path $RootDir "src-tauri\resources\models"
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$Models = @(
    @{ Url = "https://huggingface.co/openai/clip-vit-base-patch32/resolve/3d74acf9a28c67741b2f4f2ea7635f0aaf6f0268/tokenizer.json"; File = "tokenizer.json"; Sha256 = "b556ac8c99757ffb677208af34bc8c6721572114111a6e0aaf5fa69ff0b8d842" },
    @{ Url = "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/text_model_quantized.onnx"; File = "text_model.onnx"; Sha256 = "73baab855d406190da9faa498cfedf65f15cf309f4cc7385b7b032e6d08e5c3a" },
    @{ Url = "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/d15189d7028b43f1d3e65039190477f6af591c2a/onnx/vision_model_quantized.onnx"; File = "vision_model.onnx"; Sha256 = "583fd1110a514667812fee7d684952aaf82a99b959760c8d7dca7e0ab9839299" },
    @{ Url = "https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/det_500m.onnx?download=true"; File = "det_500m.onnx"; Sha256 = "5e4447f50245bbd7966bd6c0fa52938c61474a04ec7def48753668a9d8b4ea3a" },
    @{ Url = "https://huggingface.co/deepghs/insightface/resolve/4e1f33d3fe0e50a0945f3a53ab94ae8977ae7ddb/buffalo_s/w600k_mbf.onnx?download=true"; File = "w600k_mbf.onnx"; Sha256 = "9cc6e4a75f0e2bf0b1aed94578f144d15175f357bdc05e815e5c4a02b319eb4f" }
)

Write-Host "Downloading $($Models.Count) models into $TargetDir..."

function Test-ModelIntegrity {
    param([string]$Path, [string]$ExpectedSha256)

    $ActualSha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    if ($ActualSha256 -ne $ExpectedSha256) {
        throw "SHA-256 mismatch: $Path; remove the rejected file before downloading again."
    }
}

foreach ($model in $Models) {
    $FilePath = Join-Path $TargetDir $model.File
    if (Test-Path -LiteralPath $FilePath) {
        Test-ModelIntegrity -Path $FilePath -ExpectedSha256 $model.Sha256
        Write-Host "# $($model.File) verified, skipping download."
    } else {
        Write-Host "Downloading $($model.File)..."
        $Headers = @{}
        if ($env:HF_TOKEN) {
            $Headers["Authorization"] = "Bearer $($env:HF_TOKEN)"
        }
        $TempFile = Join-Path $TargetDir ($model.File + ".part." + [guid]::NewGuid().ToString("N"))
        try {
            Invoke-WebRequest -Uri $model.Url -OutFile $TempFile -UseBasicParsing -Headers $Headers
            Test-ModelIntegrity -Path $TempFile -ExpectedSha256 $model.Sha256
            Move-Item -LiteralPath $TempFile -Destination $FilePath -ErrorAction Stop
            Write-Host "# $($model.File) done."
        } finally {
            if (Test-Path -LiteralPath $TempFile) {
                Remove-Item -LiteralPath $TempFile -Force
            }
        }
    }
}

Write-Host "All downloads complete!"
