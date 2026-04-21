set dotenv-load
set shell := ["bash", "-euc"]

backend_dir      := "backend"
frontend_dir     := "frontend"
frontend_dist    := frontend_dir / "app/dist"
backend_fe_dist  := backend_dir / "frontend/dist"
binary_name      := "green-ecolution"

app_version        := `git describe --tags --always --dirty 2>/dev/null || echo "dev"`
app_git_commit     := `git rev-parse --short HEAD 2>/dev/null || echo "unknown"`
app_git_branch     := `git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown"`
app_git_repository := "https://github.com/green-ecolution/green-ecolution.git"
app_build_time     := `date -u +'%Y-%m-%dT%H:%M:%SZ'`

goflags := '-ldflags=" \
  -s -w \
  -X main.version=' + app_version + ' \
  -X github.com/green-ecolution/backend/internal/storage/local/info.version=' + app_version + ' \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitCommit=' + app_git_commit + ' \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitBranch=' + app_git_branch + ' \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitRepository=' + app_git_repository + ' \
  -X github.com/green-ecolution/backend/internal/storage/local/info.buildTime=' + app_build_time + ' \
"'

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

# Auth / S3 secrets (from .env)
ge_auth_oidc_frontend_client_id     := env("GE_AUTH_OIDC_FRONTEND_CLIENT_ID", "frontend")
ge_auth_oidc_frontend_client_secret := env("GE_AUTH_OIDC_FRONTEND_CLIENT_SECRET", "")
ge_auth_oidc_backend_client_id      := env("GE_AUTH_OIDC_BACKEND_CLIENT_ID", "backend")
ge_auth_oidc_backend_client_secret  := env("GE_AUTH_OIDC_BACKEND_CLIENT_SECRET", "")
ge_s3_accesskey                     := env("GE_S3_ACCESSKEY", "root")
ge_s3_secretaccesskey               := env("GE_S3_SECRETACCESSKEY", "secret1234")

# Show available recipes
default:
    @just --list

# Install Go modules + pnpm dependencies
setup:
    @echo "Installing backend deps..."
    cd {{ backend_dir }} && go mod download
    @echo "Installing frontend deps..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing (hint: corepack enable)"; exit 1; }
    cd {{ frontend_dir }} && pnpm install

# Compile the Go binary (assumes {{ backend_fe_dist }} is prepared)
_compile-backend:
    @echo "Compiling Go binary..."
    mkdir -p {{ backend_fe_dist }}
    test -f {{ backend_fe_dist }}/index.html || echo "<!-- placeholder for go:embed -->" > {{ backend_fe_dist }}/index.html
    cd {{ backend_dir }} && CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }} .

# Build frontend (pnpm)
build-frontend:
    @echo "Building frontend..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing"; exit 1; }
    cd {{ frontend_dir }} && pnpm install --frozen-lockfile
    cd {{ frontend_dir }} && pnpm run build
    @echo "Frontend build done."

# Build the Go backend (fast; placeholder frontend/dist for go:embed)
build-backend: _compile-backend
    @echo "Backend build done."

# Full build: frontend + backend with embedded assets
build: build-frontend
    @echo "Embedding frontend into backend..."
    mkdir -p {{ backend_fe_dist }}
    cp -R {{ frontend_dist }}/. {{ backend_fe_dist }}/
    @just _compile-backend
    @echo "Build done."

# Build for all platforms
build-all: build-frontend
    @echo "Building backend for all platforms..."
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=darwin  CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-darwin  .
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=linux   CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-linux   .
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-windows .

# Build for darwin
build-darwin:
    @echo "Building backend for darwin..."
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=darwin CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-darwin .

# Build for linux
build-linux:
    @echo "Building backend for linux..."
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=linux CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-linux .

# Build for windows
build-windows:
    @echo "Building backend for windows..."
    cd {{ backend_dir }} && GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build {{ goflags }} -o ../bin/{{ binary_name }}-windows .

# Run backend binary
run: build
    @echo "Running backend..."
    ./bin/{{ binary_name }} -config ./backend/config/config.yaml

# Run backend with live reload (air)
run-live:
    @echo "Running backend live (air)..."
    cd {{ backend_dir }} && go tool air

# Run frontend dev server
fe-dev:
    cd {{ frontend_dir }} && pnpm run dev

# Preview frontend build
fe-preview:
    cd {{ frontend_dir }} && pnpm run preview

