{
  description = "Fullstack app with Tauri + Capacitor + FastAPI (offline buildable)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    pnpm2nix.url = "github:nix-community/pnpm2nix";
    poetry2nix.url = "github:nix-community/poetry2nix";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, pnpm2nix, poetry2nix, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        # Tools
        nodejs = pkgs.nodejs_20;
        python = pkgs.python312;

        # --- Backend (FastAPI) using poetry2nix ---
        backend = poetry2nix.lib.mkPoetryApplication {
          projectDir = ./back;
          python = python;
        };

        # --- Frontend: JS deps from pnpm-lock.yaml ---
        # This creates a pre-fetched, offline Nix store of node_modules
        frontendDeps = pnpm2nix.mkPnpmPackage {
          src = ./frontend;
          lockfile = ./frontend/pnpm-lock.yaml;
          nodejs = nodejs;
        };

        # --- Tauri (Rust) using naersk ---
        tauriApp = naersk.lib."${system}".buildPackage {
          pname = "tauri-desktop";
          root = ./frontend/src-tauri;
          buildInputs = with pkgs; [
            nodejs
            frontendDeps
            gtk3
            webkitgtk
            libsoup
            libxkbcommon
            appstream
          ];
          cargoBuildOptions = [ "--release" ];
          postInstall = ''
            mkdir -p $out/bin
            cp target/release/* $out/bin/
          '';
        };

        # --- Mobile (Capacitor) ---
        mobile = pkgs.stdenv.mkDerivation {
          pname = "capacitor-mobile";
          version = "1.0.0";
          src = ./frontend;
          nativeBuildInputs = [ nodejs frontendDeps ];
          buildPhase = ''
            ln -s ${frontendDeps}/node_modules node_modules
            pnpm run build
          '';
          installPhase = ''
            mkdir -p $out/dist
            cp -r dist $out/dist/
          '';
        };

      in {
        packages = {
          inherit backend tauriApp mobile;
          default = tauriApp;
        };

        devShells.default = pkgs.mkShell {
          name = "fullstack-dev";
          buildInputs = [
            nodejs
            pkgs.rustc pkgs.cargo
            pkgs.python3 poetry2nix.packages.${system}.poetry
            pkgs.git
            pkgs.android-tools
          ];
          shellHook = ''
            echo "🚀 Welcome to your fullstack dev shell"
            echo "Use 'nix build .#tauriApp', '.#mobile', or '.#backend'"
          '';
        };
      });
}
