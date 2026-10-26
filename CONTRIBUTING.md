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
- [Writing a Plugin](#writing-a-plugin)
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
just infra-up    # Start infrastructure (Postgres, Keycloak, Valhalla, etc.)
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
| `just test-clean` | Remove testcontainer leftovers (only a hard kill of the test runner can leave any behind) |
| `just lint` | Lint Rust workspace + frontend |
| `just generate` | Run code generation |
| `just migrate-up` | Apply database migrations |
| `just migrate-new <name> [module]` | Create a new migration in `backend/migrations`, or in `backend/crates/modules/<module>/migrations` when a module key is given |
| `just generate-sqlx` | Refresh sqlx offline query cache (after changing any `query!` / `query_as!`) |
| `just build-keycloak-theme` | Build the Keycloak login theme (jar + exploded theme dir under `frontend/keycloak-theme/dist_keycloak/theme/`) |
| `just keycloak-theme-dev` | Run the theme's Vite dev server against a mocked Keycloak context |
| `just bench` | Run the micro-benchmarks of `domain` and `irrigation-domain` (Criterion) |
| `just bench-check` | Execute every benchmark once without measuring; this is what CI runs |

### geco (optional developer console)

`geco` starts the local stack as one dependency graph (compose services plus the backend via
bacon and the Vite dev server), merges their logs and shows them in a terminal UI. `just`
stays the documented default; geco is a convenience on top of it and reads its recipes and
variables from the justfile.

Inside `nix develop`, `geco` is already on the `PATH`. It is a small wrapper that builds geco
from the working tree on first use and again after every change, so it always matches the
checked-out branch; the build lands in `backend/target/geco`. `nix run .#geco` runs a geco
built from the flake without entering the shell. Without Nix, install it with cargo:

```bash
cargo install --path backend/crates/devcli --locked   # installs the `geco` binary
geco                      # TUI; starts the default profile if nothing runs yet
geco up [--detach]        # start without the TUI (foreground log stream, or in the background)
geco status [--json]      # unit states; exit code 1 while a unit is not ready
geco logs backend -f      # follow one or more units; -F 'level>=warn "jwks"' filters
geco trace <request-id>   # one request's log lines, indented by span (a unique prefix is enough)
geco doctor [--fix]       # known setup pitfalls with their fix; --deep adds the slow checks
geco profile [NAME]       # list the profiles, or switch the running stack to NAME
geco restart backend      # --cascade also restarts dependent units
geco run migrate-up       # run a just recipe as a task of the running stack (plain `just` without one)
geco completions zsh      # completion script; e.g. `source <(geco completions zsh)` in ~/.zshrc
geco down                 # stop everything geco started
```

In the TUI, `?` lists the keys of the current view in a box that closes with `?` or `Esc`.
`:` or `Ctrl-K` opens a command palette that searches every action: starting,
stopping and restarting units, switching profiles, running recipes, applying doctor fixes and
setting the log filter. Recipes with parameters ask for them in the status line. Recipes listed
under `[tasks] destructive` in `geco.toml` ask for confirmation first, and `geco run` does the
same unless `--yes` is given. That flag belongs to geco and goes before the recipe. The
recipe's own flags follow its arguments, for example
`geco run --yes import-staging staging.sql --yes`; directly after the recipe name, geco would
still read `--yes` as its own. Tasks run without stdin, so a recipe that prompts needs its own
flag to skip the question. A running recipe shows up as a temporary unit and disappears once it
succeeded.

While a local unit compiles Rust code, for example bacon after a file change or the `migrate`
build in `migrations`, the TUI shows a build panel above the status line. It lists cargo's
progress per unit with the crates done out of the total, the crates compiling right now and the
elapsed time, and keeps a finished or failed build there for five seconds. For this geco sets
`CARGO_TERM_PROGRESS_WHEN=always` and `CARGO_TERM_PROGRESS_WIDTH=200` for every local unit,
since cargo draws no progress bar into a pipe otherwise; a unit that sets either variable in
its `env` keeps its own value.

In a terminal, `geco up --detach` shows the start-up live and returns once all units are ready.
It exits with 1 when the start-up settled with failures; the supervisor keeps running in that
case, so `geco logs` and `geco restart` still work. Ctrl-C only stops watching and leaves the
start running in the background. Without a terminal, or with `NO_COLOR` set, it returns as soon
as the supervisor answers.

Units and profiles (`minimal`, `full`, `prod-like`) are defined in `geco.toml` at the
repository root; a profile can extend another and override local units. Switching profiles at
runtime restarts the units whose definition or dependencies changed, together with the units
that depend on them, and also starts units that were stopped or failed; everything else keeps
running. In the TUI, `p` switches the profile.
Runtime state lives in `.geco/` and is git-ignored. geco needs Linux or macOS.

The filter language combines terms with AND: `level>=warn`, `level:warn,error`,
`unit:backend,db`, `target:sqlx` (prefix), `req:<id prefix>`, `since:5m` and quoted or bare
text, matched case-insensitively against the message. In the TUI, `/` opens the same filter
and `t` lists the backend's requests next to the selected request's trace.

`geco doctor` checks for known local-setup pitfalls: a dev database behind the
migrations, a stale `migrate` binary, an outdated `.sqlx` cache, workspace packages built from
an old `dist`, a missing or stale `domain-wasm/pkg`, testcontainer leftovers, the missing
Keycloak theme or Valhalla tiles, missing tools and occupied ports. In the TUI, `d` lists the
findings and `f` runs a fix after asking. `r` checks again and `R` adds the slow checks; while a
check or fix runs, the doctor title and the status line show it with a spinner and the elapsed
time, and the status line reports the result once it is done.

### Benchmarks

There are two separate tools, because the two questions they answer need
different instruments.

`just bench` runs Criterion micro-benchmarks over the pure, database-free
domain functions in `backend/crates/domain/benches/` (authorization, settings
resolution) and `backend/crates/modules/irrigation/domain/benches/` (cluster
recalculation, watering status), parametrized over growing input sizes. They are
fast and deterministic, and `just bench-check` executes each of them once without
measuring, which is what backend PRs run to make sure they still compile.

The large-data harness is a separate binary, `benchdb` in the `app` crate (it
fills core and irrigation tables together), which seeds a dedicated
Postgres database in steps and measures the real repository implementations
against it. It never runs automatically and it never runs against your
development database: the recipes require `BENCH_DATABASE_URL` and refuse to
start without it.

```bash
export BENCH_DATABASE_URL=postgres://postgres:postgres@localhost:5432/bench
just bench-db-seed s 90       # seed 10k trees with 90 days of sensor history
just bench-db-run             # measure once against what is there
just bench-db-campaign        # grow through every scale and measure after each
```

A campaign writes `results.csv`, the captured query plans and an
`environment.json` into `backend/target/bench/`. The CSV carries one row per
path and scale, so the growth from one scale to the next is read off it
directly.

Two things to plan for: the sensor time series dominates the data volume, so
the full default of 730 days at the largest scale is a multi-hour, multi-GB job
(pass a smaller `--history-days` for a routine run), and the absolute timings
come from your local Postgres and do not transfer to production hardware. What
transfers is the shape of the curve.

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

A change a user would notice belongs in the handbook in the same pull request: a new feature,
a renamed label, a changed workflow, a status that no longer exists. The `routes` list in a
chapter's frontmatter links the context-help button in the application to that chapter, so a
new or renamed route belongs there too.

Figures are part of that. `frontend/handbook/images/README.md` holds the brief and the capture
conditions for every screenshot in the handbook. A feature that needs its own figure gets a
brief there; a change that makes an existing screenshot wrong should say whether that figure
is to be re-shot, replaced by a different view, or dropped, rather than leaving a stale one in
place. You do not have to take the capture yourself: write a placeholder with `writePlaceholder`
from `frontend/handbook/src/placeholder.mjs` so the PDF still builds, and say so in the pull
request.

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

`just run-dev` and `just frontend-dev` render the PDF only when it is missing, so a normal
dev session does not pay for it on every start. Without Typst they print a warning and carry
on; the download link then answers 404 instead of handing out the app shell under a `.pdf`
name.

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
| `docs` | Documentation only | Documentation |
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
- Domain code (`backend/crates/domain/` and every module domain crate such as `backend/crates/modules/irrigation/domain/`) must not depend on `sqlx` (except behind the optional `sqlx` feature), `axum`, `tokio`, `reqwest`, `rumqttc`, or `tracing-subscriber`. `cargo build -p domain --no-default-features --locked` and `cargo build -p irrigation-domain --no-default-features --locked` must stay green so the crates remain portable to WASM / mobile targets.
- `domain` and `server` never depend on a module crate, and module crates depend only on the core, never on each other. Only `app` (and `domain-wasm` for the domain crates) knows the modules.
- Aggregate invariants live in private fields with intent-named methods that return `Vec<DomainEvent>`. HTTP handlers return `*View` types, never raw aggregates.
- Errors are typed: repository traits return `RepositoryError`; the HTTP layer maps to `ApiError`. Avoid `unwrap()` / `expect()` / `panic!` outside `reconstitute` paths and tests.
- Write tests next to the code (`#[cfg(test)] mod tests`) for unit tests; integration tests live in `backend/crates/app/test/api/`. Run a single one with `cargo test -p app --test api -- <name> --nocapture`.

### Writing a new list query

The cluster, vehicle and sensor list endpoints (and every list endpoint since) are built
on a shared builder, `ListSpec` in `backend/crates/server/src/infra/list/mod.rs`. A
repository's `view_search` method constructs one `ListSpec` with the `FROM` clause and
primary key (`ListSpec::new("vehicles v", "v.id")`), attaches the scope predicates that
are not the caller's own filters (organization visibility, provider), attaches the
caller's filter predicates, sets the sort, and calls `.fetch::<RowType>(&self.pool,
COLUMNS)`. `ListSpec` runs the count, the pre-filter count and the page itself from that
one description, so the three statements cannot drift apart the way three hand-written
queries would.

To add a predicate, reach for one of the constructors in `infra/list/predicate.rs`, then
pass the result to `.filter(...)` on the spec: `Predicate::text_search` for a search
field, `Predicate::any_of` for a multi-select filter, `Predicate::equals` or
`Predicate::is_present` for a single value or a presence check, and `Predicate::archived`
for the include/active-only/archived-only three-way a list with archiving needs. Each
constructor returns `None` for an absent or empty filter, so a repository can chain
`.filter(...)` calls unconditionally without an `if` for every optional query parameter.

To add a sort column, add an entry to the repository's own `&'static` `SortColumns` table
(for example `VEHICLE_SORT_COLUMNS` in `pg_vehicle.rs`), mapping the sort key string to
the SQL expression(s) that implement it. That key string comes from the domain enum's
`as_sql_key()` (`VehicleSortField` in `crates/domain/src/vehicle/sort.rs`, and its
counterparts for the other aggregates), which is the only place a new sort option
originates: add the variant there, wire it through the HTTP sort DTO, and extend the sort
column table to match. Each of these domain sort enums carries a test asserting that
`as_sql_key()` produces exactly the set of strings the corresponding `SortColumns` table
expects (`sql_keys_are_exactly_the_contracted_set` in `vehicle/sort.rs` is the pattern to
copy); adding a sort option without extending that test leaves the contract unchecked, so
update it in the same change.

