pub mod cluster;
pub mod cluster_comment;
pub mod dto;
pub mod evaluation;
pub mod plugin_ingest;
pub mod sensor;
pub mod state;
pub mod tree;

use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    authorization::{Caller, Permission},
};
use server::service::ServiceError;
use utoipa::openapi::tag::{Tag, TagBuilder};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use state::IrrigationHttpState;

/// Detail gate: an object the caller may not read, or one in an organization
/// without the module, reads as 404, so its existence cannot be probed.
pub(crate) async fn ensure_readable(
    state: &IrrigationHttpState,
    caller: Caller,
    read: Permission,
    owner: Uuid,
) -> Result<(), ServiceError> {
    let visible = state.access.visible_orgs(caller, read).await?;
    if visible.allows(Id::new(owner)) {
        Ok(())
    } else {
        Err(RepositoryError::NotFound.into())
    }
}

/// Authenticated by a plugin key rather than a user session.
pub fn public_routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    plugin_ingest::routes()
}

pub fn protected_routes() -> OpenApiRouter<Arc<IrrigationHttpState>> {
    OpenApiRouter::new()
        .merge(cluster::routes())
        .merge(cluster_comment::routes())
        .merge(evaluation::routes())
        .merge(sensor::routes())
        .merge(tree::routes())
        .merge(super::settings::http::routes())
}

pub fn tags() -> Vec<Tag> {
    [
        (
            "Tree Clusters",
            "Manage tree clusters — logical groupings of trees that share soil conditions and watering schedules. Clusters are the primary unit for watering plan assignments.",
        ),
        (
            "Trees",
            "Manage individual trees including their species, location, planting year, and associated sensors. Trees can be assigned to clusters for grouped watering management.",
        ),
        (
            "Evaluation",
            "Aggregated statistics and evaluation data across all managed resources. Provides insights on watering plan coverage by region and vehicle usage.",
        ),
        (
            "Irrigation",
            "Per-organization irrigation defaults — water demand and just-watered duration — resolved along the organization tree.",
        ),
    ]
    .into_iter()
    .map(|(name, description)| {
        TagBuilder::new()
            .name(name)
            .description(Some(description))
            .build()
    })
    .collect()
}
