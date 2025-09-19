{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    git-hooks,
  }: let
    systems = ["x86_64-linux"];

    # helper to apply a function to all systems
    forAllSystems = f:
      builtins.listToAttrs (map (system: {
          name = system;
          value = f system;
        })
        systems);
  in {
    checks = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in {
      pre-commit-check = git-hooks.lib.${system}.run {
        src = ./.;
        hooks = pkgs.lib.genAttrs [
          "convco"
          "trim-trailing-whitespace"
          "biome"
          "isort"
          "black"
          "alejandra"
          "deadnix"
        ] (_: {enable = true;});
      };
    });

    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        config.android_sdk.accept_license = true;
      };

      pyEnv = pkgs.python3.withPackages (p:
        with p; [
          aiocache
          aiohttp
          aiosqlite
          bcrypt
          email-validator
          fastapi
          fastapi-cli
          httpx
          isort
          jinja2
          markdownify
          passlib
          pydantic
          pydantic-settings
          pyjwt
          pytest
          python-dotenv
          python-multipart
          ruff
          sqlmodel
          uvicorn
        ]);

      frontDeps = with pkgs; [
        eslint
        nodejs
        typescript
        biome
        vite
        eas-cli
        android-studio-full
        androidenv.androidPkgs.androidsdk
        androidenv.androidPkgs.emulator
        webkitgtk_4_1
        cargo-tauri
        cargo
        pkg-config
        gobject-introspection
        at-spi2-atk
        atkmm
        cairo
        gdk-pixbuf
        glib
        gtk3
        harfbuzz
        librsvg
        libsoup_3
        pango
        webkitgtk_4_1
        openssl
        rustup
        jq
        rustc
        librsvg
      ];

      backDeps = with pkgs; [
        pyEnv
        black
        isort
        python3
      ];
    in {
      default = pkgs.mkShell {
        env = rec {
          ANDROID_SDK_ROOT = "${pkgs.androidenv.androidPkgs.androidsdk}/libexec/android-sdk";
          ANDROID_HOME = ANDROID_SDK_ROOT;
          PATH = "$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH";
        };
        packages = [pyEnv] ++ frontDeps ++ backDeps;

        # 👇 ensures Rust toolchain is always ready
        shellHook = ''
          ${self.checks.${system}.pre-commit-check.shellHook}
          if ! rustc --version >/dev/null 2>&1; then
            echo "📦 Installing Rust stable toolchain..."
            rustup default stable
          fi
        '';
      };
    });
  };
}
