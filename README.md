# Green Ecolution 🌿

<p align="center">
  <img src="https://github.com/user-attachments/assets/4ea25141-135a-493c-b9f6-e1cbc7a7aa41"/>
</p>

## Project Overview 🚀

Green Ecolution is a smart urban irrigation system that uses IoT sensor data to efficiently manage green spaces like parks and urban trees.
This repository serves as the central management point for the Green Ecolution ecosystem:

- Holds the backend and frontend as Git submodules
- Provides Kubernetes configurations and deployment files
- Supports local development setups

If you're looking to dive into the backend or frontend development specifically, refer to the respective submodules for detailed instructions.

## Local Deployment ⚙️

To set up a local development environment:

1. Clone the repository along with its submodules:

```bash
git clone git@github.com:green-ecolution/green-ecolution.git
git submodule update --init --recursive
```

2. Update submodules as needed when changes are pushed:

```bash
git submodule update --remote --merge
```

## Repository Structure 📂

```
.
├── backend/            # Backend service (as submodule)
├── frontend/           # Frontend application (as submodule)
└── deploy/kustomize/   # Kubernetes deployment configurations
```

## Nix & Flakes 🧊

If you want a fully reproducible development setup or prefer to avoid installing tools globally, you can use [Nix](https://nixos.org/) with Flakes.

### What does it provide?

- Reproducible build environments for backend and frontend
- Automatic shell environments (with [direnv](https://direnv.net/))
- Consistent dependency pinning via Go modules and pnpm lockfiles

### Build

To build everything in one reproducible step:

```bash
# Build only the backend
nix build github:green-ecolution/green-ecolution#backend

# Build only the frontend
nix build github:green-ecolution/green-ecolution#frontend

# Build a single backend binary that includes the compiled frontend assets
nix build github:green-ecolution/green-ecolution
```

💡 Tip: Using Nix ensures fully reproducible builds across all supported systems.

## Useful Links 🔗

- 🌐 [Green Ecolution Website](https://green-ecolution.de)
- 🖥️ [Live Demo](https://demo.green-ecolution.de)
- 📚 [Backend Repository](https://github.com/green-ecolution/backend)
- 📚 [Frontend Repository](https://github.com/green-ecolution/frontend)
