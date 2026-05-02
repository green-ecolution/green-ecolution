use async_trait::async_trait;
use chrono::DateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    shared::pagination::{Page, Pagination},
    vehicle::{
        DrivingLicense, NumberPlate, Vehicle, VehicleDraft, VehicleReader, VehicleSearchQuery,
        VehicleSnapshot, VehicleStatus, VehicleType, VehicleView, VehicleWriter,
    },
};

pub struct PgVehicleRepository {
    pool: PgPool,
}

impl PgVehicleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleReader for PgVehicleRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, RepositoryError> {
        let snap = sqlx::query_as!(
            VehicleSnapshot,
            r#"SELECT id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info
            FROM vehicles WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Vehicle::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        let snaps = sqlx::query_as!(
            VehicleSnapshot,
            r#"SELECT id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info
            FROM vehicles WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(Vehicle::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_plate(&self, plate: &NumberPlate) -> Result<Option<Vehicle>, RepositoryError> {
        let snap = sqlx::query_as!(
            VehicleSnapshot,
            r#"SELECT id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info
            FROM vehicles WHERE number_plate = $1"#,
            plate.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snap.map(Vehicle::reconstitute))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<Vehicle>) -> Result<VehicleView, RepositoryError> {
        struct Row {
            id: i32,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
            archived_at: Option<chrono::NaiveDateTime>,
            number_plate: String,
            description: Option<String>,
            water_capacity: f64,
            vehicle_type: VehicleType,
            status: VehicleStatus,
            model: String,
            driving_license: DrivingLicense,
            height: f64,
            width: f64,
            length: f64,
            weight: f64,
            provider: Option<String>,
            additional_info: Option<Value>,
        }

        let row = sqlx::query_as!(
            Row,
            r#"SELECT id, created_at, updated_at,
                      archived_at,
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS "additional_info: Value"
            FROM vehicles WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(VehicleView {
            id: row.id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            archived_at: row.archived_at.map(|dt| dt.and_utc()),
            number_plate: row.number_plate,
            description: row.description,
            water_capacity: row.water_capacity,
            status: row.status,
            vehicle_type: row.vehicle_type,
            model: row.model,
            driving_license: row.driving_license,
            height: row.height,
            width: row.width,
            length: row.length,
            weight: row.weight,
            provider: row.provider,
            additional_info: row.additional_info,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<VehicleView>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();

        struct Row {
            id: i32,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
            archived_at: Option<chrono::NaiveDateTime>,
            number_plate: String,
            description: Option<String>,
            water_capacity: f64,
            vehicle_type: VehicleType,
            status: VehicleStatus,
            model: String,
            driving_license: DrivingLicense,
            height: f64,
            width: f64,
            length: f64,
            weight: f64,
            provider: Option<String>,
            additional_info: Option<Value>,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"SELECT id, created_at, updated_at,
                      archived_at,
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS "additional_info: Value"
            FROM vehicles WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| VehicleView {
                id: row.id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                archived_at: row.archived_at.map(|dt| dt.and_utc()),
                number_plate: row.number_plate,
                description: row.description,
                water_capacity: row.water_capacity,
                status: row.status,
                vehicle_type: row.vehicle_type,
                model: row.model,
                driving_license: row.driving_license,
                height: row.height,
                width: row.width,
                length: row.length,
                weight: row.weight,
                provider: row.provider,
                additional_info: row.additional_info,
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: VehicleSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<VehicleView>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);
        let provider = query.provider.as_ref().map(|p| p.as_str().to_owned());

        struct Row {
            id: i32,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
            archived_at: Option<chrono::NaiveDateTime>,
            number_plate: String,
            description: Option<String>,
            water_capacity: f64,
            vehicle_type: VehicleType,
            status: VehicleStatus,
            model: String,
            driving_license: DrivingLicense,
            height: f64,
            width: f64,
            length: f64,
            weight: f64,
            provider: Option<String>,
            additional_info: Option<Value>,
        }

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM vehicles
            WHERE ($1::text IS NULL OR provider = $1)
              AND ($2::vehicle_type IS NULL OR type = $2)
              AND ($3::bool OR archived_at IS NULL)
              AND (NOT $4::bool OR archived_at IS NOT NULL)"#,
            provider,
            query.vehicle_type as Option<VehicleType>,
            query.with_archived,
            query.only_archived,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            Row,
            r#"SELECT id, created_at, updated_at,
                      archived_at,
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS "additional_info: Value"
            FROM vehicles
            WHERE ($1::text IS NULL OR provider = $1)
              AND ($2::vehicle_type IS NULL OR type = $2)
              AND ($3::bool OR archived_at IS NULL)
              AND (NOT $4::bool OR archived_at IS NOT NULL)
            ORDER BY water_capacity DESC
            LIMIT $5 OFFSET $6"#,
            provider,
            query.vehicle_type as Option<VehicleType>,
            query.with_archived,
            query.only_archived,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|row| VehicleView {
                id: row.id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                archived_at: row.archived_at.map(|dt| dt.and_utc()),
                number_plate: row.number_plate,
                description: row.description,
                water_capacity: row.water_capacity,
                status: row.status,
                vehicle_type: row.vehicle_type,
                model: row.model,
                driving_license: row.driving_license,
                height: row.height,
                width: row.width,
                length: row.length,
                weight: row.weight,
                provider: row.provider,
                additional_info: row.additional_info,
            })
            .collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_type(
        &self,
        vehicle_type: VehicleType,
        pagination: Pagination,
    ) -> Result<Page<VehicleView>, RepositoryError> {
        let query = VehicleSearchQuery {
            vehicle_type: Some(vehicle_type),
            ..Default::default()
        };
        self.view_search(query, pagination).await
    }
}

#[async_trait]
impl VehicleWriter for PgVehicleRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: VehicleDraft) -> Result<Vehicle, RepositoryError> {
        let snap = sqlx::query_as!(
            VehicleSnapshot,
            r#"INSERT INTO vehicles (number_plate, description, water_capacity, type, status,
                                     model, driving_license, height, length, width, weight,
                                     provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info"#,
            draft.number_plate.as_str(),
            draft.description,
            draft.water_capacity.liters(),
            draft.vehicle_type as VehicleType,
            draft.status as VehicleStatus,
            draft.model.as_str(),
            draft.driving_license as DrivingLicense,
            draft.dimension.height,
            draft.dimension.length,
            draft.dimension.width,
            draft.dimension.weight,
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Vehicle::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, vehicle: &Vehicle) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE vehicles SET
                number_plate = $2,
                description = $3,
                water_capacity = $4,
                type = $5,
                status = $6,
                model = $7,
                driving_license = $8,
                height = $9,
                length = $10,
                width = $11,
                weight = $12,
                provider = $13,
                additional_informations = $14,
                archived_at = $15
            WHERE id = $1"#,
            vehicle.id.value(),
            vehicle.number_plate.as_str(),
            vehicle.description.as_deref(),
            vehicle.water_capacity.liters(),
            vehicle.vehicle_type as VehicleType,
            vehicle.status as VehicleStatus,
            vehicle.model.as_str(),
            vehicle.driving_license as DrivingLicense,
            vehicle.dimension.height,
            vehicle.dimension.length,
            vehicle.dimension.width,
            vehicle.dimension.weight,
            vehicle.provenance().provider().map(|p| p.as_str()),
            vehicle.provenance().additional_info(),
            vehicle.archived_at().map(|dt| dt.naive_utc()),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Vehicle>) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM vehicles WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
