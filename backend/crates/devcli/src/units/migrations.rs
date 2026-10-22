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

/// Fails on the first line that is not a version, so a broken query never reads as "nothing
/// applied".
pub fn parse_applied(output: &str) -> Result<BTreeSet<i64>, String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.parse().map_err(|_| line.to_owned()))
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
        let applied = parse_applied(&output).map_err(|line| {
            let line: String = line.chars().take(80).collect();
            UnitError::Verify(format!(
                "unexpected output from the migration check: `{line}`"
            ))
        })?;
        let pending = missing(&expected, &applied);
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
        let applied = parse_applied("20260101000000\n\n20260102000000\n").unwrap();
        let expected = BTreeSet::from([20260101000000, 20260102000000, 20260103000000]);

        assert_eq!(missing(&expected, &applied), [20260103000000]);
        assert!(missing(&applied, &applied).is_empty());
    }

    #[test]
    fn rejects_output_that_is_not_a_version_list() {
        assert_eq!(
            parse_applied("20260101000000\n?column?\n"),
            Err("?column?".to_owned())
        );
    }

    #[test]
    fn an_empty_result_means_nothing_applied() {
        assert!(parse_applied("\n").unwrap().is_empty());
    }
}
