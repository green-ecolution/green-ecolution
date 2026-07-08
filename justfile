set dotenv-load
set shell := ["bash", "-euc"]

backend_dir      := "backend"
frontend_dir     := "frontend"
frontend_dist    := frontend_dir / "app/dist"
binary_name      := "green-ecolution"
valhalla_tiles_dir := ".docker/infra/valhalla/custom_files"

app_version        := `git describe --tags --always --dirty 2>/dev/null || echo "dev"`
app_git_commit     := `git rev-parse --short HEAD 2>/dev/null || echo "unknown"`
app_git_branch     := `git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown"`
app_build_time     := `date -u +'%Y-%m-%dT%H:%M:%SZ'`

postgres_user     := env("POSTGRES_USER", "postgres")
postgres_password := env("POSTGRES_PASSWORD", "postgres")
postgres_db       := env("POSTGRES_DB", "postgres")
postgres_host     := env("POSTGRES_HOST", "localhost")
postgres_port     := env("POSTGRES_PORT", "5432")

export USER_ID := `echo "$(id -u):$(id -g)"`

domain   := env("DOMAIN", "green-ecolution.dev")
local_ip := `ip -4 route get 1.1.1.1 2>/dev/null | awk '{print $7; exit}' || ipconfig getifaddr en0 2>/dev/null || echo '127.0.0.1'`

porkbun_api_key        := env("PORKBUN_API_KEY", "")
porkbun_secret_api_key := env("PORKBUN_SECRET_API_KEY", "")

app_host           := if porkbun_api_key != "" { local_ip + "." + domain } else { "localhost" }
bind_addr          := if porkbun_api_key != "" { "0.0.0.0" } else { "127.0.0.1" }
traefik_config     := if porkbun_api_key != "" { "traefik.yaml" } else { "traefik-no-tls.yaml" }
traefik_entrypoint := if porkbun_api_key != "" { "websecure" } else { "web" }
app_proto          := if porkbun_api_key != "" { "https" } else { "http" }
app_port           := if porkbun_api_key != "" { "3443" } else { "3000" }
s3_dev_endpoint    := if porkbun_api_key != "" { "s3." + app_host + ":" + app_port } else { app_host + ":" + app_port }
s3_use_ssl         := if porkbun_api_key != "" { "true" } else { "false" }

db_url := "postgres://" + postgres_user + ":" + postgres_password + "@" + postgres_host + ":" + postgres_port + "/" + postgres_db
sqlx_prepare_db := "sqlx_prepare"
sqlx_prepare_db_url := "postgres://" + postgres_user + ":" + postgres_password + "@" + postgres_host + ":" + postgres_port + "/" + sqlx_prepare_db

# Show available recipes
default:
    @just --list

# Install Rust toolchain components + pnpm dependencies
setup:
    @echo "Checking Rust toolchain..."
    @command -v cargo >/dev/null 2>&1 || { echo "cargo missing (install rustup)"; exit 1; }
    cd {{ backend_dir }} && cargo fetch --locked
    @echo "Installing frontend deps..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing (hint: corepack enable)"; exit 1; }
    cd {{ frontend_dir }} && pnpm install

# Compile the Rust binary
_compile-backend:
    @echo "Compiling Rust binary..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo build --release --locked --bin {{ binary_name }}
    mkdir -p bin
    cp {{ backend_dir }}/target/release/{{ binary_name }} bin/{{ binary_name }}

# Build the domain WASM bindings into frontend/packages/domain-wasm/pkg
build-domain-wasm:
    @echo "Building domain WASM bindings..."
    @command -v wasm-pack >/dev/null 2>&1 || { echo "wasm-pack missing (cargo install wasm-pack)"; exit 1; }
    cd {{ backend_dir }} && wasm-pack build crates/domain-wasm --target bundler --out-dir ../../../{{ frontend_dir }}/packages/domain-wasm/pkg --release

# Build frontend (pnpm)
build-frontend: build-domain-wasm
    @echo "Building frontend..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing"; exit 1; }
    cd {{ frontend_dir }} && pnpm install --frozen-lockfile
    cd {{ frontend_dir }} && pnpm run build
    @echo "Frontend build done."

# Build the Rust backend
build-backend: _compile-backend
    @echo "Backend build done."

# Full build: frontend + backend
build: build-frontend _compile-backend
    @echo "Build done."

# Build for all platforms
build-all: build-frontend
    @echo "Building backend for all platforms..."
    @just build-linux
    @just build-darwin
    @just build-windows

