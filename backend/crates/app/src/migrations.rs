use std::collections::BTreeMap;

use futures::future::BoxFuture;
use server::module::Module;
use sqlx::{
    error::BoxDynError,
    migrate::{Migration, MigrationSource, Migrator},
};

static CORE: Migrator = sqlx::migrate!("../../migrations");

#[derive(Debug, thiserror::Error)]
pub enum MigrationSetupError {
    #[error("migration version {version} exists in both {first} and {second}")]
    DuplicateVersion {
        version: i64,
        first: String,
        second: String,
    },
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

#[derive(Debug)]
struct Merged(Vec<Migration>);

impl MigrationSource<'static> for Merged {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<Migration>, BoxDynError>> {
        Box::pin(async move { Ok(self.0) })
    }
}

/// `-r` migrations ship an up and a down file under one version, so the
/// version alone is not unique. Sorting up before down is what sqlx expects.
fn direction(migration: &Migration) -> u8 {
    u8::from(migration.migration_type.is_down_migration())
}

/// The core keeps its history in `backend/migrations`, every module adds its
/// own directory. Versions are timestamps, so one sorted list and one
/// `_sqlx_migrations` table hold both; separate migrators would report each
/// other's rows as missing.
pub async fn migrator(modules: &[Box<dyn Module>]) -> Result<Migrator, MigrationSetupError> {
    let mut by_version: BTreeMap<(i64, u8), (String, Migration)> = BTreeMap::new();
    let sources = std::iter::once(("core".to_string(), CORE.iter().cloned().collect::<Vec<_>>()))
        .chain(
            modules
                .iter()
                .map(|m| (m.key().to_string(), m.migrations())),
        );
    for (owner, migrations) in sources {
        for migration in migrations {
            let key = (migration.version, direction(&migration));
            if let Some((first, _)) = by_version.get(&key) {
                return Err(MigrationSetupError::DuplicateVersion {
                    version: migration.version,
                    first: first.clone(),
                    second: owner,
                });
            }
            by_version.insert(key, (owner.clone(), migration));
        }
    }
    let merged = by_version.into_values().map(|(_, m)| m).collect();
    Ok(Migrator::new(Merged(merged)).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::module::ModuleKey;
    use server::module::{ModuleBuildError, ModuleContext, ModuleRuntime};
    use std::borrow::Cow;

    struct WithMigrations(Vec<Migration>);

    impl Module for WithMigrations {
        fn key(&self) -> ModuleKey {
            ModuleKey::new("fake").unwrap()
        }
        fn default_enabled(&self) -> bool {
            false
        }
        fn openapi(&self) -> utoipa::openapi::OpenApi {
            utoipa::openapi::OpenApiBuilder::new().build()
        }
        fn migrations(&self) -> Vec<Migration> {
            self.0.clone()
        }
        fn build(&self, _ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
            unreachable!("migration tests never build a module")
        }
    }

    fn migration(version: i64) -> Migration {
        Migration::new(
            version,
            Cow::Borrowed("fake"),
            sqlx::migrate::MigrationType::Simple,
            Cow::Borrowed("SELECT 1"),
            false,
        )
    }

    #[tokio::test]
    async fn without_modules_the_core_history_is_unchanged() {
        let merged = migrator(&[]).await.unwrap();
        let versions: Vec<i64> = merged.iter().map(|m| m.version).collect();
        let core: Vec<i64> = CORE.iter().map(|m| m.version).collect();
        assert_eq!(versions, core);
    }

    #[tokio::test]
    async fn module_migrations_are_sorted_into_the_history() {
        let latest = CORE.iter().map(|m| m.version).max().unwrap();
        let module: Box<dyn Module> = Box::new(WithMigrations(vec![migration(latest + 1)]));
        let merged = migrator(&[module]).await.unwrap();
        assert_eq!(merged.iter().last().unwrap().version, latest + 1);
        assert_eq!(merged.iter().count(), CORE.iter().count() + 1);
    }

    #[tokio::test]
    async fn a_reversible_pair_shares_one_version() {
        use sqlx::migrate::MigrationType::{ReversibleDown, ReversibleUp};
        let latest = CORE.iter().map(|m| m.version).max().unwrap() + 1;
        let pair = |kind, sql| {
            Migration::new(
                latest,
                Cow::Borrowed("fake"),
                kind,
                Cow::Borrowed(sql),
                false,
            )
        };
        let module: Box<dyn Module> = Box::new(WithMigrations(vec![
            pair(ReversibleDown, "SELECT 2"),
            pair(ReversibleUp, "SELECT 1"),
        ]));
        let merged = migrator(&[module]).await.unwrap();
        let tail: Vec<_> = merged.iter().rev().take(2).collect();
        assert!(tail[0].migration_type.is_down_migration());
        assert!(tail[1].migration_type.is_up_migration());
        assert_eq!(merged.iter().count(), CORE.iter().count() + 2);
    }

    #[tokio::test]
    async fn a_version_taken_by_the_core_is_rejected() {
        let taken = CORE.iter().next().unwrap().version;
        let module: Box<dyn Module> = Box::new(WithMigrations(vec![migration(taken)]));
        assert!(matches!(
            migrator(&[module]).await,
            Err(MigrationSetupError::DuplicateVersion { first, second, .. })
                if first == "core" && second == "fake"
        ));
    }
}
