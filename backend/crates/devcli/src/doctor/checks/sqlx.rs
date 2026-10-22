use std::{fs, time::Duration};

use async_trait::async_trait;
use tokio::process::Command;

use super::tools::find_in_path;
use crate::{
    doctor::{Check, Cost, Env, Finding, Fix, Severity, Trigger},
    units::process::output_in_own_group,
};

/// Paths `git status --porcelain` reports as changed, deletions included (a refresh may only
/// delete cache files); renames resolve to the new path.
pub fn changed_paths(porcelain: &str) -> Vec<String> {
    porcelain
        .lines()
        .filter_map(|line| {
            let path = line.get(3..)?;
            let path = path.rsplit_once(" -> ").map_or(path, |(_, new)| new);
            Some(path.trim_matches('"').to_owned())
        })
        .collect()
}

const QUERY_MACROS: [&str; 4] = ["query!(", "query_as!(", "query_scalar!(", "query_file!("];

pub fn uses_query_macros(source: &str) -> bool {
    QUERY_MACROS.iter().any(|name| source.contains(name))
}

/// Changed Rust files that use query macros, unless the cache itself changed too.
pub fn stale_cache(changed: &[String], uses_macros: impl Fn(&str) -> bool) -> Vec<String> {
    if changed
        .iter()
        .any(|path| path.starts_with("backend/.sqlx/"))
    {
        return Vec::new();
    }
    changed
        .iter()
        .filter(|path| path.ends_with(".rs") && uses_macros(path))
        .cloned()
        .collect()
}

pub fn redact(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        text.to_owned()
    } else {
        text.replace(secret, "***")
    }
}

pub struct SqlxCache;

#[async_trait]
impl Check for SqlxCache {
    fn id(&self) -> &'static str {
        "sqlx-cache"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let output = Command::new("git")
            .args([
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--",
                "backend",
            ])
            .current_dir(&env.root)
            .output()
            .await
            .map_err(|err| format!("cannot run git: {err}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        let changed = changed_paths(&String::from_utf8_lossy(&output.stdout));
        let stale = stale_cache(&changed, |path| {
            fs::read_to_string(env.root.join(path)).is_ok_and(|source| uses_query_macros(&source))
        });
        if stale.is_empty() {
            return Ok(Vec::new());
        }
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Error,
                "query macros changed but .sqlx/ did not",
            )
            .detail(format!(
                "Builds with SQLX_OFFLINE=true, CI included, fail until the cache is refreshed. \
                 Changed files:\n  {}",
                stale.join("\n  ")
            ))
            .fix(Fix::just("generate-sqlx")),
        ])
    }

    fn reacts_to(&self, trigger: &Trigger<'_>) -> bool {
        matches!(trigger, Trigger::Log(record) if record.message.contains("no cached data for this query"))
    }
}

/// cargo can wait on the build-directory lock (bacon) or a stalled connection indefinitely.
const PREPARE_TIMEOUT: Duration = Duration::from_secs(600);

pub struct SqlxPrepare;

#[async_trait]
impl Check for SqlxPrepare {
    fn id(&self) -> &'static str {
        "sqlx-cache-prepare"
    }

    fn cost(&self) -> Cost {
        Cost::Deep
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let path = std::env::var_os("PATH").unwrap_or_default();
        if find_in_path("cargo-sqlx", &path).is_none() {
            return Err("cargo-sqlx is not installed".to_owned());
        }
        let url = env.vars.get("db_url").map_err(|err| err.to_string())?;
        let password = env
            .vars
            .get("postgres_password")
            .map_err(|err| err.to_string())?;
        let mut command = Command::new("cargo");
        command
            .args([
                "sqlx",
                "prepare",
                "--workspace",
                "--check",
                "--",
                "--all-targets",
            ])
            .current_dir(env.root.join("backend"))
            .env("DATABASE_URL", &url);
        let output = tokio::time::timeout(PREPARE_TIMEOUT, output_in_own_group(command))
            .await
            .map_err(|_| "cargo sqlx prepare --check did not finish within 10 minutes".to_owned())?
            .map_err(|err| format!("cannot run cargo sqlx: {err}"))?;
        if output.status.success() {
            return Ok(Vec::new());
        }
        let stderr = redact(&String::from_utf8_lossy(&output.stderr), &password);
        let lines: Vec<&str> = stderr.lines().collect();
        let tail = lines[lines.len().saturating_sub(5)..].join("\n");
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Error,
                ".sqlx/ does not match the queries",
            )
            .detail(format!(
                "`cargo sqlx prepare --check` against the dev database failed:\n{tail}"
            ))
            .fix(Fix::just("generate-sqlx")),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = " M backend/crates/server/src/infra/pg_tree.rs
?? backend/crates/server/src/infra/pg_new.rs
R  backend/crates/server/src/old.rs -> backend/crates/server/src/renamed.rs
 D backend/crates/server/src/gone.rs
 M backend/crates/server/Cargo.toml
";

    #[test]
    fn reads_changed_paths_from_porcelain_output() {
        assert_eq!(
            changed_paths(STATUS),
            [
                "backend/crates/server/src/infra/pg_tree.rs",
                "backend/crates/server/src/infra/pg_new.rs",
                "backend/crates/server/src/renamed.rs",
                "backend/crates/server/src/gone.rs",
                "backend/crates/server/Cargo.toml",
            ]
        );
    }

    #[test]
    fn recognises_the_checked_query_macros() {
        assert!(uses_query_macros("sqlx::query!(\"SELECT 1\")"));
        assert!(uses_query_macros("query_as!(TreeRow, \"…\")"));
        assert!(uses_query_macros("sqlx::query_scalar!(\"…\")"));
        assert!(!uses_query_macros("sqlx::query(\"SELECT 1\")"));
    }

    #[test]
    fn a_touched_cache_clears_the_finding() {
        let changed = changed_paths(STATUS);
        let with_macros = |path: &str| path.ends_with("pg_tree.rs") || path.ends_with("Cargo.toml");

        assert_eq!(
            stale_cache(&changed, with_macros),
            ["backend/crates/server/src/infra/pg_tree.rs"]
        );

        let mut touched = changed.clone();
        touched.push("backend/.sqlx/query-abc.json".into());
        assert!(stale_cache(&touched, with_macros).is_empty());
    }

    #[test]
    fn a_deleted_cache_file_counts_as_a_refresh() {
        let changed = changed_paths(
            " M backend/crates/server/src/infra/pg_tree.rs\n D backend/.sqlx/query-abc.json\n",
        );
        assert!(stale_cache(&changed, |_| true).is_empty());
    }

    #[test]
    fn the_password_never_leaves_the_check() {
        assert_eq!(
            redact("error: postgres://app:s3cret@localhost/x refused", "s3cret"),
            "error: postgres://app:***@localhost/x refused"
        );
        assert_eq!(redact("nothing", ""), "nothing");
    }

    #[test]
    fn the_offline_cache_error_triggers_the_check() {
        let record = crate::logs::parse_line(
            "backend",
            crate::config::LogFormat::Plain,
            "error: `SQLX_OFFLINE=true` but there is no cached data for this query, run `cargo sqlx prepare`",
            chrono::Utc::now(),
        );
        assert!(SqlxCache.reacts_to(&Trigger::Log(&record)));
        assert_eq!(SqlxPrepare.cost(), Cost::Deep);
    }
}
