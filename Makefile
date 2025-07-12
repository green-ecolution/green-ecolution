MAIN_PACKAGE_PATH := .
BINARY_NAME := green-ecolution-backend
APP_VERSION ?= $(shell git describe --tags --always --dirty)
APP_GIT_COMMIT ?= $(shell git rev-parse --short HEAD)
APP_GIT_BRANCH ?= $(shell git rev-parse --abbrev-ref HEAD)
APP_GIT_REPOSITORY ?= https://github.com/green-ecolution/backend
APP_BUILD_TIME ?= $(shell date -u +'%Y-%m-%dT%H:%M:%SZ')
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
MOCKERY_VERSION := v2.52.2
POSTGRES_USER ?= postgres
POSTGRES_PASSWORD ?= postgres
POSTGRES_DB ?= postgres
POSTGRES_HOST ?= localhost
POSTGRES_PORT ?= 5432
export USER_ID ?= "$(shell id -u):$(shell id -g)"

.PHONY: help
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all                               Build for all platforms"
	@echo "  build/all                         Build for all platforms"
	@echo "  build/darwin                      Build for darwin"
	@echo "  build/linux                       Build for linux"
	@echo "  build/windows                     Build for windows"
	@echo "  build                             Build"
	@echo "  generate                          Generate"
	@echo "  generate/client                   Generate client pkg"
	@echo "  setup                             Install dependencies"
	@echo "  setup/macos                       Install dependencies for macOS"
	@echo "  clean                             Clean"
	@echo "  run                               Run"
	@echo "  run/live                          Run live"
	@echo "  run/docker ENV=[dev,stage,prod]   Run docker container (default: dev)"
	@echo "  infra/up                          Run infrastructure in docker compose (postgres and pgadmin)"
	@echo "  infra/stop                        Run infrastructure stop"
	@echo "  infra/down                        Run infrastructure down (delete)"
	@echo "  migrate/new name=<name>           Create new migration"
	@echo "  migrate/up                        Migrate up"
	@echo "  migrate/down                      Migrate down"
	@echo "  migrate/reset                     Migrate reset"
	@echo "  migrate/status                    Migrate status"
	@echo "  seed/up                           Seed up"
	@echo "  seed/reset                        Seed reset"
	@echo "  tidy                              Fmt and Tidy"
	@echo "  lint                              Lint"
	@echo "  test                              Test"
	@echo "  test/verbose                      Test verbose"
	@echo "  config/all                        Encrypt all config"
	@echo "  config/enc  ENV=[dev,stage,prod]  Encrypt config"
	@echo "  config/dec  ENV=[dev,stage,prod]  Decrypt config"
	@echo "  config/edit ENV=[dev,stage,prod]  Edit config"
	@echo "  debug                             Debug"

.PHONY: all
all: build

.PHONY: build/all
build/all: 
	@echo "Building for all..."
	GOARCH=amd64 GOOS=darwin CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-darwin $(MAIN_PACKAGE_PATH)
	GOARCH=amd64 GOOS=linux CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-linux $(MAIN_PACKAGE_PATH)
	GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-windows $(MAIN_PACKAGE_PATH)

.PHONY: build/darwin
build/darwin: 
	@echo "Building for darwin..."
	GOARCH=amd64 GOOS=darwin CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-darwin $(MAIN_PACKAGE_PATH)

.PHONY: build/linux
build/linux: 
	@echo "Building for linux..."
	GOARCH=amd64 GOOS=linux CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-linux $(MAIN_PACKAGE_PATH)

.PHONY: build/windows
build/windows: 
	@echo "Building for windows..."
	GOARCH=amd64 GOOS=windows CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME)-windows $(MAIN_PACKAGE_PATH)

.PHONY: build
build: 
	@echo "Building..."
	CGO_ENABLED=1 go build $(GOFLAGS) -o bin/$(BINARY_NAME) $(MAIN_PACKAGE_PATH)

.PHONY: generate
generate:
	@echo "Generating..."
	go tool sqlc generate
	go generate 

.PHONY: generate/client
generate/client:
	@echo "Generating client..."
	@./scripts/openapi-generator.sh client docs/swagger.yaml pkg/client
	cd pkg/client && go fmt ./... && go mod tidy

.PHONY: setup
setup:
	@echo "Installing..."
	go mod download

.PHONY: setup/macos
setup/macos:
	@echo "Installing..."
	brew install goose
	brew install sqlc
	brew install yq
	brew install delve
	brew install proj
	brew install geos
	brew install sops
	brew install age
	go mod download

