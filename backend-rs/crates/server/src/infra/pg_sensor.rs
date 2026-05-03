use async_trait::async_trait;
use sqlx::PgPool;

use domain::{
    RepositoryError,
    sensor::{
        Sensor, SensorDraft, SensorId, SensorReader, SensorReadingReader, SensorReadingWriter,
        SensorSearchQuery, SensorSnapshot, SensorStatus, SensorView, SensorWriter,
        data::{SensorReading, SensorReadingDraft, SensorReadingSnapshot, SensorReadingView},
    },
    shared::pagination::{Page, Pagination},
};

pub struct PgSensorRepository {
    pool: PgPool,
}

impl PgSensorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SensorReader for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: &SensorId) -> Result<Sensor, RepositoryError> {
        let snap = sqlx::query_as!(
            SensorSnapshot,
            r#"SELECT id,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info
            FROM sensors WHERE id = $1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Sensor::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[SensorId]) -> Result<Vec<Sensor>, RepositoryError> {
        let ids: Vec<&str> = ids.iter().map(SensorId::as_str).collect();

        let snaps = sqlx::query_as!(
            SensorSnapshot,
            r#"SELECT id,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info
            FROM sensors WHERE id = ANY($1::text[])"#,
            &ids as &[&str]
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(Sensor::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info
            FROM sensors WHERE id = $1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let latest_reading = sqlx::query!(
            r#"SELECT id, sensor_id, created_at, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC LIMIT 1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| SensorReadingView {
            id: r.id,
            sensor_id: r.sensor_id,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
            data: r.data,
        });

        Ok(SensorView {
            id: row.id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            status: row.status,
            latitude: row.latitude,
            longitude: row.longitude,
            provider: row.provider,
            additional_info: row.additional_info,
            latest_reading,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[SensorId]) -> Result<Vec<SensorView>, RepositoryError> {
        let ids: Vec<&str> = ids.iter().map(SensorId::as_str).collect();

        let rows = sqlx::query!(
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info
            FROM sensors WHERE id = ANY($1::text[])"#,
            &ids as &[&str]
        )
        .fetch_all(&self.pool)
        .await?;

        // latest_reading omitted to avoid N+1 per-row subqueries on batch endpoints
        Ok(rows
            .into_iter()
            .map(|r| SensorView {
                id: r.id,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
                status: r.status,
                latitude: r.latitude,
                longitude: r.longitude,
                provider: r.provider,
                additional_info: r.additional_info,
                latest_reading: None,
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: SensorSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<SensorView>, RepositoryError> {
        let provider = query.provider.as_ref().map(|p| p.as_str().to_owned());
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM sensors
            WHERE ($1::text IS NULL OR provider = $1)"#,
            provider
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query!(
            r#"SELECT id, created_at, updated_at,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info
            FROM sensors
            WHERE ($1::text IS NULL OR provider = $1)
            ORDER BY id
            LIMIT $2 OFFSET $3"#,
            provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        // latest_reading omitted to avoid N+1 per-row subqueries on list endpoints
        let items = rows
            .into_iter()
            .map(|r| SensorView {
                id: r.id,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
                status: r.status,
                latitude: r.latitude,
                longitude: r.longitude,
                provider: r.provider,
                additional_info: r.additional_info,
                latest_reading: None,
            })
            .collect();

        Ok(Page { items, total })
    }
}

#[async_trait]
impl SensorWriter for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: SensorDraft) -> Result<Sensor, RepositoryError> {
        let lat = draft.coordinate.latitude();
        let lng = draft.coordinate.longitude();

        let snap = sqlx::query_as!(
            SensorSnapshot,
            r#"INSERT INTO sensors (id, status, latitude, longitude, geometry, provider, additional_informations)
            VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), $5, $6)
            RETURNING id,
                      status AS "status: SensorStatus",
                      latitude, longitude,
                      provider,
                      additional_informations AS additional_info"#,
            draft.id.as_str(),
            draft.status as SensorStatus,
            lat,
            lng,
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info().cloned(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Sensor::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, sensor: &Sensor) -> Result<(), RepositoryError> {
        let lat = sensor.coordinate.latitude();
        let lng = sensor.coordinate.longitude();

        let result = sqlx::query!(
            r#"UPDATE sensors SET
                status = $2,
                latitude = $3,
                longitude = $4,
                geometry = ST_SetSRID(ST_MakePoint($4, $3), 4326),
                provider = $5,
                additional_informations = $6
            WHERE id = $1"#,
            sensor.id.as_str(),
            sensor.status as SensorStatus,
            lat,
            lng,
            sensor.provenance.provider().map(|p| p.as_str()),
            sensor.provenance.additional_info().cloned(),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: &SensorId) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM sensors WHERE id = $1", id.as_str())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

#[async_trait]
impl SensorReadingReader for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn history(
        &self,
        sensor_id: &SensorId,
        limit: i64,
    ) -> Result<Vec<SensorReading>, RepositoryError> {
        let snaps = sqlx::query_as!(
            SensorReadingSnapshot,
            r#"SELECT id, sensor_id, created_at AS recorded_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC LIMIT $2"#,
            sensor_id.as_str(),
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(SensorReading::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn latest(&self, sensor_id: &SensorId) -> Result<Option<SensorReading>, RepositoryError> {
        let snap = sqlx::query_as!(
            SensorReadingSnapshot,
            r#"SELECT id, sensor_id, created_at AS recorded_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC LIMIT 1"#,
            sensor_id.as_str(),
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snap.map(SensorReading::reconstitute))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_history(
        &self,
        sensor_id: &SensorId,
        limit: i64,
    ) -> Result<Vec<SensorReadingView>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT id, sensor_id, created_at, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY created_at DESC LIMIT $2"#,
            sensor_id.as_str(),
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SensorReadingView {
                id: r.id,
                sensor_id: r.sensor_id,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
                data: r.data,
            })
            .collect())
    }
}

#[async_trait]
impl SensorReadingWriter for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn record(&self, draft: SensorReadingDraft) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO sensor_data (sensor_id, data) VALUES ($1, $2)"#,
            draft.sensor_id.as_str(),
            draft.data,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