# Backend + frontend dev via Traefik
run-dev:
    @echo "Starting dev environment ({{ app_host }})..."
    @echo "  Backend:  {{ app_proto }}://{{ app_host }}:{{ app_port }}/api"
    @echo "  Frontend: {{ app_proto }}://{{ app_host }}:{{ app_port }}"
    APP_HOST="{{ app_host }}" \
    TRAEFIK_ENTRYPOINT="{{ traefik_entrypoint }}" \
      envsubst < .docker/infra/traefik/dynamic/dev-services.yaml.tmpl \
      > .docker/infra/traefik/dynamic/dev-services.yaml
    trap 'rm -f .docker/infra/traefik/dynamic/dev-services.yaml; kill 0' EXIT; \
      ( cd {{ backend_dir }} && \
        GE_SERVER_APP_URL={{ app_proto }}://{{ app_host }}:{{ app_port }} \
        GE_SERVER_PORT=3030 \
        GE_SERVER_LOGS_LEVEL=debug \
        GE_SERVER_LOGS_FORMAT=text \
        GE_SERVER_DATABASE_HOST={{ postgres_host }} \
        GE_SERVER_DATABASE_PORT={{ postgres_port }} \
        GE_SERVER_DATABASE_TIMEOUT=30s \
        GE_SERVER_DATABASE_NAME={{ postgres_db }} \
        GE_SERVER_DATABASE_USERNAME={{ postgres_user }} \
        GE_SERVER_DATABASE_PASSWORD={{ postgres_password }} \
        GE_AUTH_ENABLE=true \
        GE_AUTH_OIDC_PROVIDER_BASE_URL={{ app_proto }}://auth.{{ app_host }}:{{ app_port }} \
        GE_AUTH_OIDC_PROVIDER_HEALTH_URL=http://auth.{{ app_host }}:{{ app_port }}/health/ready \
        GE_AUTH_OIDC_PROVIDER_DOMAIN_NAME=green-ecolution \
        GE_AUTH_OIDC_PROVIDER_AUTH_URL={{ app_proto }}://auth.{{ app_host }}:{{ app_port }}/realms/green-ecolution/protocol/openid-connect/auth \
        GE_AUTH_OIDC_PROVIDER_TOKEN_URL={{ app_proto }}://auth.{{ app_host }}:{{ app_port }}/realms/green-ecolution/protocol/openid-connect/token \
        GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_ID={{ ge_auth_oidc_frontend_client_id }} \
        GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_SECRET={{ ge_auth_oidc_frontend_client_secret }} \
        GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_ID={{ ge_auth_oidc_backend_client_id }} \
        GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_SECRET={{ ge_auth_oidc_backend_client_secret }} \
        GE_ROUTING_ENABLE=true \
        GE_ROUTING_START_POINT=9.434764259345679,54.768731253913806 \
        GE_ROUTING_END_POINT=9.434764259345679,54.768731253913806 \
        GE_ROUTING_WATERING_POINT=9.434764259345679,54.768731253913806 \
        GE_ROUTING_VALHALLA_HOST={{ app_proto }}://valhalla.{{ app_host }}:{{ app_port }} \
        GE_ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST={{ app_proto }}://vroom.{{ app_host }}:{{ app_port }} \
        GE_S3_ENABLE=true \
        GE_S3_ENDPOINT={{ s3_dev_endpoint }} \
        GE_S3_REGION=us-east-1 \
        GE_S3_USE_SSL={{ s3_use_ssl }} \
        GE_S3_ROUTE_GPX_BUCKET=gpx-routes \
        GE_S3_ROUTE_GPX_ACCESSKEY={{ ge_s3_accesskey }} \
        GE_S3_ROUTE_GPX_SECRETACCESSKEY={{ ge_s3_secretaccesskey }} \
        GE_MQTT_ENABLE=false \
        GE_MAP_CENTER=54.792277136221905,9.43580607453268 \
        GE_MAP_BBOX=54.714822,9.285796,54.860127,9.583800 \
        GE_MAP_NEAREST_TREE_MAX_RADIUS=500 \
        GE_MAP_NEAREST_TREE_DEFAULT_LIMIT=10 \
        GE_MAP_NEAREST_TREE_MAX_LIMIT=50 \
        go tool air ) & \
      ( cd {{ frontend_dir }} && USE_TRAEFIK=1 VITE_BACKEND_BASEURL=/api pnpm run dev ) & \
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

