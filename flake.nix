{
  description = "Minimal Rust + winit/eframe dev shell with nixGLIntel";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs_pandas.url = "github:NixOS/nixpkgs/1f49db7743a3436f1378f80674f93eb45b3474f4";
    nixgl.url = "github:nix-community/nixGL";
  };

  outputs = { self, nixpkgs, nixgl, nixpkgs_pandas, ... }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    pkgs_pandas = nixpkgs_pandas.legacyPackages.${system};
    nixglPkgs = nixgl.packages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (import ./nix/python.nix { inherit pkgs; pythonVersion = "python313"; })

        rustup
        rustc
        cargo
        rustfmt
        clippy

        pkg-config

        yt-dlp

        alsa-lib
        xorg.libX11
        libxkbcommon
        xorg.libXcursor
        xorg.libXi

        nixglPkgs.nixGLIntel
      ];

      LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
        xorg.libX11
        libxkbcommon
        xorg.libXcursor
        xorg.libXi
        alsa-lib
      ];
    };

    packages.${system} = rec {

      norbert = pkgs_pandas.python310Packages.callPackage ./nix/python/norbert.nix {};
      spleeter = pkgs_pandas.python310Packages.callPackage ./nix/python/spleeter.nix {
        inherit norbert;
        tensorflow = pkgs_pandas.python310Packages.tensorflow;
        numba = pkgs_pandas.python310Packages.numba;
      };
      default = pkgs.rustPlatform.buildRustPackage {
      pname = "pwe-karaoke";
      version = "0.1.0";

      src = ./.;

      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      nativeBuildInputs = with pkgs; [
        rustc
        cargo
        pkg-config
      ];

      buildInputs = with pkgs; [
        alsa-lib
        xorg.libX11
        libxkbcommon
        xorg.libXcursor
        xorg.libXi
      ];

      LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
        xorg.libX11
        libxkbcommon
        xorg.libXcursor
        xorg.libXi
        alsa-lib
      ];
    };
  };
  };
}

