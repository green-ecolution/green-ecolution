use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use crate::{
    configuration::{DatabaseSettings, Settings},
    http::{AppState, router},
    infra::pg_region::PgRegionRepository,
};

pub struct Application {
    port: u16,
    listener: TcpListener,
    state: Arc<AppState>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&config.database)
            .await
            .expect("failed to connect to database");

        let address = format!("{}:{}", config.application.host, config.application.port);
        Self::build_with_pool(pool, &address).await
    }

    pub async fn build_with_pool(pool: PgPool, address: &str) -> Result<Self, std::io::Error> {
        let state = Arc::new(AppState {
            region_repo: Arc::new(PgRegionRepository::new(pool)),
        });

        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();

        Ok(Self {
            port,
            listener,
            state,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let app = router(self.state);
        tracing::info!("listening on {}", self.listener.local_addr()?);
        axum::serve(self.listener, app).await
    }
}

pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(config.connection_options())
        .await
}