# Build for darwin (requires `rustup target add x86_64-apple-darwin` and a cross linker)
build-darwin:
    @echo "Building backend for darwin (x86_64)..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo build --release --locked --target x86_64-apple-darwin --bin {{ binary_name }}
    mkdir -p bin
    cp {{ backend_dir }}/target/x86_64-apple-darwin/release/{{ binary_name }} bin/{{ binary_name }}-darwin

# Build for linux (musl static binary; requires `rustup target add x86_64-unknown-linux-musl`)
build-linux:
    @echo "Building backend for linux (x86_64-musl)..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo build --release --locked --target x86_64-unknown-linux-musl --bin {{ binary_name }}
    mkdir -p bin
    cp {{ backend_dir }}/target/x86_64-unknown-linux-musl/release/{{ binary_name }} bin/{{ binary_name }}-linux

# Build for windows (requires `rustup target add x86_64-pc-windows-gnu` and mingw-w64)
build-windows:
    @echo "Building backend for windows (x86_64-gnu)..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo build --release --locked --target x86_64-pc-windows-gnu --bin {{ binary_name }}
    mkdir -p bin
    cp {{ backend_dir }}/target/x86_64-pc-windows-gnu/release/{{ binary_name }}.exe bin/{{ binary_name }}-windows.exe

# Run backend binary
run: build
    @echo "Running backend..."
    cd {{ backend_dir }} && APP_ENVIRONMENT=local ../bin/{{ binary_name }}

# Run backend with live reload (bacon)
run-live:
    @echo "Running backend live (bacon)..."
    @command -v bacon >/dev/null 2>&1 || { echo "bacon missing — run: cargo install bacon"; exit 1; }
    cd {{ backend_dir }} && SQLX_OFFLINE=true APP_ENVIRONMENT=local bacon --headless --job run -- --bin {{ binary_name }}

# Run frontend dev server
fe-dev:
    cd {{ frontend_dir }} && pnpm run dev

# Preview frontend build
fe-preview:
    cd {{ frontend_dir }} && pnpm run preview

# Backend + frontend dev via Traefik
run-dev:
    @command -v bacon >/dev/null 2>&1 || { echo "bacon missing — run: cargo install bacon"; exit 1; }
    @echo "Starting dev environment ({{ app_host }})..."
    @echo "  Backend:  {{ app_proto }}://{{ app_host }}:{{ app_port }}/api"
    @echo "  Frontend: {{ app_proto }}://{{ app_host }}:{{ app_port }}"
    @sed -e 's|${APP_HOST}|{{ app_host }}|g' \
        -e 's|${TRAEFIK_ENTRYPOINT}|{{ traefik_entrypoint }}|g' \
        .docker/infra/traefik/dynamic/dev-services.yaml.tmpl \
        > .docker/infra/traefik/dynamic/dev-services.yaml
    trap 'rm -f .docker/infra/traefik/dynamic/dev-services.yaml; kill 0' EXIT; \
      ( cd {{ backend_dir }} && \
        APP_ENVIRONMENT=local \
        SQLX_OFFLINE=true \
        APP_APPLICATION__HOST=0.0.0.0 \
        APP_APPLICATION__BASE_URL={{ app_proto }}://{{ app_host }}:{{ app_port }} \
        APP_DATABASE__HOST={{ postgres_host }} \
        APP_DATABASE__PORT={{ postgres_port }} \
        APP_DATABASE__DATABASE_NAME={{ postgres_db }} \
        APP_DATABASE__USERNAME={{ postgres_user }} \
        APP_DATABASE__PASSWORD={{ postgres_password }} \
        APP_LOG__LEVEL=debug \
        APP_LOG__FORMAT=pretty \
        bacon --headless --job run -- --bin {{ binary_name }} ) & \
      ( cd {{ frontend_dir }} && \
        USE_TRAEFIK=1 \
        VITE_BACKEND_BASEURL=/api \
        PUBLIC_DEV_URL={{ app_proto }}://{{ app_host }}:{{ app_port }} \
        pnpm run dev ) & \
      wait

