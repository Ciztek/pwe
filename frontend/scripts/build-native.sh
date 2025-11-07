#!/usr/bin/env bash
set -euo pipefail

# Minimal helper to build mobile (Capacitor Android) and desktop (Tauri) artifacts.
# Usage:
#   ./build-native.sh android-apk
#   ./build-native.sh android-aab
#   ./build-native.sh tauri-linux
#   ./build-native.sh tauri-windows
#   ./build-native.sh all

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_DIR="$ROOT_DIR/.tauri-tools"
ANDROID_DIR="$ROOT_DIR/android"
TAURI_DIR="$ROOT_DIR/src-tauri"

# Central build output folder for all packaged artifacts
BUILD_DIR="$ROOT_DIR/.build"
mkdir -p "$BUILD_DIR"

readonly PKG_MANAGER="$(if [ -f "$ROOT_DIR/pnpm-lock.yaml" ] && command -v pnpm >/dev/null 2>&1; then echo "pnpm"; elif command -v npm >/dev/null 2>&1; then echo "npm"; else echo "npm"; fi)"

echo "Root: $ROOT_DIR"
echo "Package manager: $PKG_MANAGER"
# Note: verbose/debug info is printed after argument parsing so variables exist

run_install() {
  echo "==> Installing node dependencies (if needed)"
  if [ "$PKG_MANAGER" = "pnpm" ]; then
    pnpm install --frozen-lockfile || pnpm install
  else
    npm ci || npm install
  fi
}

# Global flags
VERBOSE=0
SKIP_WEB=0

vecho() {
  if [ "$VERBOSE" -eq 1 ]; then
    echo "$@"
  fi
}

build_web() {
  echo "==> Building web assets"
  if [ "$PKG_MANAGER" = "pnpm" ]; then
    pnpm run build
  else
    npm run build
  fi
}

