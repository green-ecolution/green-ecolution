//! Irrigation as a module. In this stage it only implements the core's ports;
//! trees, clusters and their services still live in the core and move into the
//! module crate in stage 4.

use std::sync::Arc;

use domain::{authorization::ResourceKey, module::ModuleKey};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    http::AppState,
    module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime},
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

fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
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

    fn build(&self, _ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
        Ok(ModuleRuntime::new(ModuleRoutes::empty()))
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
