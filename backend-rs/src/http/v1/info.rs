use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    http::{
        AppState,
        v1::dto::info::{
            AppInfoResponse, DataStatisticsResponse, GitInfoResponse, MapInfoResponse,
            ServerInfoResponse, ServiceStatusResponse, ServicesInfoResponse, VersionInfoResponse,
        },
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(get_info))
        .routes(routes!(get_map_info))
        .routes(routes!(get_server_info))
        .routes(routes!(get_services_info))
        .routes(routes!(get_statistics))
}

#[utoipa::path(get, path = "/info", tag = "Info",
    operation_id = "getInfo",
    summary = "Get application info",
    description = "Returns version, git, and map configuration metadata.",
    responses(
        (status = 200, description = "Application info", body = AppInfoResponse),
        (status = 500, description = "Internal server error"),
    )
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

#[utoipa::path(get, path = "/info/map", tag = "Info",
    operation_id = "getMapInfo",
    summary = "Get map configuration",
    description = "Returns the map center coordinates and bounding box.",
    responses(
        (status = 200, description = "Map configuration", body = MapInfoResponse),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_map_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MapInfoResponse>, ServiceError> {
    let map = state.info_provider.map_info().await?;
    Ok(Json(MapInfoResponse::from(&map)))
}

#[utoipa::path(get, path = "/info/server", tag = "Info",
    operation_id = "getServerInfo",
    summary = "Get server info",
    description = "Returns server OS, hostname, and network information.",
    responses(
        (status = 200, description = "Server information", body = ServerInfoResponse),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_server_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ServerInfoResponse>, ServiceError> {
    let server = state.info_provider.server_info().await?;
    Ok(Json(ServerInfoResponse::from(&server)))
}

#[utoipa::path(get, path = "/info/services", tag = "Info",
    operation_id = "getServicesInfo",
    summary = "Get services health",
    description = "Returns health status of all connected services (database, routing, etc.).",
    responses(
        (status = 200, description = "Services health status", body = ServicesInfoResponse),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_services_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ServicesInfoResponse>, ServiceError> {
    let service = state.info_provider.services_info().await?;
    Ok(Json(ServicesInfoResponse {
        items: vec![ServiceStatusResponse::from(&service)],
    }))
}

#[utoipa::path(get, path = "/info/statistics", tag = "Info",
    operation_id = "getStatistics",
    summary = "Get data statistics",
    description = "Returns counts of all managed resources.",
    responses(
        (status = 200, description = "Data statistics", body = DataStatisticsResponse),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_statistics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DataStatisticsResponse>, ServiceError> {
    let stats = state.info_provider.statistics_info().await?;
    Ok(Json(DataStatisticsResponse::from(&stats)))
}
