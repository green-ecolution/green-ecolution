# Contributing to Green Ecolution

Thank you for your interest in contributing to Green Ecolution! This document provides guidelines and information on how to contribute.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Coding Standards](#coding-standards)
- [License](#license)

## Code of Conduct

Be respectful and constructive in all interactions. We welcome contributors of all backgrounds and experience levels.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/green-ecolution.git`
3. Add upstream remote: `git remote add upstream https://github.com/green-ecolution/green-ecolution.git`

## Development Setup

### Requirements

- Rust toolchain (rustup, includes cargo)
- Node.js + pnpm (`corepack enable`)
- Docker + Docker Compose
- `just` — command runner (`cargo install just`)
- `wasm-pack` (`cargo install wasm-pack`) — builds the domain WASM bindings, required by `just build`
- `bacon` (`cargo install bacon`) — live reload, required by `just run-dev` / `just run-live`
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features rustls,postgres`) for migrations and offline-cache regeneration

### Installation

```bash
just setup       # cargo fetch + pnpm install
just infra-up    # Start infrastructure (Postgres, Keycloak, MinIO, etc.)
just run-live    # Run backend with bacon hot reload
```

Frontend dev server (separate terminal):

```bash
just fe-dev
```

For a reproducible environment, use `nix develop`.

### Common Commands

| Command | Description |
|---------|-------------|
| `just test` | Run all tests (Rust workspace + frontend) |
| `just lint` | Lint Rust workspace + frontend |
| `just generate` | Run code generation |
| `just migrate-up` | Apply database migrations |
| `just generate-sqlx` | Refresh sqlx offline query cache (after changing any `query!` / `query_as!`) |

## Making Changes

### Branch Strategy

We use [Git-Flow](https://danielkummer.github.io/git-flow-cheatsheet/):

- `main` - Production-ready code
- `feature/*` - New features
- `bugfix/*` - Bug fixes
- `hotfix/*` - Urgent production fixes

**Create your branch from `main`:**

```bash
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name
```

### Making Changes

1. Write your code
2. Add tests for new functionality
3. Ensure all tests pass: `just test`
4. Ensure linting passes: `just lint`
5. Update documentation if needed

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/). This is important because commits directly affect changelog generation.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | Changelog Section |
|------|-------------|-------------------|
| `feat` | New feature | Features |
| `fix` | Bug fix | Bug Fixes |
| `docs` | Documentation only | - |
| `style` | Code style (formatting, etc.) | - |
| `refactor` | Code refactoring | - |
| `perf` | Performance improvement | Performance |
| `test` | Adding/updating tests | - |
| `chore` | Maintenance tasks | - |
| `ci` | CI/CD changes | - |

### Breaking Changes

Use `!` after the type or add `BREAKING CHANGE:` in the footer:

```
feat!: remove deprecated API endpoint
```

### Examples

```bash
# Feature
feat(tree): add bulk import functionality

# Bug fix
fix(sensor): correct moisture calculation

# Breaking change
feat(api)!: change response format for tree endpoints

BREAKING CHANGE: Tree response now includes nested cluster object
```

## Pull Request Process

1. **Update your branch:**

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Push your branch:**

   ```bash
   git push -u origin feature/your-feature-name
   ```

3. **Open a Pull Request** to `main` branch

4. **Fill out the PR template:**
   - Summary of changes
   - Link to issue (`close #123`)
   - Problem description
   - Solution description
   - Complete the Definition of Done checklist

### Definition of Done

Before requesting review:

- [ ] Code compiles without errors
- [ ] All tests pass (`just test`)
- [ ] No new linter warnings (`just lint`)
- [ ] Documentation updated (if applicable)
- [ ] Acceptance criteria from issue fulfilled

### Review Process

- At least 1 approval required
- Address all review comments
- Keep commits clean (squash if needed)

## Issue Guidelines

### Bug Reports

Use the [Bug Report template](https://github.com/green-ecolution/green-ecolution/issues/new?template=bug.yml):

- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Version information

### Feature Requests

Use the [Feature Request template](https://github.com/green-ecolution/green-ecolution/issues/new?template=feature.yml):

- Clear description of the feature
- Use case explanation
- Acceptance criteria

### Before Creating an Issue

- Search existing issues to avoid duplicates
- Check if the issue is already addressed in `main`

## Coding Standards

### Rust (Backend)

- Format with `cargo fmt --all` and lint with `cargo clippy --workspace --all-targets --all-features -- -D warnings` before pushing.
- Build with `--locked` and rely on `SQLX_OFFLINE=true` for CI; refresh the cache with `just generate-sqlx` whenever a `query!` / `query_as!` invocation changes.
- Domain code (`backend/crates/domain/`) must not depend on `sqlx`, `axum`, `tokio`, `reqwest`, `rumqttc`, or `tracing-subscriber`. `cargo build -p domain --no-default-features --locked` must stay green so the crate remains portable to WASM / mobile targets.
- Aggregate invariants live in private fields with intent-named methods that return `Vec<DomainEvent>`. HTTP handlers return `*View` types, never raw aggregates.
- Errors are typed: repository traits return `RepositoryError`; the HTTP layer maps to `ApiError`. Avoid `unwrap()` / `expect()` / `panic!` outside `reconstitute` paths and tests.
- Write tests next to the code (`#[cfg(test)] mod tests`) for unit tests; integration tests live in `backend/crates/server/test/api/`.

### TypeScript/React (Frontend)

- Use TypeScript strict mode
- Follow ESLint configuration
- Use existing UI components from `@green-ecolution/ui`
- Keep components focused and reusable

### General

- Prefer editing existing files over creating new ones
- Keep changes minimal and focused
- Don't add features beyond what's requested
- Avoid over-engineering

## Project Structure

```
backend/                  Cargo workspace (Rust API)
  Cargo.toml                 workspace manifest (resolver = "3")
  crates/
    domain/                  portable domain layer (aggregates, value objects,
                             repository traits, domain events, EventBus port).
                             No dependency on sqlx, axum, tokio — reusable on
                             WASM / mobile targets.
    server/                  server crate (axum router, Postgres adapters,
                             Keycloak, MQTT, composition root). Uses `domain`
                             with the `sqlx` feature enabled. Produces the
                             `green-ecolution` and `migrate` binaries.
  migrations/                sqlx-managed SQL migrations (workspace root)
  seeds/                     SQL seed data
  config/                    YAML config (base, local, production)
  .sqlx/                     committed offline query metadata
frontend/
  app/                       Main React application
  packages/
    ui/                      Shared UI components
    backend-client/          Generated OpenAPI client
    plugin-interface/        Plugin system interface
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing documentation in `CLAUDE.md`
- Review API docs at [app.green-ecolution.de/api/v1/swagger](https://app.green-ecolution.de/api/v1/swagger/index.html)

## License

By contributing to Green Ecolution, you agree that your contributions will be licensed under the [AGPL-3.0 License](LICENSE).

---

Thank you for contributing to Green Ecolution!
