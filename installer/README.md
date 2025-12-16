# PWE Karaoke Installer Build Guide

## Prerequisites

### Linux (Ubuntu/Debian)

```bash
# Install system dependencies (required for cargo-bundle)
sudo apt install -y \
    libssl-dev \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Install cargo-bundle
cargo install cargo-bundle
```

### macOS

```bash
brew install create-dmg
cargo install cargo-bundle
```

### Windows

1. Install [WiX Toolset](https://wixtoolset.org/releases/)
2. Add WiX to PATH
3. Install cargo-bundle:

```powershell
cargo install cargo-bundle
```

## Building Installers

**Important**: Installers can only be built on their target platform:

- **Linux**: Run on Linux to create `.deb` and `.AppImage`
- **macOS**: Run on macOS to create `.dmg`
- **Windows**: Run on Windows to create `.msi`

For automated multi-platform builds, use GitHub Actions (see `.github/workflows/release.yml`).

### Quick Build (All Platforms)

```bash
./installer/build-installers.sh
```

### Manual Build

#### Linux (.deb package)

```bash
cargo bundle --release --format deb
```

Output: `target/release/bundle/deb/pwe-karaoke_0.1.0_amd64.deb`

#### Linux (AppImage)

```bash
cargo bundle --release --format appimage
```

Output: `target/release/bundle/appimage/pwe-karaoke_0.1.0_amd64.AppImage`

#### macOS (.dmg)

```bash
cargo bundle --release --format dmg
```

Output: `target/release/bundle/dmg/PWE Karaoke_0.1.0_x64.dmg`

#### Windows (.msi)

```bash
cargo bundle --release --format msi
```

Output: `target/release/bundle/msi/PWE Karaoke_0.1.0_x64_en-US.msi`

## Installation

### Linux

```bash
# .deb package
sudo dpkg -i target/release/bundle/deb/pwe-karaoke_*.deb

# AppImage
chmod +x target/release/bundle/appimage/pwe-karaoke_*.AppImage
./target/release/bundle/appimage/pwe-karaoke_*.AppImage
```

### macOS

```bash
open target/release/bundle/dmg/PWE\ Karaoke_*.dmg
# Drag to Applications folder
```

### Windows

```powershell
# Run the MSI installer
target\release\bundle\msi\PWE Karaoke_*.msi
```

## Distribution

The installers in `target/release/bundle/` are ready for distribution:

- **Linux**: `.deb` for Debian/Ubuntu, `.AppImage` for universal Linux
- **macOS**: `.dmg` disk image with drag-to-install
- **Windows**: `.msi` installer package

## Updating Version

Update version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"

[package.metadata.bundle]
version = "0.2.0"
```

Then rebuild installers.
