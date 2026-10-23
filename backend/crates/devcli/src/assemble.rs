use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use bollard::Docker;

use crate::{
    config::{
        BuiltinHook, ConfigError, GecoConfig, JustVars, LocalKind, ReadyProbe, UnitConfig,
        VarError, VarSource, VerifyKind, render,
    },
    graph::{Graph, GraphError},
    state::ProcessRegistry,
    units::{
        Unit, UnitKind,
        compose::{ComposeCli, ComposeProject, ComposeUnit},
        hooked::{DevServicesFile, HookedUnit},
        migrations::MigrationsApplied,
        process::{ProcessSpec, ProcessUnit},
        unavailable::UnavailableUnit,
    },
};

pub struct Assembly {
    pub units: Vec<Arc<dyn Unit>>,
    pub graph: Graph,
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AssembleError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error(transparent)]
    Vars(#[from] VarError),
    #[error("[unit.{0}] has no `kind`, but there is no compose service with that name")]
    UnknownComposeService(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Local(LocalKind),
    Compose { oneshot: bool },
    Unavailable,
}

pub fn plan_units(
    config: &GecoConfig,
    compose: Option<&ComposeProject>,
) -> Result<(BTreeMap<String, Source>, Graph), AssembleError> {
    let mut sources = BTreeMap::new();
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Some(project) = compose {
        for (name, service) in &project.services {
            sources.insert(
                name.clone(),
                Source::Compose {
                    oneshot: service.oneshot,
                },
            );
            edges.insert(name.clone(), service.depends_on.clone());
        }
    }
    for (name, unit) in &config.unit {
        if let Some(kind) = unit.kind {
            sources.insert(name.clone(), Source::Local(kind));
            edges.insert(name.clone(), unit.after.clone());
            continue;
        }
        if let Some(deps) = edges.get_mut(name) {
            deps.extend(unit.after.iter().cloned());
            continue;
        }
        if compose.is_some() {
            return Err(AssembleError::UnknownComposeService(name.clone()));
        }
        sources.insert(name.clone(), Source::Unavailable);
        edges.insert(name.clone(), unit.after.clone());
    }
    if compose.is_none() {
        // Without Docker every name we cannot resolve must be a compose service.
        let unresolved: Vec<String> = edges
            .values()
            .flatten()
            .chain(config.profile.values().flat_map(|profile| &profile.units))
            .filter(|name| !edges.contains_key(*name))
            .cloned()
            .collect();
        for name in unresolved {
            sources.insert(name.clone(), Source::Unavailable);
            edges.entry(name).or_default();
        }
    }
    Ok((sources, Graph::new(edges)?))
}

pub fn select(full: &Graph, roots: &[String]) -> Result<Graph, AssembleError> {
    Ok(full.subgraph(&full.closure(roots)?))
}

pub async fn assemble(
    root: &Path,
    config: &GecoConfig,
    profile: &str,
    registry: ProcessRegistry,
) -> Result<Assembly, AssembleError> {
    let resolved = config.resolve(profile)?;
    let roots = resolved.units;
    let config = &GecoConfig {
        unit: resolved.unit,
        ..config.clone()
    };
    let cli = ComposeCli::new(root.to_path_buf());
    let mut warnings = Vec::new();
    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) if docker.ping().await.is_ok() => Some(docker),
        _ => {
            warnings.push("Docker is not reachable; compose units cannot start".to_owned());
            None
        }
    };
    let project = match &docker {
        Some(_) => match cli.project().await {
            Ok(project) => Some(project),
            Err(err) => {
                warnings.push(err.to_string());
                None
            }
        },
        None => None,
    };
    let unavailable_reason = warnings.last().cloned().unwrap_or_default();
    let (sources, full) = plan_units(config, project.as_ref())?;
    let graph = select(&full, &roots)?;
    let vars = JustVars::new(root.to_path_buf());

    let mut units = Vec::with_capacity(graph.start_order().len());
    for name in graph.start_order() {
        let unit_config = config.unit.get(name).cloned().unwrap_or_default();
        let base: Arc<dyn Unit> = match (sources.get(name), &docker, &project) {
            (Some(Source::Local(kind)), _, _) => Arc::new(ProcessUnit::new(
                process_spec(root, name, *kind, &unit_config, &vars)?,
                registry.clone(),
            )),
            (Some(Source::Compose { oneshot }), Some(docker), Some(project)) => {
                let kind = if *oneshot {
                    UnitKind::Oneshot
                } else {
                    UnitKind::Service
                };
                Arc::new(ComposeUnit::new(
                    name.clone(),
                    project.name.clone(),
                    kind,
                    docker.clone(),
                    cli.clone(),
                ))
            }
            _ => Arc::new(UnavailableUnit::new(
                name.clone(),
                unavailable_reason.clone(),
            )),
        };
        units.push(decorate(base, root, &unit_config, &vars, &cli)?);
    }
    Ok(Assembly {
        units,
        graph,
        warnings,
    })
}

