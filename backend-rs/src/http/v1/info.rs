use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::dto::info::{AppInfoResponse, GitInfoResponse, MapInfoResponse, VersionInfoResponse},
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_info))
}

#[utoipa::path(get, path = "/info", tag = "Info",
    responses((status = 200, description = "Application info", body = AppInfoResponse))
)]
pub async fn get_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AppInfoResponse>, ServiceError> {
    let app = state.info_provider.app_info().await?;

    let response = AppInfoResponse {
        version: app.version,
        version_info: VersionInfoResponse::from(&app.version_info),
        rust_version: app.rust_version,
        build_time: app.build_time.to_rfc3339(),
        git: GitInfoResponse::from(&app.git),
        map: MapInfoResponse::from(&app.map),
    };

    Ok(Json(response))
}
