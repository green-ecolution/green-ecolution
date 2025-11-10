# Green Ecolution Backend 🌿

<p align="center">
  <img src="https://github.com/user-attachments/assets/4ea25141-135a-493c-b9f6-e1cbc7a7aa41"/>
</p>

Smart irrigation is essential to saving water, reducing staff workload, and cutting costs. This project provides the backend for **Green Ecolution** — a digital system to manage urban greenery efficiently.

👉 For the frontend implementation, see the [frontend folder](../frontend).

## Overview 🧠

The backend provides APIs and services for:

- 🌳 Tree and vegetation management  
- 🌿 Tree clusters and spatial grouping  
- 📡 Sensor data ingestion (via MQTT / LoRaWAN)  
- 🗺️ Watering route optimization and scheduling  
- 🚛 Vehicle tracking  
- 👤 User and authentication management

Sensors are connected to LoRaWAN-based microcontrollers. Sensor data is transmitted via MQTT gateways and processed by the backend for analysis and route calculation.

Developed in collaboration with **TBZ Flensburg**, this system was originally created as part of the **Applied Computer Science Master's program** at the **University of Applied Sciences Flensburg**, and is designed to be easily adaptable for other municipalities and organizations.

For further information, visit:

- 🌐 [Project website](https://green-ecolution.de/)
- 🖥️ [Live demo](https://demo.green-ecolution.de)
- 🎓 [University of Applied Sciences Flensburg](https://hs-flensburg.de/en)

## Quick Start 🚀

### 1. Build and run the backend (with Docker Compose)

The easiest way to start the full stack (backend + infrastructure) locally is via **Make**:

```bash
make run/docker
```

This command:

- Downloads the Schleswig-Holstein OpenStreetMap tileset (for Valhalla routing)
- Builds the backend and frontend
- Starts all services (Postgres, Keycloak, MinIO, Valhalla, Vroom, Traefik, etc.)

If you want to manually manage containers:

```bash
docker compose -f compose.yaml -f compose.app.yaml up -d --build
```

By default, all core services (auth, routing, S3, database) are enabled.
MQTT is disabled by default and can be configured manually if needed.

👉 Once the service is up and running, the backend is available at: [http://localhost:3000](http://localhost:3000)

**API Documentation**

- 📘 [Swagger UI (local)](http://localhost:3000/api/v1/swagger/index.html)
- 📘 [Swagger UI (production)](https://app.green-ecolution.de/api/v1/swagger/index.html)
- 📘 [OpenAPI Spec](http://localhost:3000/api/v1/swagger/doc.json)

**Service Endpoints**

| Service         | URL                                                              |
| --------------- | ---------------------------------------------------------------- |
| Backend API     | [http://localhost:3000](http://localhost:3000)                   |
| Traefik (proxy) | [http://traefik.localhost:3000](http://traefik.localhost:3000)   |
| Keycloak        | [http://auth.localhost:3000](http://auth.localhost:3000)         |
| MinIO Console   | [http://minio.localhost:3000](http://minio.localhost:3000)       |
| S3 API          | [http://s3.localhost:3000](http://s3.localhost:3000)             |
| pgAdmin         | [http://pgadmin.localhost:3000](http://pgadmin.localhost:3000)   |
| Valhalla        | [http://valhalla.localhost:3000](http://valhalla.localhost:3000) |
| Vroom           | [http://vroom.localhost:3000](http://vroom.localhost:3000)       |

## Architecture Overview 🏗️

The backend follows a **layered architecture** with clear separation of concerns:

```
┌─────────────────────────────────────┐
│  HTTP Server (Fiber) / MQTT Server  │  ← Entry points
├─────────────────────────────────────┤
│         Handlers (v1/*)             │  ← HTTP request handlers
├─────────────────────────────────────┤
│    Services (domain logic)          │  ← Business logic layer
├─────────────────────────────────────┤
│    Repositories (data access)       │  ← Data abstraction layer
├─────────────────────────────────────┤
│  Storage (Postgres, S3, Keycloak)   │  ← Data stores
└─────────────────────────────────────┘
```

**Key Components:**

- **Entities** (`internal/entities/`) - Core domain models (Tree, Sensor, Vehicle, etc.)
- **Handlers** (`internal/server/http/handler/v1/`) - HTTP request handlers organized by domain
- **Services** (`internal/service/domain/`) - Business logic orchestration
- **Repositories** (`internal/storage/postgres/`) - Database access layer (using sqlc)
- **Event System** (`internal/worker/`) - Pub/sub for domain events and side effects
- **MQTT Server** (`internal/server/mqtt/`) - IoT sensor data ingestion
- **Mappers** (`internal/server/http/entities/mapper/`) - DTO conversions (auto-generated with goverter)

**Database:**

- PostgreSQL with PostGIS extensions for spatial data
- `sqlc` for type-safe SQL query generation
- `goose` for database migrations
- Queries defined in `storage/postgres/queries/*.sql`

**Authentication:**

- OIDC via Keycloak
- JWT middleware for protected endpoints
- Can be disabled for local development

**External Integrations:**

- **Valhalla** - Route calculation engine
- **Vroom** - Vehicle routing optimization
- **MinIO/S3** - Object storage for GPX files
- **MQTT** - Sensor data ingestion (optional)

## Technologies Used ⚙️

- [Go (Golang)](https://go.dev/) as the main language
- [Fiber](https://docs.gofiber.io/) as the web framework
- [sqlc](https://sqlc.dev/) for type-safe SQL queries
- [goose](https://github.com/pressly/goose) for database migrations
- [goverter](https://goverter.jmattheis.de/) for type conversions
- [swag](https://github.com/swaggo/swag) for OpenAPI/Swagger docs
- [env](https://github.com/caarlos0/env) for environment configuration
- [godotenv](https://github.com/joho/godotenv) for `.env` file handling
- [Testify](https://github.com/stretchr/testify) for unit testing
- [Mockery](https://github.com/vektra/mockery) for mock generation

### Prerequisites

- [Golang](https://go.dev/)
- [Air](https://github.com/air-verse/air) for live reload
- [Mockery](https://github.com/vektra/mockery) `v2.52.2` for generating mocks
- [Make](https://www.gnu.org/software/make/)
- [Docker](https://github.com/docker)
- [Docker Compose](https://github.com/docker/compose)
- Optional: [Nix](https://nixos.org/) (for fully reproducible environments)

### Initial Setup ⚙️

Install required tools:

```bash
make setup
```

Generate code:

```bash
make generate
```

Start the local infrastructure (Postgres, Valhalla, Keycloak, etc.):

```bash
make infra/up
```

### Running the Project ▶️

With live reload:

```bash
make run/live
```

Without live reload:

```bash
make run
```

👉 Once the service is up and running, you can access it at: [http://localhost:3000](http://localhost:3000)

### Testing 🧪

```bash
make test
```

Verbose mode:

```bash
make test/verbose
```

### Code Quality 🧹

Format and tidy Go modules:

```bash
make tidy
```

```bash
make lint
```

### Database Migrations 🧱

Create a new migration:

```bash
make migrate/new name=create_users_table
```

Apply migrations:

```bash
make migrate/up
```

Rollback:

```bash
make migrate/down
```

Seed example data:

```bash
make seed/up
```

## How to Contribute 🤝

We welcome contributions! Please follow these guidelines:

1. Fork this repository.
1. Create a topic branch off develop.
1. Commit your changes.
1. Push your branch to your fork.
1. Open a Pull Request.

This project follows:

- [Git-Flow Workflow](https://danielkummer.github.io/git-flow-cheatsheet/) for branching and releases.
- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for commit messages.

Thank you for helping us improve Green Ecolution! 🌿