# Download Valhalla OSM data if missing
_ensure-valhalla:
    mkdir -p .docker/infra/valhalla/custom_files
    test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || \
      wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf \
        -O .docker/infra/valhalla/custom_files/sh.osm.pbf

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
run-docker: _acme-init _ensure-valhalla
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
    APP_BUILD_TIME="now" \
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

# Run sqlc code generation
generate-sqlc:
    @echo "Generating sqlc..."
    cd {{ backend_dir }} && go tool sqlc generate

# Run go:generate directives (mocks, OpenAPI entities, etc.)
generate-go:
    @echo "Running go generate..."
    cd {{ backend_dir }} && go generate ./...

# Run frontend code generation (pnpm generate:local)
generate-frontend:
    @echo "Generating frontend..."
    @command -v pnpm >/dev/null 2>&1 || { echo "pnpm missing (hint: corepack enable)"; exit 1; }
    cd {{ frontend_dir }} && pnpm install --frozen-lockfile
    cd {{ frontend_dir }} && pnpm generate:local

# Run all backend code generation (sqlc + go generate)
generate-backend: generate-sqlc generate-go
    @echo "Backend generation done."

# Run all code generation (backend + frontend)
generate: generate-backend generate-frontend
    @echo "All code generation done."

# Generate backend client from OpenAPI spec
generate-client:
    @echo "Generating backend client (openapi)..."
    cd {{ backend_dir }} && ./scripts/openapi-generator.sh client docs/swagger.yaml pkg/client
    cd {{ backend_dir }}/pkg/client && go fmt ./... && go mod tidy

# Create a new database migration
migrate-new name:
    @echo "Create new migration..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/migrations create {{ name }} sql

# Apply all pending migrations
migrate-up:
    @echo "Migrating up..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/migrations postgres "{{ db_url }}" up

# Rollback one migration
migrate-down:
    @echo "Migrating down..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/migrations postgres "{{ db_url }}" down

# Rollback all migrations
migrate-reset:
    @echo "Migrating reset..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/migrations postgres "{{ db_url }}" reset

# Show migration status
migrate-status:
    @echo "Migration status..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/migrations postgres "{{ db_url }}" status

# Apply seed data (runs migrate-up first)
seed-up: migrate-up
    @echo "Seeding up..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "{{ db_url }}" up

# Reset seed data (runs migrate-up first)
seed-reset: migrate-up
    @echo "Seeding reset..."
    cd {{ backend_dir }} && go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "{{ db_url }}" reset

# Format Go code and tidy modules
tidy:
    @echo "Go fmt & tidy..."
    cd {{ backend_dir }} && go fmt ./...
    cd {{ backend_dir }} && go mod tidy

# Lint Go + frontend
lint:
    @echo "Go fmt + Frontend lint..."
    cd {{ backend_dir }} && go fmt ./...
    cd {{ frontend_dir }} && pnpm run lint

# Run Go tests + frontend tests
test:
    @echo "Go tests..."
    cd {{ backend_dir }} && go test -cover ./...
    @echo "Frontend tests..."
    cd {{ frontend_dir }} && pnpm run test

# Run Go tests with verbose output
test-verbose:
    @echo "Go tests (verbose)..."
    cd {{ backend_dir }} && go test -v -cover ./...

# Clean build artifacts
clean:
    @echo "Cleaning..."
    cd {{ backend_dir }} && go clean
    rm -rf bin
    rm -rf {{ backend_dir }}/docs
    rm -rf {{ backend_dir }}/tmp
    rm -rf {{ backend_dir }}/internal/server/http/entities/info/generated
    rm -rf {{ backend_dir }}/internal/server/http/entities/sensor/generated
    rm -rf {{ backend_dir }}/internal/server/http/entities/tree/generated
    rm -rf {{ backend_dir }}/internal/server/mqtt/entities/sensor/generated
    rm -rf {{ backend_dir }}/internal/service/_mock
    rm -rf {{ backend_dir }}/internal/storage/_mock
    rm -rf {{ backend_dir }}/internal/storage/postgres/_sqlc
    rm -rf {{ backend_dir }}/internal/storage/postgres/mapper/generated
    rm -rf .docker/infra/valhalla/custom_files
    rm -rf {{ frontend_dist }}

# Update Nix hashes (frontend + backend)
nix-update-hashes:
    @echo "Updating Nix hashes (frontend + backend)..."
    nix-shell -p nix-update --run "nix-update --flake --version=skip frontend"
    nix-shell -p nix-update --run "nix-update --flake --version=skip backend"
