use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;

use super::{UnitError, compose::ComposeCli, hooked::Verify};

pub fn migration_versions(dir: &Path) -> io::Result<BTreeSet<i64>> {
    let mut versions = BTreeSet::new();
    for entry in fs::read_dir(dir)? {
        let name = entry?.file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".sql") {
            continue;
        }
        if let Some(version) = name
            .split('_')
            .next()
            .and_then(|prefix| prefix.parse().ok())
        {
            versions.insert(version);
        }
    }
    Ok(versions)
}

pub fn parse_applied(output: &str) -> BTreeSet<i64> {
    output
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

pub fn missing(expected: &BTreeSet<i64>, applied: &BTreeSet<i64>) -> Vec<i64> {
    expected.difference(applied).copied().collect()
}

/// Catches the stale `migrate` binary that prints "Migrations applied" without applying the
/// newest migration.
pub struct MigrationsApplied {
    dir: PathBuf,
    cli: ComposeCli,
    user: String,
    database: String,
}

impl MigrationsApplied {
    pub fn new(dir: PathBuf, cli: ComposeCli, user: String, database: String) -> Self {
        Self {
            dir,
            cli,
            user,
            database,
        }
    }
}

#[async_trait]
impl Verify for MigrationsApplied {
    async fn verify(&self) -> Result<(), UnitError> {
        let expected = migration_versions(&self.dir)?;
        let output = self
            .cli
            .run(&[
                "exec",
                "-T",
                "db",
                "psql",
                "-U",
                self.user.as_str(),
                "-d",
                self.database.as_str(),
                "-tAc",
                "SELECT version FROM _sqlx_migrations WHERE success",
            ])
            .await?;
        let pending = missing(&expected, &parse_applied(&output));
        match pending.last() {
            None => Ok(()),
            Some(newest) => Err(UnitError::Verify(format!(
                "{} migration(s) not applied, newest {newest}. A stale target/debug/migrate \
                 binary reports success without applying them; rebuild it against the live \
                 database (DATABASE_URL=... cargo build --bin migrate) and rerun `just migrate-up`",
                pending.len()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_versions_from_migration_file_names() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "20260101000000_init.sql",
            "20260102000000_add_trees.sql",
            "README.md",
        ] {
            fs::write(dir.path().join(name), "").unwrap();
        }

        let versions = migration_versions(dir.path()).unwrap();

        assert_eq!(
            versions.into_iter().collect::<Vec<_>>(),
            [20260101000000, 20260102000000]
        );
    }

    #[test]
    fn parses_psql_output_and_finds_gaps() {
        let applied = parse_applied("20260101000000\n\n20260102000000\nnot a number\n");
        let expected = BTreeSet::from([20260101000000, 20260102000000, 20260103000000]);

        assert_eq!(missing(&expected, &applied), [20260103000000]);
        assert!(missing(&applied, &applied).is_empty());
    }
}
