use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;

use super::{ConfigError, GecoConfig, LogFormat, ReadyProbe, UnitConfig};

/// A profile's changes to a local unit; unset fields keep the unit's own value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitOverride {
    pub run: Option<String>,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub after: Option<Vec<String>>,
    pub ready: Option<ReadyProbe>,
    pub log_format: Option<LogFormat>,
}

impl UnitOverride {
    fn apply(&self, unit: &mut UnitConfig) {
        if let Some(run) = &self.run {
            unit.run = Some(run.clone());
        }
        if let Some(cwd) = &self.cwd {
            unit.cwd = Some(cwd.clone());
        }
        if let Some(after) = &self.after {
            unit.after = after.clone();
        }
        if let Some(ready) = &self.ready {
            unit.ready = Some(ready.clone());
        }
        if let Some(log_format) = self.log_format {
            unit.log_format = log_format;
        }
        unit.env.extend(
            self.env
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }
}

/// A profile with its `extends` chain and overrides folded in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProfile {
    pub name: String,
    pub units: Vec<String>,
    pub unit: BTreeMap<String, UnitConfig>,
}

impl GecoConfig {
    pub fn resolve(&self, name: &str) -> Result<ResolvedProfile, ConfigError> {
        let mut units: Vec<String> = Vec::new();
        let mut unit = self.unit.clone();
        for profile_name in self.chain(name)? {
            let profile = self.profile(&profile_name)?;
            for root in &profile.units {
                if !units.contains(root) {
                    units.push(root.clone());
                }
            }
            for (target, change) in &profile.overrides {
                if let Some(config) = unit.get_mut(target) {
                    change.apply(config);
                }
            }
        }
        Ok(ResolvedProfile {
            name: name.to_owned(),
            units,
            unit,
        })
    }

    /// From the farthest ancestor down to `name`.
    pub(super) fn chain(&self, name: &str) -> Result<Vec<String>, ConfigError> {
        let mut chain: Vec<String> = Vec::new();
        let mut current = Some(name.to_owned());
        while let Some(profile_name) = current {
            if chain.contains(&profile_name) {
                return Err(ConfigError::ProfileCycle(name.to_owned()));
            }
            current = self.profile(&profile_name)?.extends.clone();
            chain.push(profile_name);
        }
        chain.reverse();
        Ok(chain)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::{ConfigError, GecoConfig};

    const CONFIG: &str = r#"
[unit.backend]
kind = "process"
cwd = "backend"
run = "bacon --headless"
after = ["migrations"]
env = { APP_LOG__FORMAT = "json", APP_LOG__LEVEL = "debug" }

[unit.build-release]
kind = "oneshot"
run = "just _compile-backend"

[profile.minimal]
units = ["db", "backend"]

[profile.full]
extends = "minimal"
units = ["valhalla"]

[profile.prod-like]
extends = "minimal"
units = ["build-release"]

[profile.prod-like.override.backend]
run = "../bin/green-ecolution"
after = ["build-release", "migrations"]
env = { APP_LOG__LEVEL = "info" }
"#;

    fn parse(text: &str) -> Result<GecoConfig, ConfigError> {
        GecoConfig::parse(text, Path::new("geco.toml"))
    }

    #[test]
    fn extends_collects_the_units_of_the_whole_chain() {
        let config = parse(CONFIG).unwrap();
        assert_eq!(
            config.resolve("full").unwrap().units,
            ["db", "backend", "valhalla"]
        );
        assert_eq!(config.resolve("minimal").unwrap().units, ["db", "backend"]);
    }

    #[test]
    fn overrides_replace_fields_and_merge_env() {
        let config = parse(CONFIG).unwrap();
        let backend = &config.resolve("prod-like").unwrap().unit["backend"];

        assert_eq!(backend.run.as_deref(), Some("../bin/green-ecolution"));
        assert_eq!(backend.after, ["build-release", "migrations"]);
        assert_eq!(backend.env["APP_LOG__LEVEL"], "info");
        assert_eq!(backend.env["APP_LOG__FORMAT"], "json");
        assert_eq!(backend.cwd.as_deref(), Some(Path::new("backend")));

        let plain = &config.resolve("minimal").unwrap().unit["backend"];
        assert_eq!(plain.run.as_deref(), Some("bacon --headless"));
    }

    #[test]
    fn a_child_override_wins_over_its_parent() {
        let text = format!(
            "{CONFIG}\n[profile.prod-debug]\nextends = \"prod-like\"\n[profile.prod-debug.override.backend]\nenv = {{ APP_LOG__LEVEL = \"trace\" }}\n"
        );
        let config = parse(&text).unwrap();
        let backend = &config.resolve("prod-debug").unwrap().unit["backend"];
        assert_eq!(backend.env["APP_LOG__LEVEL"], "trace");
        assert_eq!(backend.run.as_deref(), Some("../bin/green-ecolution"));
    }

    #[test]
    fn broken_chains_are_reported_at_load_time() {
        let unknown = parse("[profile.a]\nextends = \"nope\"\n").unwrap_err();
        assert!(matches!(unknown, ConfigError::UnknownProfile(name) if name == "nope"));

        let cycle =
            parse("[profile.a]\nextends = \"b\"\n[profile.b]\nextends = \"a\"\n").unwrap_err();
        assert!(matches!(cycle, ConfigError::ProfileCycle(_)));
        assert!(cycle.to_string().contains("profile"));
    }

    #[test]
    fn only_local_units_can_be_overridden() {
        let err = parse("[profile.a]\nunits = [\"db\"]\n[profile.a.override.db]\nrun = \"x\"\n")
            .unwrap_err();
        assert!(matches!(err, ConfigError::Invalid { ref unit, .. } if unit == "db"));
        let unknown_field = parse(&format!(
            "{CONFIG}\n[profile.x.override.backend]\nkind = \"oneshot\"\n"
        ))
        .unwrap_err();
        assert!(
            unknown_field.to_string().contains("kind"),
            "{unknown_field}"
        );
    }
}
