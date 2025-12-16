#!/bin/bash
# Build installers for multiple platforms

set -e

echo "Building PWE Karaoke installers..."

# Ensure cargo bin directory is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Check if cargo-bundle is installed
if ! command -v cargo-bundle &> /dev/null; then
    echo "Error: cargo-bundle not found!"
    echo "Please run: make installer-deps"
    exit 1
fi

# Build release binary
echo "Building release binary..."
cargo build --release --features custom-font

# Create installers
echo "Creating installers..."

# Linux: Create deb package
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Creating .deb package..."
    cargo bundle --release --format deb

    echo "Creating AppImage..."
    cargo bundle --release --format appimage

    # Cross-compile for Windows
    echo ""
    echo "Cross-compiling for Windows..."
    if rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
        echo "Building Windows binary..."
        cargo build --release --target x86_64-pc-windows-gnu --features custom-font

        echo "Windows .exe created at: target/x86_64-pc-windows-gnu/release/pwe-karaoke.exe"
        echo ""
        echo "Note: .msi installer requires WiX Toolset on Windows."
        echo "You can distribute the .exe directly or create an installer on Windows."
    else
        echo "Windows target not installed. Skipping Windows build."
        echo "To enable: rustup target add x86_64-pc-windows-gnu"
    fi
fi

# macOS: Create DMG
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Creating macOS .dmg..."
    cargo bundle --release --format dmg
    cargo bundle --release --format app
fi

# Windows: Create MSI (requires wix toolset)
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "Creating Windows .msi..."
    cargo bundle --release --format msi
fi

echo "Installers created in target/release/bundle/"
ls -lh target/release/bundle/
