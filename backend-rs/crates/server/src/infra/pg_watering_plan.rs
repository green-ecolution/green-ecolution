use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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

/// Flat row shape shared by `view_by_id` and `view_search` on
/// `watering_plans`. `From` derives `created_at` from the UUID v7 id;
/// transporter/trailer come from the role column on the vehicle join table.
#[allow(dead_code)] // fields are read via the `From<WateringPlanViewRow>` impl
struct WateringPlanViewRow {
    id: RawId,
    updated_at: NaiveDateTime,
    date: NaiveDate,
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
    transporter_id: Option<RawId>,
    trailer_id: Option<RawId>,
    cluster_ids: Vec<RawId>,
}

impl From<WateringPlanViewRow> for WateringPlanView {
    fn from(row: WateringPlanViewRow) -> Self {
        let created_at = Id::<WateringPlan>::new(row.id)
            .created_at()
            .expect("watering_plans.id is minted as uuid v7");
        let transporter_id = row.transporter_id;
        let trailer_id = row.trailer_id;
        Self {
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
            transporter_id: Option<RawId>,
            trailer_id: Option<RawId>,
            cluster_ids: Vec<RawId>,
        }

        let row = sqlx::query_as!(
            Row,
            r#"SELECT wp.id, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'transporter'))[1] AS "transporter_id: RawId",
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'trailer'))[1] AS "trailer_id: RawId",
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

        let transporter_id = row.transporter_id;
        let trailer_id = row.trailer_id;

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
        let row = sqlx::query_as!(
            WateringPlanViewRow,
            r#"SELECT wp.id, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'transporter'))[1] AS "transporter_id: RawId",
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'trailer'))[1] AS "trailer_id: RawId",
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

        Ok(row.into())
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

        let rows = sqlx::query_as!(
            WateringPlanViewRow,
            r#"SELECT wp.id, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'transporter'))[1] AS "transporter_id: RawId",
                      (ARRAY_AGG(vwp.vehicle_id) FILTER (WHERE vwp.role = 'trailer'))[1] AS "trailer_id: RawId",
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

        let items = rows.into_iter().map(Into::into).collect();

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

/// Persists the plan row and syncs both join tables inside the caller's
/// transaction. Cluster rows are diffed rather than rewritten so surviving
/// rows keep their `consumed_water` (a full delete + reinsert silently reset
/// recorded evaluations to the column default).
async fn persist_plan(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    plan: &WateringPlan,
) -> Result<(), RepositoryError> {
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
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }

    sqlx::query!(
        "DELETE FROM vehicle_watering_plans WHERE watering_plan_id = $1",
        plan.id.value()
    )
    .execute(&mut **tx)
    .await?;

    insert_vehicle_roles(
        tx,
        plan.id.value(),
        plan.transporter_id().map(|id| id.value()),
        plan.trailer_id().map(|id| id.value()),
    )
    .await?;

    let cluster_id_values: Vec<RawId> = plan.cluster_ids().to_values();
    sqlx::query!(
        "DELETE FROM tree_cluster_watering_plans WHERE watering_plan_id = $1 AND tree_cluster_id <> ALL($2::uuid[])",
        plan.id.value(),
        &cluster_id_values,
    )
    .execute(&mut **tx)
    .await?;

    if !cluster_id_values.is_empty() {
        sqlx::query!(
            r#"INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id)
               SELECT UNNEST($1::uuid[]), $2
               ON CONFLICT (tree_cluster_id, watering_plan_id) DO NOTHING"#,
            &cluster_id_values,
            plan.id.value(),
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// Writes the transporter/trailer join rows with their role. The role column
/// (not uuid order) is what `by_id` / the view queries decode the slots from.
async fn insert_vehicle_roles(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    plan_id: RawId,
    transporter_id: Option<RawId>,
    trailer_id: Option<RawId>,
) -> Result<(), RepositoryError> {
    if let Some(id) = transporter_id {
        sqlx::query!(
            "INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id, role) VALUES ($1, $2, 'transporter')",
            id,
            plan_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    if let Some(id) = trailer_id {
        sqlx::query!(
            "INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id, role) VALUES ($1, $2, 'trailer')",
            id,
            plan_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
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

        insert_vehicle_roles(
            &mut tx,
            plan_id.value(),
            draft.transporter_id.map(|id| id.value()),
            draft.trailer_id.map(|id| id.value()),
        )
        .await?;

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
        persist_plan(&mut tx, plan).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_finished(
        &self,
        plan: &WateringPlan,
        evaluations: &[WateringPlanEvaluation],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        persist_plan(&mut tx, plan).await?;

        let cluster_ids: Vec<RawId> = evaluations.iter().map(|e| e.cluster_id.value()).collect();
        let amounts: Vec<f64> = evaluations.iter().map(|e| e.consumed_water).collect();
        sqlx::query!(
            r#"UPDATE tree_cluster_watering_plans t
               SET consumed_water = e.consumed_water
               FROM (SELECT UNNEST($2::uuid[]) AS cluster_id, UNNEST($3::float8[]) AS consumed_water) e
               WHERE t.watering_plan_id = $1 AND t.tree_cluster_id = e.cluster_id"#,
            plan.id.value(),
            &cluster_ids,
            &amounts,
        )
        .execute(&mut *tx)
        .await?;

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
