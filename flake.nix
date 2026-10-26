{
  description = "development environment for green-ecolution";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      ...
    }:
    let
      nodeVersion = 24;

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ self.overlays.default ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        nodejs = final."nodejs_${toString nodeVersion}";
      };

      packages = forEachSupportedSystem (
        { pkgs }:
        let
          lib = pkgs.lib;

          version = (lib.importTOML ./backend/Cargo.toml).workspace.package.version;

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

          si = self.sourceInfo or { };
          gitCommit = si.rev or si.dirtyRev or "local";
          gitBranch = "local";

          buildDate =
            if si ? lastModifiedDate then
              let
                d = si.lastModifiedDate;
                sub = builtins.substring;
              in
              "${sub 0 4 d}-${sub 4 2 d}-${sub 6 2 d}T${sub 8 2 d}:${sub 10 2 d}:${sub 12 2 d}Z"
            else
              "unknown";
        in
        rec {
          domain-wasm = pkgs.rustPlatform.buildRustPackage {
            inherit meta version;
            pname = "domain-wasm";
            src = lib.cleanSource ./backend;

            cargoLock = {
              lockFile = ./backend/Cargo.lock;
            };

            # clang/lld: host artifacts (proc-macros) pick up the linker
            # override in backend/.cargo/config.toml
            nativeBuildInputs = with pkgs; [
              wasm-bindgen-cli
              clang
              lld
            ];

            auditable = false;
            doCheck = false;

            # custom buildPhase: cargoBuildHook pins --target to the host
            # triple and cannot cross-build to wasm32
            buildPhase = ''
              runHook preBuild
              cargo build --release --offline -j $NIX_BUILD_CORES \
                -p domain-wasm --target wasm32-unknown-unknown
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              wasm-bindgen --target bundler --out-dir $out \
                target/wasm32-unknown-unknown/release/domain_wasm.wasm
              runHook postInstall
            '';
          };

          frontend = pkgs.stdenv.mkDerivation rec {
            inherit meta version;
            pname = "frontend";
            src = lib.cleanSource ./frontend;

            nativeBuildInputs = with pkgs; [
              nodejs
              pnpm
              pnpmConfigHook
            ];

            env.VITE_BACKEND_BASEURL = "/api";

            pnpmDeps = pkgs.fetchPnpmDeps {
              inherit pname version src;
              fetcherVersion = 4;
              hash = "sha256-yklJn6IVYQt/n0vPtpmlFqRyqmap87PETLym3JEfYwg=";
            };

            # domain-wasm/pkg is wasm-pack output, gitignored and thus absent
            # from the flake source; inject the Nix-built bindings instead.
            preBuild = ''
              mkdir -p packages/domain-wasm/pkg
              cp -r ${domain-wasm}/. packages/domain-wasm/pkg/
            '';

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

          backend = pkgs.rustPlatform.buildRustPackage {
            inherit meta version;
            pname = "green-ecolution";
            src = lib.cleanSource ./backend;

            cargoLock = {
              lockFile = ./backend/Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              clang
              lld
            ];

            cargoBuildFlags = [
              "--bin"
              "green-ecolution"
              "--bin"
              "migrate"
            ];

            # Tests rely on testcontainers (live Postgres)
            doCheck = false;

            env = {
              SQLX_OFFLINE = "true";
              GE_GIT_COMMIT = gitCommit;
              GE_GIT_BRANCH = gitBranch;
              GE_BUILD_TIME = buildDate;
            };
          };

          geco = pkgs.rustPlatform.buildRustPackage {
            inherit version;
            pname = "geco";
            src = lib.cleanSource ./backend;

            cargoLock = {
              lockFile = ./backend/Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              clang
              lld
            ];

            cargoBuildFlags = [
              "-p"
              "devcli"
            ];

            # Tests read geco.toml from the repository root, outside of src
            doCheck = false;

            meta = meta // {
              description = "geco: developer console for the Green Ecolution stack";
              mainProgram = "geco";
            };
          };

          default = backend;
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          # Builds from the working tree rather than the flake source, so geco
          # always matches the checked-out branch. Own target dir: bacon holds the
          # lock on backend/target while geco runs it. Thin LTO and one codegen
          # unit from the release profile would make every rebuild a full one.
          # The overrides apply to the build only; geco passes its environment on
          # to bacon and the just recipes, which must keep the regular settings.
          geco = pkgs.writeShellScriptBin "geco" ''
            root=$(${pkgs.git}/bin/git rev-parse --show-toplevel) || exit 1
            target="$root/backend/target/geco"
            CARGO_TARGET_DIR="$target" \
            CARGO_PROFILE_RELEASE_LTO=false \
            CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
            CARGO_PROFILE_RELEASE_INCREMENTAL=true \
              cargo build --quiet --release --locked \
                --manifest-path "$root/backend/Cargo.toml" -p devcli || exit 1
            exec "$target/release/geco" "$@"
          '';
        in
        {
          default = pkgs.mkShell {
            name = "dev-shell";
            packages = with pkgs; [
              git
              just
              # Frontend/Node
              nodejs
              pnpm
              # Handbook PDF
              typst
              # Keycloak theme build (keycloakify shells out to mvn)
              jdk21
              maven
              # Rust
              rustc
              cargo
              rustfmt
              clippy
              rust-analyzer
              cargo-deny
              sqlx-cli
              clang
              lld
              bacon
              wasm-pack
              geco
            ];
            shellHook = ''
              echo "Dev shell loaded 🧪"
              export PS1="(dev) $PS1"
            '';
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs }: pkgs.alejandra);

      checks = forEachSupportedSystem (
        { pkgs }: {
          inherit (self.packages.${pkgs.system})
            domain-wasm
            frontend
            backend
            default
            ;
        }
      );
    };
}
