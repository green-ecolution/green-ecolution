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

**Core Stack:**

- [React 19](https://react.dev/) — UI library
- [TypeScript](https://www.typescriptlang.org/) — type safety
- [Vite](https://vitejs.dev/) — fast dev server and bundler
- [pnpm](https://pnpm.io/) — workspace-based package manager (required)

**Routing & State Management:**

- [TanStack Router](https://tanstack.com/router) — type-safe file-based routing
- [TanStack Query](https://tanstack.com/query) — server state management
- [Zustand](https://github.com/pmndrs/zustand) — client state management

**Forms & Validation:**

- [React Hook Form](https://react-hook-form.com/) — form handling
- [Zod](https://zod.dev/) — schema validation

**UI & Visualization:**

- [Leaflet](https://leafletjs.com/) — interactive maps
- [shadcn/ui](https://ui.shadcn.com/) — UI components
- [Tailwind CSS](https://tailwindcss.com/) — utility-first CSS

**Development Tools:**

- [ESLint](https://eslint.org/) + [Prettier](https://prettier.io/) — code quality
- [OpenAPI Generator](https://openapi-generator.tech/) — auto-generated API client
- [Module Federation](https://module-federation.io/) — plugin system

## Project Structure 📁

This is a **pnpm workspace** monorepo with three packages:

```
frontend/
├── app/                      # Main React application
│   ├── src/
│   │   ├── routes/          # File-based routing (TanStack Router)
│   │   │   ├── _protected/  # Protected routes (requires auth)
│   │   │   └── index.tsx    # Public routes
│   │   ├── components/      # React components
│   │   ├── store/           # Zustand stores (auth, map, user)
│   │   ├── hooks/           # Custom React hooks
│   │   ├── api/             # API client wrappers
│   │   └── lib/             # Utilities and helpers
│   └── package.json
│
├── packages/
│   ├── backend-client/      # Auto-generated OpenAPI client
│   │   └── src/             # Generated from backend Swagger spec
│   │
│   └── plugin-interface/    # Plugin system types and interfaces
│       └── src/             # Shared plugin contracts
│
└── pnpm-workspace.yaml      # Workspace configuration
```

**Build Order:** `backend-client` and `plugin-interface` must be built before the main `app`.

## Plugin System 🔌

The frontend supports **runtime plugins** using **Module Federation**:

- Plugins are loaded dynamically at runtime
- Each plugin can contribute routes, components, and functionality
- Plugin interface is defined in `packages/plugin-interface/`
- Plugins register via backend API and communicate via typed contracts

See [`packages/plugin-interface/README.md`](packages/plugin-interface/README.md) for plugin development guide.

## Local development 💻

### Requirements

- [Node.js](https://nodejs.org/en/) (recommended via `corepack` or `fnm`)
- [pnpm](https://pnpm.io/)
- [just](https://github.com/casey/just)
- Optional: [Nix](https://nixos.org/) for reproducible setup

### Setup ⚙️

From the repository root (recommended):

```bash
just setup
```

or manually inside the frontend folder:

```bash
pnpm install
```

## Running the Project ▶️

Start the development server:

```bash
just fe-dev
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
just build-frontend
```

The build output is placed in:

```bash
frontend/dist/
```

When running `just build` from the repository root, the build artifacts are automatically embedded into the backend binary for unified deployment.

### Regenerating API Client 🔄

The `packages/backend-client` is auto-generated from the backend's OpenAPI specification:

```bash
# From frontend/ directory
pnpm run generate          # Generate from running local backend
pnpm run generate:stage    # Generate from staging API
pnpm run generate:local    # Generate from local API (explicit)
```

After regenerating, rebuild the client:

```bash
pnpm run build:backend-client
```

### Linting & Testing ✅

```bash
just lint
just test
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
