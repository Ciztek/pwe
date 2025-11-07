{ lib
, buildNpmPackage
, nodejs_20
, pnpm
, rustup
, pkg-config
, openssl
, gtk3
, glib
, webkitgtk_4_1
, jdk17
, gradle
, gcc
, gnumake
}:

buildNpmPackage {
  pname = "epicovid-frontend";
  version = "0.0.1";

  src = ./.;

  npmLockFile = ./pnpm-lock.yaml;

  npmBuildScript = "build";

  nativeBuildInputs = [
    nodejs_20
    pnpm
    rustup
    pkg-config
    openssl
    gtk3
    glib
    webkitgtk_4_1
    jdk17
    gradle
    gcc
    gnumake
  ];

  installPhase = ''
    runHook preInstall

    mkdir -p $out
    cp -r dist/* $out/

    runHook postInstall
  '';

  shellHook = ''
    echo "🚀 EpiCovid frontend dev environment ready"
    echo "Node.js version: $(node -v)"
    echo "Rustup available — run 'rustup-init' if first time"
    echo "pnpm available — run 'pnpm install' to set up deps"
    echo "For Android builds: set ANDROID_HOME or ANDROID_SDK_ROOT"
    echo "For WSL/headless: LIBGL_ALWAYS_SOFTWARE=1 is pre-set"
  '';

  # Environment vars useful for dev & CI
  LIBGL_ALWAYS_SOFTWARE = "1";
  MESA_LOADER_DRIVER_OVERRIDE = "llvmpipe";

  meta = {
    description = "Frontend of the EpiCovid project (React + Capacitor + Tauri)";
    license = lib.licenses.bsd3;
    maintainers = with lib.maintainers; [ sigmanificient ];
    platforms = lib.platforms.all;
    mainProgram = "vite";
  };
}
