//! Waste as a module. Its domain types live in `waste-domain`; the services,
//! adapters and routes here reach the core only through its public types and
//! the `ModuleContext`.

pub mod error;
pub mod http;
pub mod infra;
pub mod service;
pub mod targets;

use std::sync::Arc;

use domain::{authorization::ResourceKey, module::ModuleKey};
use server::module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime};
use waste_domain::settings::FullThreshold;

use http::state::WasteHttpState;
use infra::pg_waste_bin::PgWasteBinRepository;
use service::waste_bin_service::{FixedThreshold, WasteBinService};

pub use waste_domain::{KEY, key};

/// The `modules.waste` section of the YAML.
#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WasteConfig {
    #[serde(default = "default_full_threshold_percent")]
    pub full_threshold_percent: u8,
}

fn default_full_threshold_percent() -> u8 {
    FullThreshold::DEFAULT
}

pub mod resources {
    use domain::authorization::ResourceKey;

    pub const WASTE_BIN: ResourceKey = ResourceKey::declare("waste_bin");
    pub const COLLECTION_PLAN: ResourceKey = ResourceKey::declare("collection_plan");
    pub const ALL: [ResourceKey; 2] = [WASTE_BIN, COLLECTION_PLAN];
}

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");

pub struct WasteModule;

impl Module for WasteModule {
    fn key(&self) -> ModuleKey {
        key()
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn openapi(&self) -> utoipa::openapi::OpenApi {
        let mut api = http::protected_routes().into_openapi();
        api.tags = Some(http::tags());
        api
    }

    fn resources(&self) -> &'static [ResourceKey] {
        &resources::ALL
    }

    fn migrations(&self) -> Vec<sqlx::migrate::Migration> {
        MIGRATOR.iter().cloned().collect()
    }

    fn build(&self, ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
        let _config: WasteConfig = ctx.module_config(&key())?;
        let bins = Arc::new(PgWasteBinRepository::new(ctx.pool.clone()));
        let threshold = FullThreshold::new(i64::from(FullThreshold::DEFAULT))
            .expect("the default threshold is in range");
        let bin_service = Arc::new(WasteBinService::new(
            bins.clone(),
            bins,
            ctx.sensors.clone(),
            ctx.plans.clone(),
            ctx.comments.clone(),
            ctx.settings.clone(),
            Arc::new(FixedThreshold(threshold)),
        ));
        let state = Arc::new(WasteHttpState {
            bins: bin_service,
            access: ctx.access.clone(),
            authorization: ctx.authorization.clone(),
            users: ctx.users.clone(),
        });
        Ok(ModuleRuntime::new(ModuleRoutes {
            public: utoipa_axum::router::OpenApiRouter::new(),
            protected: http::protected_routes().with_state(state),
        }))
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
    fn string_values_from_env_overrides_fill_the_numeric_field() {
        let config: WasteConfig = settings_with(&[("modules.waste.full_threshold_percent", "90")])
            .module_config(KEY)
            .unwrap();
        assert_eq!(config.full_threshold_percent, 90);
    }

    #[test]
    fn an_absent_section_yields_the_defaults() {
        let config: WasteConfig = settings_with(&[]).module_config(KEY).unwrap();
        assert_eq!(config.full_threshold_percent, 80);
    }

    #[test]
    fn a_misspelled_key_is_rejected_at_load() {
        let err = settings_with(&[("modules.waste.full_treshold_percent", "90")])
            .module_config::<WasteConfig>(KEY)
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown field"), "{err}");
        assert!(err.contains("full_treshold_percent"), "{err}");
    }

    #[test]
    fn default_enabled_does_not_reach_the_module_config() {
        let config: WasteConfig = settings_with(&[("modules.waste.default_enabled", "true")])
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
        let section: config::Value = file.get("modules.waste").unwrap();
        let from_yaml: WasteConfig = section.try_deserialize().unwrap();
        let defaults: WasteConfig = settings_with(&[]).module_config(KEY).unwrap();
        assert_eq!(from_yaml, defaults);
    }

    #[test]
    fn key_matches_the_constant() {
        assert_eq!(key().as_str(), KEY);
    }

    #[test]
    fn waste_is_off_by_default() {
        assert!(!WasteModule.default_enabled());
        let profiles = [
            include_str!("../../../../../config/base.yaml"),
            include_str!("../../../../../config/local.yaml"),
            include_str!("../../../../../config/staging.yaml"),
            include_str!("../../../../../config/demo.yaml"),
            include_str!("../../../../../config/production.yaml"),
        ];
        for yaml in profiles {
            let file = config::Config::builder()
                .add_source(config::File::from_str(yaml, config::FileFormat::Yaml))
                .build()
                .unwrap();
            assert!(
                file.get::<config::Value>("modules.waste.default_enabled")
                    .is_err(),
                "no profile may set modules.waste.default_enabled"
            );
        }
    }

    #[test]
    fn migrations_ship_both_tables() {
        let names: Vec<_> = WasteModule
            .migrations()
            .iter()
            .map(|m| m.description.to_string())
            .collect();
        assert_eq!(names, ["create waste bins", "create waste settings"]);
    }

    #[test]
    fn resources_are_declared() {
        assert_eq!(
            WasteModule
                .resources()
                .iter()
                .map(|r| r.as_str())
                .collect::<Vec<_>>(),
            ["waste_bin", "collection_plan"]
        );
    }
}
