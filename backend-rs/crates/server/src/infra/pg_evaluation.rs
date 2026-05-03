use sqlx::PgPool;

use domain::{
    RepositoryError,
    evaluation::{EvaluationRepository, RegionEvaluation, VehicleEvaluation},
};

pub struct PgEvaluationRepository {
    pool: PgPool,
}

impl PgEvaluationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EvaluationRepository for PgEvaluationRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn regions_with_watering_plan(&self) -> Result<Vec<RegionEvaluation>, RepositoryError> {
        let rows = sqlx::query_as!(
            RegionEvaluation,
            r#"SELECT r.name AS "name!", COUNT(DISTINCT twp.watering_plan_id)::int AS "watering_plan_count!: i32"
            FROM regions r
            INNER JOIN tree_clusters tc ON r.id = tc.region_id
            INNER JOIN tree_cluster_watering_plans twp ON tc.id = twp.tree_cluster_id
            GROUP BY r.name
            ORDER BY COUNT(DISTINCT twp.watering_plan_id) DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn vehicle_with_watering_plan(&self) -> Result<Vec<VehicleEvaluation>, RepositoryError> {
        let rows = sqlx::query_as!(
            VehicleEvaluation,
            r#"SELECT v.number_plate AS "number_plate!", COUNT(vwp.watering_plan_id)::int AS "watering_plan_count!: i32"
            FROM vehicles v
            INNER JOIN vehicle_watering_plans vwp ON v.id = vwp.vehicle_id
            GROUP BY v.number_plate
            ORDER BY COUNT(vwp.watering_plan_id) DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn total_consumed_water(&self) -> Result<f64, RepositoryError> {
        let total = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(consumed_water), 0)::float8 AS \"total!\" FROM tree_cluster_watering_plans"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(total)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn watering_plan_user(&self) -> Result<u64, RepositoryError> {
        let count =
            sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM user_watering_plans"#)
                .fetch_one(&self.pool)
                .await? as u64;

        Ok(count)
    }
}
