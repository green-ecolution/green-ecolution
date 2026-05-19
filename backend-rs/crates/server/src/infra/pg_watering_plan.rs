use async_trait::async_trait;
use chrono::NaiveTime;
use serde_json::Value;
use sqlx::PgPool;

use domain::{
    Id, IdSliceExt, RawId, RepositoryError,
    cluster::TreeCluster,
    shared::pagination::{Page, Pagination},
    watering_plan::{
        WateringPlan, WateringPlanDraft, WateringPlanEvaluation, WateringPlanReader,
        WateringPlanSearchQuery, WateringPlanSnapshot, WateringPlanStatus, WateringPlanView,
        WateringPlanWriter,
    },
};

pub struct PgWateringPlanRepository {
    pool: PgPool,
}

impl PgWateringPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WateringPlanReader for PgWateringPlanRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, RepositoryError> {
        struct Row {
            id: RawId,
            date: chrono::NaiveDate,
            description: String,
            status: WateringPlanStatus,
            distance: Option<f64>,
            total_water_required: Option<f64>,
            cancellation_note: String,
            gpx_url: Option<String>,
            refill_count: i32,
            duration: f64,
            provider: Option<String>,
            additional_informations: Option<Value>,
            vehicle_ids: Vec<RawId>,
            cluster_ids: Vec<RawId>,
        }

