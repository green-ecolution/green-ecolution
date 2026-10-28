//! Irrigation as a module. Its domain types live in `irrigation-domain`; the
//! services, adapters and routes below reach the core only through its public
//! types and the `ModuleContext`.

pub mod error;
pub mod handlers;
pub mod http;
pub mod infra;
pub mod planning;
pub mod service;
pub mod settings;
pub mod targets;

use std::{sync::Arc, time::Duration};

use domain::{authorization::ResourceKey, module::ModuleKey};
use server::module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime};

use http::state::{IrrigationHttpState, NearestTreeLimits};
use infra::{
    pg_cluster::PgTreeClusterRepository, pg_evaluation::PgEvaluationRepository,
    pg_plugin_tree_refs::PgPluginTreeRefs, pg_tree::PgTreeRepository,
};
use service::{
    cluster_service::ClusterService, evaluation_service::EvaluationService,
    plugin_ingest_service::PluginIngestService, sensor_overview::IrrigationSensorService,
    tree_service::TreeService,
};
use settings::{
    IrrigationDefaults, repository::PgIrrigationSettings, service::IrrigationSettingsService,
};

pub use irrigation_domain::{KEY, key};

/// The `modules.irrigation` section of the YAML.
#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
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

impl Module for IrrigationModule {
    fn key(&self) -> ModuleKey {
        key()
    }

    fn default_enabled(&self) -> bool {
        true
    }

    fn openapi(&self) -> utoipa::openapi::OpenApi {
        let mut api = http::public_routes()
            .merge(http::protected_routes())
            .into_openapi();
        api.tags = Some(http::tags());
        api
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
        let settings = Arc::new(PgIrrigationSettings::new(
            ctx.pool.clone(),
            defaults,
            ctx.organizations.clone(),
        ));
        let trees = Arc::new(PgTreeRepository::new(ctx.pool.clone()));
        let clusters = Arc::new(PgTreeClusterRepository::new(ctx.pool.clone()));

        let tree_service = Arc::new(TreeService::new(
            trees.clone(),
            trees.clone(),
            clusters.clone(),
            ctx.sensors.clone(),
            ctx.event_bus.clone(),
        ));
        let cluster_service = Arc::new(ClusterService::new(
            clusters.clone(),
            clusters.clone(),
            trees.clone(),
            trees.clone(),
            ctx.sensors.clone(),
            ctx.event_bus.clone(),
            ctx.comments.clone(),
            ctx.plans.clone(),
        ));
        // Reuses `TreeService::delete` for its own delete flow so `TreeDeleted`
        // fires and cluster centroid/status stay in sync, instead of a second
        // copy of that logic.
        let plugin_ingest = Arc::new(PluginIngestService::new(
            trees.clone(),
            trees.clone(),
            Arc::new(PgPluginTreeRefs::new(ctx.pool.clone())),
            tree_service.clone(),
            ctx.event_bus.clone(),
            ctx.authorization.clone(),
            ctx.access.clone(),
        ));

        let state = Arc::new(IrrigationHttpState {
            trees: tree_service,
            clusters: cluster_service.clone(),
            evaluation: Arc::new(EvaluationService::new(Arc::new(
                PgEvaluationRepository::new(ctx.pool.clone()),
            ))),
            plugin_ingest,
            sensor_overview: Arc::new(IrrigationSensorService::new(
                ctx.sensors.clone(),
                trees.clone(),
                clusters.clone(),
            )),
            settings: Arc::new(IrrigationSettingsService::new(settings.clone())),
            sensors: ctx.sensors.clone(),
            plans: ctx.plans.clone(),
            regions: ctx.regions.clone(),
            comments: ctx.comments.clone(),
            access: ctx.access.clone(),
            authorization: ctx.authorization.clone(),
            organizations: ctx.organizations.clone(),
            users: ctx.users.clone(),
            plugin_auth: ctx.plugin_auth.clone(),
            nearest_tree_limits: NearestTreeLimits {
                max_radius_meters: config.nearest_tree_max_radius,
                default_limit: config.nearest_tree_default_limit,
                max_limit: config.nearest_tree_max_limit,
            },
        });
        let mut runtime = ModuleRuntime::new(ModuleRoutes {
            public: http::public_routes().with_state(state.clone()),
            protected: http::protected_routes().with_state(state),
        });
        runtime.event_handlers = handlers::all(
            trees.clone(),
            trees.clone(),
            clusters.clone(),
            clusters.clone(),
            ctx.regions.clone(),
            ctx.sensor_readings.clone(),
            &ctx.sensors,
        );
        runtime
            .background_tasks
            .push(Box::pin(infra::watering_status_expiry::run(
                cluster_service,
                ctx.organizations.clone(),
                settings.clone(),
                Duration::from_secs(config.just_watered_sweep_interval_secs),
            )));
        runtime.targets = Some(Arc::new(targets::IrrigationTargets::new(
            trees,
            clusters.clone(),
        )));
        runtime.planning = Some(Arc::new(planning::IrrigationPlanning::new(
            clusters,
            ctx.organizations.clone(),
            settings,
        )));
        runtime.statistics = Some(Arc::new(infra::statistics::IrrigationStatistics::new(
            ctx.pool.clone(),
        )));
        Ok(runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use server::configuration::Settings;

    fn settings_with(overrides: &[(&str, &str)]) -> Settings {
        let mut builder = config::Config::builder();
        for (k, v) in overrides {
            builder = builder.set_override(*k, *v).unwrap();
        }
        let modules: std::collections::BTreeMap<String, server::configuration::ModuleSettings> =
            builder.build().unwrap().get("modules").unwrap_or_default();
        let mut settings = Settings::for_test(server::configuration::AuthSettings {
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
        let config: IrrigationConfig = settings.module_config(KEY).unwrap();
        assert_eq!(config.tree_demand_liters, 90.0);
        assert_eq!(config.just_watered_ttl_secs, 7200);
        assert_eq!(config.nearest_tree_default_limit, 7);
        assert_eq!(config.nearest_tree_max_limit, 50);
    }

    #[test]
    fn an_absent_section_yields_the_defaults() {
        let config: IrrigationConfig = settings_with(&[]).module_config(KEY).unwrap();
        assert_eq!(config.tree_demand_liters, 80.0);
        assert_eq!(config.just_watered_sweep_interval_secs, 3_600);
    }

    #[test]
    fn a_misspelled_key_is_rejected_at_load() {
        let err = settings_with(&[("modules.irrigation.tree_demand_litres", "90")])
            .module_config::<IrrigationConfig>(KEY)
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown field"), "{err}");
        assert!(err.contains("tree_demand_litres"), "{err}");
    }

    #[test]
    fn default_enabled_does_not_reach_the_module_config() {
        let config: IrrigationConfig =
            settings_with(&[("modules.irrigation.default_enabled", "true")])
                .module_config(KEY)
                .unwrap();
        assert_eq!(config, settings_with(&[]).module_config(KEY).unwrap());
    }

    #[test]
    fn base_yaml_matches_the_code_defaults() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../config/base.yaml");
        let file = config::Config::builder()
            .add_source(config::File::new(path, config::FileFormat::Yaml))
            .build()
            .unwrap();
        let section: config::Value = file.get("modules.irrigation").unwrap();
        let from_yaml: IrrigationConfig = section.try_deserialize().unwrap();
        let defaults: IrrigationConfig = settings_with(&[]).module_config(KEY).unwrap();
        assert_eq!(from_yaml, defaults);
    }

    #[test]
    fn key_matches_the_constant() {
        assert_eq!(key().as_str(), KEY);
    }
}