Finally, add an integration test per filter and per sort option in
`backend/crates/app/test/api/`, following the existing naming
(`vehicle_list_searches_plate_model_and_description`,
`vehicle_list_filters_by_derived_status`, `vehicle_list_sorts_by_water_capacity_descending`,
`vehicle_list_can_include_and_isolate_archived_vehicles`). These are what actually catch a
wrong column name or a sort direction applied to the wrong expression.

That last point matters because these queries are deliberately not covered by sqlx's
compile-time checks: `ListSpec` assembles its SQL at runtime through `QueryBuilder`, since
the set of active filters and the sort column are only known once a request arrives, and
`query!`/`query_as!` can only check a query whose text is fixed at compile time. A typo in
a column name inside a `SortColumns` table or a `Predicate` expression therefore compiles
without complaint and fails only when a test (or a request) actually runs it, which is
exactly why every filter and sort key needs its own integration test rather than relying
on the workspace build to catch a mistake here.

This is safe regardless, for two reasons that hold in every repository built on
`ListSpec`. Every value a request supplies, a search term, a selected status, a page
number, reaches SQL only through `push_bind`, never through string interpolation, so a
request body can never inject SQL text no matter what it contains. And the query text
itself, meaning table names, joins, column names and the `WHERE` clause structure, is
authored entirely in the repository by whoever writes the code; a request can only ever
select among fixed, closed alternatives (a domain enum's variants, a boolean, a UUID from a
known set), never supply a column name or expression of its own. `infra/list/mod.rs`
states this as the module's own invariant, and it is what makes runtime-assembled SQL
here no less safe than a `query!` macro would be.

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

