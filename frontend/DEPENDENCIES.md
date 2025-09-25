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

- OpenJDK 11 or 17 (for Android / Gradle builds).
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
