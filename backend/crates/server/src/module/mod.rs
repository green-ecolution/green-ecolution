//! The host side of feature modules: what a module implements, what it is
//! given at startup and what it hands back.

use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use domain::{
    authorization::ResourceKey,
    module::{ModuleKey, ModuleStatistics},
    organization::OrganizationReader,
    planning::PlanningProvider,
    region::RegionReader,
    sensor::SensorReadingReader,
    settings::SettingsResolver,
    target::TargetCatalog,
};
use sqlx::PgPool;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    configuration::Settings,
    http::extractors::PluginAuthenticator,
    service::{
        authorization::AuthorizationService,
        comment_service::CommentService,
        event_bus::{EventBus, EventHandler},
        module_access::ModuleAccess,
        operation_plan_service::OperationPlanService,
        sensor_parsers::SensorPayloadParser,
        sensor_service::SensorService,
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
    /// An event handler must hold this as `Weak` (or not at all): the service owns the
    /// event bus, so a strong reference keeps the whole graph alive.
    pub sensors: Arc<SensorService>,
    /// Same rule as `sensors`: hold as `Weak` in event handlers.
    pub plans: Arc<OperationPlanService>,
    /// For handlers that read the readings an event points at.
    pub sensor_readings: Arc<dyn SensorReadingReader>,
    pub regions: Arc<dyn RegionReader>,
    pub comments: Arc<CommentService>,
    pub plugin_auth: PluginAuthenticator,
}

impl ModuleContext {
    /// The module's own `modules.<key>` section, minus `default_enabled`.
    pub fn module_config<T: serde::de::DeserializeOwned>(
        &self,
        key: &ModuleKey,
    ) -> Result<T, ModuleBuildError> {
        self.config
            .module_config(key.as_str())
            .map_err(|e| ModuleBuildError {
                module: key.clone(),
                reason: format!("invalid modules.{key} section: {e}"),
            })
    }
}

/// Routes relative to `/api/v1`. `protected` runs behind the same
/// authentication layer as the core's protected routes. Both carry their
/// state already (`.with_state(..)`), so module handlers cannot extract the
/// core's `AppState`.
pub struct ModuleRoutes {
    pub public: OpenApiRouter,
    pub protected: OpenApiRouter,
}

impl ModuleRoutes {
    pub fn empty() -> Self {
        Self {
            public: OpenApiRouter::new(),
            protected: OpenApiRouter::new(),
        }
    }
}

/// What the HTTP layer mounts for one module. The routers carry the module's
/// paths but not its tag descriptions, which only `Module::openapi` has.
pub struct ModuleMount {
    pub routes: ModuleRoutes,
    pub openapi: utoipa::openapi::OpenApi,
}

pub struct ModuleRuntime {
    pub routes: ModuleRoutes,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
    /// Spawned once at startup, like the core's sweeps.
    pub background_tasks: Vec<BackgroundTask>,
    pub targets: Option<Arc<dyn TargetCatalog>>,
    pub planning: Option<Arc<dyn PlanningProvider>>,
    pub statistics: Option<Arc<dyn ModuleStatistics>>,
    /// Decoders for the module's own sensor models, used by the MQTT ingestor.
    pub sensor_parsers: Vec<Arc<dyn SensorPayloadParser>>,
}

impl ModuleRuntime {
    pub fn new(routes: ModuleRoutes) -> Self {
        Self {
            routes,
            event_handlers: Vec::new(),
            background_tasks: Vec::new(),
            targets: None,
            planning: None,
            statistics: None,
            sensor_parsers: Vec::new(),
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
    #[error("route {path} of module {module} collides with {existing}, served by {owner}")]
    RouteCollision {
        path: String,
        module: ModuleKey,
        existing: String,
        owner: String,
    },
}

/// Fails before anything is mounted. axum would only panic on a few exact
/// overlaps; this names the module that caused the clash.
pub fn check_modules(
    core: &utoipa::openapi::OpenApi,
    modules: &[Box<dyn Module>],
) -> Result<(), ModuleSetupError> {
    let mut owners: BTreeMap<String, (String, String)> = core
        .paths
        .paths
        .keys()
        .map(|path| (route_shape(path), (path.clone(), "the core".to_string())))
        .collect();
    let mut keys = BTreeSet::new();
    for module in modules {
        let key = module.key();
        if !keys.insert(key.clone()) {
            return Err(ModuleSetupError::DuplicateKey(key));
        }
        for path in module.openapi().paths.paths.keys() {
            let shape = route_shape(path);
            if let Some((existing, owner)) = owners.get(&shape) {
                return Err(ModuleSetupError::RouteCollision {
                    path: path.clone(),
                    module: key,
                    existing: existing.clone(),
                    owner: owner.clone(),
                });
            }
            owners.insert(shape, (path.clone(), format!("module {key}")));
        }
    }
    Ok(())
}

/// The router treats `/x/{id}` and `/x/{other}` as the same route and panics
/// when both are mounted, so parameter names must not decide a collision.
fn route_shape(path: &str) -> String {
    let mut shape = String::with_capacity(path.len());
    let mut chars = path.chars();
    while let Some(c) = chars.next() {
        if c != '{' {
            shape.push(c);
            continue;
        }
        let name: String = chars.by_ref().take_while(|&c| c != '}').collect();
        shape.push_str(if name.starts_with('*') { "{*}" } else { "{}" });
    }
    shape
}

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa_axum::routes;

    #[utoipa::path(get, path = "/organizations")]
    async fn shadows_core() {}

    #[utoipa::path(get, path = "/organizations/{id}/modules")]
    async fn shadows_core_under_another_name() {}

    #[utoipa::path(get, path = "/fake-things")]
    async fn fake_things() {}

    #[utoipa::path(get, path = "/fake-things/{id}")]
    async fn fake_thing() {}

    #[utoipa::path(get, path = "/fake-things/{thing_id}/parts")]
    async fn fake_thing_parts() {}

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

    fn renamed_shadow() -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(shadows_core_under_another_name))
            .into_openapi()
    }

    fn thing() -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(fake_thing))
            .into_openapi()
    }

    fn thing_parts() -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(fake_thing_parts))
            .into_openapi()
    }

    fn core() -> utoipa::openapi::OpenApi {
        crate::http::v1::core_openapi()
    }

    #[test]
    fn a_core_route_under_another_parameter_name_is_rejected() {
        let modules: Vec<Box<dyn Module>> = vec![Box::new(Fake {
            key: "alpha",
            api: renamed_shadow,
        })];
        let err = check_modules(&core(), &modules).unwrap_err();
        assert!(matches!(
            err,
            ModuleSetupError::RouteCollision { ref path, ref existing, ref owner, .. }
                if path == "/organizations/{id}/modules"
                    && existing == "/organizations/{org_id}/modules"
                    && owner == "the core"
        ));
    }

    #[test]
    fn differently_named_parameters_on_distinct_routes_pass() {
        let modules: Vec<Box<dyn Module>> = vec![
            Box::new(Fake {
                key: "alpha",
                api: thing,
            }),
            Box::new(Fake {
                key: "beta",
                api: thing_parts,
            }),
        ];
        assert!(check_modules(&core(), &modules).is_ok());
    }

    #[test]
    fn route_shape_ignores_parameter_names() {
        assert_eq!(route_shape("/a/{id}/b/{*rest}"), "/a/{}/b/{*}");
        assert_eq!(route_shape("/a/b"), "/a/b");
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
