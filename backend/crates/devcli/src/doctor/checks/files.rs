use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use async_trait::async_trait;

use super::MANUAL_MIGRATION;
use crate::doctor::{Check, Env, Finding, Fix, Severity};

pub(super) const THEME_DIR: &str = "frontend/keycloak-theme/dist_keycloak/theme/green-ecolution";
pub(super) const TILES: &str = ".docker/infra/valhalla/custom_files/valhalla_tiles.tar";

/// Newest modification time below `path`, ignoring `node_modules`; `None` when nothing is there.
pub fn newest_mtime(path: &Path) -> io::Result<Option<SystemTime>> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    if !metadata.is_dir() {
        return Ok(Some(metadata.modified()?));
    }
    let mut newest = None;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_name() == "node_modules" {
            continue;
        }
        newest = newest.max(newest_mtime(&entry.path())?);
    }
    Ok(newest)
}

fn newest_of(paths: &[PathBuf]) -> Result<Option<SystemTime>, String> {
    paths.iter().try_fold(None, |newest, path| {
        Ok(newest.max(newest_mtime(path).map_err(|err| err.to_string())?))
    })
}

const PACKAGES: [(&str, &str); 3] = [
    ("ui", "src"),
    ("backend-client", "src"),
    ("plugin-interface", "lib"),
];

pub struct PackageDist;

#[async_trait]
impl Check for PackageDist {
    fn id(&self) -> &'static str {
        "package-dist"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let mut findings = Vec::new();
        for (name, sources) in PACKAGES {
            let dir = env.root.join("frontend/packages").join(name);
            let package = format!("@green-ecolution/{name}");
            let source = newest_of(&[dir.join(sources)])?;
            let built = newest_of(&[dir.join("dist")])?;
            let finding = match (source, built) {
                (_, None) => {
                    Finding::new(self.id(), Severity::Error, format!("{package} has no dist"))
                }
                (Some(source), Some(built)) if source > built => Finding::new(
                    self.id(),
                    Severity::Hint,
                    format!("{package} dist is older than its sources"),
                ),
                _ => continue,
            };
            findings.push(
                finding
                    .subject(name)
                    .detail(
                        "Workspace packages are consumed from dist, so the app, its tests and \
                         its build still see the old code.",
                    )
                    .fix(Fix {
                        command: format!("pnpm --filter {package} run build"),
                        cwd: "frontend".to_owned(),
                        destructive: false,
                    }),
            );
        }
        Ok(findings)
    }
}

pub struct DomainWasm;

#[async_trait]
impl Check for DomainWasm {
    fn id(&self) -> &'static str {
        "domain-wasm"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let sources = newest_of(&[
            env.root.join("backend/crates/domain/src"),
            env.root.join("backend/crates/domain-wasm/src"),
        ])?;
        let built = newest_of(&[env.root.join("frontend/packages/domain-wasm/pkg")])?;
        let finding = match (sources, built) {
            (_, None) => Finding::new(self.id(), Severity::Error, "domain-wasm/pkg is missing"),
            (Some(sources), Some(built)) if sources > built => Finding::new(
                self.id(),
                Severity::Hint,
                "domain-wasm/pkg is older than the domain sources",
            ),
            _ => return Ok(Vec::new()),
        };
        Ok(vec![
            finding
                .detail(
                    "The frontend validates forms with the domain compiled to WebAssembly; \
                     the dev server and the frontend build need an up-to-date pkg/.",
                )
                .fix(Fix::just("build-domain-wasm")),
        ])
    }
}

pub struct KeycloakTheme;

#[async_trait]
impl Check for KeycloakTheme {
    fn id(&self) -> &'static str {
        "keycloak-theme"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        if env.root.join(THEME_DIR).is_dir() {
            return Ok(Vec::new());
        }
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Error,
                "the Keycloak login theme is not built",
            )
            .detail(format!("compose mounts {THEME_DIR} into Keycloak."))
            .fix(Fix::just("build-keycloak-theme")),
        ])
    }
}

pub struct ValhallaTiles;

#[async_trait]
impl Check for ValhallaTiles {
    fn id(&self) -> &'static str {
        "valhalla-tiles"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        if env.root.join(TILES).is_file() {
            return Ok(Vec::new());
        }
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Hint,
                "the Valhalla routing tiles are missing",
            )
            .detail(
                "Only the full profile runs Valhalla; building the tiles takes a while and \
                     replaces the tiles directory.",
            )
            .fix(Fix::just("valhalla-tiles").destructive()),
        ])
    }
}

pub struct MigrateBinary;

