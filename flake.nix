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
      android-composition = pkgs.callPackage ./frontend/composition.nix { };
      front = pkgs.callPackage ./frontend { };
      back = pkgs.callPackage ./back { };
    });
  };
}