        let row = sqlx::query_as!(
            Row,
            r#"SELECT wp.id, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      COALESCE(ARRAY_AGG(DISTINCT vwp.vehicle_id) FILTER (WHERE vwp.vehicle_id IS NOT NULL), ARRAY[]::uuid[]) AS "vehicle_ids!: Vec<RawId>",
                      COALESCE(ARRAY_AGG(DISTINCT twp.tree_cluster_id) FILTER (WHERE twp.tree_cluster_id IS NOT NULL), ARRAY[]::uuid[]) AS "cluster_ids!: Vec<RawId>"
            FROM watering_plans wp
            LEFT JOIN vehicle_watering_plans vwp ON vwp.watering_plan_id = wp.id
            LEFT JOIN tree_cluster_watering_plans twp ON twp.watering_plan_id = wp.id
            WHERE wp.id = $1
            GROUP BY wp.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let transporter_id = row.vehicle_ids.first().copied();
        let trailer_id = row.vehicle_ids.get(1).copied();

        Ok(WateringPlan::reconstitute(WateringPlanSnapshot {
            id: row.id,
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description).filter(|s| !s.is_empty()),
            status: row.status,
            distance: row.distance,
            total_water_required: row.total_water_required,
            cluster_ids: row.cluster_ids,
            transporter_id,
            trailer_id,
            cancellation_note: Some(row.cancellation_note).filter(|s| !s.is_empty()),
            gpx_url: row.gpx_url.and_then(|u| u.parse().ok()),
            refill_count: row.refill_count,
            duration: std::time::Duration::from_secs_f64(row.duration),
            provider: row.provider,
            additional_info: row.additional_informations,
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlanView, RepositoryError> {
        struct Row {
            id: RawId,
            updated_at: chrono::NaiveDateTime,
            date: chrono::NaiveDate,
            description: String,
            status: WateringPlanStatus,
            distance: Option<f64>,
            total_water_required: Option<f64>,
            cancellation_note: String,
            gpx_url: Option<String>,
            refill_count: i32,
            duration: f64,
            provider: Option<String>,
            additional_informations: Option<Value>,
            vehicle_ids: Vec<RawId>,
            cluster_ids: Vec<RawId>,
        }

        let row = sqlx::query_as!(
            Row,
            r#"SELECT wp.id, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      COALESCE(ARRAY_AGG(DISTINCT vwp.vehicle_id) FILTER (WHERE vwp.vehicle_id IS NOT NULL), ARRAY[]::uuid[]) AS "vehicle_ids!: Vec<RawId>",
                      COALESCE(ARRAY_AGG(DISTINCT twp.tree_cluster_id) FILTER (WHERE twp.tree_cluster_id IS NOT NULL), ARRAY[]::uuid[]) AS "cluster_ids!: Vec<RawId>"
            FROM watering_plans wp
            LEFT JOIN vehicle_watering_plans vwp ON vwp.watering_plan_id = wp.id
            LEFT JOIN tree_cluster_watering_plans twp ON twp.watering_plan_id = wp.id
            WHERE wp.id = $1
            GROUP BY wp.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let transporter_id = row.vehicle_ids.first().copied();
        let trailer_id = row.vehicle_ids.get(1).copied();
        let created_at = Id::<WateringPlan>::new(row.id)
            .created_at()
            .unwrap_or_default();

        Ok(WateringPlanView {
            id: row.id,
            created_at,
            updated_at: row.updated_at.and_utc(),
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description).filter(|s| !s.is_empty()),
            status: row.status,
            distance: row.distance,
            total_water_required: row.total_water_required,
            cluster_ids: row.cluster_ids,
            transporter_id,
            trailer_id,
            cancellation_note: Some(row.cancellation_note).filter(|s| !s.is_empty()),
            gpx_url: row.gpx_url.and_then(|u| u.parse().ok()),
            refill_count: row.refill_count,
            duration: std::time::Duration::from_secs_f64(row.duration),
            provider: row.provider,
            additional_info: row.additional_informations,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: WateringPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlanView>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);
        let provider = query.provider.as_ref().map(|p| p.as_str().to_owned());

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM watering_plans
            WHERE ($1::text IS NULL OR provider = $1)"#,
            provider,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        struct Row {
            id: RawId,
            updated_at: chrono::NaiveDateTime,
            date: chrono::NaiveDate,
            description: String,
            status: WateringPlanStatus,
            distance: Option<f64>,
            total_water_required: Option<f64>,
            cancellation_note: String,
            gpx_url: Option<String>,
            refill_count: i32,
            duration: f64,
            provider: Option<String>,
            additional_informations: Option<Value>,
            vehicle_ids: Vec<RawId>,
            cluster_ids: Vec<RawId>,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"SELECT wp.id, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      COALESCE(ARRAY_AGG(DISTINCT vwp.vehicle_id) FILTER (WHERE vwp.vehicle_id IS NOT NULL), ARRAY[]::uuid[]) AS "vehicle_ids!: Vec<RawId>",
                      COALESCE(ARRAY_AGG(DISTINCT twp.tree_cluster_id) FILTER (WHERE twp.tree_cluster_id IS NOT NULL), ARRAY[]::uuid[]) AS "cluster_ids!: Vec<RawId>"
            FROM watering_plans wp
            LEFT JOIN vehicle_watering_plans vwp ON vwp.watering_plan_id = wp.id
            LEFT JOIN tree_cluster_watering_plans twp ON twp.watering_plan_id = wp.id
            WHERE ($1::text IS NULL OR wp.provider = $1)
            GROUP BY wp.id
            ORDER BY wp.date DESC
            LIMIT $2 OFFSET $3"#,
            provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|row| {
                let transporter_id = row.vehicle_ids.first().copied();
                let trailer_id = row.vehicle_ids.get(1).copied();
                let created_at = Id::<WateringPlan>::new(row.id)
                    .created_at()
                    .unwrap_or_default();
                WateringPlanView {
                    id: row.id,
                    created_at,
                    updated_at: row.updated_at.and_utc(),
                    date: row.date.and_time(NaiveTime::MIN).and_utc(),
                    description: Some(row.description).filter(|s| !s.is_empty()),
                    status: row.status,
                    distance: row.distance,
                    total_water_required: row.total_water_required,
                    cluster_ids: row.cluster_ids,
                    transporter_id,
                    trailer_id,
                    cancellation_note: Some(row.cancellation_note).filter(|s| !s.is_empty()),
                    gpx_url: row.gpx_url.and_then(|u| u.parse().ok()),
                    refill_count: row.refill_count,
                    duration: std::time::Duration::from_secs_f64(row.duration),
                    provider: row.provider,
                    additional_info: row.additional_informations,
                }
            })
            .collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn evaluations(
        &self,
        plan_id: Id<WateringPlan>,
    ) -> Result<Vec<WateringPlanEvaluation>, RepositoryError> {
        struct Row {
            tree_cluster_id: RawId,
            consumed_water: f64,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"SELECT tree_cluster_id, consumed_water
            FROM tree_cluster_watering_plans
            WHERE watering_plan_id = $1"#,
            plan_id.value()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| WateringPlanEvaluation {
                watering_plan_id: plan_id,
                cluster_id: Id::new(row.tree_cluster_id),
                consumed_water: row.consumed_water,
            })
            .collect())
    }
}

#[async_trait]
impl WateringPlanWriter for PgWateringPlanRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: WateringPlanDraft) -> Result<WateringPlan, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let plan_id = Id::<WateringPlan>::new_v7();
        sqlx::query!(
            r#"INSERT INTO watering_plans (id, date, description, status, provider, additional_informations)
            VALUES ($1, $2, $3, 'planned', $4, $5)"#,
            plan_id.value(),
            draft.date.date_naive(),
            draft.description.as_deref().unwrap_or(""),
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info(),
        )
        .execute(&mut *tx)
        .await?;

        let mut vehicle_ids: Vec<RawId> = Vec::new();
        if let Some(ref id) = draft.transporter_id {
            vehicle_ids.push(id.value());
        }
        if let Some(ref id) = draft.trailer_id {
            vehicle_ids.push(id.value());
        }
        if !vehicle_ids.is_empty() {
            sqlx::query!(
                "INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) SELECT UNNEST($1::uuid[]), $2",
                &vehicle_ids,
                plan_id.value()
            )
            .execute(&mut *tx)
            .await?;
        }

        let cluster_id_values: Vec<RawId> = draft.cluster_ids.to_values();
        if !cluster_id_values.is_empty() {
            sqlx::query!(
                "INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id) SELECT UNNEST($1::uuid[]), $2",
                &cluster_id_values,
                plan_id.value()
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.by_id(plan_id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, plan: &WateringPlan) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query!(
            r#"UPDATE watering_plans SET
                date = $2,
                description = $3,
                status = $4,
                cancellation_note = $5,
                distance = $6,
                total_water_required = $7,
                refill_count = $8,
                duration = $9,
                gpx_url = $10,
                provider = $11,
                additional_informations = $12
            WHERE id = $1"#,
            plan.id.value(),
            plan.date.date_naive(),
            plan.description.as_deref().unwrap_or(""),
            plan.status() as WateringPlanStatus,
            plan.cancellation_note().unwrap_or(""),
            plan.distance.as_ref().map(|d| d.meters()),
            plan.total_water_required,
            plan.refill_count as i32,
            plan.duration.as_secs_f64(),
            plan.gpx_url.as_ref().map(|u| u.as_str()),
            plan.provenance().provider().map(|p| p.as_str()),
            plan.provenance().additional_info(),
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        // rewrite vehicle join rows
        sqlx::query!(
            "DELETE FROM vehicle_watering_plans WHERE watering_plan_id = $1",
            plan.id.value()
        )
        .execute(&mut *tx)
        .await?;

        let mut vehicle_ids: Vec<RawId> = Vec::new();
        if let Some(id) = plan.transporter_id() {
            vehicle_ids.push(id.value());
        }
        if let Some(id) = plan.trailer_id() {
            vehicle_ids.push(id.value());
        }
        if !vehicle_ids.is_empty() {
            sqlx::query!(
                "INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) SELECT UNNEST($1::uuid[]), $2",
                &vehicle_ids,
                plan.id.value()
            )
            .execute(&mut *tx)
            .await?;
        }

        // rewrite cluster join rows
        sqlx::query!(
            "DELETE FROM tree_cluster_watering_plans WHERE watering_plan_id = $1",
            plan.id.value()
        )
        .execute(&mut *tx)
        .await?;

        let cluster_id_values: Vec<RawId> = plan.cluster_ids().to_values();
        if !cluster_id_values.is_empty() {
            sqlx::query!(
                "INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id) SELECT UNNEST($1::uuid[]), $2",
                &cluster_id_values,
                plan.id.value()
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_evaluations(
        &self,
        plan_id: Id<WateringPlan>,
        evaluations: &[WateringPlanEvaluation],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "DELETE FROM tree_cluster_watering_plans WHERE watering_plan_id = $1",
            plan_id.value()
        )
        .execute(&mut *tx)
        .await?;

        for eval in evaluations {
            sqlx::query!(
                "INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water) VALUES ($1, $2, $3)",
                eval.cluster_id.value(),
                plan_id.value(),
                eval.consumed_water,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<WateringPlan>) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "DELETE FROM vehicle_watering_plans WHERE watering_plan_id = $1",
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "DELETE FROM tree_cluster_watering_plans WHERE watering_plan_id = $1",
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        let result = sqlx::query!("DELETE FROM watering_plans WHERE id = $1", id.value())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn propagate_last_watered(
        &self,
        cluster_ids: &[Id<TreeCluster>],
        ts: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError> {
        let ids: Vec<RawId> = cluster_ids.to_values();
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE tree_clusters SET last_watered = $2 WHERE id = ANY($1::uuid[])",
            &ids,
            ts.naive_utc(),
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE trees SET last_watered = $2 WHERE tree_cluster_id = ANY($1::uuid[])",
            &ids,
            ts.naive_utc(),
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