.PHONY: clean
clean:
	@echo "Cleaning..."
	go clean
	rm -rf bin
	rm -rf docs
	rm -rf tmp
	rm -rf .docker/infra/valhalla/custom_files
	rm -rf internal/server/http/entities/info/generated
	rm -rf internal/server/http/entities/sensor/generated
	rm -rf internal/server/http/entities/tree/generated
	rm -rf internal/server/mqtt/entities/sensor/generated
	rm -rf internal/service/_mock
	rm -rf internal/storage/_mock
	rm -rf internal/storage/postgres/_sqlc
	rm -rf internal/storage/postgres/mapper/generated

.PHONY: run
run: build
	@echo "Running..."
	./bin/$(BINARY_NAME)

.PHONY: run/live
run/live:
	@echo "Running live..."
	go tool air

.PHONY: run/docker
run/docker: 
	@echo "Running infra..."
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf

	APP_VERSION=$(APP_VERSION) \
	APP_GIT_COMMIT=$(APP_GIT_COMMIT) \
	APP_GIT_BRANCH=$(APP_GIT_BRANCH) \
	APP_BUILD_TIME=$(APP_BUILD_TIME) \
	docker compose -f compose.yaml -f compose.app.yaml up -d --build

.PHONY: infra/up
infra/up:
	@echo "Running infra..."
	mkdir -p .docker/infra/valhalla/custom_files
	test -f .docker/infra/valhalla/custom_files/sh.osm.pbf || wget https://download.geofabrik.de/europe/germany/schleswig-holstein-latest.osm.pbf -O .docker/infra/valhalla/custom_files/sh.osm.pbf

	docker compose up -d

.PHONY: infra/stop
infra/stop:
	@echo "Running infra stop..."
	docker compose -f compose.yaml stop

.PHONY: infra/down
infra/down:
	@echo "Running infra delete..."
	docker compose -f compose.yaml down -v

.PHONY: migrate/new
migrate/new:
	@echo "Migrating up..."
	@if [ -z "$(name)" ]; then \
		echo "error: name is required"; \
		echo "usage: make migrate/new name=name_of_migration"; \
		exit 1; \
	fi
	go tool goose -dir internal/storage/postgres/migrations create $(name) sql

.PHONY: migrate/up
migrate/up:
	@echo "Migrating up..."
	go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" up

.PHONY: migrate/down
migrate/down:
	@echo "Migrating down..."
	go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" down

.PHONY: migrate/reset
migrate/reset:
	@echo "Migrating reset..."
	go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" reset

.PHONY: migrate/status
migrate/status:
	@echo "Migrating status..."
	go tool goose -dir internal/storage/postgres/migrations postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" status

.PHONY: seed/up
seed/up: migrate/up
	@echo "Seeding up..."
	go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" up

.PHONY: seed/reset
seed/reset: migrate/up
	@echo "Seeding reset..."
	go tool goose -dir internal/storage/postgres/seed -no-versioning postgres "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(POSTGRES_HOST):$(POSTGRES_PORT)/$(POSTGRES_DB)" reset

.PHONY: tidy
tidy:
	@echo "Fmt and Tidying..."
	go fmt ./...
	go mod tidy

.PHONY: lint
lint:
	@echo "Go fmt..."
	go fmt ./...
	@echo "Linting..."
	go tool golangci-lint run

.PHONY: test
test:
	@echo "Testing..."
	go test -cover ./...

.PHONY: test/verbose
test/verbose:
	@echo "Testing..."
	go test -v -cover ./...

.PHONY: config/all
config/all:
	@echo "Decrypt all config..."
	sops -d config/config.dev.enc.yaml > config/config.dev.yaml; \
	sops -d config/config.stage.enc.yaml > config/config.stage.yaml; \
	sops -d config/config.prod.enc.yaml > config/config.prod.yaml; \

.PHONY: config/enc
config/enc:
	sops -e config/config.$(ENV).yaml > config/config.$(ENV).enc.yaml; \

.PHONY: config/dec
config/dec:
	@echo "Decrypting config..."
	sops -d config/config.$(ENV).enc.yaml > config/config.$(ENV).yaml; \

.PHONY: config/edit
config/edit:
	@echo "Editing config..."
	sops edit config/config.$(ENV).enc.yaml \

.PHONY: debug
debug:
	@echo "Debugging..."
	dlv debug
