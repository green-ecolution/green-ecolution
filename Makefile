SHELL := bash
-include .env

BACKEND_DIR      := backend
FRONTEND_DIR     := frontend
FRONTEND_DIST    := $(FRONTEND_DIR)/app/dist
BACKEND_FE_DIST  := $(BACKEND_DIR)/frontend/dist

MAIN_PACKAGE_PATH := .
BINARY_NAME       := green-ecolution

APP_VERSION        ?= $(shell git describe --tags --always --dirty)
APP_GIT_COMMIT     ?= $(shell git rev-parse --short HEAD)
APP_GIT_BRANCH     ?= $(shell git rev-parse --abbrev-ref HEAD)
APP_GIT_REPOSITORY ?= https://github.com/green-ecolution.git
APP_BUILD_TIME     ?= $(shell date -u +'%Y-%m-%dT%H:%M:%SZ')

define GOFLAGS
-ldflags=" \
	-s -w \
  -X main.version=$(APP_VERSION) \
  -X github.com/green-ecolution/backend/internal/storage/local/info.version=$(APP_VERSION) \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitCommit=$(APP_GIT_COMMIT) \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitBranch=$(APP_GIT_BRANCH) \
  -X github.com/green-ecolution/backend/internal/storage/local/info.gitRepository=$(APP_GIT_REPOSITORY) \
  -X github.com/green-ecolution/backend/internal/storage/local/info.buildTime=$(APP_BUILD_TIME) \
"
endef

MOCKERY_VERSION   := v2.52.2
POSTGRES_USER     ?= postgres
POSTGRES_PASSWORD ?= postgres
POSTGRES_DB       ?= postgres
POSTGRES_HOST     ?= localhost
POSTGRES_PORT     ?= 5432
export USER_ID    ?= "$(shell id -u):$(shell id -g)"

define IN
	@set -e; cd $(1); $(2)
endef
FRUN = $(call IN,$(FRONTEND_DIR),$(1))
BRUN = $(call IN,$(BACKEND_DIR),$(1))

.PHONY: help
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Common:"
	@echo "  setup                 Install deps (go + pnpm)"
	@echo "  build                 Build backend + frontend"
	@echo "  build/backend         Build backend (host)"
	@echo "  build/frontend        Build frontend (pnpm)"
	@echo "  run                   Run backend binary"
	@echo "  run/live              Run backend with live reload (air)"
	@echo "  run/dev               Backend + frontend dev via Traefik HTTPS"
	@echo "  run/docker            Build+run app + infra via compose"
	@echo "  infra/up|stop|down    Infra via compose"
	@echo "  dns/setup             Create Porkbun DNS records for local dev"
	@echo "  dns/cleanup           Remove Porkbun DNS records"
	@echo "  lint                  Lint go + frontend"
	@echo "  test                  Test go + frontend"
	@echo "  clean                 Clean artifacts"
	@echo ""
	@echo "Cross:"
	@echo "  build/all|linux|darwin|windows  Cross compile backend"
	@echo ""
	@echo "DB:"
	@echo "  migrate/new name=...  Create new migration"
	@echo "  migrate/(up|down|reset|status)"
	@echo "  seed/(up|reset)"
	@echo ""
	@echo "Gen:"
	@echo "  generate              Backend codegen (sqlc, go:generate)"
	@echo "  generate/client       Backend client (openapi script)"
	@echo ""
	@echo "Frontend extra:"
	@echo "  fe/dev                pnpm dev (frontend workspace)"
	@echo "  fe/preview            pnpm preview (after build)"

.PHONY: all
all: build

.PHONY: build
build: build/backend
	@echo "Build done."

.PHONY: build/frontend
build/frontend:
	@echo "Building frontend..."
	@if ! command -v pnpm >/dev/null 2>&1; then echo "pnpm missing (hint: corepack enable)"; exit 1; fi
	$(call FRUN,pnpm install --frozen-lockfile)
	$(call FRUN,pnpm run build)

