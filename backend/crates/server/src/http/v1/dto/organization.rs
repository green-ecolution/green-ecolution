use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::organization::OrganizationView;

/// A node in the organization tree.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "01980000-0000-7000-8000-000000000001",
    "parent_id": null,
    "name": "Green Ecolution",
    "created_at": "2024-06-15T10:30:00+00:00"
}))]
pub struct OrganizationResponse {
    /// Unique organization identifier (UUID v7).
    pub id: String,
    /// Parent organization; null only for the single root.
    pub parent_id: Option<String>,
    /// Display name, unique among siblings.
    pub name: String,
    /// Creation time derived from the UUID v7 id (null for seeded ids without a real timestamp).
    pub created_at: Option<String>,
}

impl From<&OrganizationView> for OrganizationResponse {
    fn from(value: &OrganizationView) -> Self {
        Self {
            id: value.id.to_string(),
            parent_id: value.parent_id.map(|p| p.to_string()),
            name: value.name.as_str().to_string(),
            created_at: value.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Request body for creating an organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "name": "TBZ Flensburg", "parent_id": "01980000-0000-7000-8000-000000000001" }))]
pub struct OrganizationCreateRequest {
    pub name: String,
    pub parent_id: Uuid,
}

/// Request body for renaming an organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "name": "TBZ Flensburg" }))]
pub struct OrganizationRenameRequest {
    pub name: String,
}