# Backend + frontend locally in production mode via Traefik (release binary + prod frontend build; no devtools/debug)
run-prod: build-domain-wasm _compile-backend
    @echo "Building frontend (production)..."
    cd {{ frontend_dir }} && pnpm install --frozen-lockfile
    cd {{ frontend_dir }} && VITE_BACKEND_BASEURL=/api APP_VERSION={{ app_version }} pnpm run build
    @echo "Starting prod-like environment ({{ app_host }})..."
    @echo "  App: {{ app_proto }}://{{ app_host }}:{{ app_port }}"
    @sed -e 's|${APP_HOST}|{{ app_host }}|g' \
        -e 's|${TRAEFIK_ENTRYPOINT}|{{ traefik_entrypoint }}|g' \
        .docker/infra/traefik/dynamic/dev-services.yaml.tmpl \
        > .docker/infra/traefik/dynamic/dev-services.yaml
    trap 'rm -f .docker/infra/traefik/dynamic/dev-services.yaml; kill 0' EXIT; \
      ( cd {{ backend_dir }} && \
        APP_ENVIRONMENT=local \
        APP_APPLICATION__HOST=0.0.0.0 \
        APP_APPLICATION__BASE_URL={{ app_proto }}://{{ app_host }}:{{ app_port }} \
        APP_DATABASE__HOST={{ postgres_host }} \
        APP_DATABASE__PORT={{ postgres_port }} \
        APP_DATABASE__DATABASE_NAME={{ postgres_db }} \
        APP_DATABASE__USERNAME={{ postgres_user }} \
        APP_DATABASE__PASSWORD={{ postgres_password }} \
        ../bin/{{ binary_name }} ) & \
      ( cd {{ frontend_dir }} && USE_TRAEFIK=1 pnpm run preview ) & \
      wait

# Set up ACME storage for Let's Encrypt
_acme-init:
    @if [ -n "{{ porkbun_api_key }}" ]; then \
      echo "Setting up ACME storage for Let's Encrypt..."; \
      mkdir -p .docker/infra/traefik/acme; \
      test -f .docker/infra/traefik/acme/acme.json || \
        { touch .docker/infra/traefik/acme/acme.json && chmod 600 .docker/infra/traefik/acme/acme.json; }; \
      echo "ACME storage ready."; \
    else \
      echo "No Porkbun API keys set — running without TLS."; \
    fi

# Build patched Valhalla tiles into custom_files using the streamlet pipeline
# images. Same flow as the streamlet CI, but builds locally and skips the
# S3 upload (-m), so no secrets are needed. CONSTRUCTION=1 pulls in the TBZ
# roadworks changeset; default skips it for reproducible dev tiles.
_build-valhalla-tiles:
    #!/usr/bin/env bash
    set -euo pipefail
    skip="$([[ "${CONSTRUCTION:-0}" == "1" ]] && echo false || echo true)"
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    mkdir -p "{{ valhalla_tiles_dir }}"
    echo "Building patched PBF (SKIP_CONSTRUCTION=$skip)..."
    docker run --rm -u "$USER_ID" \
      -v "$work:/work" \
      -e DATA_DIR=/work -e OUTPUT_PATH=/work \
      -e OUTPUT_FILENAME=flensburg-updated.osm.pbf \
      -e SKIP_CONSTRUCTION="$skip" \
      ghcr.io/green-ecolution/streamlet/pbf-patch:latest
    echo "Building Valhalla tiles into {{ valhalla_tiles_dir }}..."
    docker run --rm -u "$USER_ID" \
      -v "$(pwd)/{{ valhalla_tiles_dir }}:/custom_files" \
      -v "$work:/data" \
      -e PBF_PATH=/data/flensburg-updated.osm.pbf \
      ghcr.io/green-ecolution/streamlet/generate-valhalla:latest -m
    rm -f "{{ valhalla_tiles_dir }}/flensburg-updated.osm.pbf"
    rmdir "{{ valhalla_tiles_dir }}/data" 2>/dev/null || true

# Build patched Valhalla tiles only if they are missing
_ensure-valhalla:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f "{{ valhalla_tiles_dir }}/valhalla_tiles.tar" ]; then
      echo "Valhalla tiles present — skipping build."
    else
      just _build-valhalla-tiles
    fi

# (Re)build patched Valhalla tiles from scratch (CONSTRUCTION=1 for TBZ roadworks)
valhalla-tiles:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Removing existing Valhalla tiles..."
    # Wipe as root inside a container: the valhalla serve container may leave
    # root-owned files behind that the host user cannot delete directly.
    if [ -d "{{ valhalla_tiles_dir }}" ]; then
      docker run --rm -u 0 --entrypoint find \
        -v "$(pwd)/{{ valhalla_tiles_dir }}:/custom_files" \
        ghcr.io/green-ecolution/streamlet/generate-valhalla:latest \
        /custom_files -mindepth 1 -delete
    fi
    just _build-valhalla-tiles

