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

/// The core's directory first, then the directory of every module that ships migrations.
pub fn migration_dirs(backend: &Path) -> io::Result<Vec<PathBuf>> {
    let mut dirs = vec![backend.join("migrations")];
    let modules = backend.join("crates/modules");
    if !modules.is_dir() {
        return Ok(dirs);
    }
    let mut of_modules = Vec::new();
    for entry in fs::read_dir(modules)? {
        let dir = entry?.path().join("migrations");
        if dir.is_dir() {
            of_modules.push(dir);
        }
    }
    of_modules.sort();
    dirs.extend(of_modules);
    Ok(dirs)
}

pub fn expected_versions(dirs: &[PathBuf]) -> io::Result<BTreeSet<i64>> {
    let mut versions = BTreeSet::new();
    for dir in dirs {
        versions.extend(migration_versions(dir)?);
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

pub async fn applied(
    cli: &ComposeCli,
    user: &str,
    database: &str,
) -> Result<BTreeSet<i64>, UnitError> {
    let output = cli
        .run(&[
            "exec",
            "-T",
            "db",
            "psql",
            "-U",
            user,
            "-d",
            database,
            "-tAc",
            "SELECT version FROM _sqlx_migrations WHERE success",
        ])
        .await?;
    parse_applied(&output).map_err(|line| {
        let line: String = line.chars().take(80).collect();
        UnitError::Verify(format!(
            "unexpected output from the migration check: `{line}`"
        ))
    })
}

/// Catches the stale `migrate` binary that prints "Migrations applied" without applying the
/// newest migration.
pub struct MigrationsApplied {
    backend: PathBuf,
    cli: ComposeCli,
    user: String,
    database: String,
}

impl MigrationsApplied {
    pub fn new(backend: PathBuf, cli: ComposeCli, user: String, database: String) -> Self {
        Self {
            backend,
            cli,
            user,
            database,
        }
    }
}

#[async_trait]
impl Verify for MigrationsApplied {
    async fn verify(&self) -> Result<(), UnitError> {
        let expected = expected_versions(&migration_dirs(&self.backend)?)?;
        let applied = applied(&self.cli, &self.user, &self.database).await?;
        let pending = missing(&expected, &applied);
        match pending.last() {
            None => Ok(()),
            Some(newest) => Err(UnitError::Verify(format!(
                "{} migration(s) not applied, newest {newest}. A stale target/debug/migrate \
                 binary reports success without applying them; `geco doctor --fix` (or `f` in \
                 the TUI) rebuilds it and migrates again",
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

    fn relative(dirs: &[PathBuf], backend: &Path) -> Vec<String> {
        dirs.iter()
            .map(|dir| {
                dir.strip_prefix(backend)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }

    #[test]
    fn collects_the_core_directory_and_every_module_directory_that_has_migrations() {
        let backend = tempfile::tempdir().unwrap();
        for dir in [
            "migrations",
            "crates/modules/b/migrations",
            "crates/modules/a/migrations",
            "crates/modules/c/server",
        ] {
            fs::create_dir_all(backend.path().join(dir)).unwrap();
        }

        let dirs = migration_dirs(backend.path()).unwrap();

        assert_eq!(
            relative(&dirs, backend.path()),
            [
                "migrations",
                "crates/modules/a/migrations",
                "crates/modules/b/migrations"
            ]
        );
    }

    #[test]
    fn a_backend_without_modules_yields_only_the_core_directory() {
        let backend = tempfile::tempdir().unwrap();
        fs::create_dir_all(backend.path().join("migrations")).unwrap();

        let dirs = migration_dirs(backend.path()).unwrap();

        assert_eq!(relative(&dirs, backend.path()), ["migrations"]);
    }

    #[test]
    fn expected_versions_are_the_union_over_all_directories() {
        let backend = tempfile::tempdir().unwrap();
        let core = backend.path().join("migrations");
        let module = backend.path().join("crates/modules/a/migrations");
        fs::create_dir_all(&core).unwrap();
        fs::create_dir_all(&module).unwrap();
        fs::write(core.join("20260101000000_a.sql"), "").unwrap();
        fs::write(module.join("20260102000000_b.sql"), "").unwrap();

        let versions = expected_versions(&[core, module]).unwrap();

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