#[async_trait]
impl Check for MigrateBinary {
    fn id(&self) -> &'static str {
        "migrate-binary"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let Ok(built) = fs::metadata(env.root.join("backend/target/debug/migrate"))
            .and_then(|metadata| metadata.modified())
        else {
            return Ok(Vec::new());
        };
        let newest = newest_of(&[env.root.join("backend/migrations")])?;
        if !newest.is_some_and(|newest| newest > built) {
            return Ok(Vec::new());
        }
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Hint,
                "target/debug/migrate is older than the newest migration",
            )
            .detail(format!(
                "A stale migrate binary reports \"Migrations applied\" without applying the new \
                 migration.\n{MANUAL_MIGRATION}"
            ))
            .fix(Fix::just("migrate-up")),
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        time::{Duration, SystemTime},
    };

    use super::*;
    use crate::doctor::{Check, Severity, fake::test_env};

    fn at(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + secs)
    }

    fn touch(root: &Path, path: &str, secs: u64) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(at(secs))
            .unwrap();
    }

    async fn ids(check: &dyn Check, root: &Path) -> Vec<(String, Severity)> {
        check
            .diagnose(&test_env(root))
            .await
            .unwrap()
            .into_iter()
            .map(|f| (f.id, f.severity))
            .collect()
    }

    #[test]
    fn newest_mtime_walks_directories_but_skips_node_modules() {
        let dir = tempfile::tempdir().unwrap();
        touch(dir.path(), "a/one.ts", 10);
        touch(dir.path(), "a/deep/two.ts", 20);
        touch(dir.path(), "a/node_modules/x/three.js", 99);

        assert_eq!(newest_mtime(&dir.path().join("a")).unwrap(), Some(at(20)));
        assert_eq!(newest_mtime(&dir.path().join("missing")).unwrap(), None);
    }

    #[tokio::test]
    async fn packages_with_a_missing_or_older_dist_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(root, "frontend/packages/ui/src/index.ts", 20);
        touch(root, "frontend/packages/ui/dist/index.js", 10);
        touch(root, "frontend/packages/backend-client/src/index.ts", 10);
        touch(root, "frontend/packages/backend-client/dist/index.js", 20);
        touch(root, "frontend/packages/plugin-interface/lib/index.ts", 10);

        assert_eq!(
            ids(&PackageDist, root).await,
            [
                ("package-dist:ui".to_owned(), Severity::Hint),
                ("package-dist:plugin-interface".to_owned(), Severity::Error),
            ]
        );
        let findings = PackageDist.diagnose(&test_env(root)).await.unwrap();
        let fix = findings[0].fix.as_ref().unwrap();
        assert_eq!(fix.command, "pnpm --filter @green-ecolution/ui run build");
        assert_eq!(fix.cwd, "frontend");
    }

    #[tokio::test]
    async fn domain_wasm_must_exist_and_be_newer_than_the_domain() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(root, "backend/crates/domain/src/lib.rs", 20);
        assert_eq!(
            ids(&DomainWasm, root).await,
            [("domain-wasm".to_owned(), Severity::Error)]
        );

        touch(root, "frontend/packages/domain-wasm/pkg/domain_wasm.js", 10);
        assert_eq!(
            ids(&DomainWasm, root).await,
            [("domain-wasm".to_owned(), Severity::Hint)]
        );

        touch(root, "frontend/packages/domain-wasm/pkg/domain_wasm.js", 30);
        assert!(ids(&DomainWasm, root).await.is_empty());
    }

    #[tokio::test]
    async fn the_keycloak_theme_and_valhalla_tiles_must_exist() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert_eq!(
            ids(&KeycloakTheme, root).await,
            [("keycloak-theme".to_owned(), Severity::Error)]
        );
        let tiles = ValhallaTiles.diagnose(&test_env(root)).await.unwrap();
        assert_eq!(tiles[0].severity, Severity::Hint);
        assert!(tiles[0].fix.as_ref().unwrap().destructive);

        fs::create_dir_all(root.join(THEME_DIR)).unwrap();
        touch(root, TILES, 1);
        assert!(ids(&KeycloakTheme, root).await.is_empty());
        assert!(ids(&ValhallaTiles, root).await.is_empty());
    }

    #[tokio::test]
    async fn a_migrate_binary_older_than_the_newest_migration_is_stale() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(root, "backend/migrations/20260101000000_init.sql", 20);
        assert!(
            ids(&MigrateBinary, root).await.is_empty(),
            "no binary, nothing to say"
        );

        touch(root, "backend/target/debug/migrate", 10);
        let findings = MigrateBinary.diagnose(&test_env(root)).await.unwrap();
        assert_eq!(findings[0].id, "migrate-binary");
        assert_eq!(findings[0].fix.as_ref().unwrap().command, "just migrate-up");
        assert!(findings[0].detail.contains("cargo sqlx migrate run"));

        touch(root, "backend/target/debug/migrate", 30);
        assert!(ids(&MigrateBinary, root).await.is_empty());
    }
}
