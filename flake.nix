{
  description = "development environment for green-ecolution";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    goVersion = 25;
    nodeVersion = 24;

    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [self.overlays.default];
          };
        });
  in {
    overlays.default = final: prev: {
      go = final."go_1_${toString goVersion}";
      nodejs = final."nodejs_${toString nodeVersion}";
    };

    packages = forEachSupportedSystem ({pkgs}: let
      lib = pkgs.lib;

      meta = {
        description = "Green Ecolution – Smart irrigation system to optimize water use, reduce operational workload, and lower costs.";
        longDescription = ''
          Green Ecolution is an intelligent irrigation platform designed to enable sustainable and efficient green space management.
          By optimizing water usage and reducing manual maintenance efforts, it helps organizations save resources and costs, while supporting environmental goals.
        '';
        homepage = "https://green-ecolution.de";
        changelog = "https://github.com/green-ecolution/green-ecolution/releases";
        license = lib.licenses.agpl3Only;
        maintainers = [
          {
            name = "Cedrik Hoffmann";
            email = "choffmann@green-ecolution.de";
            github = "choffmann";
            githubId = 73289312;
          }
        ];
        mainProgram = "green-ecolution";
        platforms = lib.platforms.linux ++ lib.platforms.darwin;
      };

      si = self.sourceInfo or {};
      gitCommit = si.rev or si.dirtyRev or "local";
      gitBranch = "local";

      buildDate =
        if si ? lastModifiedDate
        then builtins.substring 2 6 si.lastModifiedDate
        else "000000";
    in rec {
      frontend = pkgs.stdenv.mkDerivation rec {
        inherit meta;
        pname = "frontend";
        version = "0.3.0"; # x-release-please-version
        src = lib.cleanSource ./frontend;

        nativeBuildInputs = with pkgs; [nodejs pnpm pnpmConfigHook];

        env.VITE_BACKEND_BASEURL = "/api";

        pnpmDeps = pkgs.fetchPnpmDeps {
          inherit pname version src;
          fetcherVersion = 2;
          hash = "sha256-3YCqhIqioc6BE9yfFRmnKn4AmU/Ksdy8V13o4mXXN+w=";
        };

        buildPhase = ''
          runHook preBuild
          pnpm build
          runHook postBuild
        '';

        installPhase = ''
          mkdir -p $out
          cp -r app/dist/* $out/
        '';

        dontFixup = true;
      };

      backend = pkgs.rustPlatform.buildRustPackage rec {
        inherit meta;
        pname = "green-ecolution";
        version = "0.3.0"; # x-release-please-version
        src = lib.cleanSource ./backend;

        cargoLock = {
          lockFile = ./backend/Cargo.lock;
        };

        nativeBuildInputs = with pkgs; [clang lld];

        cargoBuildFlags = ["--bin" "green-ecolution" "--bin" "migrate"];

        # Tests rely on testcontainers (live Postgres)
        doCheck = false;

        env = {
          SQLX_OFFLINE = "true";
          GE_GIT_COMMIT = gitCommit;
          GE_GIT_BRANCH = gitBranch;
          GE_BUILD_TIME = buildDate;
        };
      };

      default = backend;
    });

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        name = "dev-shell";
        packages = with pkgs; [
          git
          # Backend/Go
          go
          gotools
          golangci-lint
          gnumake
          yq-go
          delve
          # Frontend/Node
          nodejs
          pnpm
          # Rust
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer
          sqlx-cli
          clang
          lld
          bacon
          wasm-pack
        ];
        shellHook = ''
          echo "Dev shell loaded 🧪"
          export PS1="(dev) $PS1"
        '';
      };
    });

    formatter = forEachSupportedSystem ({pkgs}: pkgs.alejandra);

    checks = forEachSupportedSystem ({pkgs}: {
      inherit (self.packages.${pkgs.system}) frontend backend default;
    });
  };
}
