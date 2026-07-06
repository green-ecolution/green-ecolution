use sqlx::PgPool;

use domain::RepositoryError;
use domain::info::{DataStatistics, StatisticsReader};

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
    async fn statistics(&self) -> Result<DataStatistics, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM trees)                                AS "tree_count!: i64",
                (SELECT COUNT(*) FROM sensors)                              AS "sensor_count!: i64",
                (SELECT COUNT(*) FROM vehicles WHERE archived_at IS NULL)   AS "vehicle_count!: i64",
                (SELECT COUNT(*) FROM tree_clusters WHERE archived = false) AS "cluster_count!: i64",
                (SELECT COUNT(*) FROM watering_plans)                       AS "watering_plan_count!: i64"
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DataStatistics {
            tree_count: row.tree_count,
            sensor_count: row.sensor_count,
            vehicle_count: row.vehicle_count,
            cluster_count: row.cluster_count,
            watering_plan_count: row.watering_plan_count,
        })
    }
}
