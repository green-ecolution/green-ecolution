use std::sync::Arc;

use crate::http::extractors::Json;
use axum::{
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
            OrganizationCreateRequest, OrganizationDetailResponse, OrganizationResponse,
            OrganizationUpdateRequest,
        },
        v1::scope::resolve_target_org,
    },
    service::ServiceError,
};
use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    organization::{OrganizationDraft, OrganizationName},
    shared::address::Address,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_organizations, create_organization))
        .routes(routes!(
            get_organization,
            update_organization,
            delete_organization
        ))
}

#[utoipa::path(get, path = "/organizations", tag = "Organizations",
    operation_id = "listOrganizations",
    summary = "List visible organizations",
    description = "Returns the caller's organization subtree as a flat list; clients rebuild the tree via parent_id. Requires organization:read.",
    responses(
        (status = 200, description = "The organizations visible to the caller", body = Vec<OrganizationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Acting user has no organization and none was given"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
) -> Result<Json<Vec<OrganizationResponse>>, ServiceError> {
    let read = Permission::new(Resource::Organization, Action::Read);
    let scope = resolve_target_org(&state, user.id, None).await?;
    state
        .authorization_service
        .require(user.id, read, scope)
        .await?;
    // The gate only proves the caller may read *somewhere*; a tenant must not
    // see the addresses and contact people of sibling tenants.
    let visible = state
        .authorization_service
        .visible_orgs_for(user.id, read)
        .await?;
    let views = state.organization_service.list().await?;
    Ok(Json(
        views
            .iter()
            .filter(|view| visible.allows(view.id))
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "/organizations/{org_id}", tag = "Organizations",
    operation_id = "getOrganization",
    summary = "Get a single organization",
    description = "Includes the resolved contact person. Requires organization:read.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    responses(
        (status = 200, description = "The organization", body = OrganizationDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationDetailResponse>, ServiceError> {
    let id = Id::new(org_id);
    state
        .authorization_service
        .require(
            user.id,
            Permission::new(Resource::Organization, Action::Read),
            id,
        )
        .await?;
    let view = state.organization_service.detail(id).await?;
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

#[utoipa::path(put, path = "/organizations/{org_id}", tag = "Organizations",
    operation_id = "updateOrganization",
    summary = "Update an organization",
    description = "Replaces name, address and contact person. An omitted field clears the stored value. The contact person must be a member of this organization.",
    params(("org_id" = Uuid, Path, description = "Organization id")),
    request_body = OrganizationUpdateRequest,
    responses(
        (status = 200, description = "Updated", body = OrganizationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Name conflict or root organization"),
        (status = 422, description = "Contact person is not a member of this organization"),
        (status = 500, description = "Internal server error"),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(org_id): Path<Uuid>,
    Json(req): Json<OrganizationUpdateRequest>,
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
    let address = req.address.map(Address::try_from).transpose()?;
    let view = state
        .organization_service
        .update(
            id,
            OrganizationName::new(req.name)?,
            address,
            req.contact_person_id,
        )
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
