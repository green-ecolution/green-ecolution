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
            r#"SELECT r.name AS "name!", COUNT(DISTINCT ops.plan_id)::int AS "watering_plan_count!: i32"
            FROM regions r
            INNER JOIN tree_clusters tc ON r.id = tc.region_id
            INNER JOIN operation_plan_stops ops ON tc.id = ops.stop_id
            INNER JOIN operation_plans op ON op.id = ops.plan_id
            WHERE op.module_key = 'irrigation'
              AND ($1::uuid[] IS NULL OR tc.organization_id = ANY($1))
            GROUP BY r.name
            ORDER BY COUNT(DISTINCT ops.plan_id) DESC"#,
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
            r#"SELECT v.number_plate AS "number_plate!", COUNT(opv.plan_id)::int AS "watering_plan_count!: i32"
            FROM vehicles v
            INNER JOIN operation_plan_vehicles opv ON v.id = opv.vehicle_id
            INNER JOIN operation_plans op ON op.id = opv.plan_id
            WHERE op.module_key = 'irrigation'
              AND ($1::uuid[] IS NULL OR v.organization_id = ANY($1))
              AND ($2::uuid[] IS NULL OR op.organization_id = ANY($2))
            GROUP BY v.number_plate
            ORDER BY COUNT(opv.plan_id) DESC"#,
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
            r#"SELECT COALESCE(SUM(ops.amount), 0)::float8 AS "total!"
            FROM operation_plan_stops ops
            INNER JOIN operation_plans op ON op.id = ops.plan_id
            INNER JOIN tree_clusters tc ON tc.id = ops.stop_id
            WHERE op.module_key = 'irrigation'
              AND ($1::uuid[] IS NULL OR tc.organization_id = ANY($1))"#,
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
            FROM operation_plan_users opu
            INNER JOIN operation_plans op ON op.id = opu.plan_id
            WHERE op.module_key = 'irrigation'
              AND ($1::uuid[] IS NULL OR op.organization_id = ANY($1))"#,
            visible_ids.as_deref(),
        )
        .fetch_one(&self.pool)
        .await? as u64;

        Ok(count)
    }
}
