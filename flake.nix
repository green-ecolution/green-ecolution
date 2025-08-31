{
  description = "Development enviroment for go";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = inputs: let
    goVersion = 24;
    nodeVersion = 24;
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [inputs.self.overlays.default];
          };
        });
  in {
    overlays.default = final: prev: {
      go = final."go_1_${toString goVersion}";
      nodejs = final."nodejs_${toString nodeVersion}";
    };

    packages = forEachSupportedSystem ({pkgs}: let
      meta = {
        description = "Green Ecolution – Smart irrigation system to optimize water use, reduce operational workload, and lower costs.";
        longDescription = ''
          Green Ecolution is an intelligent irrigation platform designed to enable sustainable and efficient green space management.
          By optimizing water usage and reducing manual maintenance efforts, it helps organizations save resources and costs, while supporting environmental goals.
        '';
        homepage = "https://green-ecolution.de";
        changelog = "https://github.com/green-ecolution/green-ecolution/releases";
        license = {
          spdxId = "AGPL-3.0-only";
          fullName = "GNU Affero General Public License v3.0 only";
        };
        maintainers = [
          {
            name = "Cedrik Hoffmann";
            email = "dev@choffmann.io";
            github = "choffmann";
            githubId = 73289312;
          }
        ];
        mainProgram = "green-ecolution";
        platforms = pkgs.lib.platforms.linux ++ pkgs.lib.platforms.darwin;
      };
    in rec {
      frontend = pkgs.stdenv.mkDerivation rec {
        inherit meta;
        pname = "frontend";
        version = "1.3.0-nightly.20250830";
        src = pkgs.fetchFromGitHub {
          owner = "green-ecolution";
          repo = "frontend";
          rev = "965eb919deb05aa89bbb4676fcdaedd103832295";
          hash = "sha256-sO78KtCnfJxonFQV2gpPV7GmNrJpeDilqbv8+RMuUgE=";
        };

        nativeBuildInputs = with pkgs; [
          nodejs
          pnpm.configHook
        ];

        VITE_BACKEND_BASEURL = "/api";
        pnpmDeps = pkgs.pnpm.fetchDeps {
          inherit pname version src;
          hash = "sha256-uJmF5lhlryDuu5Me0Y4UUuxfWv6+2aymHmPT6YHf3bE=";
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

      backend = pkgs.buildGoModule rec {
        inherit meta;
        pname = "backend";
        version = "1.3.0-nightly.20250831";
        src = pkgs.fetchFromGitHub {
          owner = "green-ecolution";
          repo = "backend";
          rev = "0bb2dffd40ff09e236214dd5355cc678394abf26";
          hash = "sha256-9AN1hpdTOdfBjbuVuRBUvcdW2gVWhGj7pczBLfUV2nQ=";
        };
        ldflags = [
          "-s"
          "-w"
          "-X main.version=${version}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.version=${version}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitCommit=${src.rev}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitBranch=${src.rev}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitRepository=${src.url}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.buildTime=${"unkown"}"
        ];

        nativeBuildInputs = with pkgs; [proj pkg-config];
        buildInputs = [pkgs.geos];

        doCheck = false;
        excludedPackages = "pkg/*";
        vendorHash = "sha256-ixHDuyFeifRu7USUxwKqxTLHECrJsWTARQKn4HlT3nY=";
        env.CGO_ENABLED = 1;
      };

      default = backend.overrideAttrs (old: final: {
        name = "green-ecolution";
        preBuild = ''
          mkdir -p frontend/dist
          cp -r ${frontend}/* frontend/dist/
        '';
      });
    });

    devShells = forEachSupportedSystem ({pkgs}: let
      update = pkgs.pkgs.writeShellApplication {
        name = "update";
        text = ''
          if [ $# -ge 1 ] && [ -n "$1" ]; then
              version_arg="branch"
            if [ "$1" == "unstable" ]; then
                version_arg="branch"
            elif [ "$1" == "branch" ] && [ -n "$2" ]; then
                version_arg="branch=$2"
            elif [ "$1" == "version" ] && [ -n "$2" ]; then
                version_arg="$2"
            else
                echo "Usage: "
            fi
          else
              version_arg="branch"
          fi

          ${pkgs.nix-update}/bin/nix-update backend --flake "--version=$version_arg" --build
          ${pkgs.nix-update}/bin/nix-update frontend --flake "--version=$version_arg" --build
        '';
      };
    in {
      default = pkgs.mkShell {
        name = "root-shell";
        packages = with pkgs; [
          git
          update
        ];

        shellHook = ''
          echo "Welcome to the root shell 🚀"
          export PS1="(root-shell) $PS1"
        '';
      };

      backend = pkgs.mkShell {
        name = "backend-shell";
        packages = with pkgs; [
          go
          gotools
          golangci-lint
          geos
          proj
          gnumake
          pkg-config
          yq-go
          delve
        ];

        shellHook = ''
          echo "Backend dev shell loaded 🐳"
          export PS1="(backend-shell) $PS1"
        '';
      };

      frontend = pkgs.mkShell {
        name = "frontend-shell";
        packages = with pkgs; [
          nodejs
          pnpm
        ];

        shellHook = ''
          echo "Frontend dev shell loaded 🌿"
          export PS1="(frontend-shell) $PS1"
        '';
      };
    });
  };
}