.PHONY: build/backend
build/backend: build/frontend
	@echo "Building backend..."
	@mkdir -p "$(BACKEND_FE_DIST)"
	@cp -R "$(FRONTEND_DIST)/." "$(BACKEND_FE_DIST)/"
	$(call BRUN,CGO_ENABLED=1 go build $(GOFLAGS) -o "../bin/$(BINARY_NAME)" $(MAIN_PACKAGE_PATH))

.PHONY: build/all
build/all:
	@echo "Building backend for all..."
	$(call BRUN,GOARCH=amd64 GOOS=darwin  CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-darwin  $(MAIN_PACKAGE_PATH))
	$(call BRUN,GOARCH=amd64 GOOS=linux   CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-linux   $(MAIN_PACKAGE_PATH))
	$(call BRUN,GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-windows $(MAIN_PACKAGE_PATH))

.PHONY: build/darwin
build/darwin:
	@echo "Building backend for darwin..."
	$(call BRUN,GOARCH=amd64 GOOS=darwin CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-darwin $(MAIN_PACKAGE_PATH))

.PHONY: build/linux
build/linux:
	@echo "Building backend for linux..."
	$(call BRUN,GOARCH=amd64 GOOS=linux CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-linux $(MAIN_PACKAGE_PATH))

.PHONY: build/windows
build/windows:
	@echo "Building backend for windows..."
	$(call BRUN,GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build $(GOFLAGS) -o ../bin/$(BINARY_NAME)-windows $(MAIN_PACKAGE_PATH))

.PHONY: run
run: build/backend
	@echo "Running backend..."
	./bin/$(BINARY_NAME) -config ./backend/config/config.yaml

.PHONY: run/live
run/live:
	@echo "Running backend live (air)..."
	$(call BRUN,go tool air)

.PHONY: setup
setup:
	@echo "Installing backend deps..."
	$(call BRUN,go mod download)
	@echo "Installing frontend deps..."
	@if ! command -v pnpm >/dev/null 2>&1; then echo "pnpm missing"; exit 1; fi
	$(call FRUN,pnpm install)

.PHONY: clean
clean:
	@echo "Cleaning..."
	$(call BRUN,go clean)
	rm -rf bin
	# backend generated/temporary
	rm -rf $(BACKEND_DIR)/docs
	rm -rf $(BACKEND_DIR)/tmp
	rm -rf $(BACKEND_DIR)/internal/server/http/entities/info/generated
	rm -rf $(BACKEND_DIR)/internal/server/http/entities/sensor/generated
	rm -rf $(BACKEND_DIR)/internal/server/http/entities/tree/generated
	rm -rf $(BACKEND_DIR)/internal/server/mqtt/entities/sensor/generated
	rm -rf $(BACKEND_DIR)/internal/service/_mock
	rm -rf $(BACKEND_DIR)/internal/storage/_mock
	rm -rf $(BACKEND_DIR)/internal/storage/postgres/_sqlc
	rm -rf $(BACKEND_DIR)/internal/storage/postgres/mapper/generated
	# infra cache
	rm -rf .docker/infra/valhalla/custom_files
	# frontend artifacts
	rm -rf $(FRONTEND_DIR)/dist

DOMAIN   ?= green-ecolution.dev
LOCAL_IP ?= $(shell ip -4 route get 1.1.1.1 | awk '{print $$7; exit}')

ifdef PORKBUN_API_KEY
  APP_HOST            ?= $(LOCAL_IP).$(DOMAIN)
  BIND_ADDR           ?= 0.0.0.0
  TRAEFIK_CONFIG      ?= traefik.yaml
  TRAEFIK_ENTRYPOINT  ?= websecure
  APP_PROTO           ?= https
  APP_PORT            ?= 3443
  S3_DEV_ENDPOINT     ?= s3.$(APP_HOST):$(APP_PORT)
  S3_USE_SSL          ?= true
else
  APP_HOST            ?= localhost
  BIND_ADDR           ?= 127.0.0.1
  TRAEFIK_CONFIG      ?= traefik-no-tls.yaml
  TRAEFIK_ENTRYPOINT  ?= web
  APP_PROTO           ?= http
  APP_PORT            ?= 3000
  S3_DEV_ENDPOINT     ?= $(APP_HOST):$(APP_PORT)
  S3_USE_SSL          ?= false
