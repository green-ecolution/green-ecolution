use sqlx::PgPool;

use domain::{
    RepositoryError,
    authorization::Visibility,
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
    async fn regions_with_watering_plan(
        &self,
        visible: Visibility,
    ) -> Result<Vec<RegionEvaluation>, RepositoryError> {
        let visible_ids = visible.into_raw_ids();
        let rows = sqlx::query_as!(
            RegionEvaluation,
            r#"SELECT r.name AS "name!", COUNT(DISTINCT twp.watering_plan_id)::int AS "watering_plan_count!: i32"
            FROM regions r
            INNER JOIN tree_clusters tc ON r.id = tc.region_id
            INNER JOIN tree_cluster_watering_plans twp ON tc.id = twp.tree_cluster_id
            WHERE ($1::uuid[] IS NULL OR tc.organization_id = ANY($1))
            GROUP BY r.name
            ORDER BY COUNT(DISTINCT twp.watering_plan_id) DESC"#,
            visible_ids.as_deref(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn vehicle_with_watering_plan(
        &self,
        visible_vehicle: Visibility,
        visible_plan: Visibility,
    ) -> Result<Vec<VehicleEvaluation>, RepositoryError> {
        let visible_vehicle_ids = visible_vehicle.into_raw_ids();
        let visible_plan_ids = visible_plan.into_raw_ids();
        let rows = sqlx::query_as!(
            VehicleEvaluation,
            r#"SELECT v.number_plate AS "number_plate!", COUNT(vwp.watering_plan_id)::int AS "watering_plan_count!: i32"
            FROM vehicles v
            INNER JOIN vehicle_watering_plans vwp ON v.id = vwp.vehicle_id
            INNER JOIN watering_plans wp ON wp.id = vwp.watering_plan_id
            WHERE ($1::uuid[] IS NULL OR v.organization_id = ANY($1))
              AND ($2::uuid[] IS NULL OR wp.organization_id = ANY($2))
            GROUP BY v.number_plate
            ORDER BY COUNT(vwp.watering_plan_id) DESC"#,
            visible_vehicle_ids.as_deref(),
            visible_plan_ids.as_deref(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn total_consumed_water(&self, visible: Visibility) -> Result<f64, RepositoryError> {
        let visible_ids = visible.into_raw_ids();
        let total = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(tcwp.consumed_water), 0)::float8 AS "total!"
            FROM tree_cluster_watering_plans tcwp
            INNER JOIN tree_clusters tc ON tc.id = tcwp.tree_cluster_id
            WHERE ($1::uuid[] IS NULL OR tc.organization_id = ANY($1))"#,
            visible_ids.as_deref(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(total)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn watering_plan_user(&self, visible: Visibility) -> Result<u64, RepositoryError> {
        let visible_ids = visible.into_raw_ids();
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64"
            FROM user_watering_plans uwp
            INNER JOIN watering_plans wp ON wp.id = uwp.watering_plan_id
            WHERE ($1::uuid[] IS NULL OR wp.organization_id = ANY($1))"#,
            visible_ids.as_deref(),
        )
        .fetch_one(&self.pool)
        .await? as u64;

        Ok(count)
    }
}
