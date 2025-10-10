{
  lib,
  pkgs,
  buildNpmPackage,
  pnpm,
  nodejs_20,
  rustPlatform,
}: {
  web = buildNpmPackage {
    name = "EpiCoFront";
    version = "0.0.1";

    src = ../frontend;

    npmLockFile = ../frontend/pnpm-lock.yaml;
    npmDepsHash = "sha256-iwMwRF9bFuQe373N0JKJ5mIJ4fUH9msuc6nA0jsvi9U=";

    nativeBuildInputs = [
      nodejs_20
      pnpm
    ];

    buildPhase = ''
      pnpm run build
    '';

    installPhase = ''
      mkdir -p $out
      cp -r dist/* $out/
    '';

    meta = {
      description = "Web frontend of EpiCovid";
      license = lib.licenses.bsd3;
      maintainers = with lib.maintainers; [cizniarova];
    };
  };

  desktop = rustPlatform.buildRustPackage (finalAttrs: {
    pname = "EpiCoDesk";
    version = "1.0.0";

    src = pkgs.lib.cleanSource ../frontend;

    cargoRoot = "src-tauri";
    buildAndTestSubdir = finalAttrs.cargoRoot;
    cargoHash = "sha256-Fxtraeoy+dAu6JCVSsJECExzSDp2JDJqc8tnRqSqMuU=";

    npmDeps = pkgs.fetchNpmDeps {
      name = "${finalAttrs.pname}-${finalAttrs.version}-npm-deps";
      inherit (finalAttrs) src;
      hash = "sha256-iwMwRF9bFuQe373N0JKJ5mIJ4fUH9msuc6nA0jsvi9U=";
    };

    nativeBuildInputs =
      [
        pkgs.cargo-tauri.hook

        nodejs_20
        pnpm
        pkgs.npmHooks.npmConfigHook

        pkgs.pkg-config
      ]
      ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
        pkgs.wrapGAppsHook4
      ];

    buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux [
      pkgs.glib-networking
      pkgs.openssl
      pkgs.webkitgtk_4_1
    ];

    tauriBuildFlags = ["--debug"];

    meta = {
      description = "Desktop Tauri frontend for EpiCovid";
      license = lib.licenses.bsd3;
      maintainers = with lib.maintainers; [cizniarova];
    };
  });
}
