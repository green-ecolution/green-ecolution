use std::{collections::BTreeMap, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};

use crate::config::TasksConfig;

pub trait RecipeSource: Send + Sync {
    /// The output of `just --dump --dump-format json`.
    fn dump(&self) -> Result<String, CatalogError>;
}

/// `--dump` prints the justfile's expressions, not their values, so no secret is evaluated.
pub struct JustRecipes {
    root: PathBuf,
}

impl JustRecipes {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl RecipeSource for JustRecipes {
    fn dump(&self) -> Result<String, CatalogError> {
        let output = Command::new("just")
            .args(["--dump", "--dump-format", "json"])
            .current_dir(&self.root)
            .output()
            .map_err(|err| CatalogError::Just(err.to_string()))?;
        if !output.status.success() {
            return Err(CatalogError::Just(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        String::from_utf8(output.stdout).map_err(|err| CatalogError::Parse(err.to_string()))
    }
}

#[cfg(test)]
pub struct StaticRecipes(pub String);

#[cfg(test)]
impl RecipeSource for StaticRecipes {
    fn dump(&self) -> Result<String, CatalogError> {
        Ok(self.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    #[error("cannot list the just recipes: {0}")]
    Just(String),
    #[error("cannot read the just recipe list: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
    Singular,
    Star,
    Plus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    /// Only a literal default; one that just computes from an expression is `None`.
    pub default: Option<String>,
    pub optional: bool,
    pub kind: ParamKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub doc: Option<String>,
    pub group: Option<String>,
    pub private: bool,
    pub destructive: bool,
    pub params: Vec<Param>,
}

impl Recipe {
    pub fn command(&self, program: &str, args: &[String]) -> String {
        std::iter::once(program.to_owned())
            .chain(std::iter::once(self.name.clone()))
            .chain(args.iter().map(|arg| quote(arg)))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn signature(&self) -> String {
        let params = self
            .params
            .iter()
            .map(|param| match (param.kind, param.optional) {
                (ParamKind::Singular, false) => format!("<{}>", param.name),
                (ParamKind::Singular, true) => format!("[{}]", param.name),
                (ParamKind::Plus, _) => format!("<{}…>", param.name),
                (ParamKind::Star, _) => format!("[{}…]", param.name),
            });
        std::iter::once(self.name.clone())
            .chain(params)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    recipes: BTreeMap<String, Recipe>,
    aliases: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Dump {
    recipes: BTreeMap<String, DumpRecipe>,
    #[serde(default)]
    aliases: BTreeMap<String, DumpAlias>,
}

#[derive(Deserialize)]
struct DumpRecipe {
    name: String,
    doc: Option<String>,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    attributes: Vec<serde_json::Value>,
    #[serde(default)]
    parameters: Vec<DumpParam>,
}

#[derive(Deserialize)]
struct DumpParam {
    name: String,
    #[serde(default)]
    default: serde_json::Value,
    kind: ParamKind,
}

#[derive(Deserialize)]
struct DumpAlias {
    target: String,
}

impl Catalog {
    pub fn parse(json: &str, tasks: &TasksConfig) -> Result<Self, CatalogError> {
        let dump: Dump =
            serde_json::from_str(json).map_err(|err| CatalogError::Parse(err.to_string()))?;
        let recipes = dump
            .recipes
            .into_values()
            .map(|recipe| {
                let group = recipe.attributes.iter().find_map(|attribute| {
                    attribute
                        .get("group")
                        .and_then(|group| group.as_str())
                        .map(str::to_owned)
                });
                let params = recipe
                    .parameters
                    .into_iter()
                    .map(|param| Param {
                        default: param.default.as_str().map(str::to_owned),
                        optional: !param.default.is_null() || param.kind == ParamKind::Star,
                        name: param.name,
                        kind: param.kind,
                    })
                    .collect();
                let destructive = tasks.destructive.contains(&recipe.name);
                (
                    recipe.name.clone(),
                    Recipe {
                        name: recipe.name,
                        doc: recipe.doc,
                        group,
                        private: recipe.private,
                        destructive,
                        params,
                    },
                )
            })
            .collect();
        let aliases = dump
            .aliases
            .into_iter()
            .map(|(name, alias)| (name, alias.target))
            .collect();
        Ok(Self { recipes, aliases })
    }

    pub fn get(&self, name: &str) -> Option<&Recipe> {
        let name = self.aliases.get(name).map_or(name, String::as_str);
        self.recipes.get(name)
    }

    pub fn recipes(&self) -> impl Iterator<Item = &Recipe> {
        self.recipes.values()
    }

    pub fn aliases(&self) -> impl Iterator<Item = (&str, &str)> {
        self.aliases
            .iter()
            .map(|(alias, target)| (alias.as_str(), target.as_str()))
    }
}

/// POSIX quoting for `sh -c`; plain words stay readable in the log.
pub fn quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_./=:@%+,-".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Trimmed from `just --dump --dump-format json` of this repository.
    pub(crate) const DUMP: &str = r#"{
      "aliases": {"fe-dev": {"attributes": [], "name": "fe-dev", "target": "frontend-dev"}},
      "recipes": {
        "_compose": {"attributes": ["positional-arguments"], "doc": "Positional arguments keep args with spaces intact.", "name": "_compose", "parameters": [{"default": null, "kind": "star", "name": "ARGS"}], "private": true},
        "frontend-dev": {"attributes": [{"group": "run"}], "doc": "Run the frontend dev server", "name": "frontend-dev", "parameters": [], "private": false},
        "import-staging": {"attributes": [{"group": "db"}], "doc": "Destructive: drops schema public. Pass --yes to skip the prompt.", "name": "import-staging", "parameters": [{"default": "staging.sql", "kind": "singular", "name": "file"}, {"default": null, "kind": "star", "name": "ARGS"}], "private": false},
        "migrate-new": {"attributes": [{"group": "db"}], "doc": "Create a new migration file", "name": "migrate-new", "parameters": [{"default": null, "kind": "singular", "name": "name"}], "private": false},
        "migrate-reset": {"attributes": [{"group": "db"}], "doc": "Wipe schema and re-apply all migrations", "name": "migrate-reset", "parameters": [], "private": false},
        "migrate-up": {"attributes": [{"group": "db"}], "dependencies": [{"arguments": [], "recipe": "_migrate-build"}], "doc": "Apply all pending migrations", "name": "migrate-up", "parameters": [], "private": false},
        "tidy": {"attributes": [], "doc": null, "name": "tidy", "parameters": [{"default": ["variable", "backend_dir"], "kind": "singular", "name": "dir"}], "private": false}
      }
    }"#;

    pub(crate) fn catalog() -> Catalog {
        Catalog::parse(
            DUMP,
            &TasksConfig {
                destructive: vec!["migrate-reset".into(), "import-staging".into()],
            },
        )
        .unwrap()
    }

    #[test]
    fn reads_doc_group_privacy_and_destructiveness() {
        let catalog = catalog();
        let up = catalog.get("migrate-up").unwrap();
        assert_eq!(up.doc.as_deref(), Some("Apply all pending migrations"));
        assert_eq!(up.group.as_deref(), Some("db"));
        assert!(!up.private && !up.destructive);
        assert!(catalog.get("_compose").unwrap().private);
        assert!(catalog.get("migrate-reset").unwrap().destructive);
        assert_eq!(catalog.get("tidy").unwrap().group, None);
    }

    #[test]
    fn reads_parameters() {
        let catalog = catalog();
        assert_eq!(
            catalog.get("import-staging").unwrap().params,
            [
                Param {
                    name: "file".into(),
                    default: Some("staging.sql".into()),
                    optional: true,
                    kind: ParamKind::Singular,
                },
                Param {
                    name: "ARGS".into(),
                    default: None,
                    optional: true,
                    kind: ParamKind::Star,
                },
            ]
        );
        let name = &catalog.get("migrate-new").unwrap().params[0];
        assert!(!name.optional);
        let dir = &catalog.get("tidy").unwrap().params[0];
        assert_eq!(
            dir.default, None,
            "an expression default has no literal value"
        );
        assert!(dir.optional);
    }

    #[test]
    fn aliases_resolve_to_their_recipe() {
        let catalog = catalog();
        assert_eq!(catalog.get("fe-dev").unwrap().name, "frontend-dev");
        assert_eq!(
            catalog.aliases().collect::<Vec<_>>(),
            [("fe-dev", "frontend-dev")]
        );
        assert!(catalog.get("nope").is_none());
    }

    #[test]
    fn commands_quote_their_arguments() {
        let catalog = catalog();
        let staging = catalog.get("import-staging").unwrap();
        assert_eq!(
            staging.command("just", &["my dump.sql".into(), "--yes".into()]),
            "just import-staging 'my dump.sql' --yes"
        );
        assert_eq!(
            catalog.get("migrate-up").unwrap().command("just", &[]),
            "just migrate-up"
        );
        assert_eq!(quote("it's"), r"'it'\''s'");
        assert_eq!(quote(""), "''");
        assert_eq!(quote("a=b,c:d/e.f@g%h+i-j_k"), "a=b,c:d/e.f@g%h+i-j_k");
    }

    #[test]
    fn signatures_mark_optional_and_variadic_parameters() {
        let catalog = catalog();
        assert_eq!(
            catalog.get("migrate-new").unwrap().signature(),
            "migrate-new <name>"
        );
        assert_eq!(
            catalog.get("import-staging").unwrap().signature(),
            "import-staging [file] [ARGS…]"
        );
        assert_eq!(catalog.get("migrate-up").unwrap().signature(), "migrate-up");
    }

    #[test]
    fn a_broken_dump_is_reported() {
        assert!(matches!(
            Catalog::parse("not json", &TasksConfig::default()),
            Err(CatalogError::Parse(_))
        ));
    }
}
