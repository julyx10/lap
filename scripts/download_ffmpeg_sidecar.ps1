[CmdletBinding()]
param(
    [ValidateSet("x64", "x86_64", "arm64", "aarch64")]
    [string]$Arch
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$TargetDir = Join-Path $RootDir "src-tauri\resources\ffmpeg"
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

if (-not $Arch) {
    $procArch = $env:PROCESSOR_ARCHITECTURE
    if ($procArch -eq "ARM64") {
        $Arch = "arm64"
    }
    else {
        $Arch = "x64"
    }
    Write-Host "No -Arch specified; auto-detected $Arch (PROCESSOR_ARCHITECTURE=$procArch)."
}

switch ($Arch) {
    { $_ -in "x64", "x86_64" } {
        $TripleSuffix = "x86_64-pc-windows-msvc"
    }
    { $_ -in "arm64", "aarch64" } {
        $TripleSuffix = "aarch64-pc-windows-msvc"
    }
    default {
        throw "Unsupported architecture: $Arch"
    }
}

$ReleaseBase = "https://github.com/julyx10/lap-binaries/releases/download/ffmpeg-8.1"

$Downloads = @(
    "ffmpeg-$TripleSuffix.exe",
    "ffprobe-$TripleSuffix.exe"
)

Write-Host "Downloading FFmpeg sidecar files ($Arch) into $TargetDir..."
foreach ($Filename in $Downloads) {
    $Url = "$ReleaseBase/$Filename"
    $Dest = Join-Path $TargetDir $Filename

    if (Test-Path $Dest) {
        Write-Host "# $Filename already exists, skipping."
        continue
    }

    Write-Host "# Downloading $Filename..."
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
    }
    catch {
        Write-Error "Failed to download $Url. If this is an arm64 build and the binary has not been published yet, upload ffmpeg-aarch64-pc-windows-msvc.exe and ffprobe-aarch64-pc-windows-msvc.exe to the julyx10/lap-binaries release tag 'ffmpeg-8.1', or temporarily use the x64 binaries via Windows 11 ARM64 emulation."
        throw
    }
    Write-Host "# $Filename downloaded."
}

Write-Host "All FFmpeg sidecar files downloaded into $TargetDir."
