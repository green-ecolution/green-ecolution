mod vars;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[cfg(test)]
pub use vars::StaticVars;
pub use vars::{JustVars, VarError, VarSource, render};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogFormat {
    #[default]
    Plain,
    TracingJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadyProbe {
    Http(String),
    Tcp(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalKind {
    Process,
    Oneshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerifyKind {
    MigrationsApplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinHook {
    TraefikDevServices,
}

/// A local unit when `kind` is set; otherwise additions to the compose service of the same name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitConfig {
    pub kind: Option<LocalKind>,
    pub run: Option<String>,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub after: Vec<String>,
    pub ready: Option<ReadyProbe>,
    pub verify: Option<VerifyKind>,
    #[serde(default)]
    pub log_format: LogFormat,
    #[serde(default)]
    pub pre_start: Vec<String>,
    #[serde(default)]
    pub builtin_hooks: Vec<BuiltinHook>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileConfig {
    pub units: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    #[serde(default = "default_capacity")]
    pub capacity: usize,
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            capacity: default_capacity(),
        }
    }
}

fn default_capacity() -> usize {
    50_000
}

fn default_profile() -> String {
    "minimal".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GecoConfig {
    #[serde(default = "default_profile")]
    pub default_profile: String,
    #[serde(default)]
    pub logs: LogsConfig,
    #[serde(default)]
    pub unit: BTreeMap<String, UnitConfig>,
    #[serde(default)]
    pub profile: BTreeMap<String, ProfileConfig>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("{}: unit `{unit}`: {reason}", path.display())]
    Invalid {
        path: PathBuf,
        unit: String,
        reason: &'static str,
    },
    #[error("unknown profile `{0}`")]
    UnknownProfile(String),
}

impl GecoConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text, path)
    }

    pub fn parse(text: &str, path: &Path) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        config.validate(path)?;
        Ok(config)
    }

    pub fn profile(&self, name: &str) -> Result<&ProfileConfig, ConfigError> {
        self.profile
            .get(name)
            .ok_or_else(|| ConfigError::UnknownProfile(name.to_owned()))
    }

    fn validate(&self, path: &Path) -> Result<(), ConfigError> {
        for (name, unit) in &self.unit {
            let invalid = |reason| ConfigError::Invalid {
                path: path.to_path_buf(),
                unit: name.clone(),
                reason,
            };
            let compose_only = unit.run.is_none()
                && unit.cwd.is_none()
                && unit.ready.is_none()
                && unit.verify.is_none()
                && unit.env.is_empty()
                && unit.log_format == LogFormat::Plain;
            match unit.kind {
                Some(_) if unit.run.as_deref().is_none_or(str::is_empty) => {
                    return Err(invalid("`run` is required for local units"));
                }
                Some(LocalKind::Process) if unit.verify.is_some() => {
                    return Err(invalid("`verify` is only allowed on oneshot units"));
                }
                None if !compose_only => {
                    return Err(invalid(
                        "compose services only accept `after`, `pre_start` and `builtin_hooks`",
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
default_profile = "minimal"

[unit.migrations]
kind = "oneshot"
run = "just migrate-up"
after = ["db"]
verify = "migrations-applied"

[unit.backend]
kind = "process"
cwd = "backend"
run = "bacon --headless --job run"
after = ["migrations"]
ready = { http = "http://localhost:3030/api/ready" }
log_format = "tracing-json"
env = { APP_LOG__FORMAT = "json" }

[unit.reverse-proxy]
pre_start = ["just _acme-init"]
builtin_hooks = ["traefik-dev-services"]

[profile.minimal]
units = ["backend"]
"#;

    fn parse(text: &str) -> Result<GecoConfig, ConfigError> {
        GecoConfig::parse(text, Path::new("geco.toml"))
    }

    #[test]
    fn parses_local_units_and_compose_additions() {
        let config = parse(SAMPLE).unwrap();

        let backend = &config.unit["backend"];
        assert_eq!(backend.kind, Some(LocalKind::Process));
        assert_eq!(
            backend.ready,
            Some(ReadyProbe::Http("http://localhost:3030/api/ready".into()))
        );
        assert_eq!(backend.log_format, LogFormat::TracingJson);
        assert_eq!(backend.env["APP_LOG__FORMAT"], "json");
        assert_eq!(
            config.unit["migrations"].verify,
            Some(VerifyKind::MigrationsApplied)
        );

        let proxy = &config.unit["reverse-proxy"];
        assert_eq!(proxy.kind, None);
        assert_eq!(proxy.builtin_hooks, vec![BuiltinHook::TraefikDevServices]);

        assert_eq!(config.profile("minimal").unwrap().units, vec!["backend"]);
        assert_eq!(config.logs.capacity, 50_000);
    }

    #[test]
    fn defaults_the_profile_to_minimal() {
        assert_eq!(parse("").unwrap().default_profile, "minimal");
    }

    #[test]
    fn local_units_require_run() {
        let err = parse("[unit.x]\nkind = \"process\"\n").unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { ref unit, .. } if unit == "x"));
        let text = err.to_string();
        assert!(text.contains("geco.toml"), "{text}");
        assert!(text.contains("unit `x`"), "{text}");
    }

    #[test]
    fn compose_additions_reject_process_fields() {
        let err = parse("[unit.db]\nrun = \"x\"\n").unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { .. }));
    }

    #[test]
    fn verify_is_only_allowed_on_oneshots() {
        let err =
            parse("[unit.x]\nkind = \"process\"\nrun = \"x\"\nverify = \"migrations-applied\"\n")
                .unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { .. }));
    }

    #[test]
    fn unknown_fields_report_file_and_field() {
        let err = parse("[unit.x]\nkind = \"process\"\nrun = \"x\"\nrestart = true\n").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("geco.toml"), "{text}");
        assert!(text.contains("restart"), "{text}");
    }

    #[test]
    fn unknown_profiles_are_errors() {
        assert!(matches!(
            parse("").unwrap().profile("full"),
            Err(ConfigError::UnknownProfile(_))
        ));
    }
}
