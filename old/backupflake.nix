{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self, nixpkgs, git-hooks
  }: let
    applySystems = nixpkgs.lib.genAttrs ["x86_64-linux"];
    forAllSystems = f: applySystems (system:
      f (import nixpkgs {
        inherit system;
        config = {
          android_sdk.accept_license = true;
          allowUnfree = true;
        };
      })
    );
  in {
    formatter = forAllSystems (pkgs: pkgs.alejandra);

    checks = forAllSystems (
      pkgs: {
        pre-commit-check = git-hooks.lib.${pkgs.system}.run {
          src = ./.;
          hooks = {
            biome = {
              enable = true;
              name = "biome hook (format only)";
              entry = ''
                ${pkgs.lib.getExe pkgs.biome} format --write ./.
              '';
            };
          }
          // pkgs.lib.genAttrs [
            "black"
            "convco"
            "isort"
            "trim-trailing-whitespace"
            "deadnix"
            "alejandra"
          ] (_: {enable = true;});
        };
      }
    );

    devShells = forAllSystems (pkgs: let
      compo = self.packages.${pkgs.system}.android-composition;

      py-env = pkgs.python3.withPackages (_:
        with self.packages.${pkgs.system}.back;
          dependencies ++ optional-dependencies.dev
      );

      front-env = (with self.packages.${pkgs.system}.front; nativeBuildInputs);
    in {
      base = pkgs.mkShell {
        inherit (self.checks.${pkgs.system}.pre-commit-check) shellHook;

        packages = with pkgs; [
          biome
          nodejs
          pnpm
          jdk
          gradle
          py-env
          front-env
        ];
      };

      default = pkgs.mkShell {
        inputsFrom = [ self.devShells.${pkgs.system}.base ];

        env.ANDROID_SDK_ROOT = "${compo.androidsdk}/libexec/android-sdk";

        packages = (with compo; [
          androidsdk
          platform-tools
          build-tools
        ]);
      };

      with-emulator = let
        compo' = compo.override {
          includeEmulator = true;
          includeSystemImages = true;
          abiVersions = [ "x86_64" ];
          systemImageTypes = [ "google_apis" ];
        };
      in pkgs.mkShell {
        inputsFrom = [ self.devShells.${pkgs.system}.base ];

        env.ANDROID_SDK_ROOT = "${compo'.androidsdk}/libexec/android-sdk";

        packages = (with compo'; [
          androidsdk
          emulator
          platform-tools
        ]);
      };
    });

packages = forAllSystems (pkgs: {
  # ✅ Already existing
  android-composition = pkgs.callPackage ./frontend/composition.nix { };
  front = pkgs.callPackage ./frontend { };
  back = pkgs.callPackage ./back { };

  # 🖥️ Desktop build (Tauri)
  front-desktop = pkgs.stdenv.mkDerivation rec {
    pname = "epicovid-frontend-desktop";
    version = "0.0.1";

    src = ./.;

    nativeBuildInputs = [
      pkgs.rustup
      pkgs.cargo
      pkgs.gcc
      pkgs.pkg-config
      pkgs.openssl
      pkgs.gtk3
      pkgs.glib
      pkgs.webkitgtk_4_1
      pkgs.nodejs
      pkgs.pnpm
    ];


    frontendNodeModules = self.packages.${pkgs.system}.front;
    buildPhase = ''
      echo "=== Copying frontend assets and node_modules ==="
      cp -r ${frontendNodeModules}/. $PWD/frontend

      echo "=== Building Tauri app ==="
      cargo tauri build
    '';

    installPhase = ''
      mkdir -p $out
      cp -r target/release/bundle $out/
    '';
  };

  # 📱 Android build (Capacitor)
front-android = pkgs.stdenv.mkDerivation rec {
  pname = "epicovid-frontend-android";
  version = "0.0.1";

  src = ./.;

  nativeBuildInputs = with pkgs; with self.packages.${pkgs.system}.android-composition; [
    gradle
    jdk
    androidsdk
    platform-tools
    build-tools
    pnpm
  ];

  # Reuse already-built frontend node_modules from .#front
  frontendNodeModules = self.packages.${pkgs.system}.front;

  buildPhase = ''
    echo "=== Copying frontend assets and node_modules ==="
    cp -r ${frontendNodeModules}/. $PWD/frontend

    echo "=== Syncing Capacitor project ==="
    cd frontend
    $(which cap) sync android

    echo "=== Running Gradle build ==="
    cd android
    gradle assembleDebug
  '';

  installPhase = ''
    mkdir -p $out
    cp -r android/app/build/outputs $out/
  '';
};

});
  };
}