# --- Android JDK helpers (prefer JDK 21, fallback to 17) ---
find_java_home_by_major() {
  local major="$1"
  # If JAVA_HOME is set and matches the desired major, use it
  if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/java" ]; then
    local v
    v="$("$JAVA_HOME/bin/java" -version 2>&1 | awk -F '"' '/version/ {print $2; exit}')"
    case "$v" in
      ${major}*|1.${major}.*) echo "$JAVA_HOME"; return 0 ;;
    esac
  fi
  # Common locations on Ubuntu and other distros
  local candidates=(
    "/usr/lib/jvm/java-${major}-openjdk-amd64"
    "/usr/lib/jvm/java-${major}-openjdk"
    "/usr/lib/jvm/temurin-${major}-jdk-amd64"
    "/usr/lib/jvm/temurin-${major}-jdk"
    "/usr/lib/jvm/zulu-${major}-amd64"
  )
  # Add wildcard as last resort
  candidates+=(/usr/lib/jvm/*-${major}-*)
  for p in "${candidates[@]}"; do
    if [ -d "$p" ] && [ -x "$p/bin/java" ]; then
      local v
      v="$("$p/bin/java" -version 2>&1 | awk -F '"' '/version/ {print $2; exit}')"
      case "$v" in
        ${major}*|1.${major}.*) echo "$p"; return 0 ;;
      esac
    fi
  done
  return 1
}

ensure_android_jdk() {
  local JH
  if JH="$(find_java_home_by_major 21)"; then
    export ORG_GRADLE_JAVA_HOME="$JH"
    export JAVA_HOME="$JH"
    export PATH="$JH/bin:$PATH"
    vecho "Using JDK 21 at: $JH"
    return 0
  fi
  echo "Android build requires JDK 21. Current: $(java -version 2>&1 | head -n1 || echo 'java not found')"
  echo "Install on Ubuntu: sudo apt update && sudo apt install -y openjdk-21-jdk"
  echo "Then re-run the build."
  return 1
}

build_android_apk() {
  echo "==> Building Android APK (Capacitor)"
  cd "$ROOT_DIR"
  # Load local signing env if present (not committed)
  if [ -f "$ANDROID_DIR/keystore.env" ]; then
    # shellcheck disable=SC1090
    . "$ANDROID_DIR/keystore.env"
    # Ensure variables are exported for Gradle (System.getenv)
    for __v in EPICOVID_STORE_FILE EPICOVID_STORE_PASSWORD EPICOVID_KEY_ALIAS EPICOVID_KEY_PASSWORD; do
      if [ -n "${!__v:-}" ]; then export "${__v}"; fi
    done
  fi
  ensure_android_jdk || return 1
  # ensure web assets copied
  npx cap sync android
  if [ ! -d "$ANDROID_DIR" ]; then
    echo "Android project not found at $ANDROID_DIR"
    return 1
  fi
  cd "$ANDROID_DIR"
  echo "Running gradle assembleRelease (this requires Android SDK/NDK, JDK, ANDROID_HOME set)"
  ./gradlew --stop || true
  ./gradlew assembleRelease
  APK_DIR="$ANDROID_DIR/app/build/outputs/apk/release"
  echo "APK: $APK_DIR"
  mkdir -p "$BUILD_DIR/android"
  shopt -s nullglob; files=("$APK_DIR"/*); shopt -u nullglob
  if [ ${#files[@]} -gt 0 ]; then
    cp -a "$APK_DIR"/* "$BUILD_DIR/android/" || true
    echo "Copied APK(s) to $BUILD_DIR/android/"
    if [ -z "${EPICOVID_STORE_FILE:-}" ] || [ -z "${EPICOVID_STORE_PASSWORD:-}" ] || [ -z "${EPICOVID_KEY_ALIAS:-}" ] || [ -z "${EPICOVID_KEY_PASSWORD:-}" ]; then
      echo "Note: Release APK likely unsigned (signing env vars not set). Set:"
      echo "  EPICOVID_STORE_FILE, EPICOVID_STORE_PASSWORD, EPICOVID_KEY_ALIAS, EPICOVID_KEY_PASSWORD"
      echo "Then re-run: ./scripts/build-native.sh android-apk -s"
    else
      echo "Release APK should be signed using provided keystore env vars."
    fi
  else
    echo "No APKs found in $APK_DIR (build likely failed)."
  fi
}

build_android_aab() {
  echo "==> Building Android AAB (bundle)"
  cd "$ROOT_DIR"
  if [ -f "$ANDROID_DIR/keystore.env" ]; then
    # shellcheck disable=SC1090
    . "$ANDROID_DIR/keystore.env"
  fi
  ensure_android_jdk || return 1
  npx cap sync android
  cd "$ANDROID_DIR"
  ./gradlew bundleRelease
  AAB_DIR="$ANDROID_DIR/app/build/outputs/bundle/release"
  echo "AAB: $AAB_DIR"
  mkdir -p "$BUILD_DIR/android"
  shopt -s nullglob; files=("$AAB_DIR"/*); shopt -u nullglob
  if [ ${#files[@]} -gt 0 ]; then
    cp -a "$AAB_DIR"/* "$BUILD_DIR/android/" || true
    echo "Copied AAB(s) to $BUILD_DIR/android/"
  else
    echo "No AABs found in $AAB_DIR (build likely failed)."
  fi
}

build_android_apk_debug() {
  echo "==> Building Android APK (Debug)"
  cd "$ROOT_DIR"
  ensure_android_jdk || return 1
  npx cap sync android
  cd "$ANDROID_DIR"
  echo "Running gradle assembleDebug"
  ./gradlew --stop || true
  ./gradlew assembleDebug
  local DBG_APK="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
  if [ -f "$DBG_APK" ]; then
    mkdir -p "$BUILD_DIR/android"
    cp -a "$DBG_APK" "$BUILD_DIR/android/" || true
    echo "Copied debug APK to $BUILD_DIR/android/"
  else
    echo "Debug APK not found at $DBG_APK"
  fi
}

ensure_tauri_tools() {
  mkdir -p "$TOOLS_DIR"
  # small helper to download a file if missing
  dl_if_missing() {
    local url="$1"
    local dest="$2"
    if [ ! -f "$dest" ]; then
      echo "Downloading: $url -> $dest"
      curl -fsSL "$url" -o "$dest"
      chmod +x "$dest" || true
    fi
  }

  # These are the common resources Tauri tries to use for AppImage bundling.
  # We keep them local and add to PATH so tauri/linuxdeploy can find them.
  dl_if_missing "https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-x86_64" "$TOOLS_DIR/AppRun-x86_64"
  dl_if_missing "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage" "$TOOLS_DIR/linuxdeploy-x86_64.AppImage"
  dl_if_missing "https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh" "$TOOLS_DIR/linuxdeploy-plugin-gtk.sh"
  dl_if_missing "https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gstreamer/master/linuxdeploy-plugin-gstreamer.sh" "$TOOLS_DIR/linuxdeploy-plugin-gstreamer.sh"
  dl_if_missing "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage" "$TOOLS_DIR/linuxdeploy-plugin-appimage-x86_64.AppImage"

  # make sure PATH includes the tools dir for this script invocation
  export PATH="$TOOLS_DIR:$PATH"
  echo "Tauri helper tools ensured in $TOOLS_DIR (added to PATH for this shell only)"

  # Create small wrapper launchers for AppImage tools to avoid failures on systems
  # without FUSE or where AppImage isn't directly executable in this environment
  create_appimage_wrapper() {
    local appimg="$1"
    local wrapper="$2"
    if [ -f "$appimg" ]; then
      cat > "$wrapper" <<'EOF'
#!/usr/bin/env bash
# Robust AppImage wrapper: try direct execution, then fallback to extraction into a temp dir
set -euo pipefail
APPIMAGE_PATH="__APPIMAGE_PATH__"

# Try direct execution first (guard errors without exiting due to set -e)
set +e
"${APPIMAGE_PATH}" "$@"
code=$?
set -e
if [ $code -eq 0 ]; then exit 0; fi

# Fallback: extract and run inside temporary dir (avoids FUSE requirement)
TMPDIR="\$(mktemp -d /tmp/appimage-extract.XXXXXX)"
cleanup() {
  rm -rf "\${TMPDIR}" || true
}
trap cleanup EXIT

echo "[appimage-wrapper] extracting \${APPIMAGE_PATH} to \${TMPDIR}"
APPIMAGE_EXTRACT_AND_RUN=1 "${APPIMAGE_PATH}" --appimage-extract >/dev/null 2>&1 || {
  echo "[appimage-wrapper] failed to extract \${APPIMAGE_PATH}" >&2
  exit 1
}

if [ -x "\${TMPDIR}/squashfs-root/AppRun" ]; then
  echo "[appimage-wrapper] running extracted AppRun"
  exec "${TMPDIR}/squashfs-root/AppRun" "$@"
elif [ -x "\${TMPDIR}/AppRun" ]; then
  exec "${TMPDIR}/AppRun" "$@"
else
  echo "[appimage-wrapper] no AppRun found in extracted AppImage" >&2
  exit 1
fi
EOF
      # Inject the real AppImage path into the wrapper content
      sed -i "s#__APPIMAGE_PATH__#${appimg//\//\\/}#g" "$wrapper"
      chmod +x "$wrapper" || true
      vecho "Created AppImage wrapper: $wrapper -> $appimg"
    fi
  }

  # Create wrappers for the AppImage-based helpers so tauri/linuxdeploy can invoke them.
  create_appimage_wrapper "$TOOLS_DIR/linuxdeploy-x86_64.AppImage" "$TOOLS_DIR/linuxdeploy"
  create_appimage_wrapper "$TOOLS_DIR/linuxdeploy-plugin-appimage-x86_64.AppImage" "$TOOLS_DIR/linuxdeploy-plugin-appimage"

  # Ensure shell plugins are executable
  chmod +x "$TOOLS_DIR/linuxdeploy-plugin-gtk.sh" "$TOOLS_DIR/linuxdeploy-plugin-gstreamer.sh" || true
}


# Pre-flight checks for Tauri bundling on Linux. Prints hints for common distros.
preflight_tauri_checks() {
  vecho "Running Tauri preflight checks"
  local missing=()
  command -v patchelf >/dev/null 2>&1 || missing+=(patchelf)
  command -v pkg-config >/dev/null 2>&1 || missing+=(pkg-config)
  # libglib tooling (glib-compile-schemas) is part of libglib2.0-bin on Debian
  command -v glib-compile-schemas >/dev/null 2>&1 || missing+=(libglib)
  # fuse binary presence
  command -v fusermount >/dev/null 2>&1 || command -v fuse >/dev/null 2>&1 || missing+=(fuse)
  # mksquashfs is required by appimage tooling (squashfs-tools)
  command -v mksquashfs >/dev/null 2>&1 || missing+=(squashfs-tools)

  # Optional but recommended: desktop-file-validate and appstreamcli (do not fail if missing)
  local optional_missing=()
  command -v desktop-file-validate >/dev/null 2>&1 || optional_missing+=(desktop-file-utils)
  command -v appstreamcli >/dev/null 2>&1 || optional_missing+=(appstream)

  # Rust toolchain required for tauri builds
  command -v cargo >/dev/null 2>&1 || missing+=(cargo)
  command -v rustc >/dev/null 2>&1 || missing+=(rustc)

  if [ ${#missing[@]} -ne 0 ]; then
    echo "Tauri preflight: missing system dependencies: ${missing[*]}"
    # Print distro-specific install hints when /etc/os-release is available
    if [ -f /etc/os-release ]; then
      . /etc/os-release
      case "${ID:-}" in
        debian|ubuntu|linuxmint)
          echo "Install with: sudo apt update && sudo apt install -y patchelf pkg-config libglib2.0-bin fuse squashfs-tools"
          ;;
        fedora|rhel|centos)
          echo "Install with: sudo dnf install -y patchelf pkgconfig glib2-utils fuse squashfs-tools"
          ;;
        arch)
          echo "Install with: sudo pacman -Syu patchelf pkgconf glib2 fuse2 squashfs-tools"
          ;;
        *)
          echo "Install packages for your distro: patchelf pkg-config (or pkgconf) glib utilities (glib-compile-schemas), fuse, and squashfs-tools (mksquashfs)."
          ;;
      esac
    else
      echo "Install packages: patchelf pkg-config glib utilities (glib-compile-schemas), fuse, and squashfs-tools (mksquashfs)."
    fi
      return 1
  fi

  if [ ${#optional_missing[@]} -ne 0 ]; then
    echo "Note: optional tools missing: ${optional_missing[*]} (recommended for better metadata validation)."
    if [ -f /etc/os-release ]; then
      . /etc/os-release
      case "${ID:-}" in
        debian|ubuntu|linuxmint)
          echo "You can install them with: sudo apt install -y desktop-file-utils appstream"
          ;;
        fedora|rhel|centos)
          echo "You can install them with: sudo dnf install -y desktop-file-utils appstream"
          ;;
        arch)
          echo "You can install them with: sudo pacman -Syu desktop-file-utils appstream"
          ;;
      esac
    fi
  fi

  # Check GTK/WebKit dev packages via pkg-config (required to compile wry/webkitgtk stack)
  if command -v pkg-config >/dev/null 2>&1; then
    local pc_missing=()
    # Minimums aligned with errors in your log and common Tauri requirements
    pkg-config --exists 'glib-2.0 >= 2.70'        || pc_missing+=(glib-2.0)
    pkg-config --exists 'gobject-2.0 >= 2.70'     || pc_missing+=(gobject-2.0)
    pkg-config --exists 'gio-2.0 >= 2.70'         || pc_missing+=(gio-2.0)
    pkg-config --exists 'gdk-3.0 >= 3.22'         || pc_missing+=(gdk-3.0)
    pkg-config --exists 'pango >= 1.40'           || pc_missing+=(pango)
    pkg-config --exists 'cairo >= 1.16'           || pc_missing+=(cairo)
    pkg-config --exists 'gdk-pixbuf-2.0 >= 2.42'  || pc_missing+=(gdk-pixbuf-2.0)
    pkg-config --exists 'atk >= 2.36'             || pc_missing+=(atk)
    # WebKitGTK (version may vary by distro)
    if ! pkg-config --exists 'webkit2gtk-4.1' && ! pkg-config --exists 'webkit2gtk-4.0'; then
      pc_missing+=(webkit2gtk)
    fi

    if [ ${#pc_missing[@]} -ne 0 ]; then
      echo "Tauri preflight: missing development packages (pkg-config): ${pc_missing[*]}"
      if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "${ID:-}" in
          debian|ubuntu|linuxmint)
            echo "Install with (Ubuntu/Debian):"
            echo "  sudo apt update && sudo apt install -y build-essential pkg-config libgtk-3-dev libglib2.0-dev libgdk-pixbuf-2.0-dev libpango1.0-dev libcairo2-dev libatk1.0-dev libayatana-appindicator3-dev libwebkit2gtk-4.1-dev || sudo apt install -y libwebkit2gtk-4.0-dev"
            ;;
          fedora|rhel|centos)
            echo "Install with (Fedora/RHEL):"
            echo "  sudo dnf install -y gcc gcc-c++ make pkgconfig gtk3-devel glib2-devel gdk-pixbuf2-devel pango-devel cairo-devel atk-devel webkit2gtk4.1-devel || sudo dnf install -y webkit2gtk3-devel"
            ;;
          arch)
            echo "Install with (Arch):"
            echo "  sudo pacman -Syu base-devel pkgconf gtk3 glib2 gdk-pixbuf2 pango cairo atk webkit2gtk"
            ;;
          *)
            echo "Install the development headers for GTK3 + WebKitGTK (webkit2gtk) for your distro."
            ;;
        esac
      else
        echo "Install the development headers for GTK3 + WebKitGTK (webkit2gtk) for your distro."
      fi
      return 1
    fi
  fi
  vecho "Tauri preflight checks passed"
    # Quick runtime sanity check: try running `cargo metadata` inside tauri dir to surface permission errors
    if command -v cargo >/dev/null 2>&1; then
      if ! (cd "$TAURI_DIR" && cargo metadata --format-version 1 --no-deps >/dev/null 2>&1); then
        echo "Warning: 'cargo metadata' failed when run in $TAURI_DIR. This often indicates missing Rust toolchain components or permission issues."
        echo "Run: cd $TAURI_DIR && cargo metadata --format-version 1 --no-deps to see the error."
        echo "Install Rust (rustup) or use Nix to provide cargo/rustc. Example: 'curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh' or 'nix shell nixpkgs#rustc nixpkgs#cargo --run "./scripts/build-native.sh tauri-linux"'"
        return 1
      fi
    fi
  return 0
}


build_tauri_linux() {
  echo "==> Building Tauri (Linux)"
  cd "$ROOT_DIR"
  # ensure local linuxdeploy and friends so tauri can run them (avoids 'failed to run linuxdeploy')
  ensure_tauri_tools
  # Enforce preflight checks
  mkdir -p "$BUILD_DIR/logs"
  preflight_tauri_checks || {
    echo "Preflight checks failed — aborting Tauri linux build. See hints above to install required system packages."
    return 1
  }

  # Hint tauri-bundler to use our local tools explicitly
  export APPIMAGE_EXTRACT_AND_RUN=1
  # Prefer system linuxdeploy if available, else fallback to local wrappers
  if command -v linuxdeploy >/dev/null 2>&1 && command -v linuxdeploy-plugin-appimage >/dev/null 2>&1; then
    export TAURI_BUNDLER_LINUXDEPLOY="$(command -v linuxdeploy)"
    export TAURI_BUNDLER_LINUXDEPLOY_PLUGIN_APPIMAGE="$(command -v linuxdeploy-plugin-appimage)"
    vecho "Using system linuxdeploy: $TAURI_BUNDLER_LINUXDEPLOY"
  else
    export TAURI_BUNDLER_LINUXDEPLOY="$TOOLS_DIR/linuxdeploy"
    export TAURI_BUNDLER_LINUXDEPLOY_PLUGIN_APPIMAGE="$TOOLS_DIR/linuxdeploy-plugin-appimage"
    vecho "Using local linuxdeploy wrappers from $TOOLS_DIR"
  fi
  export TAURI_BUNDLER_APPIMAGE_APPDIR_APP_RUN="$TOOLS_DIR/AppRun-x86_64"

  # The function accepts an optional param:
  #   --all   -> try to build all supported bundles (AppImage, deb, rpm, ...)
  #   <distro> -> request bundles for a specific distro (script will build all and then filter)
  local subarg="${1:-}"
  local filter_pattern=""

  if [ -n "$subarg" ]; then
    if [ "$subarg" = "--all" ] || [ "$subarg" = "all" ]; then
      export BUILD_ALL_TAURI_BUNDLES=1
      vecho "Requested: build all tauri bundles"
    else
      # treat subarg as distro name and build all bundles, then pick the one that matches
      DISTRO_LC="$(echo "$subarg" | tr '[:upper:]' '[:lower:]')"
      case "$DISTRO_LC" in
        ubuntu|debian|mint)
          filter_pattern='*.deb'
          ;;
        fedora|centos|rhel)
          filter_pattern='*.rpm'
          ;;
        arch|manjaro)
          # arch typically uses tar.xz or tar.gz packages for distribution
          filter_pattern='*.tar.*'
          ;;
        generic|appimage)
          filter_pattern='*.AppImage'
          ;;
        *)
          vecho "Unknown distro hint: $subarg; will attempt build and copy whatever is produced"
          filter_pattern=''
          ;;
      esac
      export BUILD_ALL_TAURI_BUNDLES=1
    fi
  else
    # default behaviour: respect BUILD_ALL_TAURI_BUNDLES env if set externally, otherwise prefer distro-default
    export BUILD_ALL_TAURI_BUNDLES="${BUILD_ALL_TAURI_BUNDLES:-0}"
    if [ "$BUILD_ALL_TAURI_BUNDLES" -eq 0 ]; then
      # try to detect distro for informational purposes
      if [ -f /etc/os-release ]; then
        . /etc/os-release
        vecho "Detected distro: ${ID:-unknown} (${ID_LIKE:-})"
      else
        vecho "Could not detect distro (/etc/os-release not present)"
      fi
    fi
  fi

  vecho "BUILD_ALL_TAURI_BUNDLES=$BUILD_ALL_TAURI_BUNDLES"

  # Build targeting the native Linux triple. Capture detailed logs for debugging.
  LOG1="$BUILD_DIR/logs/tauri-linux-build-1.log"
  LOG2="$BUILD_DIR/logs/tauri-linux-build-2-fallback.log"

  echo "Running tauri build (attempt 1) — logs: $LOG1"
  if ! npx tauri build --target x86_64-unknown-linux-gnu 2>&1 | tee "$LOG1"; then
    echo "Tauri build failed on attempt 1. Will retry with BUILD_ALL_TAURI_BUNDLES=0 (fallback). See $LOG1 for details."
    export BUILD_ALL_TAURI_BUNDLES=0
    echo "Running tauri build (attempt 2 - fallback) — logs: $LOG2"
    if ! npx tauri build --target x86_64-unknown-linux-gnu 2>&1 | tee "$LOG2"; then
      echo "Tauri build failed on fallback as well. See logs: $LOG1 and $LOG2"
      return 1
    fi
  fi

  BUNDLE_DIR="$TAURI_DIR/target/release/bundle"
  echo "Tauri bundles: $BUNDLE_DIR"
  if [ -d "$BUNDLE_DIR" ]; then
    mkdir -p "$BUILD_DIR/tauri/linux"
    if [ -n "$filter_pattern" ]; then
      # copy only matching files (filter pattern may include shell glob chars)
      shopt_saved=$(set +o)
      # use cp with glob; fallback to copying everything if nothing matches
      matches=$(ls $BUNDLE_DIR/ 2>/dev/null | grep -E "$(echo "$filter_pattern" | sed 's/\./\\./g; s/\*/.*/g')" || true)
      if [ -n "$matches" ]; then
        cp -a $BUNDLE_DIR/$filter_pattern "$BUILD_DIR/tauri/linux/" || true
        echo "Copied filtered Tauri Linux bundles ($filter_pattern) to $BUILD_DIR/tauri/linux/"
      else
        echo "No bundles matching pattern '$filter_pattern' found; copying all bundles instead"
        cp -a "$BUNDLE_DIR"/* "$BUILD_DIR/tauri/linux/" || true
        echo "Copied Tauri Linux bundles to $BUILD_DIR/tauri/linux/"
      fi
      eval "$shopt_saved" || true
    else
      cp -a "$BUNDLE_DIR"/* "$BUILD_DIR/tauri/linux/" || true
      echo "Copied Tauri Linux bundles to $BUILD_DIR/tauri/linux/"
    fi
  fi
}

build_tauri_windows() {
  echo "==> Building Tauri (Windows)"
  # Cross-building for Windows on Linux is complex (MSVC toolchain needed) — prefer building on native Windows.
  if [ "$(uname -s | tr '[:upper:]' '[:lower:]')" != "mingw"* ] && [ "$(uname -s | tr '[:upper:]' '[:lower:]')" != "cygwin"* ] && [ "$(uname -s)" != "Windows_NT" ]; then
    echo "Note: you are not on Windows. Building Windows installers on Linux requires a cross toolchain or a Windows build environment."
    echo "Recommended options:"
    echo " - Build on a Windows machine (run: pnpm run build && npx tauri build --target x86_64-pc-windows-msvc)"
    echo " - Use CI (GitHub Actions with windows-latest) to produce Windows artifacts"
    echo "Attempting a best-effort build (may fail):"
    # Prefer system NSIS (Linux makensis) if available to avoid trying to run makensis.exe
    if command -v makensis >/dev/null 2>&1; then
      export TAURI_BUNDLER_NSIS_BIN="$(command -v makensis)"
      vecho "Using system NSIS: $TAURI_BUNDLER_NSIS_BIN"
    else
      echo "NSIS 'makensis' not found. Installer bundling may fail. Install with: sudo apt install -y nsis"
    fi
    # Try to build; even if bundling fails (makensis/msi), still copy the .exe
    if ! npx tauri build --target x86_64-pc-windows-gnu; then
      echo "Windows bundling failed — will try to collect built .exe if available."
    fi
  else
    # On Windows environment
    npx tauri build --target x86_64-pc-windows-msvc
  fi

  W_BUNDLE_DIR="$TAURI_DIR/target/release/bundle"
  echo "Windows bundles: $W_BUNDLE_DIR"
  if [ -d "$W_BUNDLE_DIR" ]; then
    # Copy only the Windows installer(s) (NSIS) into .build, not the application exe
    installers=()
    if [ -d "$W_BUNDLE_DIR/nsis" ]; then
      # Prefer *setup*.exe naming produced by NSIS bundler
      while IFS= read -r -d '' f; do installers+=("$f"); done < <(find "$W_BUNDLE_DIR/nsis" -maxdepth 1 -type f -name "*setup*.exe" -print0)
      # Fallback: any .exe under nsis dir
      if [ ${#installers[@]} -eq 0 ]; then
        while IFS= read -r -d '' f; do installers+=("$f"); done < <(find "$W_BUNDLE_DIR/nsis" -maxdepth 1 -type f -name "*.exe" -print0)
      fi
    fi
    # Fallback: search anywhere under bundle for *setup*.exe
    if [ ${#installers[@]} -eq 0 ]; then
      while IFS= read -r -d '' f; do installers+=("$f"); done < <(find "$W_BUNDLE_DIR" -type f -name "*setup*.exe" -print0)
    fi

    if [ ${#installers[@]} -gt 0 ]; then
      mkdir -p "$BUILD_DIR/tauri/windows"
      for f in "${installers[@]}"; do
        cp -a "$f" "$BUILD_DIR/tauri/windows/" || true
      done
      echo "Copied Windows installer(s) to $BUILD_DIR/tauri/windows/"
    else
      echo "No Windows installer (.exe) found in $W_BUNDLE_DIR. Ensure NSIS bundling succeeded."
      echo "Tip: on Linux, install NSIS (sudo apt install -y nsis) or build on Windows/MSVC."
    fi
  fi
}

print_usage() {
  cat <<'EOF'
Usage: build-native.sh [OPTIONS] <target>

Options:
  -h, --help      Show this help and exit

Targets:
  android-apk     Build an Android APK using Capacitor/Gradle (requires Android SDK/JDK/ANDROID_HOME)
  android-aab     Build an Android App Bundle (AAB) using Gradle (requires Android SDK/JDK/ANDROID_HOME)
  tauri-linux     Build Tauri desktop bundles for Linux (AppImage/deb/rpm where supported)
  tauri-windows   Build Tauri desktop bundles for Windows (prefer building on Windows/CI)
  android-apk-debug Build a debug-signed APK for quick install via adb (no release signing)
  all             Run web build and attempt all platform builds; collects artifacts under .build/

Notes / prerequisites (short):
  - Node toolchain: pnpm (preferred) or npm
  - Capacitor Android: Android SDK, JDK, ANDROID_HOME set in environment
  - Tauri: Rust (stable), system deps (libgtk, libglib, patchelf, fuse on some distros)
  - Windows installer: building on Windows or CI is recommended (MSVC toolchain)

Examples:
  ./scripts/build-native.sh android-apk
  ./scripts/build-native.sh tauri-linux
  ./scripts/build-native.sh all

For more detailed troubleshooting tips, open the script and read the comments near the tauri build helpers.
EOF
}

# Parse flags (support short and long options) using getopt so flags may appear anywhere.
## Portable flag parsing: accept global flags anywhere and leave unknown args intact
new_args=()
while [ $# -gt 0 ]; do
  case "$1" in
    -h|--help)
      print_usage
      exit 0
      ;;
    -v|--verbose)
      VERBOSE=1
      shift
      ;;
    -s|--skip-web)
      SKIP_WEB=1
      shift
      ;;
    --)
      shift
      while [ $# -gt 0 ]; do new_args+=("$1"); shift; done
      break
      ;;
    -*)
      # Unknown global option: fail
      echo "Unknown global option: $1"
      print_usage
      exit 2
      ;;
    *)
      new_args+=("$1")
      shift
      ;;
  esac
done

# Restore remaining args (first non-option is target)
set -- "${new_args[@]:-}"
main_target="${1:-}"
shift || true
target_args=("$@")

if [ -z "$main_target" ]; then
  print_usage
  exit 2
fi

case "$main_target" in
  android-apk)
    run_install
    if [ "$SKIP_WEB" -eq 0 ]; then build_web; else vecho "Skipping web build (SKIP_WEB=1)"; fi
    build_android_apk
    ;;

  android-aab)
    run_install
    if [ "$SKIP_WEB" -eq 0 ]; then build_web; else vecho "Skipping web build (SKIP_WEB=1)"; fi
    build_android_aab
    ;;

  android-apk-debug)
    run_install
    if [ "$SKIP_WEB" -eq 0 ]; then build_web; else vecho "Skipping web build (SKIP_WEB=1)"; fi
    build_android_apk_debug
    ;;

  tauri-linux)
    run_install
    if [ "$SKIP_WEB" -eq 0 ]; then build_web; else vecho "Skipping web build (SKIP_WEB=1)"; fi
    build_tauri_linux "${target_args[@]}"
    ;;

  tauri-windows)
    run_install
    if [ "$SKIP_WEB" -eq 0 ]; then build_web; else vecho "Skipping web build (SKIP_WEB=1)"; fi
    build_tauri_windows "${target_args[@]}"
    ;;

  all)
    run_install
    build_web
    # copy web assets
    if [ -d "$ROOT_DIR/dist" ]; then
      mkdir -p "$BUILD_DIR/web"
      cp -a "$ROOT_DIR/dist"/* "$BUILD_DIR/web/" || true
      echo "Copied web assets to $BUILD_DIR/web/"
    fi

    build_android_apk || true
    build_android_aab || true
    build_tauri_linux || true
    build_tauri_windows || true

    # iOS sync (only sync; IPA creation must be done on macOS/Xcode)
    if command -v npx >/dev/null 2>&1; then
      npx cap sync ios || true
      echo "iOS project synced; open Xcode to build the ipa."
    fi
    echo "All build artifacts (collected) are available under $BUILD_DIR"
    ;;

  *)
    print_usage
    exit 2
    ;;
esac

echo "Done."