endif

.PHONY: certs/generate
certs/generate:
ifdef PORKBUN_API_KEY
	@echo "Setting up ACME storage for Let's Encrypt..."
	@mkdir -p .docker/infra/traefik/acme
	@test -f .docker/infra/traefik/acme/acme.json || \
		{ touch .docker/infra/traefik/acme/acme.json && chmod 600 .docker/infra/traefik/acme/acme.json; }
	@echo "ACME storage ready."
else
	@echo "No Porkbun API keys set — running without TLS."
endif

.PHONY: dns/setup
dns/setup:
	@echo "Setting up DNS records for $(APP_HOST) -> $(LOCAL_IP)..."
	@test -n "$(PORKBUN_API_KEY)" || { echo "error: PORKBUN_API_KEY not set"; exit 1; }
	@test -n "$(PORKBUN_SECRET_API_KEY)" || { echo "error: PORKBUN_SECRET_API_KEY not set"; exit 1; }
	@echo "Creating A record: $(APP_HOST) -> $(LOCAL_IP)"
	@curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/create/$(DOMAIN)" \
		-H "Content-Type: application/json" \
		-d '{"apikey":"$(PORKBUN_API_KEY)","secretapikey":"$(PORKBUN_SECRET_API_KEY)","type":"A","name":"$(LOCAL_IP)","content":"$(LOCAL_IP)","ttl":"600"}'
	@echo ""
	@echo "Creating wildcard A record: *.$(APP_HOST) -> $(LOCAL_IP)"
	@curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/create/$(DOMAIN)" \
		-H "Content-Type: application/json" \
		-d '{"apikey":"$(PORKBUN_API_KEY)","secretapikey":"$(PORKBUN_SECRET_API_KEY)","type":"A","name":"*.$(LOCAL_IP)","content":"$(LOCAL_IP)","ttl":"600"}'
	@echo ""
	@echo "DNS records created/updated."

.PHONY: dns/cleanup
dns/cleanup:
	@echo "Removing DNS records for $(APP_HOST)..."
	@test -n "$(PORKBUN_API_KEY)" || { echo "error: PORKBUN_API_KEY not set"; exit 1; }
	@test -n "$(PORKBUN_SECRET_API_KEY)" || { echo "error: PORKBUN_SECRET_API_KEY not set"; exit 1; }
	@curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/deleteByNameType/$(DOMAIN)/A/$(LOCAL_IP)" \
		-H "Content-Type: application/json" \
		-d '{"apikey":"$(PORKBUN_API_KEY)","secretapikey":"$(PORKBUN_SECRET_API_KEY)"}' || true
	@curl -sf -X POST "https://api.porkbun.com/api/json/v3/dns/deleteByNameType/$(DOMAIN)/A/*.$(LOCAL_IP)" \
		-H "Content-Type: application/json" \
		-d '{"apikey":"$(PORKBUN_API_KEY)","secretapikey":"$(PORKBUN_SECRET_API_KEY)"}' || true
	@echo "DNS records removed."

.PHONY: run/docker
run/docker:
	@echo "Running compose (infra + app)..."
	@$(MAKE) certs/generate
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf
	APP_HOST="$(APP_HOST)" \
	BIND_ADDR="$(BIND_ADDR)" \
	TRAEFIK_CONFIG="$(TRAEFIK_CONFIG)" \
	TRAEFIK_ENTRYPOINT="$(TRAEFIK_ENTRYPOINT)" \
	APP_PROTO="$(APP_PROTO)" \
	APP_PORT="$(APP_PORT)" \
	PORKBUN_API_KEY="$(PORKBUN_API_KEY)" \
	PORKBUN_SECRET_API_KEY="$(PORKBUN_SECRET_API_KEY)" \
	APP_VERSION="$(APP_VERSION)" \
	APP_GIT_COMMIT="$(APP_GIT_COMMIT)" \
	APP_GIT_BRANCH="$(APP_GIT_BRANCH)" \
	APP_BUILD_TIME="now" \
	docker compose -f compose.yaml -f compose.app.yaml up -d --build

