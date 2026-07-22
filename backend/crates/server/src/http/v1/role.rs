use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{AppState, auth::extractor::AuthUserExtractor},
    service::ServiceError,
};
use domain::Id;

use super::dto::role::RoleResponse;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(list_roles_for_organization))
}

#[utoipa::path(get, path = "/organizations/{org_id}/roles", tag = "Roles",
    operation_id = "listRolesForOrganization",
    summary = "List roles owned by an organization",
    description = "Returns every role instantiated for the given organization, including the copies of the five templates created alongside it.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "Roles owned by the organization", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(organization.id = %org_id))]
pub async fn list_roles_for_organization(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, ServiceError> {
    let views = state.role_service.by_organization(Id::new(org_id)).await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}
