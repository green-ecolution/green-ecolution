use std::sync::Arc;

use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    authorization::{Action, Caller, Permission},
    shared::pagination::Pagination,
};
use server::{
    http::{
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path, Query},
        v1::{
            dto::{ListResponse, TransferRequest},
            error::ErrorBody,
            scope,
        },
    },
    service::ServiceError,
};
use waste_domain::waste_bin::WasteBinUpdate;

use super::{
    dto::{WasteBinCreateRequest, WasteBinListParams, WasteBinResponse, WasteBinUpdateRequest},
    state::WasteHttpState,
};
use crate::{
    resources::WASTE_BIN,
    service::waste_bin_service::{FillStateFilter, WasteBinListQuery, WasteBinState},
};

pub fn routes() -> OpenApiRouter<Arc<WasteHttpState>> {
    OpenApiRouter::new()
        .routes(routes!(list_waste_bins, create_waste_bin))
        .routes(routes!(get_waste_bin, update_waste_bin, delete_waste_bin))
        .routes(routes!(transfer_waste_bin))
}

/// Loads a bin the caller may read; an invisible one, or one in an
/// organization without the module, reads as 404.
async fn visible_bin(
    state: &WasteHttpState,
    user: Caller,
    id: Uuid,
) -> Result<WasteBinState, ServiceError> {
    let bin = state.bins.get(Id::new(id)).await?;
    let visible = state
        .access
        .visible_orgs(user, Permission::new(WASTE_BIN, Action::Read))
        .await?;
    if !visible.allows(bin.view.organization_id) {
        return Err(RepositoryError::NotFound.into());
    }
    Ok(bin)
}

