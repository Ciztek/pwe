#!/usr/bin/env bash
set -euo pipefail

# Simple helper to build Capacitor (Android) APK/AAB and Tauri bundles.
# Usage:
#   ./build-native.sh android-apk     # build Android release APK
#   ./build-native.sh android-aab     # build Android AAB (bundle)
#   ./build-native.sh tauri-linux     # build Linux bundles (AppImage, deb, ...)
#   ./build-native.sh tauri-windows   # build Windows installer (on Windows / proper toolchain)

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Installing dependencies (skip if already done)"
if command -v pnpm >/dev/null 2>&1; then
  PM=pnpm
  $PM install
else
  PM=npm
  echo "pnpm not found, falling back to npm install (may update package-lock.json)"
  $PM install
fi

echo "==> Building web assets"
$PM run build

detect_wsl() {
  # return 0 if running under WSL
  if [ -f /proc/version ] && grep -qi microsoft /proc/version; then
    return 0
  fi
  return 1
}

ensure_rust_target() {
  local target="$1"
  if rustup target list --installed | grep -q "^${target}$"; then
    echo "Rust target ${target} already installed"
  else
    echo "Adding Rust target ${target}"
    rustup target add "${target}"
  fi
}

case "${1:-}" in
  android-apk)
    echo "==> Syncing Capacitor and building Android APK"
    npx cap sync android
    cd android
    # Debug: ./gradlew assembleDebug
    ./gradlew assembleRelease
    echo "APK should be at: android/app/build/outputs/apk/release/"
    ;;

  android-aab)
    echo "==> Syncing Capacitor and building Android AAB (bundle)"
    npx cap sync android
    cd android
    ./gradlew bundleRelease
    echo "AAB should be at: android/app/build/outputs/bundle/release/"
    ;;

  tauri-linux)
    echo "==> Building Tauri (Linux) - bundles in src-tauri/target/release/bundle/"
    # Use npx to run the local tauri cli
    npx tauri build --target x86_64-unknown-linux-gnu
    echo "Bundles: src-tauri/target/release/bundle/"
    ;;

  tauri-windows)
    echo "==> Building Tauri (Windows) - will choose an appropriate Rust target based on environment"
    if detect_wsl; then
      echo "Detected WSL environment — building Windows GNU target (x86_64-pc-windows-gnu)."
      ensure_rust_target x86_64-pc-windows-gnu
      npx tauri build --target x86_64-pc-windows-gnu
    else
      # try to detect if running on Windows (MINGW/Cygwin) via OSTYPE or uname
      OS_NAME=$(uname -s 2>/dev/null || echo unknown)
      if echo "$OS_NAME" | grep -qi "mingw\|cygwin\|msys"; then
        echo "Detected Windows-like environment ($OS_NAME) — using MSVC target (x86_64-pc-windows-msvc)."
        ensure_rust_target x86_64-pc-windows-msvc
        npx tauri build --target x86_64-pc-windows-msvc
      else
        echo "Not on Windows or WSL; attempting MSVC target but this will likely fail unless MSVC toolchain is installed."
        echo "If you want a GNU Windows build, run this script on WSL or install mingw and use x86_64-pc-windows-gnu."
        ensure_rust_target x86_64-pc-windows-msvc || true
        npx tauri build --target x86_64-pc-windows-msvc
      fi
    fi
    echo "Windows bundles: src-tauri/target/release/bundle/"
    ;;

  *)
    echo "Usage: $0 {android-apk|android-aab|tauri-linux|tauri-windows}"
    exit 2
    ;;
esac

echo "Done."
