use sqlx::PgPool;

use domain::RepositoryError;
use domain::info::{CoreCounts, StatisticsReader};

pub struct PgStatisticsRepo {
    pool: PgPool,
}

impl PgStatisticsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StatisticsReader for PgStatisticsRepo {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn core_counts(&self) -> Result<CoreCounts, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM sensors)                              AS "sensor_count!: i64",
                (SELECT COUNT(*) FROM vehicles WHERE archived_at IS NULL)   AS "vehicle_count!: i64",
                (SELECT COUNT(*) FROM operation_plans)                      AS "plan_count!: i64"
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CoreCounts {
            sensor_count: row.sensor_count,
            vehicle_count: row.vehicle_count,
            plan_count: row.plan_count,
        })
    }
}
