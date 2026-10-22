use std::{collections::HashMap, path::PathBuf, process::Command, sync::Mutex};

use crate::sync::lock;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VarError {
    #[error("unclosed placeholder in `{0}`")]
    Unclosed(String),
    #[error("cannot evaluate just variable `{name}`: {reason}")]
    Evaluate { name: String, reason: String },
    #[error("`{0}` is not a just variable name")]
    InvalidName(String),
}

pub trait VarSource {
    fn get(&self, name: &str) -> Result<String, VarError>;
}

pub fn render(template: &str, vars: &dyn VarSource) -> Result<String, VarError> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or_else(|| VarError::Unclosed(template.to_owned()))?;
        out.push_str(&vars.get(after[..end].trim())?);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Reads variables one at a time via `just --evaluate <name>`; evaluating all of them would
/// pull the Porkbun API keys into memory for no reason.
pub struct JustVars {
    root: PathBuf,
    cache: Mutex<HashMap<String, String>>,
}

impl JustVars {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            cache: Mutex::default(),
        }
    }
}

impl VarSource for JustVars {
    fn get(&self, name: &str) -> Result<String, VarError> {
        // The name becomes an argument of `just`; `{{--dump}}` must not turn into a flag.
        let valid = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid {
            return Err(VarError::InvalidName(name.to_owned()));
        }
        if let Some(value) = lock(&self.cache).get(name) {
            return Ok(value.clone());
        }
        let evaluate_error = |reason: String| VarError::Evaluate {
            name: name.to_owned(),
            reason,
        };
        let output = Command::new("just")
            .arg("--evaluate")
            .arg(name)
            .current_dir(&self.root)
            .output()
            .map_err(|err| evaluate_error(err.to_string()))?;
        if !output.status.success() {
            return Err(evaluate_error(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let value = String::from_utf8_lossy(&output.stdout).into_owned();
        lock(&self.cache).insert(name.to_owned(), value.clone());
        Ok(value)
    }
}

#[cfg(test)]
pub struct StaticVars(pub HashMap<&'static str, &'static str>);

#[cfg(test)]
impl VarSource for StaticVars {
    fn get(&self, name: &str) -> Result<String, VarError> {
        self.0
            .get(name)
            .map(|value| (*value).to_owned())
            .ok_or_else(|| VarError::Evaluate {
                name: name.to_owned(),
                reason: "not set".to_owned(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn vars() -> StaticVars {
        StaticVars(HashMap::from([
            ("app_host", "geco.test"),
            ("app_port", "3443"),
        ]))
    }

    #[test]
    fn replaces_placeholders_and_ignores_inner_whitespace() {
        let out = render("https://{{app_host}}:{{ app_port }}/api", &vars()).unwrap();
        assert_eq!(out, "https://geco.test:3443/api");
    }

    #[test]
    fn leaves_text_without_placeholders_alone() {
        assert_eq!(render("pnpm run dev", &vars()).unwrap(), "pnpm run dev");
    }

    #[test]
    fn rejects_unclosed_placeholders() {
        assert!(matches!(
            render("{{app_host", &vars()),
            Err(VarError::Unclosed(_))
        ));
    }

    #[test]
    fn just_vars_reject_names_that_are_no_identifiers() {
        let dir = tempfile::tempdir().unwrap();
        let vars = JustVars::new(dir.path().to_path_buf());

        for name in ["--dump", "-l", "a b", "x;y", ""] {
            assert_eq!(
                vars.get(name),
                Err(VarError::InvalidName(name.to_owned())),
                "{name:?}"
            );
        }
        assert_eq!(
            render("{{--dump}}", &vars),
            Err(VarError::InvalidName("--dump".to_owned()))
        );
    }

    #[test]
    fn reports_unknown_variables() {
        assert!(matches!(
            render("{{nope}}", &vars()),
            Err(VarError::Evaluate { name, .. }) if name == "nope"
        ));
    }
}
