$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$TargetDir = Join-Path $RootDir "src-tauri\resources\ffmpeg"
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$ReleaseBase = "https://github.com/julyx10/lap-binaries/releases/download/ffmpeg-8.1"
$Downloads = @(
    "ffmpeg-x86_64-pc-windows-msvc.exe",
    "ffprobe-x86_64-pc-windows-msvc.exe"
)

Write-Host "Downloading FFmpeg sidecar files into $TargetDir..."
foreach ($Filename in $Downloads) {
    $Url = "$ReleaseBase/$Filename"
    $Dest = Join-Path $TargetDir $Filename

    if (Test-Path $Dest) {
        Write-Host "✓ $Filename already exists, skipping."
        continue
    }

    Write-Host "⬇ Downloading $Filename..."
    Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
    Write-Host "✓ $Filename downloaded."
}

Write-Host "All FFmpeg sidecar files downloaded into $TargetDir."