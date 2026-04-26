use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    shared::{
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        water_capacity::WaterCapacity,
    },
    vehicle::{
        DrivingLicense, Vehicle, VehicleCreate, VehicleDimension, VehicleQuery, VehicleRepository,
        VehicleStatus, VehicleType, VehicleUpdate,
    },
};

struct VehicleRow {
    id: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    number_plate: String,
    description: String,
    water_capacity: f64,
    model: String,
    archived_at: Option<NaiveDateTime>,
    vehicle_type: VehicleType,
    status: VehicleStatus,
    driving_license: DrivingLicense,
    height: f64,
    length: f64,
    width: f64,
    weight: f64,
    provider: Option<String>,
    additional_informations: Option<Value>,
}

impl TryFrom<VehicleRow> for Vehicle {
    type Error = RepositoryError;

    fn try_from(row: VehicleRow) -> Result<Self, Self::Error> {
        Ok(Vehicle {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            archived_at: row.archived_at.map(|dt| dt.and_utc()),
            number_plate: row.number_plate,
            description: Some(row.description),
            water_capacity: WaterCapacity::new(row.water_capacity)?,
            status: row.status,
            vehicle_type: row.vehicle_type,
            model: row.model,
            driving_license: row.driving_license,
            dimension: VehicleDimension {
                height: row.height,
                width: row.width,
                length: row.length,
                weight: row.weight,
            },
            provider_info: ProviderInfo {
                provider: row.provider.unwrap_or_default(),
                additional_info: row.additional_informations.unwrap_or_default(),
            },
        })
    }
}

pub struct PgVehicleRepository {
    pool: PgPool,
}

impl PgVehicleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl VehicleRepository for PgVehicleRepository {
    async fn all(
        &self,
        query: VehicleQuery,
        pagination: Pagination,
    ) -> Result<Page<Vehicle>, RepositoryError> {
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM vehicles
            WHERE ($1::text IS NULL OR provider = $1)
              AND ($2::vehicle_type IS NULL OR type = $2)
              AND ($3::bool OR archived_at IS NULL)
              AND (NOT $4::bool OR archived_at IS NOT NULL)"#,
            query.provider,
            query.vehicle_type as Option<VehicleType>,
            query.with_archived,
            query.only_archived,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query_as!(
            VehicleRow,
            r#"SELECT id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations
            FROM vehicles
            WHERE ($1::text IS NULL OR provider = $1)
              AND ($2::vehicle_type IS NULL OR type = $2)
              AND ($3::bool OR archived_at IS NULL)
              AND (NOT $4::bool OR archived_at IS NOT NULL)
            ORDER BY water_capacity DESC
            LIMIT $5 OFFSET $6"#,
            query.provider,
            query.vehicle_type as Option<VehicleType>,
            query.with_archived,
            query.only_archived,
            pagination.limit as i64,
            pagination.offset as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(Vehicle::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page { items, total })
    }

    async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, RepositoryError> {
        sqlx::query_as!(
            VehicleRow,
            r#"SELECT id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations
            FROM vehicles WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        sqlx::query_as!(
            VehicleRow,
            r#"SELECT id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations
            FROM vehicles WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Vehicle::try_from)
        .collect()
    }

    async fn by_plate(&self, plate: &str) -> Result<Vehicle, RepositoryError> {
        sqlx::query_as!(
            VehicleRow,
            r#"SELECT id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations
            FROM vehicles WHERE number_plate = $1"#,
            plate
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    async fn create(&self, entity: VehicleCreate) -> Result<Vehicle, RepositoryError> {
        sqlx::query_as!(
            VehicleRow,
            r#"INSERT INTO vehicles (number_plate, description, water_capacity, type, status,
                                     model, driving_license, height, length, width, weight,
                                     provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations"#,
            entity.number_plate,
            entity.description,
            entity.water_capacity.liters(),
            entity.vehicle_type as VehicleType,
            entity.status as VehicleStatus,
            entity.model,
            entity.driving_license as DrivingLicense,
            entity.dimension.height,
            entity.dimension.length,
            entity.dimension.width,
            entity.dimension.weight,
            entity.provider_info.provider,
            entity.provider_info.additional_info,
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()
    }

    async fn update(
        &self,
        id: Id<Vehicle>,
        entity: VehicleUpdate,
    ) -> Result<Vehicle, RepositoryError> {
        sqlx::query_as!(
            VehicleRow,
            r#"UPDATE vehicles SET
                number_plate = COALESCE($2, number_plate),
                description = COALESCE($3, description),
                water_capacity = COALESCE($4, water_capacity),
                type = COALESCE($5, type),
                status = COALESCE($6, status),
                model = COALESCE($7, model),
                driving_license = COALESCE($8, driving_license),
                height = COALESCE($9, height),
                length = COALESCE($10, length),
                width = COALESCE($11, width),
                weight = COALESCE($12, weight),
                provider = COALESCE($13, provider),
                additional_informations = COALESCE($14, additional_informations)
            WHERE id = $1
            RETURNING id, created_at, updated_at, number_plate, description,
                      water_capacity, model, archived_at,
                      type AS "vehicle_type: VehicleType",
                      status AS "status: VehicleStatus",
                      driving_license AS "driving_license: DrivingLicense",
                      height, length, width, weight,
                      provider, additional_informations"#,
            id.value(),
            entity.number_plate,
            entity.description,
            entity.water_capacity.map(|wc| wc.liters()),
            entity.vehicle_type as Option<VehicleType>,
            entity.status as Option<VehicleStatus>,
            entity.model,
            entity.driving_license as Option<DrivingLicense>,
            entity.dimension.map(|d| d.height),
            entity.dimension.map(|d| d.length),
            entity.dimension.map(|d| d.width),
            entity.dimension.map(|d| d.weight),
            entity.provider_info.as_ref().map(|p| p.provider.as_str()),
            entity
                .provider_info
                .as_ref()
                .map(|p| p.additional_info.clone()),
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    async fn archive(&self, id: Id<Vehicle>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE vehicles SET archived_at = NOW() WHERE id = $1",
            id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Id<Vehicle>) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM vehicles WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
