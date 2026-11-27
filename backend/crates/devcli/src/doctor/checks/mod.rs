pub mod db;
pub mod files;
pub mod ports;
pub mod sqlx;
pub mod tools;

use std::{path::Path, sync::Arc};

use super::{Check, Fix};
use crate::units::migrations::migration_dirs;

/// Shown with every finding whose fix is [`migrate_fix`]: once the working tree has new
/// `query!` call sites, the migrate binary cannot be rebuilt with the offline cache.
pub const MANUAL_MIGRATION: &str = "If `just migrate-up` cannot build the migrate binary because \
the sqlx offline cache lacks new queries, migrate by hand from backend/:
  1. for dir in \"$(pwd)/migrations\" \"$(pwd)\"/crates/modules/*/migrations; do [ -d \"$dir\" ] && \
cargo sqlx migrate run --ignore-missing --source \"$dir\" --database-url \"$(just --evaluate db_url)\"; done
  2. DATABASE_URL=\"$(just --evaluate db_url)\" cargo build --bin migrate
  3. refresh .sqlx/: recreate the sqlx_prepare database, run ./target/debug/migrate up against it, \
then cargo sqlx prepare --workspace -- --all-targets";

/// `migrate!` embeds the migrations at compile time and no crate has a `rerun-if-changed` for
/// them, so only touching the file that embeds them makes cargo pick up a new migration. The
/// core embeds its own in `app`, a module embeds its directory from its server crate root.
pub fn migrate_fix(root: &Path) -> Fix {
    let backend = root.join("backend");
    let mut touched = vec!["backend/crates/app/src/migrations.rs".to_owned()];
    for dir in migration_dirs(&backend).unwrap_or_default().iter().skip(1) {
        let embedding = dir.with_file_name("server").join("src/lib.rs");
        if let Ok(relative) = embedding.strip_prefix(root)
            && embedding.is_file()
        {
            touched.push(relative.to_string_lossy().into_owned());
        }
    }
    Fix {
        command: format!("touch {} && just migrate-up", touched.join(" ")),
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn the_migrate_fix_touches_only_the_core_file_without_module_migrations() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("backend/crates/modules/b/server/src")).unwrap();
        fs::write(
            root.path()
                .join("backend/crates/modules/b/server/src/lib.rs"),
            "",
        )
        .unwrap();

        assert_eq!(
            migrate_fix(root.path()).command,
            "touch backend/crates/app/src/migrations.rs && just migrate-up"
        );
    }

    #[test]
    fn the_migrate_fix_touches_the_crate_root_of_every_module_that_embeds_migrations() {
        let root = tempfile::tempdir().unwrap();
        for dir in [
            "backend/crates/modules/a/migrations",
            "backend/crates/modules/a/server/src",
            "backend/crates/modules/c/migrations",
            "backend/crates/modules/c/server/src",
        ] {
            fs::create_dir_all(root.path().join(dir)).unwrap();
        }
        for module in ["a", "c"] {
            fs::write(
                root.path()
                    .join(format!("backend/crates/modules/{module}/server/src/lib.rs")),
                "",
            )
            .unwrap();
        }

        assert_eq!(
            migrate_fix(root.path()).command,
            "touch backend/crates/app/src/migrations.rs \
             backend/crates/modules/a/server/src/lib.rs \
             backend/crates/modules/c/server/src/lib.rs && just migrate-up"
        );
    }
}
