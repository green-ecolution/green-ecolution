pub mod db;
pub mod files;
pub mod ports;
pub mod sqlx;
pub mod tools;

use std::sync::Arc;

use super::{Check, Fix};

/// Shown with every finding whose fix is [`migrate_fix`]: once the working tree has new
/// `query!` call sites, the migrate binary cannot be rebuilt with the offline cache.
pub const MANUAL_MIGRATION: &str = "If `just migrate-up` cannot build the migrate binary because \
the sqlx offline cache lacks new queries, migrate by hand from backend/:
  1. cargo sqlx migrate run --source \"$(pwd)/migrations\" --database-url \"$(just --evaluate db_url)\"
  2. DATABASE_URL=\"$(just --evaluate db_url)\" cargo build --bin migrate
  3. refresh .sqlx/: recreate the sqlx_prepare database, run ./target/debug/migrate up against it, \
then cargo sqlx prepare --workspace -- --all-targets";

/// `migrate!` embeds the migrations at compile time and the server has no `rerun-if-changed`
/// for them, so only touching the file that embeds them makes cargo pick up a new migration.
pub fn migrate_fix() -> Fix {
    Fix {
        command: "touch backend/crates/app/src/migrations.rs && just migrate-up".to_owned(),
        cwd: String::new(),
        destructive: false,
    }
}

pub fn all() -> Vec<Arc<dyn Check>> {
    vec![
        Arc::new(db::DevDbMigrations),
        Arc::new(files::MigrateBinary),
        Arc::new(sqlx::SqlxCache),
        Arc::new(sqlx::SqlxPrepare),
        Arc::new(files::PackageDist),
        Arc::new(files::DomainWasm),
        Arc::new(db::Testcontainers),
        Arc::new(files::KeycloakTheme),
        Arc::new(files::ValhallaTiles),
        Arc::new(tools::Tools::default()),
        Arc::new(tools::DockerDaemon),
        Arc::new(ports::Ports),
    ]
}
