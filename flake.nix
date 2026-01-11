{
  description = "development enviroment for green-ecolution";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  } @ inputs: let
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

      si = self.sourceInfo or {};
      gitCommit = si.rev or "local";
      gitBranch = si.ref or "local";
      gitRepo = si.url or "local";

      buildDate = let
        d = si.lastModifiedDate or si.lastModified or null;
      in
        if d == null
        then "000000"
        else builtins.substring 2 6 d;
    in rec {
      frontend = pkgs.stdenv.mkDerivation rec {
        inherit meta;
        pname = "frontend";
        version = "0.1.1"; # x-release-please-version
        src = lib.cleanSource ./frontend;

        nativeBuildInputs = with pkgs; [nodejs pnpm.configHook];

        VITE_BACKEND_BASEURL = "/api";

        pnpmDeps = pkgs.pnpm.fetchDeps {
          inherit pname version src;
          fetcherVersion = 2;
          hash = "sha256-9r9W+cbwZ5HZPgcuYfHjsNDJBldUyEE6jCmWgC+jt1Y=";
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
        version = "0.1.1"; # x-release-please-version
        src = lib.cleanSource ./backend;

        subPackages = ["."];
        ldflags = [
          "-s"
          "-w"
          "-X main.version=${version}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.version=${version}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitCommit=${gitCommit}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitBranch=${gitBranch}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.gitRepository=${gitRepo}"
          "-X github.com/green-ecolution/backend/internal/storage/local/info.buildTime=${buildDate}"
        ];

        nativeBuildInputs = with pkgs; [proj pkg-config];
        buildInputs = [pkgs.geos];

        doCheck = false;
        excludedPackages = "pkg/*";
        vendorHash = "sha256-YSEm5ZqRdx8Qc83gch15daJxMFW67lz1FAjF0+g+vkI=";
        env.CGO_ENABLED = 1;

        postInstall = ''
          if [ -e "$out/bin/backend" ]; then
            mv "$out/bin/backend" "$out/bin/green-ecolution"
          fi
        '';
      };

      default = backend.overrideAttrs (_: {
        name = "green-ecolution";
        preBuild = ''
          mkdir -p frontend/dist
          cp -r ${frontend}/* frontend/dist/
        '';
      });
    });

    devShells = forEachSupportedSystem ({pkgs}: let
      ksops = pkgs.kustomize-sops.overrideAttrs (old: {
        installPhase = ''
          runHook preInstall
          mkdir -p $out
          dir="$GOPATH/bin"
          mv "$dir/kustomize-sops" "$dir/ksops" || true
          [ -e "$dir" ] && cp -r $dir $out
          runHook postInstall
        '';
      });
    in {
      default = pkgs.mkShell {
        name = "dev-shell";
        packages = with pkgs; [
          git
          # Backend/Go
          go
          gotools
          golangci-lint
          geos
          proj
          gnumake
          pkg-config
          yq-go
          delve
          # Frontend/Node
          nodejs
          pnpm
          # Deploy tooling
          kustomize
          ksops
        ];
        shellHook = ''
          echo "Dev shell loaded 🧪"
          export PS1="(dev) $PS1"
        '';
      };
    });

    nixosConfigurations.ge-dev = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = {inherit inputs self;};
      modules = [./nixos/ge-dev.nix];
    };

    # nix run .#dev-vm
    apps.x86_64-linux.dev-vm = {
      type = "app";
      program = "${self.nixosConfigurations.ge-dev.config.system.build.vm}/bin/run-ge-dev-vm";
    };
  };
}
