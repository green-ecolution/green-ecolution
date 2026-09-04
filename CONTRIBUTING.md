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
- `bacon` (`cargo install bacon`) — live reload, required by `just run-dev`
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features rustls,postgres`) for migrations and offline-cache regeneration
- JDK 21 + Apache Maven — required to build the Keycloak login theme (`just build-keycloak-theme`)

### Installation

```bash
just setup       # cargo fetch + pnpm install + build frontend workspace packages + domain WASM
just infra-up    # Start infrastructure (Postgres, Keycloak, MinIO, etc.)
just run-dev     # Backend + frontend dev via Traefik (bacon live reload)
```

Frontend dev server only (backend must run separately):

```bash
just frontend-dev    # alias: fe-dev
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
| `just build-keycloak-theme` | Build the Keycloak login theme (jar + exploded theme dir under `frontend/keycloak-theme/dist_keycloak/theme/`) |
| `just keycloak-theme-dev` | Run the theme's Vite dev server against a mocked Keycloak context |

### Keycloak Login Theme

`just setup` builds the theme automatically, and `just infra-up` prints a warning if the built
theme directory is missing, but it is still worth knowing what can go wrong:

- The theme build needs a JDK and Apache Maven. `keycloakify build` packages the theme jar by
  shelling out to `mvn clean install`; without `mvn` on the `PATH` it aborts before writing
  anything. `nix develop` provides both; otherwise install a JDK and Maven from your
  distribution.
- If the theme was never built (or the build failed), `compose.yaml` mounts an empty directory
  into Keycloak's theme path, and Keycloak falls back to its default login page. This looks like
  nothing is wrong, so run `just build-keycloak-theme` and refresh the page if a local login
  doesn't look branded.

### User Handbook

The application's user documentation lives in `frontend/handbook/`. Chapters are Markdown
files with YAML frontmatter, one per file, under `frontend/handbook/content/de/`; a single
generator turns them into both the in-app help under `/help` and a PDF, so a chapter never
has to be written twice for the two outputs.

Only the Markdown constructs the generator can render identically in both outputs are
permitted, and that set is closed: anything outside it (an unsupported block or inline node,
a heading deeper than `###`, a bare relative link, and so on) fails the build with an error
naming what was rejected, rather than quietly rendering differently between the app and the
PDF. The generator itself (`frontend/handbook/src/blocks.mjs` and `inline.mjs`) is the source
of truth for what is allowed.

To render the PDF locally:

```bash
just handbook-pdf
```

This requires Typst, which ships in the Nix development shell (`nix develop`); outside Nix,
install it separately and match the version pinned in `frontend/Dockerfile`, since
`just handbook-pdf` warns when the local version drifts from it.

`just build-frontend` (and therefore `just build`) renders the PDF as well, because the
handbook page offers it as a download and a `dist` without it would serve the SPA fallback
under the download link. Typst is required for those builds too.

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

#### User-facing text

All user-facing text lives in the translation catalogs, never hardcoded in a
component. Add the German string to `frontend/app/src/locales/de/<namespace>.json`
and the English one to `.../en/<namespace>.json`, then read it with
`useTranslation('<namespace>')`. Component copy in `frontend/packages/ui` goes
into that package's own catalog at `src/i18n/catalog.ts` instead.

`frontend/app` enforces this with the `i18next/no-literal-string` ESLint rule
(`frontend/app/eslint.config.js`), scoped to `src/**/*.{ts,tsx}` with an
allowlist for product names, technical literals (units, aria roles, test ids,
routes) and the debug routes/components, which are deliberately untranslated.
`frontend/packages/ui` has no equivalent lint rule — its exposure is low since
its copy is already centralized in one catalog file — so reviewers reject a
hardcoded JSX text node or quoted sentence there on sight. The `de`/`en`
parity test fails when only one of the two catalogs is updated.

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
  config/                    YAML config (base, local, staging, demo, production)
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
