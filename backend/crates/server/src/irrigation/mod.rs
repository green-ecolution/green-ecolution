//! Irrigation as a module. In this stage it only implements the core's ports;
//! trees, clusters and their services still live in the core and move into the
//! module crate in stage 4.

pub mod handlers;
pub mod settings;
pub mod targets;

use std::sync::Arc;

use domain::{authorization::ResourceKey, module::ModuleKey};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    infra::{pg_cluster::PgTreeClusterRepository, pg_tree::PgTreeRepository},
    module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime},
};
use handlers::SensorLifecycle;
use settings::{
    IrrigationDefaults, http::IrrigationHttpState, repository::PgIrrigationSettings,
    service::IrrigationSettingsService,
};

pub const KEY: &str = "irrigation";

pub mod resources {
    use domain::authorization::ResourceKey;

    pub const TREE: ResourceKey = ResourceKey::declare("tree");
    pub const TREE_CLUSTER: ResourceKey = ResourceKey::declare("tree_cluster");
    pub const WATERING_PLAN: ResourceKey = ResourceKey::declare("watering_plan");
    pub const ALL: [ResourceKey; 3] = [TREE, TREE_CLUSTER, WATERING_PLAN];
}

pub fn key() -> ModuleKey {
    ModuleKey::new(KEY).expect("static key is valid")
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
        let defaults =
            IrrigationDefaults::from_config(&ctx.config).map_err(|e| ModuleBuildError {
                module: key(),
                reason: e.to_string(),
            })?;
        let repo = Arc::new(PgIrrigationSettings::new(
            ctx.pool.clone(),
            defaults,
            ctx.organizations.clone(),
        ));
        let state = Arc::new(IrrigationHttpState {
            settings: Arc::new(IrrigationSettingsService::new(repo)),
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
            clusters,
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

    #[test]
    fn key_matches_the_constant() {
        assert_eq!(key().as_str(), KEY);
    }
}
