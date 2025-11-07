# Frontend Dependency Inventory (for Nix flake authors)

This file lists all dependencies required to build and package the `frontend` of the EpiCovid project. It includes JS (npm/pnpm) packages, dev tools, and the system-level toolchain and libraries your Nix flake should provide or document.

> Note: This is an inventory only — it does not create a flake. For platform-specific items (Xcode/CocoaPods) note that they are macOS-only and cannot be provided from Linux via Nix.

---

## JavaScript / Node packages (from `package.json`)

- dependencies (runtime)
  - `@capacitor/android` ^7.4.3
  - `@capacitor/cli` ^7.4.3
  - `@capacitor/core` ^7.4.3
  - `@capacitor/ios` ^7.4.3
  - `react` ^19.1.1
  - `react-dom` ^19.1.1
  - `recharts` ^2.5.0
  - `date-fns` ^2.30.0
  - `@tauri-apps/api` ^2.0.0

- devDependencies (build / dev tools)
  - `@eslint/js` ^9.36.0
  - `@tauri-apps/cli` ^2.8.4
  - `@types/react` ^19.1.13
  - `@types/react-dom` ^19.1.9
  - `@vitejs/plugin-react` ^5.0.3
  - `eslint` ^9.36.0
  - `eslint-plugin-react-hooks` ^5.2.0
  - `eslint-plugin-react-refresh` ^0.4.20
  - `globals` ^16.4.0
  - `typescript` ~5.8.3
  - `typescript-eslint` ^8.44.0
  - `vite` ^7.1.7

## System / OS packages & runtimes (required by scripts and native packaging)

- Node.js (LTS) — e.g. Node 18 or Node 20 (ensure compatibility with project packages).
- pnpm (optional, recommended) — scripts prefer pnpm if installed.
- rustup, rustc, cargo (stable Rust toolchain) — required for Tauri.

Rust targets to consider (install with `rustup target add`):

- `x86_64-unknown-linux-gnu` (Linux)
- `x86_64-pc-windows-msvc` (Windows MSVC; requires Visual Studio C++ toolchain on Windows)
- `x86_64-pc-windows-gnu` (Windows GNU / mingw, useful for cross-building from WSL)

- OpenJDK 17 (required by modern Android Gradle Plugin). Older AGP versions worked with 11, but current project requires 17.
- Android SDK & command-line tools:
  - `android-sdk`, `platform-tools`, `build-tools` (matching compileSdk)
  - Ensure `ANDROID_HOME` / `ANDROID_SDK_ROOT` set in environment and tools on PATH.
- Gradle: project includes Gradle wrapper; JDK + wrapper are normally sufficient.

- mingw-w64 (if cross-compiling Windows GNU from Linux/WSL)
- build-essential / gcc / g++ / make
- pkg-config
- libssl-dev (or OS-equivalent) — for crates that link OpenSSL
- libwebkit2gtk-4.0-dev, libgtk-3-dev, glib dev packages (Linux) — Tauri may require webview GTK deps depending on platform/webview backend

## Platform-specific / optional tools

- Xcode + CocoaPods (macOS only) — required to build Capacitor iOS apps.
- Visual Studio with "Desktop development with C++" (Windows) — required to build MSVC Rust target.
- keytool (from JDK) — create/sign Android keystores.
- zip/unzip, curl, wget, git — common utilities useful in CI and packaging steps.

## CLI tools referenced by repository scripts

- `npx` (bundled with npm)
- `cap` (Capacitor CLI via `@capacitor/cli`)
- `tauri` (Tauri CLI via `@tauri-apps/cli`)
- `gradle` (via project `gradlew` wrapper)

## Environment vars / runtime hints (used by scripts)

- `LIBGL_ALWAYS_SOFTWARE` and `MESA_LOADER_DRIVER_OVERRIDE` — used in `package.json` or helper scripts to force software GL in WSL or headless environments (see `tauri:dev:sw` script example).
- `ANDROID_HOME` / `ANDROID_SDK_ROOT` — Android SDK locations.
- Android signing values (keystore path, alias, passwords) — should be provided by CI secrets or developer environment; do not store in the flake.

## Notes & recommendations for the flake author

- Provide Node.js and pnpm (or npm) in the flake's `packages` or `devShell` so `pnpm install` / `npm install` works reproducibly.
- Provide Rust (rustup or pinned toolchain) — prefer installing `rustup` in the developer shell and pinning a stable toolchain.
- For building Windows bundles, prefer building on a native Windows host with MSVC installed. If building from Linux/WSL, install `mingw-w64` and use `x86_64-pc-windows-gnu` target — note linking toolchain differences.
- Android SDK is large: consider documenting a CI image (or using an official Android builder image) rather than packaging the entire SDK inside the flake.
- macOS / Xcode cannot be provided from Linux via Nix; document iOS build steps as macOS-only.

---

## Global setup (Ubuntu, non‑Nix)

If you prefer installing globally on Ubuntu instead of using Nix, these packages cover the common needs for this repo:

```bash
sudo apt update
sudo apt install -y build-essential curl wget git unzip xz-utils \
  ca-certificates gnupg lsb-release software-properties-common

# Node 20 LTS and pnpm
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
sudo npm install -g pnpm

# Java 17 for Android Gradle
sudo apt install -y openjdk-17-jdk

# Tauri/Linux packaging requirements
sudo apt install -y patchelf pkg-config libglib2.0-bin bsdtar xz-utils fuse3 fakeroot rpm libssl-dev \
  libgtk-3-dev libwebkit2gtk-4.0-dev

# Windows cross toolchain (GNU)
sudo apt install -y mingw-w64

# Rust toolchain (user-local)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable
rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu

# Android SDK (manual via commandline-tools)
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
export ANDROID_HOME="$ANDROID_SDK_ROOT"
mkdir -p "$ANDROID_SDK_ROOT"
cd /tmp && curl -LO "https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip"
unzip commandlinetools-linux-9477386_latest.zip -d "$ANDROID_SDK_ROOT/cmdline-tools"
mkdir -p "$ANDROID_SDK_ROOT/cmdline-tools/latest"
mv "$ANDROID_SDK_ROOT/cmdline-tools/"cmdline-tools/* "$ANDROID_SDK_ROOT/cmdline-tools/latest/" || true
export PATH="$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:$ANDROID_SDK_ROOT/platform-tools:$PATH"
yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --licenses
sdkmanager --sdk_root="$ANDROID_SDK_ROOT" "platform-tools" "platforms;android-33" \
  "build-tools;33.0.2" "ndk;25.2.9519653"
```

Add to your shell rc to persist:

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"
export PATH="$ANDROID_SDK_ROOT/platform-tools:$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:$JAVA_HOME/bin:$PATH"
```
