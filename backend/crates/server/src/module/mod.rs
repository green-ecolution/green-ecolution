//! The host side of feature modules: what a module implements, what it is
//! given at startup and what it hands back.

use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use domain::{
    authorization::ResourceKey, module::ModuleKey, organization::OrganizationReader,
    settings::SettingsResolver, target::TargetCatalog,
};
use sqlx::PgPool;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    configuration::Settings,
    http::AppState,
    service::{
        authorization::AuthorizationService,
        event_bus::{EventBus, EventHandler},
        module_access::ModuleAccess,
        user_service::UserService,
    },
};

pub type BackgroundTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait Module: Send + Sync {
    fn key(&self) -> ModuleKey;
    /// Activation where neither the YAML nor any organization says otherwise.
    fn default_enabled(&self) -> bool;
    /// Built from the same routes as the runtime router but without state, so
    /// `dump-openapi` and the startup checks need no database.
    fn openapi(&self) -> utoipa::openapi::OpenApi;
    /// Resources this module adds to the permission catalog.
    fn resources(&self) -> &'static [ResourceKey] {
        &[]
    }
    fn migrations(&self) -> Vec<sqlx::migrate::Migration> {
        Vec::new()
    }
    fn build(&self, ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError>;
}

#[derive(Clone)]
pub struct ModuleContext {
    pub pool: PgPool,
    pub event_bus: Arc<dyn EventBus>,
    pub authorization: Arc<AuthorizationService>,
    pub organizations: Arc<dyn OrganizationReader>,
    pub settings: Arc<dyn SettingsResolver>,
    pub access: ModuleAccess,
    pub users: Arc<UserService>,
    pub config: Arc<Settings>,
}

/// Routes relative to `/api/v1`. `protected` runs behind the same
/// authentication layer as the core's protected routes.
pub struct ModuleRoutes {
    pub public: OpenApiRouter<Arc<AppState>>,
    pub protected: OpenApiRouter<Arc<AppState>>,
}

impl ModuleRoutes {
    pub fn empty() -> Self {
        Self {
            public: OpenApiRouter::new(),
            protected: OpenApiRouter::new(),
        }
    }
}

pub struct ModuleRuntime {
    pub routes: ModuleRoutes,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    /// Spawned once at startup, like the core's sweeps.
    pub background_tasks: Vec<BackgroundTask>,
    pub targets: Option<Arc<dyn TargetCatalog>>,
}

impl ModuleRuntime {
    pub fn new(routes: ModuleRoutes) -> Self {
        Self {
            routes,
            event_handlers: Vec::new(),
            background_tasks: Vec::new(),
            targets: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("module {module} failed to build: {reason}")]
pub struct ModuleBuildError {
    pub module: ModuleKey,
    pub reason: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleSetupError {
    #[error("module {0} is registered twice")]
    DuplicateKey(ModuleKey),
    #[error("route {path} of module {module} is already served by {owner}")]
    RouteCollision {
        path: String,
        module: ModuleKey,
        owner: String,
    },
}

/// Fails before anything is mounted. axum would only panic on a few exact
/// overlaps; this names the module that caused the clash.
pub fn check_modules(
    core: &utoipa::openapi::OpenApi,
    modules: &[Box<dyn Module>],
) -> Result<(), ModuleSetupError> {
    let mut owners: BTreeMap<String, String> = core
        .paths
        .paths
        .keys()
        .map(|path| (path.clone(), "the core".to_string()))
        .collect();
    let mut keys = BTreeSet::new();
    for module in modules {
        let key = module.key();
        if !keys.insert(key.clone()) {
            return Err(ModuleSetupError::DuplicateKey(key));
        }
        for path in module.openapi().paths.paths.keys() {
            if let Some(owner) = owners.get(path) {
                return Err(ModuleSetupError::RouteCollision {
                    path: path.clone(),
                    module: key,
                    owner: owner.clone(),
                });
            }
            owners.insert(path.clone(), format!("module {key}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa_axum::routes;

    #[utoipa::path(get, path = "/organizations")]
    async fn shadows_core() {}

    #[utoipa::path(get, path = "/fake-things")]
    async fn fake_things() {}

    struct Fake {
        key: &'static str,
        api: fn() -> utoipa::openapi::OpenApi,
    }

    impl Module for Fake {
        fn key(&self) -> ModuleKey {
            ModuleKey::new(self.key).unwrap()
        }
        fn default_enabled(&self) -> bool {
            false
        }
        fn openapi(&self) -> utoipa::openapi::OpenApi {
            (self.api)()
        }
        fn build(&self, _ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
            unreachable!("startup checks never build a module")
        }
    }

    fn things() -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(fake_things))
            .into_openapi()
    }

    fn shadow() -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(shadows_core))
            .into_openapi()
    }

    fn core() -> utoipa::openapi::OpenApi {
        crate::http::v1::core_openapi()
    }

    #[test]
    fn distinct_modules_pass() {
        let modules: Vec<Box<dyn Module>> = vec![Box::new(Fake {
            key: "alpha",
            api: things,
        })];
        assert!(check_modules(&core(), &modules).is_ok());
    }

    #[test]
    fn a_route_the_core_serves_is_rejected() {
        let modules: Vec<Box<dyn Module>> = vec![Box::new(Fake {
            key: "alpha",
            api: shadow,
        })];
        let err = check_modules(&core(), &modules).unwrap_err();
        assert!(matches!(
            err,
            ModuleSetupError::RouteCollision { ref path, ref owner, .. }
                if path == "/organizations" && owner == "the core"
        ));
    }

    #[test]
    fn two_modules_on_the_same_route_are_rejected() {
        let modules: Vec<Box<dyn Module>> = vec![
            Box::new(Fake {
                key: "alpha",
                api: things,
            }),
            Box::new(Fake {
                key: "beta",
                api: things,
            }),
        ];
        let err = check_modules(&core(), &modules).unwrap_err();
        assert!(matches!(
            err,
            ModuleSetupError::RouteCollision { ref module, ref owner, .. }
                if module.as_str() == "beta" && owner == "module alpha"
        ));
    }

    #[test]
    fn a_key_registered_twice_is_rejected() {
        let modules: Vec<Box<dyn Module>> = vec![
            Box::new(Fake {
                key: "alpha",
                api: things,
            }),
            Box::new(Fake {
                key: "alpha",
                api: shadow,
            }),
        ];
        assert!(matches!(
            check_modules(&core(), &modules),
            Err(ModuleSetupError::DuplicateKey(_))
        ));
    }
}
