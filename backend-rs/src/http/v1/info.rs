use std::sync::Arc;

use axum::{Json, extract::State};
use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::http::AppState;

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().route("/info", get(get_info))
}

pub async fn get_info(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}
