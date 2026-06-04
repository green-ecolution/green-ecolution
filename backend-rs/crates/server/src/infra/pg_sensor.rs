use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use domain::{
    RepositoryError,
    sensor::{
        LorawanCredentials, Sensor, SensorDraft, SensorId, SensorReader, SensorReadingReader,
        SensorReadingWriter, SensorSearchQuery, SensorSnapshot, SensorType, SensorView,
        SensorWriter,
        data::{SensorReading, SensorReadingDraft, SensorReadingSnapshot, SensorReadingView},
        derive_connectivity,
        repository::NormalizedValue,
        view::{LorawanInfo, SensorModelSummary},
    },
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
        provenance::ProviderId,
        string_value::NonEmptyString,
    },
    uuid_v7_timestamp,
};

pub struct PgSensorRepository {
    pool: PgPool,
    offline_after: chrono::Duration,
}

impl PgSensorRepository {
    pub fn new(pool: PgPool, offline_after: chrono::Duration) -> Self {
        Self {
            pool,
            offline_after,
        }
    }
}

#[async_trait]
impl SensorReader for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: &SensorId) -> Result<Sensor, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT s.id,
                      s.activated_at,
                      s.type          AS "sensor_type: SensorType",
                      s.model_id,
                      s.provider,
                      s.additional_informations AS "additional_info: serde_json::Value",
                      sl.serial_number AS "serial_number?",
                      sl.dev_eui       AS "dev_eui?",
                      sl.app_eui       AS "app_eui?",
                      sl.app_key       AS "app_key?",
                      sl.at_pin,
                      sl.ota_pin,
                      sl.config        AS "config: serde_json::Value"
            FROM sensors s
            LEFT JOIN sensor_lorawan sl ON sl.id = s.id
            WHERE s.id = $1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let lorawan = build_lorawan(
            row.serial_number,
            row.dev_eui,
            row.app_eui,
            row.app_key,
            row.at_pin,
            row.ota_pin,
            row.config,
        )?;
        Ok(Sensor::reconstitute(SensorSnapshot {
            id: row.id,
            activated_at: row.activated_at.map(|t| t.and_utc()),
            sensor_type: row.sensor_type,
            model_id: row.model_id,
            provider: row.provider,
            additional_info: row.additional_info,
            lorawan,
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[SensorId]) -> Result<Vec<Sensor>, RepositoryError> {
        let ids: Vec<&str> = ids.iter().map(SensorId::as_str).collect();

        let rows = sqlx::query!(
            r#"SELECT s.id,
                      s.activated_at,
                      s.type          AS "sensor_type: SensorType",
                      s.model_id,
                      s.provider,
                      s.additional_informations AS "additional_info: serde_json::Value",
                      sl.serial_number AS "serial_number?",
                      sl.dev_eui       AS "dev_eui?",
                      sl.app_eui       AS "app_eui?",
                      sl.app_key       AS "app_key?",
                      sl.at_pin,
                      sl.ota_pin,
                      sl.config        AS "config: serde_json::Value"
            FROM sensors s
            LEFT JOIN sensor_lorawan sl ON sl.id = s.id
            WHERE s.id = ANY($1::text[])"#,
            &ids as &[&str]
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                let lorawan = build_lorawan(
                    r.serial_number,
                    r.dev_eui,
                    r.app_eui,
                    r.app_key,
                    r.at_pin,
                    r.ota_pin,
                    r.config,
                )?;
                Ok(Sensor::reconstitute(SensorSnapshot {
                    id: r.id,
                    activated_at: r.activated_at.map(|t| t.and_utc()),
                    sensor_type: r.sensor_type,
                    model_id: r.model_id,
                    provider: r.provider,
                    additional_info: r.additional_info,
                    lorawan,
                }))
            })
            .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: &SensorId) -> Result<SensorView, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT s.id, s.created_at, s.updated_at,
                      s.activated_at,
                      s.type          AS "sensor_type: SensorType",
                      s.provider,
                      s.additional_informations AS "additional_info: serde_json::Value",
                      sm.id           AS model_id,
                      sm.name         AS model_name,
                      t.id            AS "linked_tree_id?",
                      t.latitude      AS "tree_lat?",
                      t.longitude     AS "tree_lng?",
                      sl.serial_number AS "serial_number?",
                      sl.dev_eui       AS "dev_eui?",
                      sl.app_eui       AS "app_eui?",
                      sl.at_pin,
                      sl.ota_pin,
                      sl.config        AS "config: serde_json::Value"
            FROM sensors s
            INNER JOIN sensor_models sm ON sm.id = s.model_id
            LEFT JOIN sensor_lorawan sl ON sl.id = s.id
            LEFT JOIN trees t          ON t.sensor_id = s.id
            WHERE s.id = $1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let latest_reading = sqlx::query!(
            r#"SELECT id, sensor_id, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY id DESC LIMIT 1"#,
            id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| SensorReadingView {
            created_at: uuid_v7_timestamp(&r.id).expect("sensor_data.id is minted as uuid v7"),
            id: r.id,
            sensor_id: r.sensor_id,
            updated_at: r.updated_at.and_utc(),
            data: r.data,
        });

        let status = derive_connectivity(
            row.activated_at.map(|t| t.and_utc()),
            latest_reading.as_ref().map(|r| r.created_at),
            Utc::now(),
            self.offline_after,
        );

        Ok(SensorView {
            id: row.id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            status,
            sensor_type: row.sensor_type,
            coordinate: build_coord(row.tree_lat, row.tree_lng)?,
            linked_tree_id: row.linked_tree_id,
            provider: row.provider.map(ProviderId::reconstitute),
            additional_info: row.additional_info,
            model: SensorModelSummary {
                id: row.model_id,
                name: row.model_name,
            },
            lorawan: build_lorawan_info(
                row.serial_number,
                row.dev_eui,
                row.app_eui,
                row.at_pin,
                row.ota_pin,
                row.config,
            ),
            latest_reading,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[SensorId]) -> Result<Vec<SensorView>, RepositoryError> {
        let ids: Vec<&str> = ids.iter().map(SensorId::as_str).collect();

        let rows = sqlx::query!(
            r#"SELECT s.id, s.created_at, s.updated_at,
                      s.activated_at,
                      s.type          AS "sensor_type: SensorType",
                      s.provider,
                      s.additional_informations AS "additional_info: serde_json::Value",
                      sm.id           AS model_id,
                      sm.name         AS model_name,
                      t.id            AS "linked_tree_id?",
                      t.latitude      AS "tree_lat?",
                      t.longitude     AS "tree_lng?",
                      sl.serial_number AS "serial_number?",
                      sl.dev_eui       AS "dev_eui?",
                      sl.app_eui       AS "app_eui?",
                      sl.at_pin,
                      sl.ota_pin,
                      sl.config        AS "config: serde_json::Value",
                      lr.id            AS "last_reading_id?: Uuid",
                      lr.updated_at    AS "last_reading_updated_at?",
                      lr.data          AS "last_reading_data?"
            FROM sensors s
            INNER JOIN sensor_models sm ON sm.id = s.model_id
            LEFT JOIN sensor_lorawan sl ON sl.id = s.id
            LEFT JOIN trees t          ON t.sensor_id = s.id
            LEFT JOIN LATERAL (
                SELECT sd.id, sd.updated_at, sd.data
                FROM sensor_data sd
                WHERE sd.sensor_id = s.id
                ORDER BY sd.id DESC
                LIMIT 1
            ) lr ON true
            WHERE s.id = ANY($1::text[])"#,
            &ids as &[&str]
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                let latest_reading = build_latest_reading(
                    &r.id,
                    r.last_reading_id,
                    r.last_reading_updated_at,
                    r.last_reading_data,
                );
                let status = derive_connectivity(
                    r.activated_at.map(|t| t.and_utc()),
                    latest_reading.as_ref().map(|lr| lr.created_at),
                    Utc::now(),
                    self.offline_after,
                );
                Ok(SensorView {
                    id: r.id,
                    created_at: r.created_at.and_utc(),
                    updated_at: r.updated_at.and_utc(),
                    status,
                    sensor_type: r.sensor_type,
                    coordinate: build_coord(r.tree_lat, r.tree_lng)?,
                    linked_tree_id: r.linked_tree_id,
                    provider: r.provider.map(ProviderId::reconstitute),
                    additional_info: r.additional_info,
                    model: SensorModelSummary {
                        id: r.model_id,
                        name: r.model_name,
                    },
                    lorawan: build_lorawan_info(
                        r.serial_number,
                        r.dev_eui,
                        r.app_eui,
                        r.at_pin,
                        r.ota_pin,
                        r.config,
                    ),
                    latest_reading,
                })
            })
            .collect()
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
            r#"SELECT s.id, s.created_at, s.updated_at,
                      s.activated_at,
                      s.type          AS "sensor_type: SensorType",
                      s.provider,
                      s.additional_informations AS "additional_info: serde_json::Value",
                      sm.id           AS model_id,
                      sm.name         AS model_name,
                      t.id            AS "linked_tree_id?",
                      t.latitude      AS "tree_lat?",
                      t.longitude     AS "tree_lng?",
                      sl.serial_number AS "serial_number?",
                      sl.dev_eui       AS "dev_eui?",
                      sl.app_eui       AS "app_eui?",
                      sl.at_pin,
                      sl.ota_pin,
                      sl.config        AS "config: serde_json::Value",
                      lr.id            AS "last_reading_id?: Uuid",
                      lr.updated_at    AS "last_reading_updated_at?",
                      lr.data          AS "last_reading_data?"
            FROM sensors s
            INNER JOIN sensor_models sm ON sm.id = s.model_id
            LEFT JOIN sensor_lorawan sl ON sl.id = s.id
            LEFT JOIN trees t          ON t.sensor_id = s.id
            LEFT JOIN LATERAL (
                SELECT sd.id, sd.updated_at, sd.data
                FROM sensor_data sd
                WHERE sd.sensor_id = s.id
                ORDER BY sd.id DESC
                LIMIT 1
            ) lr ON true
            WHERE ($1::text IS NULL OR s.provider = $1)
            ORDER BY s.id
            LIMIT $2 OFFSET $3"#,
            provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| {
                let latest_reading = build_latest_reading(
                    &r.id,
                    r.last_reading_id,
                    r.last_reading_updated_at,
                    r.last_reading_data,
                );
                let status = derive_connectivity(
                    r.activated_at.map(|t| t.and_utc()),
                    latest_reading.as_ref().map(|lr| lr.created_at),
                    Utc::now(),
                    self.offline_after,
                );
                Ok(SensorView {
                    id: r.id,
                    created_at: r.created_at.and_utc(),
                    updated_at: r.updated_at.and_utc(),
                    status,
                    sensor_type: r.sensor_type,
                    coordinate: build_coord(r.tree_lat, r.tree_lng)?,
                    linked_tree_id: r.linked_tree_id,
                    provider: r.provider.map(ProviderId::reconstitute),
                    additional_info: r.additional_info,
                    model: SensorModelSummary {
                        id: r.model_id,
                        name: r.model_name,
                    },
                    lorawan: build_lorawan_info(
                        r.serial_number,
                        r.dev_eui,
                        r.app_eui,
                        r.at_pin,
                        r.ota_pin,
                        r.config,
                    ),
                    latest_reading,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()?;

        Ok(Page { items, total })
    }
}

#[async_trait]
impl SensorWriter for PgSensorRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: SensorDraft) -> Result<Sensor, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"INSERT INTO sensors (id, type, model_id, provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5)"#,
            draft.id.as_str(),
            draft.sensor_type as SensorType,
            draft.model_id.value(),
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info().cloned(),
        )
        .execute(&mut *tx)
        .await?;

        let lorawan = &draft.lorawan;
        sqlx::query!(
            r#"INSERT INTO sensor_lorawan
                (id, serial_number, dev_eui, app_eui, app_key, at_pin, ota_pin, config)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            draft.id.as_str(),
            lorawan.serial_number.as_str(),
            lorawan.dev_eui.as_str(),
            lorawan.app_eui.as_str(),
            secrecy::ExposeSecret::expose_secret(&lorawan.app_key),
            lorawan.at_pin,
            lorawan.ota_pin,
            lorawan.config,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.by_id(&draft.id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, sensor: &Sensor) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE sensors SET
                activated_at = $2,
                provider = $3,
                additional_informations = $4
            WHERE id = $1"#,
            sensor.id.as_str(),
            sensor.activated_at().map(|t| t.naive_utc()),
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
            r#"SELECT id, sensor_id, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY id DESC LIMIT $2"#,
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
            r#"SELECT id, sensor_id, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY id DESC LIMIT 1"#,
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
            r#"SELECT id, sensor_id, updated_at, data
            FROM sensor_data WHERE sensor_id = $1
            ORDER BY id DESC LIMIT $2"#,
            sensor_id.as_str(),
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SensorReadingView {
                created_at: uuid_v7_timestamp(&r.id).expect("sensor_data.id is minted as uuid v7"),
                id: r.id,
                sensor_id: r.sensor_id,
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
        let reading_id = Uuid::now_v7();
        sqlx::query!(
            r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, $3)"#,
            reading_id,
            draft.sensor_id.as_str(),
            draft.data,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn record_with_normalized(
        &self,
        sensor_id: &SensorId,
        raw: serde_json::Value,
        normalized: &[NormalizedValue],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let reading_id = Uuid::now_v7();
        sqlx::query!(
            r#"INSERT INTO sensor_data (id, sensor_id, data) VALUES ($1, $2, $3)"#,
            reading_id,
            sensor_id.as_str(),
            raw,
        )
        .execute(&mut *tx)
        .await?;

        if !normalized.is_empty() {
            let ability_ids: Vec<Uuid> = normalized.iter().map(|n| n.model_ability_id).collect();
            let values: Vec<Decimal> = normalized.iter().map(|n| n.value).collect();
            sqlx::query!(
                r#"INSERT INTO sensor_data_ability_values
                    (sensor_data_id, sensor_model_ability_id, value)
                SELECT $1, ability, val
                FROM UNNEST($2::uuid[], $3::numeric[]) AS t(ability, val)"#,
                reading_id,
                &ability_ids,
                &values as &[Decimal],
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

/// Builds [`LorawanCredentials`] from the joined row. Returns `Ok(None)` only
/// when the join missed (no `sensor_lorawan` row); otherwise validates the
/// non-empty fields and surfaces a [`RepositoryError::DataIntegrity`] on bad
/// data.
fn build_lorawan(
    serial_number: Option<String>,
    dev_eui: Option<String>,
    app_eui: Option<String>,
    app_key: Option<String>,
    at_pin: Option<String>,
    ota_pin: Option<String>,
    config: Option<serde_json::Value>,
) -> Result<Option<LorawanCredentials>, RepositoryError> {
    let (Some(serial_number), Some(dev_eui), Some(app_eui), Some(app_key)) =
        (serial_number, dev_eui, app_eui, app_key)
    else {
        return Ok(None);
    };
    Ok(Some(LorawanCredentials {
        serial_number: NonEmptyString::reconstitute(serial_number),
        dev_eui: NonEmptyString::reconstitute(dev_eui),
        app_eui: NonEmptyString::reconstitute(app_eui),
        app_key: secrecy::SecretString::from(app_key),
        at_pin,
        ota_pin,
        config,
    }))
}

fn build_lorawan_info(
    serial_number: Option<String>,
    dev_eui: Option<String>,
    app_eui: Option<String>,
    at_pin: Option<String>,
    ota_pin: Option<String>,
    config: Option<serde_json::Value>,
) -> Option<LorawanInfo> {
    let (Some(serial_number), Some(dev_eui), Some(app_eui)) = (serial_number, dev_eui, app_eui)
    else {
        return None;
    };
    Some(LorawanInfo {
        serial_number,
        dev_eui,
        app_eui,
        at_pin,
        ota_pin,
        config,
    })
}

/// Builds the embedded `latest_reading` from a LATERAL-joined newest row.
/// All three columns come from the same `LEFT JOIN LATERAL`, so they are
/// either all present (a reading exists) or all absent.
fn build_latest_reading(
    sensor_id: &str,
    id: Option<Uuid>,
    updated_at: Option<chrono::NaiveDateTime>,
    data: Option<serde_json::Value>,
) -> Option<SensorReadingView> {
    let (Some(id), Some(updated_at), Some(data)) = (id, updated_at, data) else {
        return None;
    };
    Some(SensorReadingView {
        created_at: uuid_v7_timestamp(&id).expect("sensor_data.id is minted as uuid v7"),
        id,
        sensor_id: sensor_id.to_owned(),
        updated_at: updated_at.and_utc(),
        data,
    })
}

fn build_coord(lat: Option<f64>, lng: Option<f64>) -> Result<Option<Coordinate>, RepositoryError> {
    let (Some(lat), Some(lng)) = (lat, lng) else {
        return Ok(None);
    };
    Ok(Some(Coordinate::new(lat, lng)?))
}