.PHONY: infra/up
infra/up:
	@echo "Infra up..."
	@$(MAKE) certs/generate
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf
	APP_HOST="$(APP_HOST)" \
	BIND_ADDR="$(BIND_ADDR)" \
	TRAEFIK_CONFIG="$(TRAEFIK_CONFIG)" \
	TRAEFIK_ENTRYPOINT="$(TRAEFIK_ENTRYPOINT)" \
	APP_PROTO="$(APP_PROTO)" \
	APP_PORT="$(APP_PORT)" \
	PORKBUN_API_KEY="$(PORKBUN_API_KEY)" \
	PORKBUN_SECRET_API_KEY="$(PORKBUN_SECRET_API_KEY)" \
	docker compose up -d

define DEV_BACKEND_ENV
GE_SERVER_APP_URL=$(APP_PROTO)://$(APP_HOST):$(APP_PORT) \
GE_SERVER_PORT=3030 \
GE_SERVER_LOGS_LEVEL=debug \
GE_SERVER_LOGS_FORMAT=text \
GE_SERVER_DATABASE_HOST=$(POSTGRES_HOST) \
GE_SERVER_DATABASE_PORT=$(POSTGRES_PORT) \
GE_SERVER_DATABASE_TIMEOUT=30s \
GE_SERVER_DATABASE_NAME=$(POSTGRES_DB) \
GE_SERVER_DATABASE_USERNAME=$(POSTGRES_USER) \
GE_SERVER_DATABASE_PASSWORD=$(POSTGRES_PASSWORD) \
GE_AUTH_ENABLE=true \
GE_AUTH_OIDC_PROVIDER_BASE_URL=$(APP_PROTO)://auth.$(APP_HOST):$(APP_PORT) \
GE_AUTH_OIDC_PROVIDER_HEALTH_URL=http://auth.$(APP_HOST):$(APP_PORT)/health/ready \
GE_AUTH_OIDC_PROVIDER_DOMAIN_NAME=green-ecolution \
GE_AUTH_OIDC_PROVIDER_AUTH_URL=$(APP_PROTO)://auth.$(APP_HOST):$(APP_PORT)/realms/green-ecolution/protocol/openid-connect/auth \
GE_AUTH_OIDC_PROVIDER_TOKEN_URL=$(APP_PROTO)://auth.$(APP_HOST):$(APP_PORT)/realms/green-ecolution/protocol/openid-connect/token \
GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_ID=frontend \
GE_AUTH_OIDC_PROVIDER_FRONTEND_CLIENT_SECRET=EJogoaQSrW7vlo9CK3zIo2ieC8guy9Mm \
GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_ID=backend \
GE_AUTH_OIDC_PROVIDER_BACKEND_CLIENT_SECRET=cVBGgWNlbwSLydLZfZMvTvmNCeiSmLGP \
GE_ROUTING_ENABLE=true \
GE_ROUTING_START_POINT=9.434764259345679,54.768731253913806 \
GE_ROUTING_END_POINT=9.434764259345679,54.768731253913806 \
GE_ROUTING_WATERING_POINT=9.434764259345679,54.768731253913806 \
GE_ROUTING_VALHALLA_HOST=$(APP_PROTO)://valhalla.$(APP_HOST):$(APP_PORT) \
GE_ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST=$(APP_PROTO)://vroom.$(APP_HOST):$(APP_PORT) \
GE_S3_ENABLE=true \
GE_S3_ENDPOINT=$(S3_DEV_ENDPOINT) \
GE_S3_REGION=us-east-1 \
GE_S3_USE_SSL=$(S3_USE_SSL) \
GE_S3_ROUTE_GPX_BUCKET=gpx-routes \
GE_S3_ROUTE_GPX_ACCESSKEY=root \
GE_S3_ROUTE_GPX_SECRETACCESSKEY=secret1234 \
GE_MQTT_ENABLE=false \
GE_MAP_CENTER=54.792277136221905,9.43580607453268 \
GE_MAP_BBOX=54.714822,9.285796,54.860127,9.583800
endef

