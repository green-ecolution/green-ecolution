use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/info", get(get_info))
}

pub async fn get_info(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}
