use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};

use crate::http::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/evaluation", get(get_evaluation))
}

pub async fn get_evaluation(State(_state): State<Arc<AppState>>) -> Json<()> {
    todo!()
}
