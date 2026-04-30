use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{
    RepositoryError,
    sensor::{
        Sensor, SensorCreate, SensorData, SensorQuery, SensorRepository, SensorStatus, SensorUpdate,
    },
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
    },
};

struct SensorRow {
    id: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    status: SensorStatus,
    latitude: f64,
    longitude: f64,
    provider: Option<String>,
    additional_informations: Option<Value>,
}

impl TryFrom<SensorRow> for Sensor {
    type Error = RepositoryError;

    fn try_from(row: SensorRow) -> Result<Self, Self::Error> {
        Ok(Sensor {
            id: row.id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            status: row.status,
            latest_data: None,
            coordinates: Coordinate::new(row.latitude, row.longitude)?,
            provider_info: ProviderInfo {
                provider: row.provider,
                additional_info: row.additional_informations,
            },
        })
    }
}

pub struct PgSensorRepository {
    pool: PgPool,
}

impl PgSensorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SensorRepository for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn all(
        &self,
        query: SensorQuery,
        pagination: Pagination,
    ) -> Result<Page<Sensor>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM sensors
            WHERE ($1::text IS NULL OR provider = $1)"#,
            query.provider
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            SensorRow,
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider, additional_informations
            FROM sensors
            WHERE ($1::text IS NULL OR provider = $1)
            ORDER BY id
            LIMIT $2 OFFSET $3"#,
            query.provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(Sensor::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: &str) -> Result<Sensor, RepositoryError> {
        sqlx::query_as!(
            SensorRow,
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider, additional_informations
            FROM sensors WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[String]) -> Result<Vec<Sensor>, RepositoryError> {
        sqlx::query_as!(
            SensorRow,
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider, additional_informations
            FROM sensors WHERE id = ANY($1)"#,
            ids
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Sensor::try_from)
        .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn create(&self, entity: SensorCreate) -> Result<Sensor, RepositoryError> {
        let lat = entity.coordinate.latitude();
        let lng = entity.coordinate.longitude();

        sqlx::query_as!(
            SensorRow,
            r#"INSERT INTO sensors (id, status, latitude, longitude, geometry, provider, additional_informations)
            VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), $5, $6)
            RETURNING id, created_at, updated_at, status AS "status: SensorStatus",
                      latitude, longitude, provider, additional_informations"#,
            entity.id,
            entity.status as SensorStatus,
            lat,
            lng,
            entity.provider_info.provider,
            entity.provider_info.additional_info,
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn update(&self, id: &str, entity: SensorUpdate) -> Result<Sensor, RepositoryError> {
        sqlx::query_as!(
            SensorRow,
            r#"UPDATE sensors SET
                status = COALESCE($2, status),
                provider = COALESCE($3, provider),
                additional_informations = COALESCE($4, additional_informations)
            WHERE id = $1
            RETURNING id, created_at, updated_at, status AS "status: SensorStatus",
                      latitude, longitude, provider, additional_informations"#,
            id,
            entity.status as Option<SensorStatus>,
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.provider.as_deref()),
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.additional_info.clone()),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM sensors WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn all_data(&self, sensor_id: &str) -> Result<Vec<SensorData>, RepositoryError> {
        // TODO: replace hard limit with proper pagination once the API allows it.
        // Sensor data grows unbounded (LoRaWAN ticks), so we cap reads to prevent OOM.
        const MAX_ROWS: i64 = 10_000;

        let rows = sqlx::query!(
            r#"SELECT id, sensor_id, created_at, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC
            LIMIT $2"#,
            sensor_id,
            MAX_ROWS,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SensorData {
                id: row.id,
                sensor_id: row.sensor_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                data: row.data,
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn latest_data(&self, sensor_id: &str) -> Result<SensorData, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, sensor_id, created_at, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC LIMIT 1"#,
            sensor_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(SensorData {
            id: row.id,
            sensor_id: row.sensor_id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            data: row.data,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn create_data(&self, data: SensorData) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO sensor_data (sensor_id, data) VALUES ($1, $2)"#,
            data.sensor_id,
            data.data,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