#### Accessibility

`eslint-plugin-jsx-a11y` runs in its `strict` configuration in `frontend/app`,
`frontend/packages/ui` and `frontend/keycloak-theme`, at error level, so an
accessibility regression fails the build rather than a review. Two deviations are
configured rather than left to individual files: the app allows `onBlur`/`onFocus`
on a `<form>`, because those handlers delegate from the natively focusable controls
inside it, and the rules are off in test files, where an `<a>` stub standing in for
a router link is not a defect.

Everything else is expected to be fixed rather than silenced, and a disable is the
last resort, not the first reflex. Most findings have a restructuring that removes
them for real: a primitive that spreads `{...props}` onto a heading or anchor can
destructure `children` and render them, which is what the rule was asking for; a
key handler on a container with roving tabindex belongs on the items that actually
receive focus; and an `autoFocus` inside a Radix dialog is usually redundant,
because the dialog already moves focus to the first control.

Only where a rule is genuinely wrong about correct markup, disable that one rule on
that one line with `eslint-disable-next-line` and a comment saying why. Three such
cases exist today: the input group whose click only forwards focus the way a
`<label>` does, and the two `autoFocus` uses whose behaviour is deliberate. Note
what does not count as a fix: replacing `autoFocus` with a ref and an effect
produces exactly the same behaviour where the linter cannot see it, so the disable
that records the decision is the better artefact. A blanket disable for a whole file
or directory is not acceptable either.

