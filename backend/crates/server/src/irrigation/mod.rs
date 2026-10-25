//! Irrigation as a module. Its domain types live in `irrigation-domain`; the
//! services and adapters below still sit in the server crate until they move
//! into `irrigation-server`.

pub mod handlers;
pub mod planning;
pub mod settings;
pub mod statistics;
pub mod targets;

use std::sync::Arc;

use domain::{authorization::ResourceKey, module::ModuleKey};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    configuration::Settings,
    infra::{pg_cluster::PgTreeClusterRepository, pg_tree::PgTreeRepository},
    module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime},
};
use handlers::SensorLifecycle;
use settings::{
    IrrigationDefaults, http::IrrigationHttpState, repository::PgIrrigationSettings,
    service::IrrigationSettingsService,
};

pub use irrigation_domain::{KEY, key};

/// The `modules.irrigation` section of the YAML.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct IrrigationConfig {
    #[serde(default = "default_tree_demand_liters")]
    pub tree_demand_liters: f64,
    #[serde(default = "default_just_watered_ttl_secs")]
    pub just_watered_ttl_secs: u64,
    #[serde(default = "default_just_watered_sweep_interval_secs")]
    pub just_watered_sweep_interval_secs: u64,
    #[serde(default = "default_nearest_tree_max_radius")]
    pub nearest_tree_max_radius: f64,
    #[serde(default = "default_nearest_tree_default_limit")]
    pub nearest_tree_default_limit: u32,
    #[serde(default = "default_nearest_tree_max_limit")]
    pub nearest_tree_max_limit: u32,
}

fn default_tree_demand_liters() -> f64 {
    80.0
}
fn default_just_watered_ttl_secs() -> u64 {
    86_400
}
fn default_just_watered_sweep_interval_secs() -> u64 {
    3_600
}
fn default_nearest_tree_max_radius() -> f64 {
    500.0
}
fn default_nearest_tree_default_limit() -> u32 {
    10
}
fn default_nearest_tree_max_limit() -> u32 {
    50
}

impl IrrigationConfig {
    pub fn from_settings(settings: &Settings) -> Result<Self, config::ConfigError> {
        match settings.modules.get(KEY) {
            Some(section) => section.deserialize_extra(),
            None => config::Value::new(None, config::ValueKind::Table(config::Map::new()))
                .try_deserialize(),
        }
    }
}

pub mod resources {
    use domain::authorization::ResourceKey;

    pub const TREE: ResourceKey = ResourceKey::declare("tree");
    pub const TREE_CLUSTER: ResourceKey = ResourceKey::declare("tree_cluster");
    pub const WATERING_PLAN: ResourceKey = ResourceKey::declare("watering_plan");
    pub const ALL: [ResourceKey; 3] = [TREE, TREE_CLUSTER, WATERING_PLAN];
}

#[cfg(test)]
pub(crate) fn test_catalog() -> domain::authorization::ResourceCatalog {
    domain::authorization::ResourceCatalog::new(resources::ALL).unwrap()
}

pub struct IrrigationModule;

fn routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    settings::http::routes()
}

impl Module for IrrigationModule {
    fn key(&self) -> ModuleKey {
        key()
    }

    fn default_enabled(&self) -> bool {
        true
    }

    fn openapi(&self) -> utoipa::openapi::OpenApi {
        routes().into_openapi()
    }

    fn resources(&self) -> &'static [ResourceKey] {
        &resources::ALL
    }

    fn build(&self, ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
        let config: IrrigationConfig = ctx.module_config(&key())?;
        let defaults = IrrigationDefaults::from_config(&config).map_err(|e| ModuleBuildError {
            module: key(),
            reason: e.to_string(),
        })?;
        let repo = Arc::new(PgIrrigationSettings::new(
            ctx.pool.clone(),
            defaults,
            ctx.organizations.clone(),
        ));
        let state = Arc::new(IrrigationHttpState {
            settings: Arc::new(IrrigationSettingsService::new(repo.clone())),
            access: ctx.access.clone(),
            authorization: ctx.authorization.clone(),
            organizations: ctx.organizations.clone(),
            users: ctx.users.clone(),
        });
        let trees = Arc::new(PgTreeRepository::new(ctx.pool.clone()));
        let clusters = Arc::new(PgTreeClusterRepository::new(ctx.pool.clone()));
        let mut runtime = ModuleRuntime::new(ModuleRoutes {
            public: OpenApiRouter::new(),
            protected: routes().with_state(state),
        });
        runtime.targets = Some(Arc::new(targets::IrrigationTargets::new(
            trees.clone(),
            clusters.clone(),
        )));
        runtime.planning = Some(Arc::new(planning::IrrigationPlanning::new(
            clusters,
            ctx.organizations.clone(),
            repo,
        )));
        runtime.statistics = Some(Arc::new(statistics::IrrigationStatistics::new(
            ctx.pool.clone(),
        )));
        runtime.event_handlers.push(Arc::new(SensorLifecycle::new(
            trees.clone(),
            trees,
            &ctx.sensors,
        )));
        Ok(runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with(overrides: &[(&str, &str)]) -> Settings {
        let mut builder = config::Config::builder();
        for (k, v) in overrides {
            builder = builder.set_override(*k, *v).unwrap();
        }
        let modules: std::collections::BTreeMap<String, crate::configuration::ModuleSettings> =
            builder.build().unwrap().get("modules").unwrap_or_default();
        let mut settings = Settings::for_test(crate::configuration::AuthSettings {
            enabled: false,
            issuer_url: String::new(),
            frontend_client_id: String::new(),
            backend_client_id: String::new(),
            backend_client_secret: secrecy::SecretString::from(String::new()),
            jwks_refresh_interval_secs: 1,
            jwks_refresh_timeout_secs: 1,
            default_redirect_url: String::new(),
            expected_audience: None,
        });
        settings.modules = modules;
        settings
    }

    #[test]
    fn string_values_from_env_overrides_fill_the_numeric_fields() {
        let settings = settings_with(&[
            ("modules.irrigation.tree_demand_liters", "90"),
            ("modules.irrigation.just_watered_ttl_secs", "7200"),
            ("modules.irrigation.nearest_tree_default_limit", "7"),
        ]);
        let config = IrrigationConfig::from_settings(&settings).unwrap();
        assert_eq!(config.tree_demand_liters, 90.0);
        assert_eq!(config.just_watered_ttl_secs, 7200);
        assert_eq!(config.nearest_tree_default_limit, 7);
        assert_eq!(config.nearest_tree_max_limit, 50);
    }

    #[test]
    fn an_absent_section_yields_the_defaults() {
        let config = IrrigationConfig::from_settings(&settings_with(&[])).unwrap();
        assert_eq!(config.tree_demand_liters, 80.0);
        assert_eq!(config.just_watered_sweep_interval_secs, 3_600);
    }

    #[test]
    fn key_matches_the_constant() {
        assert_eq!(key().as_str(), KEY);
    }
}