# Create Porkbun DNS records for local dev
dns-setup:
    @echo "Setting up DNS records for {{ app_host }} -> {{ local_ip }}..."
    @test -n "{{ porkbun_api_key }}" || { echo "error: PORKBUN_API_KEY not set"; exit 1; }
    @test -n "{{ porkbun_secret_api_key }}" || { echo "error: PORKBUN_SECRET_API_KEY not set"; exit 1; }
    @echo "Creating A record: {{ app_host }} -> {{ local_ip }}"
    @curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/create/{{ domain }}" \
      -H "Content-Type: application/json" \
      -d '{"apikey":"{{ porkbun_api_key }}","secretapikey":"{{ porkbun_secret_api_key }}","type":"A","name":"{{ local_ip }}","content":"{{ local_ip }}","ttl":"600"}'
    @echo ""
    @echo "Creating wildcard A record: *.{{ app_host }} -> {{ local_ip }}"
    @curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/create/{{ domain }}" \
      -H "Content-Type: application/json" \
      -d '{"apikey":"{{ porkbun_api_key }}","secretapikey":"{{ porkbun_secret_api_key }}","type":"A","name":"*.{{ local_ip }}","content":"{{ local_ip }}","ttl":"600"}'
    @echo ""
    @echo "DNS records created/updated."