The linter only sees markup. It cannot tell whether focus order, colour contrast or
a live region actually work, so it is a floor and not a substitute for testing a
change with the keyboard and a screen reader.

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
    server/                  server library (axum router, Postgres adapters,
                             Keycloak, MQTT, application services). Uses `domain`
                             with the `sqlx` feature enabled.
    app/                     composition root (`src/startup.rs`, registered
                             modules in `src/modules.rs`) and the
                             `green-ecolution`, `migrate`, `dump-openapi` and
                             `benchdb` binaries. Integration tests live in
                             `app/test/api/`.
    domain-wasm/             wasm-bindgen wrapper over the core and module
                             domain drafts, used by the frontend for validation.
    modules/
      irrigation/domain/     `irrigation-domain`: trees, clusters, soil physics;
                             portable like `domain`.
      irrigation/server/     `irrigation-server`: the irrigation module's
                             services, Postgres adapters, routes, event handlers
                             and the `import-kataster-fl` binary.
    devcli/                  `geco` developer console (TUI + CLI); not part of
                             any deployed artifact.
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
  modules/
    irrigation/              Frontend module package of the irrigation module
```

Features that can be switched on per organization are called modules. An organization
enables or disables a module with `PUT /api/v1/organizations/{org_id}/modules`, and the
setting is inherited down the organization tree the same way as the other organization
settings. Module endpoints answer with `module.disabled` (403) for an organization that
has not activated the module.

A module is a pair of crates under `backend/crates/modules/<name>/`: `<name>-domain`
for its aggregates and value objects (same portability rules as `domain`) and
`<name>-server` for services, Postgres adapters, routes and event handlers. The
server crate exports a type implementing `server::module::Module`, and
`backend/crates/app/src/modules.rs` lists it in `registered()`. The reference
implementation is the irrigation module (`irrigation-domain`, `irrigation-server`); a
minimal one for tests lives in `backend/crates/app/test/api/test_module/`
(`ThingCatalog`, `ThingPlanning`). Because the core does not depend on any module, the
compiler rejects a module that reaches into core internals; everything a module may use
is handed to `Module::build` in the `ModuleContext` (pool, event bus, authorization,
`ModuleAccess`, core readers and the narrow core services). The routers a module returns
already carry their own state, so a module handler cannot extract the core's
`AppState`.

A module plugs into the core through three extension points for permissions, targets
and planning, plus a few smaller hooks described after them.

`Module::resources()` returns the `ResourceKey`s the module adds to the permission
model; the core merges them into the `ResourceCatalog` it builds the permission set from
and rejects duplicate keys. A role permission for a resource that no registered module
declares is ignored and logged once, and a plugin permission for an unknown resource is
dropped silently, so removing a module does not break existing roles or plugins.

`TargetCatalog` (in `domain/src/target/`) lists the target kinds a module owns, as
`TargetKindSpec` entries that say whether a kind accepts sensors, whether it can be
commented on and which resource guards it. The core asks it to admit a target, locate it
and label it. The catalogs are collected in the `TargetRegistry`
(`backend/crates/server/src/service/`), which sensor attachments and target comments use
instead of knowing concrete kinds.

`PlanningProvider` (in `domain/src/planning.rs`) lets a module plan with the generic
operation plans: it names the capacity kind, the resource that guards its plans and
resolves stops to the objects behind them. The providers are collected in the
`PlanningRegistry` (`backend/crates/server/src/service/`). A module that deletes an
object which can be a stop must call `OperationPlanService::remove_stop`, so no plan
keeps a dangling stop.

`ModuleStatistics` (in `domain/src/module/statistics.rs`) returns the module's object
counts as `kind`/`count` pairs; `GET /api/v1/info/statistics` adds the module key and
lists them under `modules` next to the core counts. The labels come from the frontend,
not from the backend.

A module's own domain events travel the core event bus wrapped in
`DomainEvent::Module(ModuleEvent)`; its handlers recover them with `downcast_ref`
(irrigation wraps this in `IrrigationEvent::into_domain` / `from_domain`). Errors with a
module-specific code reach the HTTP layer as `ServiceError::Module { kind, code,
message }`, where `kind` selects 409 or 422 and `code` is the stable error code.
Long-running work such as a periodic sweep is returned as a `BackgroundTask`, which the
composition root spawns at startup.

A module reads its instance configuration from its own `modules.<key>` section of the
YAML config through `ModuleContext::module_config`; `modules.<key>.default_enabled` is
reserved for the activation default. The irrigation defaults live under
`modules.irrigation` (`tree_demand_liters`, `just_watered_ttl_secs`,
`just_watered_sweep_interval_secs`, `nearest_tree_max_radius`,
`nearest_tree_default_limit`, `nearest_tree_max_limit`). The server refuses to start
while one of the old locations of these keys (`routing.tree_demand_liters`,
`watering.just_watered_*`, `map.nearest_tree_*`) is still set, in a file or as an `APP_…`
environment variable, and the error names the new key.

### Frontend Module Packages

The frontend side of a module is a pnpm package under `frontend/modules/<name>/` (for
irrigation: `@green-ecolution/module-irrigation`). It holds the module's pages, components,
API wrappers, store, validation schemas and translation catalogs, and describes itself to
the app through a manifest. Whether a module is shown is decided at runtime by the
organization's module switches, so the app only renders the manifests of active modules.

A module package consists of:

- `module.config.ts`: `{ key, routesDirectory }`, read by Vite, Vitest and the manifest.
  It is a separate file because the manifest imports React components and cannot be
  loaded from `vite.config.ts`. The app lists every config in `frontend/app/modules.config.ts`.
- `src/manifest.tsx`: the `ModuleManifest` (type in `frontend/app/src/modules/types.ts`)
  with navigation, route permissions, resources, statistics, map, planning, sensor target
  and settings slots, and the module's i18n namespaces.
- `src/routes/`: the module's route files, mirroring the app's paths, for example
  `src/routes/_protected/trees/...` with `createFileRoute('/_protected/trees')`. The
  TanStack Router plugin picks them up through a `virtualRouteConfig`, so the URLs stay
  what they are. Wrap a module's top-level route in `moduleRoute(key, …)` (from `@/modules/moduleRoute`) so an inactive
  module renders a "module not active" page without sending any module request.

A module may import from the app with `@/…`, but the app must never import a module. The
only places in `frontend/app/src` that may reference a module package are
`modules/registry.ts` (the list of manifests), `lib/i18n/i18next.d.ts` (types of the module
catalogs) and the generated `routeTree.gen.ts`. `frontend/app/src/modules/boundary.test.ts`
enforces this and fails with the offending files; do not add exceptions, hand the data
over through a manifest slot instead. Core text must stay neutral and module-specific
wording belongs in the module's catalogs.

A module package keeps the same devDependencies as the app, including the peer variants
(`typescript` as alias for `@typescript/typescript6`, `@types/node`, `jsdom`,
`@vitest/*`). If they drift, pnpm installs a second copy of `react-i18next` or `vitest`
and translations come back empty in tests. `src/instances.test.ts` in the module checks
that the packages resolve to the same instance. Tailwind only finds a module's classes
through the `@source "../../../modules/*/src/**/*.{ts,tsx}"` line in
`frontend/app/src/css/site.css`, which `src/modules/tailwindSource.test.ts` guards; a
module outside `frontend/modules/` needs its own `@source` line. After adding or moving route files,
regenerate `routeTree.gen.ts` by starting the dev server briefly
(`timeout 30 pnpm exec vite --port 5199` in `frontend/app`) and check the diff.

