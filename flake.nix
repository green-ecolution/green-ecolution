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

    packages = let
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
      };
    in
      forEachSupportedSystem ({pkgs}: rec {
        frontend = pkgs.stdenv.mkDerivation rec {
          inherit meta;
          pname = "frontend";
          version = "1.2.1";
          src = pkgs.fetchFromGitHub {
            owner = "green-ecolution";
            repo = "frontend";
            rev = "85096d4ccb0366803d64164152f25c2303709093"; # TODO: Change to release version
            hash = "sha256-aDFKssU20C12sirQXSe+ukEk7RpNSxHBQJd7eav+43U=";
          };

          nativeBuildInputs = with pkgs; [
            nodejs
            pnpm.configHook
          ];

          VITE_BACKEND_BASEURL = "/api";
          pnpmDeps = pkgs.pnpm.fetchDeps {
            inherit pname version src;
            hash = "sha256-NFOMw449NIz2pNeQUqsx9uiApwiVnH437X8xU2UQANo=";
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
          pname = "green-ecolution";
          version = "1.2.1";
          src = pkgs.fetchFromGitHub {
            owner = "green-ecolution";
            repo = "backend";
            rev = "5886b46319b9"; # TODO: Change to release version
            hash = "sha256-VACsVns+mHhmAOdy6KYEK9tLwDxP3YaTTPhm2qEcWCY=";
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
          preBuild = ''
            ${pkgs.sqlc}/bin/sqlc generate
            go generate
          '';

          doCheck = false;
          excludedPackages = "pkg/*";
          vendorHash = "sha256-V4Swf+TycYP/4qTJcjmx497spZfpRQlN/hsrs1vlauk=";
          env.CGO_ENABLED = 1;
        };

        default = backend.overrideAttrs (old: final: {
          preBuild = ''
            ${pkgs.sqlc}/bin/sqlc generate
            go generate

            mkdir -p frontend/dist
            cp -r ${frontend}/* frontend/dist/
          '';
        });
      });

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        name = "root-shell";
        packages = with pkgs; [
          git
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