# Remove Porkbun DNS records
dns-cleanup:
    @echo "Removing DNS records for {{ app_host }}..."
    @test -n "{{ porkbun_api_key }}" || { echo "error: PORKBUN_API_KEY not set"; exit 1; }
    @test -n "{{ porkbun_secret_api_key }}" || { echo "error: PORKBUN_SECRET_API_KEY not set"; exit 1; }
    @curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/deleteByNameType/{{ domain }}/A/{{ local_ip }}" \
      -H "Content-Type: application/json" \
      -d '{"apikey":"{{ porkbun_api_key }}","secretapikey":"{{ porkbun_secret_api_key }}"}' || true
    @curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/deleteByNameType/{{ domain }}/A/*.{{ local_ip }}" \
      -H "Content-Type: application/json" \
      -d '{"apikey":"{{ porkbun_api_key }}","secretapikey":"{{ porkbun_secret_api_key }}"}' || true
    @echo "DNS records removed."

# Build + run app + infra via Docker Compose
run-docker: _acme-init _ensure-valhalla build-domain-wasm
    @echo "Running compose (infra + app)..."
    APP_HOST="{{ app_host }}" \
    BIND_ADDR="{{ bind_addr }}" \
    TRAEFIK_CONFIG="{{ traefik_config }}" \
    TRAEFIK_ENTRYPOINT="{{ traefik_entrypoint }}" \
    APP_PROTO="{{ app_proto }}" \
    APP_PORT="{{ app_port }}" \
    PORKBUN_API_KEY="{{ porkbun_api_key }}" \
    PORKBUN_SECRET_API_KEY="{{ porkbun_secret_api_key }}" \
    APP_VERSION="{{ app_version }}" \
    APP_GIT_COMMIT="{{ app_git_commit }}" \
    APP_GIT_BRANCH="{{ app_git_branch }}" \
    APP_BUILD_TIME="{{ app_build_time }}" \
    docker compose -f compose.yaml -f compose.app.yaml up -d --build

# Start infrastructure services
infra-up: _acme-init _ensure-valhalla
    @echo "Infra up..."
    APP_HOST="{{ app_host }}" \
    BIND_ADDR="{{ bind_addr }}" \
    TRAEFIK_CONFIG="{{ traefik_config }}" \
    TRAEFIK_ENTRYPOINT="{{ traefik_entrypoint }}" \
    APP_PROTO="{{ app_proto }}" \
    APP_PORT="{{ app_port }}" \
    PORKBUN_API_KEY="{{ porkbun_api_key }}" \
    PORKBUN_SECRET_API_KEY="{{ porkbun_secret_api_key }}" \
    docker compose up -d

# Stop infrastructure services
infra-stop:
    @echo "Infra stop..."
    docker compose -f compose.yaml stop

# Stop infrastructure and delete volumes
infra-down:
    @echo "Infra down (delete volumes)..."
    docker compose -f compose.yaml down -v

# Refresh sqlx offline query cache (.sqlx/) — requires running DB and sqlx-cli
generate-sqlx: _migrate-build
    @echo "Refreshing sqlx offline cache against a clean migrated DB ({{ sqlx_prepare_db }})..."
    @command -v cargo-sqlx >/dev/null 2>&1 || { echo "sqlx-cli missing (cargo install sqlx-cli --no-default-features --features rustls,postgres)"; exit 1; }
    -cd {{ backend_dir }} && DATABASE_URL="{{ sqlx_prepare_db_url }}" cargo sqlx database drop -y
    cd {{ backend_dir }} && DATABASE_URL="{{ sqlx_prepare_db_url }}" cargo sqlx database create
    cd {{ backend_dir }} && DATABASE_URL="{{ sqlx_prepare_db_url }}" ./target/debug/migrate up
    cd {{ backend_dir }} && DATABASE_URL="{{ sqlx_prepare_db_url }}" cargo sqlx prepare --workspace -- --all-targets

# Dump the OpenAPI spec from the Rust backend into the frontend client package
dump-openapi:
    @echo "Dumping OpenAPI spec from Rust backend..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo run --quiet --locked --bin dump-openapi > ../{{ frontend_dir }}/packages/backend-client/api-docs.json

# Run frontend code generation (refreshes api-docs.json from Rust backend, then runs pnpm generate:local)
generate-frontend: dump-openapi
    @echo "Generating frontend..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing (hint: corepack enable)"; exit 1; }
    cd {{ frontend_dir }} && pnpm install --frozen-lockfile
    cd {{ frontend_dir }} && pnpm generate:local

# Run all backend code generation (sqlx prepare)
generate-backend: generate-sqlx
    @echo "Backend generation done."

# Run all code generation (backend + frontend)
generate: generate-backend generate-frontend
    @echo "All code generation done."

# Generate backend client from OpenAPI spec
generate-client:
    @echo "Generating backend client (openapi)..."
    @echo "TODO: implement OpenAPI client generation for Rust backend (utoipa exposes /api-docs)"

# Create a new migration file (requires sqlx-cli for scaffolding)
migrate-new name:
    @echo "Create new migration..."
    @command -v cargo-sqlx >/dev/null 2>&1 || { echo "sqlx-cli missing (cargo install sqlx-cli --no-default-features --features rustls,postgres)"; exit 1; }
    cd {{ backend_dir }} && cargo sqlx migrate add {{ name }}

# Build the in-tree `migrate` binary
_migrate-build:
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo build --bin migrate

# Apply all pending migrations
migrate-up: _migrate-build
    @echo "Migrating up..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate up

# Drop and recreate the database, then migrate
migrate-reset: _migrate-build
    @echo "Resetting database..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate reset

# Show migration status
migrate-status: _migrate-build
    @echo "Migration status..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate info

# Import the Flensburg tree cadastre into Green Ecolution.
# Requires KATASTER_SOURCE_URL. Pass flags via ARGS, e.g. `just import-kataster-fl --dry-run`.
import-kataster-fl *ARGS:
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo run --bin import-kataster-fl -- {{ ARGS }}

# Apply seeds on top of the current DB (assumes empty/migrated schema)
seed-up: _migrate-build
    @echo "Applying seeds..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate seed

# Remove all seed data (runs *.down.sql files in reverse order)
seed-down: _migrate-build
    @echo "Removing seed data..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate seed-down

# Reset DB, migrate, then apply seeds
seed-reset: _migrate-build
    @echo "Resetting DB and applying seeds..."
    cd {{ backend_dir }} && DATABASE_URL="{{ db_url }}" ./target/debug/migrate reset --with-seed

# Format Rust code
tidy:
    @echo "cargo fmt..."
    cd {{ backend_dir }} && cargo fmt --all

# Lint Rust + frontend
lint:
    @echo "cargo fmt --check + clippy + Frontend lint..."
    cd {{ backend_dir }} && cargo fmt --all -- --check
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    cd {{ frontend_dir }} && pnpm run lint

# Run Rust + frontend tests
test:
    @echo "Rust tests..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo test --workspace --locked
    @echo "Frontend tests..."
    cd {{ frontend_dir }} && pnpm run test

# Run Rust tests with verbose output
test-verbose:
    @echo "Rust tests (verbose)..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo test --workspace --locked -- --nocapture

# Build and open the Rust API docs in the browser (includes the logging field convention from telemetry.rs)
docs:
    @echo "Building Rust docs..."
    cd {{ backend_dir }} && SQLX_OFFLINE=true cargo doc --workspace --no-deps --locked --open

# Clean build artifacts
clean:
    @echo "Cleaning..."
    cd {{ backend_dir }} && cargo clean
    rm -rf bin
    rm -rf .docker/infra/valhalla/custom_files
    rm -rf {{ frontend_dist }}

# Update Nix hashes (frontend + backend)
nix-update-hashes:
    @echo "Updating Nix hashes (frontend + backend)..."
    nix-shell -p nix-update --run "nix-update --flake --version=skip frontend"
    nix-shell -p nix-update --run "nix-update --flake --version=skip backend"
