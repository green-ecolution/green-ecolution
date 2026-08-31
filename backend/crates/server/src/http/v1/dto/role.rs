use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::role::RoleView;

use crate::service::{Malformed, ServiceError};

/// A role (named permission set), either a global template or bound to an organization.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "01980000-0000-7000-8000-0000000000b1",
    "organization_id": "01980000-0000-7000-8000-000000000001",
    "name": "Administrator",
    "description": "Voller Zugriff auf alle Ressourcen",
    "permissions": ["tree:read", "tree:create"],
    "is_template": false,
    "template_key": "administrator",
    "created_at": "2024-06-15T10:30:00+00:00"
}))]
pub struct RoleResponse {
    /// Unique role identifier (UUID v7).
    pub id: String,
    /// Owning organization; null for templates.
    pub organization_id: Option<String>,
    /// Display name, unique per organization (or among templates).
    pub name: String,
    pub description: Option<String>,
    /// Grantable capabilities, formatted as `<resource>:<action>`.
    pub permissions: Vec<String>,
    /// Whether this role is a global template (immutable, not assignable).
    pub is_template: bool,
    /// Which delivered template this role still matches verbatim, e.g.
    /// `tree_care`. Null once someone edits the name or description, or for a
    /// hand-made role. A client may translate `name` and `description` while
    /// this is set, and must show them verbatim once it is null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_key: Option<String>,
    /// Creation time derived from the UUID v7 id (null for seeded ids without a real timestamp).
    pub created_at: Option<String>,
}

impl From<&RoleView> for RoleResponse {
    fn from(value: &RoleView) -> Self {
        Self {
            id: value.id.to_string(),
            organization_id: value.organization_id.map(|o| o.to_string()),
            name: value.name.as_str().to_string(),
            description: value.description.as_ref().map(|d| d.as_str().to_string()),
            permissions: value.permissions.iter().map(|p| p.to_string()).collect(),
            is_template: value.is_template,
            template_key: value.template_key.as_ref().map(|k| k.as_str().to_string()),
            created_at: value.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Request body for creating a role, either from scratch or as a copy of an existing role/template.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "name": "Gießtrupp", "permissions": ["tree:read", "tree:update"] }))]
pub struct RoleCreateRequest {
    /// Required unless `copy_from_role_id` is set (in which case it renames the copy).
    pub name: Option<String>,
    pub description: Option<String>,
    /// Required unless `copy_from_role_id` is set.
    pub permissions: Option<Vec<String>>,
    /// Source role or template to copy the permission set from.
    pub copy_from_role_id: Option<Uuid>,
}

/// Request body for replacing a role's name, description and permission set.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "name": "Gießtrupp Nord", "description": "Bewässerung Nordbezirk", "permissions": ["tree:read", "sensor:read"] }))]
pub struct RoleUpdateRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

pub fn parse_permissions(
    raw: &[String],
) -> Result<BTreeSet<domain::authorization::Permission>, ServiceError> {
    raw.iter()
        .map(|s| s.parse::<domain::authorization::Permission>())
        .collect::<Result<_, _>>()
        .map_err(|e| ServiceError::Malformed {
            kind: Malformed::Permission,
            detail: e.to_string(),
        })
}