Handbook chapters that only make sense with a module carry `module: <key>` in their
frontmatter; they are hidden from logged-in users for organizations where the module is
off. Validation of a module's drafts comes from `domain-wasm`, whose exports carry the
module key as prefix (`irrigationValidateTreeDraft`).

## Writing a Plugin

Green Ecolution can be extended by an external plugin: a separate system that writes
data into a Green Ecolution instance (for example importing trees from a municipal
register), optionally contributes its own view embedded inside the application, or
both. A plugin is not part of this repository and does not need to be written in Rust
or TypeScript; it only needs to speak the HTTP contract described below.

A plugin is not self-registering. An administrator of the target Green Ecolution
instance installs it from **Settings → Plugins**, choosing the permissions it receives
and, if it has a view, where that view is hosted. Installing issues a one-time
plaintext API key; that key, not a login, is what authenticates every request the
plugin's own backend makes afterwards.

Two pieces cover the whole surface:

- **The ingest contract** — a plain HTTP API under `/api/v1/plugins/`, authenticated
  with the plugin's API key as a bearer token, for writing data (currently trees) into
  Green Ecolution. No SDK is required to use it, only an HTTP client.
- **The view SDK**, `@green-ecolution/plugin-interface` — if the plugin also
  contributes a view, this package provides the `postMessage` handshake between that
  view (rendered in a sandboxed iframe) and the host application, plus a small React
  helper around it.

