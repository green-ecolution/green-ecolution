pub mod files;
pub mod ports;
pub mod tools;

use std::sync::Arc;

use super::Check;

/// Shown with every finding whose fix is `just migrate-up`, for the case CLAUDE.md documents:
/// the migrate binary cannot be rebuilt offline once new `query!` call sites exist.
pub const MANUAL_MIGRATION: &str = "If `just migrate-up` cannot build the migrate binary because \
the sqlx offline cache lacks new queries, migrate by hand from backend/:
  1. cargo sqlx migrate run --source \"$(pwd)/migrations\" --database-url \"$(just --evaluate db_url)\"
  2. DATABASE_URL=\"$(just --evaluate db_url)\" cargo build --bin migrate
  3. refresh .sqlx/: recreate the sqlx_prepare database, run ./target/debug/migrate up against it, \
then cargo sqlx prepare --workspace -- --all-targets";

pub fn all() -> Vec<Arc<dyn Check>> {
    vec![
        Arc::new(files::MigrateBinary),
        Arc::new(files::PackageDist),
        Arc::new(files::DomainWasm),
        Arc::new(files::KeycloakTheme),
        Arc::new(files::ValhallaTiles),
        Arc::new(tools::Tools::default()),
        Arc::new(tools::DockerDaemon),
        Arc::new(ports::Ports),
    ]
}
