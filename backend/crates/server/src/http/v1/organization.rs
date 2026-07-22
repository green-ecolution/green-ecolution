use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        v1::dto::organization::{
            OrganizationCreateRequest, OrganizationRenameRequest, OrganizationResponse,
        },
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    organization::{OrganizationDraft, OrganizationName},
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_organizations, create_organization))
        .routes(routes!(
            get_organization,
            rename_organization,
            delete_organization
        ))
}

#[utoipa::path(get, path = "/organizations", tag = "Organizations",
    operation_id = "listOrganizations",
    summary = "List all organizations",
    description = "Returns the full organization tree as a flat list; clients rebuild the tree via parent_id.",
    responses(
        (status = 200, description = "All organizations", body = Vec<OrganizationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
) -> Result<Json<Vec<OrganizationResponse>>, ServiceError> {
    let views = state.organization_service.list().await?;
    Ok(Json(views.iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/organizations/{org_id}", tag = "Organizations",
    operation_id = "getOrganization",
    summary = "Get a single organization",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The organization", body = OrganizationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    _user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationResponse>, ServiceError> {
    let view = state.organization_service.by_id(Id::new(org_id)).await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(post, path = "/organizations", tag = "Organizations",
    operation_id = "createOrganization",
    summary = "Create an organization",
    description = "Creates a sub-organization under parent_id and instantiates copies of all role templates. Requires organization:create in the parent organization.",
    request_body = OrganizationCreateRequest,
    responses(
        (status = 201, description = "Created", body = OrganizationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Sibling name conflict"),
        (status = 422, description = "Parent does not exist"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Json(req): Json<OrganizationCreateRequest>,
) -> Result<(StatusCode, Json<OrganizationResponse>), ServiceError> {
    let parent = Id::new(req.parent_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Organization, Action::Create),
            parent,
        )
        .await?;
    let draft = OrganizationDraft {
        name: OrganizationName::new(req.name)?,
        parent_id: parent,
    };
    let view = state.organization_service.create(draft).await?;
    Ok((StatusCode::CREATED, Json((&view).into())))
}

#[utoipa::path(patch, path = "/organizations/{org_id}", tag = "Organizations",
    operation_id = "renameOrganization",
    summary = "Rename an organization",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationRenameRequest,
    responses(
        (status = 200, description = "Renamed", body = OrganizationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Name conflict or root"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn rename_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<OrganizationRenameRequest>,
) -> Result<Json<OrganizationResponse>, ServiceError> {
    let id = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Organization, Action::Update),
            id,
        )
        .await?;
    let view = state
        .organization_service
        .rename(id, OrganizationName::new(req.name)?)
        .await?;
    Ok(Json((&view).into()))
}

#[utoipa::path(delete, path = "/organizations/{org_id}", tag = "Organizations",
    operation_id = "deleteOrganization",
    summary = "Delete an empty organization",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Organization still has children or users"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<StatusCode, ServiceError> {
    let id = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Organization, Action::Delete),
            id,
        )
        .await?;
    state.organization_service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
