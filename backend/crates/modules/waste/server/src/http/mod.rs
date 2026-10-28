pub mod dto;
pub mod state;
pub mod waste_bin;

use std::sync::Arc;

use utoipa::openapi::tag::{Tag, TagBuilder};
use utoipa_axum::router::OpenApiRouter;

use state::WasteHttpState;

pub fn protected_routes() -> OpenApiRouter<Arc<WasteHttpState>> {
    OpenApiRouter::new().merge(waste_bin::routes())
}

pub fn tags() -> Vec<Tag> {
    vec![
        TagBuilder::new()
            .name("WasteBins")
            .description(Some(
                "Manage waste containers, their location and size. The fill level comes from an attached distance sensor and is classified per organization as empty, normal, full or unknown.",
            ))
            .build(),
    ]
}
