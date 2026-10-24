use std::sync::Arc;

use axum::middleware;
use utoipa_axum::router::OpenApiRouter;

use crate::http::{AppState, auth::AuthLayer};

pub mod cluster;
pub mod comment;
pub mod dto;
pub mod error;
pub mod evaluation;
pub mod gpx;
pub mod info;
pub mod module;
pub mod organization;
pub mod pagination;
pub mod plan;
pub mod plugin;
pub mod region;
pub mod role;
pub mod routing;
pub mod scope;
pub mod sensor;
pub mod settings;
pub mod tree;
pub mod user;
pub mod vehicle;

pub fn public_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(info::public_routes())
        .merge(plugin::ingest_routes())
}

pub fn protected_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(region::routes())
        .merge(cluster::routes())
        .merge(comment::routes())
        .merge(evaluation::routes())
        .merge(info::protected_routes())
        .merge(module::routes())
        .merge(organization::routes())
        .merge(plan::routes())
        .merge(plugin::routes())
        .merge(role::routes())
        .merge(routing::routes())
        .merge(sensor::routes())
        .merge(settings::routes())
        .merge(tree::routes())
        .merge(user::protected_routes())
        .merge(vehicle::routes())
}

/// The core's own routes relative to `/api/v1`, for checking module routes
/// against them before anything is mounted.
pub fn core_openapi() -> utoipa::openapi::OpenApi {
    public_router().merge(protected_router()).into_openapi()
}

pub fn router(
    auth_layer: AuthLayer,
    module_routes: Vec<crate::module::ModuleRoutes>,
) -> OpenApiRouter<Arc<AppState>> {
    let (public, protected) = module_routes.into_iter().fold(
        (public_router(), protected_router()),
        |(public, protected), routes| {
            (
                public.merge(routes.public),
                protected.merge(routes.protected),
            )
        },
    );
    public.merge(protected.layer(middleware::from_fn_with_state(
        auth_layer,
        crate::http::auth::auth_middleware,
    )))
}
