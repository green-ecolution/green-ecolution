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

- Go (with CGO enabled)
- Node.js + pnpm (`corepack enable`)
- Docker + Docker Compose

### Installation

```bash
make setup       # Install Go and pnpm dependencies
make infra/up    # Start infrastructure (Postgres, Keycloak, MinIO, etc.)
make run/live    # Run backend with hot reload
```

Frontend dev server (separate terminal):

```bash
make fe/dev
```

For a reproducible environment, use `nix develop`.

### Common Commands

| Command | Description |
|---------|-------------|
| `make test` | Run all tests |
| `make lint` | Lint Go + frontend |
| `make generate` | Run code generation |
| `make migrate/up` | Apply database migrations |

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
3. Ensure all tests pass: `make test`
4. Ensure linting passes: `make lint`
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
- [ ] All tests pass (`make test`)
- [ ] No new linter warnings (`make lint`)
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

### Go (Backend)

- Follow standard Go formatting (`gofmt`)
- Use meaningful variable and function names
- Keep functions focused and small
- Add comments only where logic isn't self-evident
- Write tests for new functionality

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
backend/          Go API server
frontend/
  app/           Main React application
  packages/
    ui/          Shared UI components
    backend-client/  Generated API client
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing documentation in `CLAUDE.md`
- Review API docs at [app.green-ecolution.de/api/v1/swagger](https://app.green-ecolution.de/api/v1/swagger/index.html)

## License

By contributing to Green Ecolution, you agree that your contributions will be licensed under the [GPL-3.0 License](LICENSE).

---

Thank you for contributing to Green Ecolution!
