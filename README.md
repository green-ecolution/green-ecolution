<p>
  <a href="https://github.com/green-ecolution/green-ecolution/releases">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/green-ecolution/green-ecolution"/>
  </a>
  <a href=""><img alt="License" src="https://img.shields.io/github/license/green-ecolution/green-ecolution.svg"/></a>
  <a href=""><img alt="Maintained yes" src="https://img.shields.io/badge/Maintained%3F-yes-green.svg"/></a>
  <a href=""><img alt="Code coverage" src="https://raw.githubusercontent.com/green-ecolution/backend/badges/.badges/develop/coverage.svg"/></a>
  <a href="https://pkg.go.dev/github.com/green-ecolution/backend">
    <img src="https://pkg.go.dev/badge/github.com/green-ecolution/backend.svg" alt="Go Reference">
  </a>
</p>

# Green Ecolution 🌿

<p align="center">
  <img src="https://github.com/user-attachments/assets/4ea25141-135a-493c-b9f6-e1cbc7a7aa41"/>
</p>

**Green Ecolution** is a smart irrigation and green-space management platform that uses IoT sensor data to optimize water usage, automate maintenance, and reduce operational costs.

## Key Features ✨

- 🌳 **Tree & Vegetation Management** - Comprehensive inventory and monitoring of urban greenery
- 📡 **IoT Sensor Integration** - Real-time data collection via LoRaWAN sensors and MQTT
- 🗺️ **Smart Route Optimization** - Automated watering route planning using Valhalla and Vroom
- 💧 **Water Usage Optimization** - Data-driven irrigation scheduling to reduce water waste
- 🚛 **Fleet Management** - Vehicle tracking and maintenance task planning
- 📊 **Analytics & Reporting** - Comprehensive evaluation and insights dashboard
- 🔌 **Plugin System** - Extensible architecture for custom integrations
- 🔐 **Enterprise Authentication** - OIDC/Keycloak integration with role-based access control

## Repository Structure 📁

```
.
├── backend/ # Go backend (API, Auth, Routing, Storage, MQTT)
├── frontend/ # Web frontend (Vite + pnpm)
├── deploy/kustomize/ # Kubernetes deployment manifests
├── compose.yaml # Local dev infrastructure (Postgres, S3, Keycloak, etc.)
├── compose.app.yaml # Application container definitions
├── flake.nix # Nix Flake for builds, DevShell, and Dev VM
└── Makefile # Unified build, test, and infra automation
```

## Getting Started ⚡

### Option A: Using **Make** (local toolchain)

**Requirements**

- Go (with CGO enabled)
- Node.js and **pnpm** (use `corepack enable`)
- Docker + Docker Compose

**Setup**

```bash
# Install dependencies for backend and frontend
make setup
```

**Build**

```bash
# Build both frontend and backend (frontend assets are embedded)
make build
```

**Run the backend**

```bash
make run
```

**Run backend with live reload**

```bash
make run/live
```

**Run with Docker Compose**

```bash
# Run full local stack (infra + app)
make run/docker

# Or just bring up the infrastructure
make infra/up
```

**Common tasks**

```bash
make generate            # Run code generation (sqlc, go:generate)
make migrate/up          # Run DB migrations
make migrate/new name=...# Create a new migration
make lint                # Lint Go and frontend code
make test                # Run tests
make clean               # Remove build artifacts
```

### Option B: Using Nix & Flakes 🧊

Use Nix for a reproducible and dependency-free dev environment.

**Enter development shell**

```bash
nix develop
```

**Build artifacts**

```bash
# Build backend only
nix build .#backend

# Build frontend only
nix build .#frontend

# Build backend binary with embedded frontend
nix build .
```

**Run the development VM (includes backend + services)**

```bash
# Launch with QEMU GUI
nix run .#dev-vm

# Headless (no graphical window)
nix run .#dev-vm -- -nographic
```

### Available services (via Traefik reverse proxy)

| Service           | URL                                                              |
| ----------------- | ---------------------------------------------------------------- |
| Backend API       | [http://localhost:3000](http://localhost:3000)                   |
| Traefik Dashboard | [http://traefik.localhost:3000](http://traefik.localhost:3000)   |
| Keycloak          | [http://auth.localhost:3000](http://auth.localhost:3000)         |
| MinIO Console     | [http://minio.localhost:3000](http://minio.localhost:3000)       |
| pgAdmin           | [http://pgadmin.localhost:3000](http://pgadmin.localhost:3000)   |
| Vroom             | [http://vroom.localhost:3000](http://vroom.localhost:3000)       |
| Valhalla          | [http://valhalla.localhost:3000](http://valhalla.localhost:3000) |

## Configuration ⚙️

All configuration is managed via environment variables — see `compose.app.yaml` and the NixOS module for examples.

### Common Variables

| Category                   | Example                                                                                                                                   |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Database**               | `GE_SERVER_DATABASE_HOST`, `GE_SERVER_DATABASE_PORT`, `GE_SERVER_DATABASE_NAME`, `GE_SERVER_DATABASE_USERNAME`, `GE_SERVER_DATABASE_PASSWORD` |
| **Auth (OIDC / Keycloak)** | `GE_AUTH_ENABLE`, `GE_AUTH_OIDC_PROVIDER_BASE_URL`, `GE_AUTH_OIDC_PROVIDER_TOKEN_URL`, etc.                                               |
| **Storage (S3 / MinIO)**   | `GE_S3_ENABLE`, `GE_S3_ENDPOINT`, `GE_S3_REGION`, `GE_S3_USE_SSL`, `GE_S3_ROUTE-GPX_BUCKET`                                               |
| **Routing**                | `GE_ROUTING_ENABLE`, `GE_ROUTING_VALHALLA_HOST`, `GE_ROUTING_VALHALLA_OPTIMIZATION_VROOM_HOST`                                            |

The `compose.yaml` and Nix setup include defaults for local development.

## Development 🧑‍💻

**Backend (Go)**

```bash
make tidy      # Format and tidy Go modules
make lint      # Lint Go code
make test      # Run tests
```

**Frontend (pnpm)**

```bash
make fe/dev        # Start dev server
make build/frontend
make fe/preview    # Preview after build
```

**Database**

```bash
make migrate/new name=create_users_table
make migrate/up
make seed/up
```

## Deployment 🚀

- Docker Compose: for local and testing deployments (`compose.yaml`, `compose.app.yaml`)
- Kubernetes: deployment manifests in `deploy/kustomize/`
- Nix Flakes: can produce reproducible builds and dev/test VMs

### How to Contribute 🤝

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

## Links 🔗

- 🌐 [Official Website](https://green-ecolution.de)
- 🖥️ [Live Demo](https://demo.green-ecolution.de)
- 🧑‍💻 [GitHub Repository](https://github.com/green-ecolution)
- 📘 [API Documentation](https://app.green-ecolution.de/api/v1/swagger/index.html)
- 🎓 [University of Applied Sciences Flensburg](https://hs-flensburg.de/en)
