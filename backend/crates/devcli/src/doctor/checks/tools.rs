use std::{
    ffi::{OsStr, OsString},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use async_trait::async_trait;
use bollard::Docker;

use crate::doctor::{Check, Env, Finding, Severity};

pub fn find_in_path(program: &str, path: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(path)
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

struct Tool {
    program: &'static str,
    install: &'static str,
    needed_for: &'static str,
    /// The minimal profile cannot start without it.
    required: bool,
}

const TOOLS: [Tool; 5] = [
    Tool {
        program: "bacon",
        install: "cargo install bacon",
        needed_for: "the backend unit",
        required: true,
    },
    Tool {
        program: "cargo-sqlx",
        install: "cargo install sqlx-cli --no-default-features --features rustls,postgres",
        needed_for: "new migrations and the sqlx offline cache",
        required: false,
    },
    Tool {
        program: "pnpm",
        install: "corepack enable pnpm",
        needed_for: "the frontend unit",
        required: true,
    },
    Tool {
        program: "wasm-pack",
        install: "cargo install wasm-pack",
        needed_for: "just build-domain-wasm",
        required: false,
    },
    Tool {
        program: "typst",
        install: "cargo install --locked typst-cli",
        needed_for: "the handbook PDF",
        required: false,
    },
];

#[derive(Default)]
pub struct Tools {
    /// `PATH` override for tests; the process's own `PATH` otherwise.
    path: Option<OsString>,
}

#[async_trait]
impl Check for Tools {
    fn id(&self) -> &'static str {
        "tools"
    }

    async fn diagnose(&self, _env: &Env) -> Result<Vec<Finding>, String> {
        let path = self
            .path
            .clone()
            .or_else(|| std::env::var_os("PATH"))
            .unwrap_or_default();
        Ok(TOOLS
            .iter()
            .filter(|tool| find_in_path(tool.program, &path).is_none())
            .map(|tool| {
                let severity = if tool.required {
                    Severity::Error
                } else {
                    Severity::Hint
                };
                Finding::new(
                    self.id(),
                    severity,
                    format!("`{}` is not installed", tool.program),
                )
                .subject(tool.program)
                .detail(format!(
                    "Needed for {}. Install it with `{}`.",
                    tool.needed_for, tool.install
                ))
            })
            .collect())
    }
}

pub struct DockerDaemon;

#[async_trait]
impl Check for DockerDaemon {
    fn id(&self) -> &'static str {
        "docker"
    }

    async fn diagnose(&self, _env: &Env) -> Result<Vec<Finding>, String> {
        let reason = match Docker::connect_with_local_defaults() {
            Err(err) => err.to_string(),
            Ok(docker) => match tokio::time::timeout(Duration::from_secs(3), docker.ping()).await {
                Ok(Ok(_)) => return Ok(Vec::new()),
                Ok(Err(err)) => err.to_string(),
                Err(_) => "no answer within 3 s".to_owned(),
            },
        };
        Ok(vec![
            Finding::new(
                self.id(),
                Severity::Error,
                "the Docker daemon is not reachable",
            )
            .detail(format!(
                "{reason}. Compose units cannot start; local processes still run."
            )),
        ])
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt};

    use super::*;
    use crate::doctor::fake::test_env;

    fn executable(dir: &std::path::Path, name: &str) {
        let path = dir.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn finds_executables_on_the_path_only() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        executable(second.path(), "bacon");
        fs::write(first.path().join("pnpm"), "not executable").unwrap();
        let path = std::env::join_paths([first.path(), second.path()]).unwrap();

        assert_eq!(
            find_in_path("bacon", &path),
            Some(second.path().join("bacon"))
        );
        assert_eq!(find_in_path("pnpm", &path), None);
    }

    #[tokio::test]
    async fn missing_tools_come_with_their_install_command() {
        let bin = tempfile::tempdir().unwrap();
        executable(bin.path(), "bacon");
        executable(bin.path(), "wasm-pack");
        executable(bin.path(), "typst");
        let tools = Tools {
            path: Some(OsString::from(bin.path())),
        };

        let findings = tools.diagnose(&test_env(bin.path())).await.unwrap();

        let ids: Vec<(&str, Severity)> = findings
            .iter()
            .map(|f| (f.id.as_str(), f.severity))
            .collect();
        assert_eq!(
            ids,
            [
                ("tools:cargo-sqlx", Severity::Hint),
                ("tools:pnpm", Severity::Error)
            ]
        );
        assert!(findings[1].detail.contains("corepack enable pnpm"));
        assert!(findings.iter().all(|f| f.fix.is_none()));
    }
}