fn process_spec(
    root: &Path,
    name: &str,
    kind: LocalKind,
    unit: &UnitConfig,
    vars: &dyn VarSource,
) -> Result<ProcessSpec, AssembleError> {
    let mut env: BTreeMap<String, String> = unit
        .env
        .iter()
        .map(|(key, value)| Ok((key.clone(), render(value, vars)?)))
        .collect::<Result<_, VarError>>()?;
    // Without a terminal cargo prints no progress bar, and the TUI's build panel reads it.
    for (key, value) in [
        ("CARGO_TERM_PROGRESS_WHEN", "always"),
        ("CARGO_TERM_PROGRESS_WIDTH", "200"),
    ] {
        env.entry(key.to_owned())
            .or_insert_with(|| value.to_owned());
    }
    let ready = match &unit.ready {
        Some(ReadyProbe::Http(url)) => Some(ReadyProbe::Http(render(url, vars)?)),
        Some(ReadyProbe::Tcp(addr)) => Some(ReadyProbe::Tcp(render(addr, vars)?)),
        None => None,
    };
    Ok(ProcessSpec {
        name: name.to_owned(),
        kind: match kind {
            LocalKind::Process => UnitKind::Service,
            LocalKind::Oneshot => UnitKind::Oneshot,
        },
        command: render(unit.run.as_deref().unwrap_or_default(), vars)?,
        cwd: unit
            .cwd
            .as_ref()
            .map_or_else(|| root.to_path_buf(), |cwd| root.join(cwd)),
        env,
        ready,
        log_format: unit.log_format,
    })
}

