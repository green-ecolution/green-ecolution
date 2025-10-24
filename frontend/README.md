# Green Ecolution Frontend 🌿

<p align="center">
  <img src="https://github.com/user-attachments/assets/4ea25141-135a-493c-b9f6-e1cbc7a7aa41"/>
</p>

Smart irrigation is essential to saving water, reducing staff workload, and cutting costs. This project provides the user interface for Green Ecolution — a digital system to manage urban greenery efficiently.

👉 For the backend implementation, see the [backend folder](../backend).

## Overview 🧠

The frontend connects to the backend API and enables users to manage and visualize:

- 🌳 Trees and vegetation data
- 🌿 Tree clusters and zones
- 📡 IoT sensors and telemetry
- 🗺️ Watering routes and plans
- 🚛 Vehicle tracking and task planning
- 👤 User and access management

Developed in collaboration with **TBZ Flensburg**, this system was originally built as part of the **Applied Computer Science Master's program** at the **University of Applied Sciences Flensburg** and is adaptable for other cities and organizations.

For further information, visit:

- [🌐 Project website](https://green-ecolution.de/)
- [🎓 University of Applied Sciences Flensburg](https://hs-flensburg.de/en)
- [🖥️ Live demo](https://demo.green-ecolution.de)

## Technologies ⚙️

- [React](https://react.dev/) — UI library
- [Vite](https://vitejs.dev/) — fast dev server and bundler
- [TypeScript](https://www.typescriptlang.org/) — type safety
- [ESLint](https://eslint.org/) + [Prettier](https://prettier.io/) — code linting and formatting
- [pnpm](https://pnpm.io/) — fast and space-efficient package manager
- [Vite Environment Variables](https://vitejs.dev/guide/env-and-mode.html) — for backend endpoint configuration

## Local development 💻

### Requirements

- [Node.js](https://nodejs.org/en/) (recommended via `corepack` or `fnm`)
- [pnpm](https://pnpm.io/)
- [Make](https://www.gnu.org/software/make/)
- Optional: [Nix](https://nixos.org/) for reproducible setup

### Setup ⚙️

From the repository root (recommended):

```bash
make setup
```

or manually inside the frontend folder:

```bash
pnpm install
```

## Running the Project ▶️

Start the development server:

```bash
make fe/dev
```

Or directly with pnpm:

```bash
pnpm run dev
```

By default, it connects to the local backend at `http://localhost:3000/api`.
To use a remote backend (e.g. staging or production):

```bash
pnpm run dev:remote
```

The frontend will be available at: 👉 <http://localhost:5173>

### Building 🏗️

Build the production-ready frontend:

```bash
make build/frontend
```

The build output is placed in:

```bash
frontend/dist/
```

When running `make build` from the repository root, the build artifacts are automatically embedded into the backend binary for unified deployment.

### Linting & Testing ✅

```bash
make lint
make test
```

or directly:

```bash
pnpm run lint
pnpm run test
```

### Environment Variables 🌍

Frontend environment variables (in `.env` or via CLI):

| Variable               | Description                                                   | Default |
| ---------------------- | ------------------------------------------------------------- | ------- |
| `VITE_BACKEND_BASEURL` | Backend API base URL                                          | `/api`  |
| `VITE_APP_ENV`         | Environment mode (`local`, `staging`, `production`, `docker`) | `local` |

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