#[utoipa::path(get, path = "/waste-bins", tag = "WasteBins",
    operation_id = "listWasteBins",
    summary = "List waste bins",
    description = "Returns a paginated list of the waste bins visible to the caller in organizations where the waste module is active, sorted by number. \
                   `number` filters by a case-insensitive substring. `fill_state` is judged by the full threshold and sensor offline period \
                   of `organization_id`, or of the caller's own organization when none is given.",
    params(WasteBinListParams),
    responses(
        (status = 200, description = "Paginated list of waste bins", body = ListResponse<WasteBinResponse>),
        (status = 400, description = "Invalid query parameter", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn list_waste_bins(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Query(params): Query<WasteBinListParams>,
) -> Result<Json<ListResponse<WasteBinResponse>>, ServiceError> {
    let pagination = Pagination::new(params.page, params.per_page);
    let visible = state
        .access
        .visible_orgs(user.caller(), Permission::new(WASTE_BIN, Action::Read))
        .await?;
    let fill_state = match params.fill_state {
        Some(fill) => Some(FillStateFilter {
            state: fill.into(),
            reference_org: scope::resolve_target_org(
                &state.users,
                &state.authorization,
                user.caller(),
                params.organization_id,
            )
            .await?,
        }),
        None => None,
    };
    let query = WasteBinListQuery {
        number: params.number,
        organization_id: params.organization_id.map(Id::new),
        fill_state,
    };
    let result = state.bins.list(query, pagination, &visible).await?;
    Ok(Json(
        ListResponse::from_page(result.page, &pagination)
            .with_total_unfiltered(result.total_unfiltered),
    ))
}

#[utoipa::path(get, path = "/waste-bins/{waste_bin_id}", tag = "WasteBins",
    operation_id = "getWasteBin",
    summary = "Get a waste bin by ID",
    params(("waste_bin_id" = Uuid, Path, description = "Waste bin ID")),
    responses(
        (status = 200, description = "Waste bin found", body = WasteBinResponse),
        (status = 404, description = "Waste bin not found or not visible", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(waste_bin.id = %id))]
pub async fn get_waste_bin(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<WasteBinResponse>, ServiceError> {
    let bin = visible_bin(&state, user.caller(), id).await?;
    Ok(Json(WasteBinResponse::from(&bin)))
}

#[utoipa::path(post, path = "/waste-bins", tag = "WasteBins",
    operation_id = "createWasteBin",
    summary = "Create a waste bin",
    description = "Creates a waste bin in `organization_id`, or in the caller's own organization when none is given.",
    request_body = WasteBinCreateRequest,
    responses(
        (status = 201, description = "Waste bin created", body = WasteBinResponse),
        (status = 400, description = "Invalid input, e.g. a depth outside 1-1000 cm (code `request.validation_failed`)", body = ErrorBody),
        (status = 403, description = "Missing waste_bin:create, or the waste module is off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 409, description = "The number is already used in the organization (code `waste_bin.number_taken`)", body = ErrorBody),
        (status = 422, description = "Malformed request body", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all)]
pub async fn create_waste_bin(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Json(req): Json<WasteBinCreateRequest>,
) -> Result<(StatusCode, Json<WasteBinResponse>), ServiceError> {
    let org = scope::resolve_target_org(
        &state.users,
        &state.authorization,
        user.caller(),
        req.organization_id,
    )
    .await?;
    state
        .access
        .require_write(
            user.caller(),
            Permission::new(WASTE_BIN, Action::Create),
            org,
        )
        .await?;
    let bin = state.bins.create(req.into_draft(org)?).await?;
    Ok((StatusCode::CREATED, Json(WasteBinResponse::from(&bin))))
}

#[utoipa::path(put, path = "/waste-bins/{waste_bin_id}", tag = "WasteBins",
    operation_id = "updateWasteBin",
    summary = "Update a waste bin",
    description = "Replaces number, position, volume and depth of a waste bin. Fill level and organization stay unchanged.",
    params(("waste_bin_id" = Uuid, Path, description = "Waste bin ID")),
    request_body = WasteBinUpdateRequest,
    responses(
        (status = 200, description = "Waste bin updated", body = WasteBinResponse),
        (status = 400, description = "Invalid input (code `request.validation_failed`)", body = ErrorBody),
        (status = 403, description = "Missing waste_bin:update, or the waste module is off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Waste bin not found or not visible", body = ErrorBody),
        (status = 409, description = "The number is already used in the organization (code `waste_bin.number_taken`)", body = ErrorBody),
        (status = 422, description = "Malformed request body", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(waste_bin.id = %id))]
pub async fn update_waste_bin(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
    Json(req): Json<WasteBinUpdateRequest>,
) -> Result<Json<WasteBinResponse>, ServiceError> {
    let current = visible_bin(&state, user.caller(), id).await?;
    state
        .access
        .require_write(
            user.caller(),
            Permission::new(WASTE_BIN, Action::Update),
            current.view.organization_id,
        )
        .await?;
    let bin = state
        .bins
        .update(Id::new(id), WasteBinUpdate::try_from(req)?)
        .await?;
    Ok(Json(WasteBinResponse::from(&bin)))
}

#[utoipa::path(delete, path = "/waste-bins/{waste_bin_id}", tag = "WasteBins",
    operation_id = "deleteWasteBin",
    summary = "Delete a waste bin",
    description = "Deletes a waste bin, releases its sensor, removes it from every collection plan and deletes its comments.",
    params(("waste_bin_id" = Uuid, Path, description = "Waste bin ID")),
    responses(
        (status = 204, description = "Waste bin deleted"),
        (status = 403, description = "Missing waste_bin:delete, or the waste module is off in the organization (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Waste bin not found or not visible", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(waste_bin.id = %id))]
pub async fn delete_waste_bin(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ServiceError> {
    let current = visible_bin(&state, user.caller(), id).await?;
    state
        .access
        .require_write(
            user.caller(),
            Permission::new(WASTE_BIN, Action::Delete),
            current.view.organization_id,
        )
        .await?;
    state.bins.delete(Id::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/waste-bins/{waste_bin_id}/organization", tag = "WasteBins",
    operation_id = "transferWasteBin",
    summary = "Transfer a waste bin to another organization",
    description = "Moves a waste bin, and an attached sensor with it, to another organization. \
                   Requires `waste_bin:update` and an active waste module in both the source and the target organization.",
    params(("waste_bin_id" = Uuid, Path, description = "Waste bin ID")),
    request_body = TransferRequest,
    responses(
        (status = 204, description = "Waste bin transferred"),
        (status = 403, description = "Missing waste_bin:update in source or target organization, or the waste module is off there (code `module.disabled`)", body = ErrorBody),
        (status = 404, description = "Waste bin or organization not found", body = ErrorBody),
        (status = 409, description = "The number is already used in the target organization (code `waste_bin.number_taken`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(waste_bin.id = %id))]
pub async fn transfer_waste_bin(
    State(state): State<Arc<WasteHttpState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ServiceError> {
    let current = visible_bin(&state, user.caller(), id).await?;
    let update = Permission::new(WASTE_BIN, Action::Update);
    state
        .access
        .require_write(user.caller(), update, current.view.organization_id)
        .await?;
    state
        .access
        .require_write(user.caller(), update, Id::new(req.organization_id))
        .await?;
    state
        .bins
        .transfer(Id::new(id), Id::new(req.organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
