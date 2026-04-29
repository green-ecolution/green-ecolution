use chrono::NaiveTime;
use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    shared::{
        distance::Distance,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
    },
    watering_plan::{
        WateringPlan, WateringPlanCreate, WateringPlanQuery, WateringPlanRepository,
        WateringPlanStatus, WateringPlanUpdate,
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

#[async_trait::async_trait]
impl WateringPlanRepository for PgWateringPlanRepository {
    async fn all(
        &self,
        query: WateringPlanQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlan>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM watering_plans
            WHERE ($1::text IS NULL OR provider = $1)"#,
            query.provider,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query!(
            r#"SELECT wp.id, wp.created_at, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      COALESCE(ARRAY_AGG(DISTINCT vwp.vehicle_id) FILTER (WHERE vwp.vehicle_id IS NOT NULL), ARRAY[]::int[]) AS "vehicle_ids!: Vec<i32>",
                      COALESCE(ARRAY_AGG(DISTINCT twp.tree_cluster_id) FILTER (WHERE twp.tree_cluster_id IS NOT NULL), ARRAY[]::int[]) AS "cluster_ids!: Vec<i32>"
            FROM watering_plans wp
            LEFT JOIN vehicle_watering_plans vwp ON vwp.watering_plan_id = wp.id
            LEFT JOIN tree_cluster_watering_plans twp ON twp.watering_plan_id = wp.id
            WHERE ($1::text IS NULL OR wp.provider = $1)
            GROUP BY wp.id
            ORDER BY wp.date DESC
            LIMIT $2 OFFSET $3"#,
            query.provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|row| {
                // TODO: Transporter/Trailer distinction is based on insertion order - needs a role column in the junction table
                let transporter_id = row.vehicle_ids.first().copied().map(Id::new);
                let trailer_id = row.vehicle_ids.get(1).copied().map(Id::new);

                WateringPlan {
                    id: Id::new(row.id),
                    created_at: row.created_at.and_utc(),
                    updated_at: row.updated_at.and_utc(),
                    date: row.date.and_time(NaiveTime::MIN).and_utc(),
                    description: Some(row.description),
                    status: row.status,
                    distance: row.distance.and_then(|d| Distance::new(d).ok()),
                    total_water_required: row.total_water_required,
                    cluster_ids: row.cluster_ids.into_iter().map(Id::new).collect(),
                    transporter_id,
                    trailer_id,
                    cancellation_note: Some(row.cancellation_note),
                    evaluation: None,
                    gpx_url: row.gpx_url.and_then(|u| u.parse().ok()),
                    refill_count: row.refill_count as u32,
                    duration: std::time::Duration::from_secs_f64(row.duration),
                    provider_info: ProviderInfo {
                        provider: row.provider,
                        additional_info: row.additional_informations,
                    },
                }
            })
            .collect();

        Ok(Page { items, total })
    }

    async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT wp.id, wp.created_at, wp.updated_at, wp.date, wp.description,
                      wp.status AS "status: WateringPlanStatus",
                      wp.distance, wp.total_water_required, wp.cancellation_note,
                      wp.gpx_url, wp.refill_count, wp.duration,
                      wp.provider, wp.additional_informations,
                      COALESCE(ARRAY_AGG(DISTINCT vwp.vehicle_id) FILTER (WHERE vwp.vehicle_id IS NOT NULL), ARRAY[]::int[]) AS "vehicle_ids!: Vec<i32>",
                      COALESCE(ARRAY_AGG(DISTINCT twp.tree_cluster_id) FILTER (WHERE twp.tree_cluster_id IS NOT NULL), ARRAY[]::int[]) AS "cluster_ids!: Vec<i32>"
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

        // TODO: Transporter/Trailer distinction is based on insertion order - needs a role column in the junction table
        let transporter_id = row.vehicle_ids.first().copied().map(Id::new);
        let trailer_id = row.vehicle_ids.get(1).copied().map(Id::new);

        Ok(WateringPlan {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description),
            status: row.status,
            distance: row.distance.and_then(|d| Distance::new(d).ok()),
            total_water_required: row.total_water_required,
            cluster_ids: row.cluster_ids.into_iter().map(Id::new).collect(),
            transporter_id,
            trailer_id,
            cancellation_note: Some(row.cancellation_note),
            evaluation: None,
            gpx_url: row.gpx_url.and_then(|u| u.parse().ok()),
            refill_count: row.refill_count as u32,
            duration: std::time::Duration::from_secs_f64(row.duration),
            provider_info: ProviderInfo {
                provider: row.provider,
                additional_info: row.additional_informations,
            },
        })
    }

    async fn create(
        &self,
        entity: WateringPlanCreate,
    ) -> Result<WateringPlan, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"INSERT INTO watering_plans (date, description, status, provider, additional_informations)
            VALUES ($1, $2, 'planned', $3, $4)
            RETURNING id, created_at, updated_at, date, description,
                      status AS "status: WateringPlanStatus",
                      distance, total_water_required, cancellation_note,
                      gpx_url, refill_count, duration,
                      provider, additional_informations"#,
            entity.date.date_naive(),
            entity.description,
            entity.provider_info.provider,
            entity.provider_info.additional_info,
        )
        .fetch_one(&mut *tx)
        .await?;

        let plan_id = row.id;

        let mut vehicle_ids = Vec::new();
        if let Some(ref id) = entity.transporter_id {
            vehicle_ids.push(id.value());
        }
        if let Some(ref id) = entity.trailer_id {
            vehicle_ids.push(id.value());
        }
        if !vehicle_ids.is_empty() {
            sqlx::query!(
                "INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) SELECT UNNEST($1::int[]), $2",
                &vehicle_ids,
                plan_id
            )
            .execute(&mut *tx)
            .await?;
        }

        let cluster_id_values: Vec<i32> = entity.cluster_ids.iter().map(|id| id.value()).collect();
        if !cluster_id_values.is_empty() {
            sqlx::query!(
                "INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id) SELECT UNNEST($1::int[]), $2",
                &cluster_id_values,
                plan_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(WateringPlan {
            id: Id::new(plan_id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description),
            status: row.status,
            distance: None,
            total_water_required: None,
            cluster_ids: entity.cluster_ids,
            transporter_id: entity.transporter_id,
            trailer_id: entity.trailer_id,
            cancellation_note: Some(row.cancellation_note),
            evaluation: None,
            gpx_url: None,
            refill_count: 0,
            duration: std::time::Duration::ZERO,
            provider_info: entity.provider_info,
        })
    }

    async fn update(
        &self,
        id: Id<WateringPlan>,
        entity: WateringPlanUpdate,
    ) -> Result<WateringPlan, RepositoryError> {
        sqlx::query!(
            r#"UPDATE watering_plans SET
                date = COALESCE($2, date),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                cancellation_note = COALESCE($5, cancellation_note),
                provider = COALESCE($6, provider),
                additional_informations = COALESCE($7, additional_informations)
            WHERE id = $1"#,
            id.value(),
            entity.date.map(|d| d.date_naive()),
            entity.description,
            entity.status as Option<WateringPlanStatus>,
            entity.cancellation_note,
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.provider.as_deref()),
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.additional_info.clone()),
        )
        .execute(&self.pool)
        .await?;

        self.by_id(id).await
    }

    async fn delete(&self, id: Id<WateringPlan>) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM watering_plans WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