Both, including full request and response examples, are documented in
[`frontend/packages/plugin-interface/README.md`](frontend/packages/plugin-interface/README.md).
The full OpenAPI schema for every endpoint, including the plugin ones, is also part of
the live API documentation linked under [Getting Help](#getting-help).

### Testing the plugin integration

The repository ships a demo plugin that exercises both halves of the contract.
`just plugin-demo-up` builds and starts it on <http://localhost:5175>, and
[`frontend/demo-plugin/README.md`](frontend/demo-plugin/README.md) walks through
installing it under **Settings → Plugins**, pasting its one-time API key and importing
demo trees. It runs behind the compose profile `plugins`, so a plain `just infra-up`
neither builds nor starts it.

Proxied plugin views are served under `http://<slug>.plugins.localhost:3000`
(`plugins.proxy.public_url`, overridable via `APP_PLUGINS__PROXY__PUBLIC_URL`).
`backend/config/local.yaml` allows the demo plugin as a proxied target:
`demo-plugin:80` when the backend runs in compose, `localhost:5175` when it
runs via `just run-dev`.

## Getting Help

- Open an issue for bugs or feature requests
- Check existing documentation in `CLAUDE.md`
- Review API docs at [app.green-ecolution.de/api/v1/swagger](https://app.green-ecolution.de/api/v1/swagger/index.html)

## License

By contributing to Green Ecolution, you agree that your contributions will be licensed under the [AGPL-3.0 License](LICENSE).

---

Thank you for contributing to Green Ecolution!
