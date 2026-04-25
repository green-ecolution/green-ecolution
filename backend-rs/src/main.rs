use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;

use crate::{
    http::{AppState, router},
    infra::pg_region::PgRegionRepository,
};

pub mod domain;
pub mod http;
pub mod infra;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = Arc::new(AppState {
        region_repo: Arc::new(PgRegionRepository::new(pool)),
    });

    let app = router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3020").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
