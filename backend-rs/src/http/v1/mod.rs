use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::http::AppState;

pub mod cluster;
pub mod dto;
pub mod error;
pub mod evaluation;
pub mod info;
pub mod pagination;
pub mod plugin;
pub mod region;
pub mod sensor;
pub mod tree;
pub mod user;
pub mod vehicle;
pub mod watering_plan;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .merge(region::routes())
        .merge(cluster::routes())
        .merge(evaluation::routes())
        .merge(info::routes())
        .merge(sensor::routes())
        .merge(tree::routes())
        .merge(user::routes())
        .merge(vehicle::routes())
        .merge(watering_plan::routes())
        .merge(plugin::routes())
}
