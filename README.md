<p>
  <a href="https://github.com/green-ecolution/green-ecolution-backend/releases">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/green-ecolution/green-ecolution-backend"/>
  </a>
  <a href="https://pkg.go.dev/github.com/green-ecolution/green-ecolution-backend">
    <img src="https://pkg.go.dev/badge/github.com/green-ecolution/green-ecolution-backend.svg" alt="Go Reference">
  </a>
  <a href=""><img alt="Code coverage" src="https://raw.githubusercontent.com/green-ecolution/green-ecolution-backend/badges/.badges/develop/coverage.svg"/></a>
  <a href=""><img alt="License" src="https://img.shields.io/github/license/green-ecolution/green-ecolution-backend.svg"/></a>
  <a href=""><img alt="Maintained yes" src="https://img.shields.io/badge/Maintained%3F-yes-green.svg"/></a>
</p>

# Green Ecolution Backend 🌿

<p align="center">
  <img src="https://github.com/user-attachments/assets/4ea25141-135a-493c-b9f6-e1cbc7a7aa41"/>
</p>

Smart irrigation is essential to saving water, reducing staff workload, and cutting costs. This project provides the backend for **Green Ecolution** — a digital system to manage urban greenery efficiently.

👉 For the frontend implementation, visit the [Green Ecolution Frontend](https://github.com/green-ecolution/green-ecolution-frontend).

The backend acts as the interface between the website and the database, managing data about:

- 🌳 Trees
- 🌿 Tree clusters
- 📡 Sensors
- 🗺️ Watering plans
- 🚛 Vehicles
- 👤 Users

In the current setup, sensors are connected to a microcontroller with an LoRaWAN modules. Sensor data is transmitted via LoRaWAN to an MQTT gateway, then forwarded to the server for processing.

Developed in collaboration with **TBZ Flensburg**, this software is designed to be adaptable for other cities. It originated as a research project within the **Applied Computer Science Master's program** at the **University of Applied Sciences Flensburg**.

For further information, visit:

- 🌐 [Project website](https://green-ecolution.de/)
- 🖥️ [Live demo](https://demo.green-ecolution.de)
- 🎓 [University of Applied Sciences Flensburg](https://hs-flensburg.de/en)

## Quick Start Guide 🚀

To quickly run the application locally:

1. Download the Docker Compose configuration:

```bash
wget https://raw.githubusercontent.com/green-ecolution/green-ecolution-backend/refs/heads/develop/.docker/docker-compose.yaml
```

2. Build and launch the application containers:

```bash
docker compose up
```

By default, external services like `auth`, `mqtt`, `routing`, and `s3` are disabled but can be enabled via configuration. See the [Demo Setup](https://github.com/green-ecolution/green-ecolution-backend/wiki/Demo-Setup) for more details.

👉 Once the service is up and running, you can access it at: [http://localhost:8123](http://localhost:8123)

## Technologies Used ⚙️

- [Go (Golang)](https://go.dev/) as the main language
- [env](https://github.com/caarlos0/env) for environment configuration
- [godotenv](https://github.com/joho/godotenv) for `.env` file handling
- [Fiber](https://docs.gofiber.io/) as the web framework
- [Testify](https://github.com/stretchr/testify) for unit testing

## Local Development Setup 💻

For a detailed step-by-step guide on setting up your local development environment, refer to the [Local Development Wiki](https://github.com/green-ecolution/green-ecolution-backend/wiki/Local-Development) 📖.

### Prerequisites

- [Golang](https://go.dev/)
- [Air](https://github.com/air-verse/air) for live reload
- [Mockery](https://github.com/vektra/mockery) `v2.52.2` for generating mocks
- [Make](https://www.gnu.org/software/make/)
- [Docker](https://github.com/docker)
- [Docker Compose](https://github.com/docker/compose)

### Initial Setup ⚙️

Install required tools:

```bash
make setup
```

Generate code:

```bash
make generate
```

Start the infrastructure services using Docker Compose. This setup includes:

- a local PostgreSQL database,
- routing services such as Valhalla, ORS and VROOM for route and vehicle scheduling calculations

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