fn decorate(
    base: Arc<dyn Unit>,
    root: &Path,
    unit: &UnitConfig,
    vars: &dyn VarSource,
    cli: &ComposeCli,
) -> Result<Arc<dyn Unit>, AssembleError> {
    if unit.pre_start.is_empty() && unit.builtin_hooks.is_empty() && unit.verify.is_none() {
        return Ok(base);
    }
    let pre_start = unit
        .pre_start
        .iter()
        .map(|command| render(command, vars))
        .collect::<Result<_, _>>()?;
    let mut hooked = HookedUnit::new(base, root.to_path_buf()).pre_start(pre_start);
    for hook in &unit.builtin_hooks {
        match hook {
            BuiltinHook::TraefikDevServices => {
                let dir: PathBuf = root.join(".docker/infra/traefik/dynamic");
                hooked = hooked.dev_services(DevServicesFile {
                    template: dir.join("dev-services.yaml.tmpl"),
                    target: dir.join("dev-services.yaml"),
                    app_host: vars.get("app_host")?,
                    entrypoint: vars.get("traefik_entrypoint")?,
                });
            }
        }
    }
    if let Some(VerifyKind::MigrationsApplied) = unit.verify {
        hooked = hooked.verify(Arc::new(MigrationsApplied::new(
            root.join("backend/migrations"),
            cli.clone(),
            vars.get("postgres_user")?,
            vars.get("postgres_db")?,
        )));
    }
    Ok(Arc::new(hooked))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{config::StaticVars, units::compose::ComposeService};

    fn compose() -> ComposeProject {
        let service = |deps: &[&str], oneshot: bool| ComposeService {
            depends_on: deps.iter().map(|d| (*d).to_owned()).collect(),
            oneshot,
        };
        ComposeProject {
            name: "green-ecolution".into(),
            services: BTreeMap::from([
                ("db".into(), service(&[], false)),
                ("keycloak-init-db".into(), service(&["db"], true)),
                ("keycloak".into(), service(&["keycloak-init-db"], false)),
                ("pgadmin".into(), service(&["db"], false)),
                ("reverse-proxy".into(), service(&[], false)),
                ("streamlet".into(), service(&["valhalla"], false)),
                ("valhalla".into(), service(&[], false)),
            ]),
        }
    }

    fn repo_config() -> GecoConfig {
        GecoConfig::parse(
            include_str!("../../../../geco.toml"),
            Path::new("geco.toml"),
        )
        .unwrap()
    }

    #[test]
    fn the_repository_config_selects_a_valid_minimal_stack() {
        let config = repo_config();
        let (sources, full) = plan_units(&config, Some(&compose())).unwrap();
        let graph = select(&full, &config.resolve("minimal").unwrap().units).unwrap();

        assert_eq!(
            graph.start_order(),
            [
                "db",
                "frontend",
                "reverse-proxy",
                "keycloak-init-db",
                "migrations",
                "keycloak",
                "backend"
            ]
        );
        assert_eq!(sources["backend"], Source::Local(LocalKind::Process));
        assert_eq!(
            sources["keycloak-init-db"],
            Source::Compose { oneshot: true }
        );
        assert!(
            select(&full, &config.resolve("full").unwrap().units)
                .unwrap()
                .contains("streamlet")
        );
    }

    #[test]
    fn additions_must_name_an_existing_compose_service() {
        let config =
            GecoConfig::parse("[unit.redis]\nafter = []\n", Path::new("geco.toml")).unwrap();
        assert!(matches!(
            plan_units(&config, Some(&compose())),
            Err(AssembleError::UnknownComposeService(name)) if name == "redis"
        ));
    }

    #[test]
    fn without_compose_unknown_names_become_unavailable() {
        let config = repo_config();
        let (sources, full) = plan_units(&config, None).unwrap();
        let graph = select(&full, &config.resolve("minimal").unwrap().units).unwrap();

        assert_eq!(sources["db"], Source::Unavailable);
        assert_eq!(sources["keycloak"], Source::Unavailable);
        assert_eq!(sources["backend"], Source::Local(LocalKind::Process));
        assert!(graph.contains("db"));
    }

    #[test]
    fn process_specs_render_placeholders() {
        let vars = StaticVars(HashMap::from([
            ("binary_name", "green-ecolution"),
            ("app_proto", "https"),
            ("app_host", "geco.test"),
            ("app_port", "3443"),
            ("postgres_host", "localhost"),
            ("postgres_port", "5432"),
            ("postgres_db", "postgres"),
            ("postgres_user", "postgres"),
            ("postgres_password", "postgres"),
        ]));
        let config = repo_config();

        let spec = process_spec(
            Path::new("/repo"),
            "backend",
            LocalKind::Process,
            &config.unit["backend"],
            &vars,
        )
        .unwrap();

        assert_eq!(
            spec.command,
            "bacon --headless --job run -- --bin green-ecolution"
        );
        assert_eq!(spec.cwd, Path::new("/repo/backend"));
        assert_eq!(spec.kind, UnitKind::Service);
        assert_eq!(
            spec.env["APP_APPLICATION__BASE_URL"],
            "https://geco.test:3443"
        );
        assert_eq!(
            spec.ready,
            Some(ReadyProbe::Http("http://localhost:3030/api/ready".into()))
        );
    }

    #[test]
    fn process_specs_ask_cargo_for_its_progress_unless_the_unit_decides() {
        let config = GecoConfig::parse(
            r#"
[unit.backend]
kind = "process"
run = "bacon"
[unit.quiet]
kind = "oneshot"
run = "cargo build"
[unit.quiet.env]
CARGO_TERM_PROGRESS_WHEN = "never"
"#,
            Path::new("geco.toml"),
        )
        .unwrap();
        let vars = StaticVars(HashMap::new());
        let spec = |name: &str, kind| {
            process_spec(Path::new("/repo"), name, kind, &config.unit[name], &vars).unwrap()
        };

        let backend = spec("backend", LocalKind::Process);
        assert_eq!(backend.env["CARGO_TERM_PROGRESS_WHEN"], "always");
        assert_eq!(backend.env["CARGO_TERM_PROGRESS_WIDTH"], "200");
        assert_eq!(
            spec("quiet", LocalKind::Oneshot).env["CARGO_TERM_PROGRESS_WHEN"],
            "never"
        );
    }

    #[test]
    fn select_follows_the_resolved_profile_and_its_overrides() {
        let config = GecoConfig::parse(
            r#"
[unit.migrations]
kind = "oneshot"
run = "true"
[unit.build-release]
kind = "oneshot"
run = "true"
[unit.backend]
kind = "process"
run = "bacon"
after = ["migrations"]
[profile.minimal]
units = ["backend"]
[profile.prod-like]
extends = "minimal"
[profile.prod-like.override.backend]
after = ["build-release", "migrations"]
"#,
            Path::new("geco.toml"),
        )
        .unwrap();

        let minimal = config.resolve("minimal").unwrap();
        let effective = GecoConfig {
            unit: minimal.unit.clone(),
            ..config.clone()
        };
        let (_, full) = plan_units(&effective, None).unwrap();
        let graph = select(&full, &minimal.units).unwrap();
        assert_eq!(graph.start_order(), ["migrations", "backend"]);

        let prod = config.resolve("prod-like").unwrap();
        let effective = GecoConfig {
            unit: prod.unit.clone(),
            ..config.clone()
        };
        let (_, full) = plan_units(&effective, None).unwrap();
        let graph = select(&full, &prod.units).unwrap();
        assert!(graph.contains("build-release"));
        assert_eq!(
            graph.dependencies("backend").collect::<Vec<_>>(),
            ["build-release", "migrations"]
        );
    }

    #[test]
    fn the_repository_config_resolves_every_profile() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let config = GecoConfig::load(&root.join("geco.toml")).unwrap();
        for profile in ["minimal", "full", "prod-like"] {
            config.resolve(profile).unwrap();
        }
        let prod = config.resolve("prod-like").unwrap();
        assert_eq!(
            prod.unit["backend"].run.as_deref(),
            Some("../bin/{{binary_name}}")
        );
        assert_eq!(
            prod.unit["frontend"].run.as_deref(),
            Some("pnpm run preview")
        );
        assert_eq!(prod.unit["frontend"].env["USE_TRAEFIK"], "1");
        assert_eq!(
            prod.unit["frontend"].ready,
            Some(crate::config::ReadyProbe::Tcp("localhost:5173".into()))
        );
    }
}
