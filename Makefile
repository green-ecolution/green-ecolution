SHELL := bash

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
	@echo "  run/docker            Build+run app + infra via compose"
	@echo "  infra/up|stop|down    Infra via compose"
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
	./bin/$(BINARY_NAME)

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

.PHONY: run/docker
run/docker:
	@echo "Running compose (infra + app)..."
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf
	APP_VERSION="$(APP_VERSION)" \
	APP_GIT_COMMIT="$(APP_GIT_COMMIT)" \
	APP_GIT_BRANCH="$(APP_GIT_BRANCH)" \
	APP_BUILD_TIME="$(APP_BUILD_TIME)" \
	docker compose -f compose.yaml -f compose.app.yaml up -d --build

.PHONY: infra/up
infra/up:
	@echo "Infra up..."
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf
	docker compose up -d

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

