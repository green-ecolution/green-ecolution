pub mod cluster;
pub mod cluster_comment;
pub mod dto;
pub mod evaluation;
pub mod plugin_ingest;
pub mod sensor;
pub mod state;
pub mod tree;

use std::sync::Arc;

use utoipa::openapi::tag::{Tag, TagBuilder};
use utoipa_axum::router::OpenApiRouter;

use state::IrrigationHttpState;

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
