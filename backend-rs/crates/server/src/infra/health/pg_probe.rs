use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use domain::info::{ServiceMessage, ServiceName, ServiceStatus};

use super::probe::HealthProbe;

pub struct PgProbe {
    pool: PgPool,
}

impl PgProbe {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthProbe for PgProbe {
    fn name(&self) -> ServiceName {
        ServiceName::Postgres
    }

    async fn check(&self) -> ServiceStatus {
        let start = Instant::now();
        let healthy = sqlx::query("SELECT 1").execute(&self.pool).await.is_ok();
        ServiceStatus {
            name: ServiceName::Postgres,
            enabled: true,
            healthy,
            response_time: start.elapsed(),
            last_checked: Utc::now(),
            message: if healthy {
                ServiceMessage::Connected
            } else {
                ServiceMessage::NoConnection
            },
        }
    }
}
