{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    git-hooks,
  }: let
    applySystems = nixpkgs.lib.genAttrs ["x86_64-linux"];
    forAllSystems = f:
      applySystems (
        system:
          f (import nixpkgs {
            inherit system;
            config = {
              android_sdk.accept_license = true;
              allowUnfree = true;
            };
          })
      );
  in {
    checks = forAllSystems (
      pkgs: {
        pre-commit-check = git-hooks.lib.${pkgs.system}.run {
          src = ./.;
          hooks =
            {
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
      py-env = pkgs.python3.withPackages (_:
        with self.packages.${pkgs.system}.back;
          dependencies ++ optional-dependencies.dev
      );
      # front-web-env = (with self.packages.${pkgs.system}.front-web; nativeBuildInputs);
      front-desktop-env = (with self.packages.${pkgs.system}.front-desktop; nativeBuildInputs);
    in {
      default = pkgs.mkShell {
          packages = [
            py-env
            # front-web-env
            front-desktop-env
          ];
        };
    });
    packages = forAllSystems (pkgs: with pkgs; rec {
      default = pkgs.symlinkJoin {
        name = "epicovid";
        paths = [
          back
          front-web
        ];
      };

      back = python3Packages.buildPythonApplication {
        name = "covid-dataviz";
        version = "0.0.1";
        pyproject = true;

        src = ./back;

        build-system = [python3Packages.hatchling];

        dependencies = with python3Packages; [
          fastapi
          uvicorn
          sqlalchemy
          passlib
          aiosqlite
        ];

        optional-dependencies = with python3Packages; {
          dev = [
            fastapi-cli
            black
            isort
          ];
        };

        meta = {
          description = "Backend of the EpiCovid project (FastAPI)";
          license = lib.licenses.bsd3;
          maintainers = with lib.maintainers; [cizniarova];
          mainProgram = "EpiCoBack";
        };
      };
      front-web = buildNpmPackage {
        name = "EpiCoFront";
        version = "0.0.1";

        src = ./frontend;

        npmLockFile = ./frontend/pnpm-lock.yaml;
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
          description = "Frontend of the EpiCovid project (React + Capacitor + Tauri)";
          license = lib.licenses.bsd3;
          maintainers = with lib.maintainers; [ cizniarova ];
        };
      };
      front-desktop = pkgs.stdenv.mkDerivation rec {
        name = "EpiCoDesk";
        version = "0.0.1";
        src = ./frontend;
        nativeBuildInputs = [
          rustup
          cargo
          gcc
          pkg-config
          openssl
          gtk3
          glib
          webkitgtk_4_1
          nodejs_20
          pnpm
        ];
        frontendNodeModules = self.packages.${pkgs.system}.front-web;
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
        meta = {
          description = "EpiCovid desktop app (Tauri)";
          license = lib.licenses.bsd3;
          maintainers = with lib.maintainers; [ cizniarova ];
        };
      };
      # front-android = pkgs.callPackage ./nix/front-android.nix {
      #   inherit (self.packages.${pkgs.system}) front-web;
      # };
      # front-android = build-gradle-application.packages.${pkgs.system}.default (pkgs: {
      #   pname = "EpiCoBile";
      #   version = "0.0.1";
      #   src = ./frontend;

      #   gradleProjectFile = ./frontend/android/settings.gradle;

      #   gradleDistribution = {
      #     url = "https://services.gradle.org/distributions/gradle-8.11.1-all.zip";
      #     sha256 = "sha256-idTnDk6E4tLfu2Pk2qU+IbJQF8xww35O6jHuUfsVCYo=";
      #   };

      #   gradleDeps = [
      #     (pkgs.fetchurl {
      #       url = "https://dl.google.com/dl/android/maven2/com/android/tools/build/gradle/8.7.2/gradle-8.7.2.jar";
      #       sha256 = "kGyVRG0MGZurpmCORLVJJCoPMmNfzUOvplY3yqLUk/Y=";
      #     })
      #     (pkgs.fetchurl {
      #       url = "https://dl.google.com/dl/android/maven2/com/google/gms/google-services/4.4.2/google-services-4.4.2.jar";
      #       sha256 = "M6xbjCDHycyB6JjZ9Ncvjl8Xo1UFGCaDzYFexpOS7I0=";
      #     })
      #   ];

      #   nativeBuildInputs = with pkgs; [
      #     jdk21_headless
      #     nodejs_20
      #     pnpm
      #   ];

      #   preBuild = ''
      #     cp -r ${self.packages.${pkgs.system}.front-web}/. frontend
      #   '';

      #   gradleTasks = [ "assembleRelease" ];

      #   installPhase = ''
      #     mkdir -p $out
      #     cp -r android/app/build/outputs $out/
      #   '';

      #   meta = {
      #     description = "EpiCovid Android app (Capacitor)";
      #     license = pkgs.lib.licenses.bsd3;
      #     maintainers = with pkgs.lib.maintainers; [ cizniarova ];
      #   };
      # });


      # Sign APK (impure, reads .env at runtime)
      sign-apk = pkgs.writeShellApplication {
        name = "sign-apk";
        runtimeInputs = [ pkgs.jdk17 pkgs.apksigner pkgs.zipalign ];
        text = ''
          set -eu

          ENV_FILE=./.env
          KEYSTORE_FILE= ./release-key.jks

          if [ ! -f "$ENV_FILE" ]; then
            echo "Missing $ENV_FILE. Creating default .env with interactive key generation..."
            mkdir -p ./android

            read -p "Keystore path (default $KEYSTORE_FILE): " ks
            ks=${ks:-$KEYSTORE_FILE}

            read -p "Key alias (default my-key-alias): " alias
            alias=${alias:-my-key-alias}

            read -s -p "Keystore password: " ks_pass
            echo
            read -s -p "Key password: " key_pass
            echo

            keytool -genkeypair \
              -v \
              -keystore "$ks" \
              -alias "$alias" \
              -keyalg RSA \
              -keysize 2048 \
              -validity 10000 \
              -storepass "$ks_pass" \
              -keypass "$key_pass" \
              -dname "CN=Your Name, OU=Dev, O=Company, L=City, S=State, C=US"

            cat > "$ENV_FILE" <<EOF
              ANDROID_KEYSTORE=$ks
              ANDROID_KEYSTORE_PASSWORD=$ks_pass
              ANDROID_KEY_PASSWORD=$key_pass
              ANDROID_KEY_ALIAS=$alias
            EOF

            echo "Generated keystore and .env at $ENV_FILE"
          fi

          # Load .env
          export $(grep -v '^#' "$ENV_FILE" | xargs)

          # Build unsigned APK
          APK_PATH=$(nix build .#android-unsigned --no-link --print-out-paths)/app-release-unsigned.apk

          echo "==> Signing $APK_PATH"
          apksigner sign \
            --ks "$ANDROID_KEYSTORE" \
            --ks-pass "pass:$ANDROID_KEYSTORE_PASSWORD" \
            --key-pass "pass:$ANDROID_KEY_PASSWORD" \
            --out signed-app.apk \
            "$APK_PATH"

          echo "==> Aligning APK"
          zipalign -f -v 4 signed-app.apk aligned-app.apk

          echo "==> Done! Final APK: aligned-app.apk"
        '';
      };
    });
  };
}
