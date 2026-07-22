use serde::Serialize;

use domain::role::RoleView;

/// A role (named permission set), either a global template or bound to an organization.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "01980000-0000-7000-8000-0000000000b1",
    "organization_id": "01980000-0000-7000-8000-000000000001",
    "name": "Administrator",
    "description": "Voller Zugriff auf alle Ressourcen",
    "permissions": ["tree:read", "tree:create"],
    "is_template": false,
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
            created_at: value.created_at.map(|t| t.to_rfc3339()),
        }
    }
}
