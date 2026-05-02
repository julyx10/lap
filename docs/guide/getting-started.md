# Getting Started

## Installation

### macOS (Apple Silicon)

1.  Download the latest `_aarch64.dmg` file from the [Releases page](https://github.com/julyx10/lap/releases).
2.  Open the disk image and drag **Lap** to your **Applications** folder.
3.  Double-click to launch.

### macOS (Intel)

1.  Download the latest `_x64.dmg` file from the [Releases page](https://github.com/julyx10/lap/releases).
2.  Open the disk image and drag **Lap** to your **Applications** folder.
3.  Double-click to launch.

### Linux (Ubuntu/Debian)

1. Download the latest `_amd64.deb` package from the [Releases page](https://github.com/julyx10/lap/releases).
2. Install it with your package manager or run `sudo apt install ./lap_<version>_amd64.deb`.
3. Launch **Lap** from your applications menu.

For better video playback support on Ubuntu/Debian/Linux Mint, install:

```bash
sudo apt install gstreamer1.0-libav gstreamer1.0-plugins-good
```

### Windows 10/11 (x64)

1. Download the latest `_x64_en-US.msi` installer from the [Releases page](https://github.com/julyx10/lap/releases).
2. Run the installer and complete the setup wizard.
3. Launch **Lap** from the Start menu or desktop shortcut.

### Windows 10/11 (ARM64)

1. Download the latest `_arm64_en-US.msi` installer from the [Releases page](https://github.com/julyx10/lap/releases).
2. Run the installer and complete the setup wizard.
3. Launch **Lap** from the Start menu or desktop shortcut.


## First Run

When you open Lap for the first time:

1.  **Grant Permissions**: Lap needs access to your folders to display photos.
2.  **Add a Library**: Point Lap to a folder containing your photos.
3.  **Let it Index**: Lap will scan your files, generate thumbnails, and build local search data. This happens on your device.

## Upgrading from v0.1.x

You can install Lap v0.2.0 directly over a v0.1.x installation. The local database is migrated automatically on first launch.
