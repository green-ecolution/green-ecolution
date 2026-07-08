use std::sync::Arc;

use axum::middleware;
use utoipa_axum::router::OpenApiRouter;

use crate::http::{AppState, auth::AuthLayer};

pub mod cluster;
pub mod dto;
pub mod error;
pub mod evaluation;
pub mod gpx;
pub mod info;
pub mod pagination;
pub mod plugin;
pub mod region;
pub mod sensor;
pub mod tree;
pub mod user;
pub mod vehicle;
pub mod watering_plan;

pub fn public_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(info::routes())
        .merge(plugin::routes())
}

pub fn protected_router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(region::routes())
        .merge(cluster::routes())
        .merge(evaluation::routes())
        .merge(sensor::routes())
        .merge(tree::routes())
        .merge(user::protected_routes())
        .merge(vehicle::routes())
        .merge(watering_plan::routes())
}

pub fn router(auth_layer: AuthLayer) -> OpenApiRouter<Arc<AppState>> {
    public_router().merge(protected_router().layer(middleware::from_fn_with_state(
        auth_layer,
        crate::http::auth::auth_middleware,
    )))
}