.PHONY: run/dev
run/dev:
	@echo "Starting dev environment ($(APP_HOST))..."
	@echo "  Backend:  $(APP_PROTO)://$(APP_HOST):$(APP_PORT)/api"
	@echo "  Frontend: $(APP_PROTO)://$(APP_HOST):$(APP_PORT)"
	@printf 'http:\n  routers:\n    backend-dev:\n      rule: "Host(\x60$(APP_HOST)\x60) && PathPrefix(\x60/api\x60)"\n      entryPoints: [$(TRAEFIK_ENTRYPOINT)]\n      service: backend-dev\n    frontend-dev:\n      rule: "Host(\x60$(APP_HOST)\x60)"\n      entryPoints: [$(TRAEFIK_ENTRYPOINT)]\n      service: frontend-dev\n  services:\n    backend-dev:\n      loadBalancer:\n        servers:\n          - url: "http://host.docker.internal:3030"\n    frontend-dev:\n      loadBalancer:\n        servers:\n          - url: "http://host.docker.internal:5173"\n' > .docker/infra/traefik/dynamic/dev-services.yaml
	@trap 'rm -f .docker/infra/traefik/dynamic/dev-services.yaml; kill 0' EXIT; \
		(cd $(BACKEND_DIR) && $(DEV_BACKEND_ENV) go tool air) & \
		(cd $(FRONTEND_DIR) && USE_TRAEFIK=1 VITE_BACKEND_BASEURL=/api pnpm run dev) & \
		wait

.PHONY: infra/stop
infra/stop:
	@echo "Infra stop..."
	docker compose -f compose.yaml stop

.PHONY: infra/down
infra/down:
	@echo "Infra down (delete volumes)..."
	docker compose -f compose.yaml down -v

.PHONY: generate
generate:
	@echo "Generating backend..."
	$(call BRUN,go tool sqlc generate)
	$(call BRUN,go generate)

.PHONY: generate/client
generate/client:
	@echo "Generating backend client (openapi)..."
	$(call BRUN,./scripts/openapi-generator.sh client docs/swagger.yaml pkg/client)
	$(call BRUN,cd pkg/client && go fmt ./... && go mod tidy)
	# TODO: Build frontend api client from file

.PHONY: migrate/new
migrate/new:
	@echo "Create new migration..."
	@if [ -z "$(name)" ]; then \
		echo "error: name is required"; \
		echo "usage: make migrate/new name=name_of_migration"; \
		exit 1; \
	fi
	$(call BRUN,go tool goose -dir internal/storage/postgres/migrations create $(name) sql)

.PHONY: migrate/up
migrate/up:
	@echo "Migrating up..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" up)

.PHONY: migrate/down
migrate/down:
	@echo "Migrating down..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" down)

.PHONY: migrate/reset
migrate/reset:
	@echo "Migrating reset..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" reset)

.PHONY: migrate/status
migrate/status:
	@echo "Migrating status..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" status)

.PHONY: seed/up
seed/up: migrate/up
	@echo "Seeding up..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" up)

.PHONY: seed/reset
seed/reset: migrate/up
	@echo "Seeding reset..."
	$(call BRUN,go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" reset)

.PHONY: tidy
tidy:
	@echo "Go fmt & tidy..."
	$(call BRUN,go fmt ./...)
	$(call BRUN,go mod tidy)

.PHONY: lint
lint:
	@echo "Go fmt + Frontend lint..."
	$(call BRUN,go fmt ./...)
	$(call FRUN,pnpm run lint)

.PHONY: test
test:
	@echo "Go tests..."
	$(call BRUN,go test -cover ./...)
	@echo "Frontend tests..."
	$(call FRUN,pnpm run test)

.PHONY: test/verbose
test/verbose:
	@echo "Go tests (verbose)..."
	$(call BRUN,go test -v -cover ./...)

