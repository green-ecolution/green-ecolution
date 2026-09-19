use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime};
use serde_json::Value;
use sqlx::PgPool;

use crate::infra::list::{ArchiveFilter, ListSpec, Predicate, SortColumns};
use domain::{
    Id, IdSliceExt, RawId, RepositoryError,
    authorization::Visibility,
    shared::pagination::{Page, Pagination, SearchPage},
    vehicle::{
        ArchiveFilterKind, DrivingLicense, NumberPlate, Vehicle, VehicleAvailability, VehicleDraft,
        VehicleReader, VehicleSearchQuery, VehicleSnapshot, VehicleStatus, VehicleType,
        VehicleView, VehicleWriter, derive_status,
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

/// Flat row shape shared by every `view_*` query on `vehicles`. The `From`
/// impl derives `created_at` from the UUID v7 id and the visible status from
/// the availability plus the `on_active_plan` flag each query computes.
#[derive(sqlx::FromRow)]
struct VehicleViewRow {
    id: RawId,
    updated_at: NaiveDateTime,
    archived_at: Option<NaiveDateTime>,
    number_plate: String,
    description: Option<String>,
    water_capacity: f64,
    vehicle_type: VehicleType,
    availability: VehicleAvailability,
    model: String,
    driving_license: DrivingLicense,
    height: f64,
    width: f64,
    length: f64,
    weight: f64,
    provider: Option<String>,
    additional_info: Option<Value>,
    organization_id: RawId,
    on_active_plan: bool,
}

impl From<VehicleViewRow> for VehicleView {
    fn from(row: VehicleViewRow) -> Self {
        let created_at = Id::<Vehicle>::new(row.id)
            .created_at()
            .expect("vehicles.id is minted as uuid v7");
        Self {
            id: row.id,
            created_at,
            updated_at: row.updated_at.and_utc(),
            archived_at: row.archived_at.map(|dt| dt.and_utc()),
            number_plate: row.number_plate,
            description: row.description,
            water_capacity: row.water_capacity,
            availability: row.availability,
            status: derive_status(row.availability, row.on_active_plan),
            vehicle_type: row.vehicle_type,
            model: row.model,
            driving_license: row.driving_license,
            height: row.height,
            width: row.width,
            length: row.length,
            weight: row.weight,
            provider: row.provider,
            additional_info: row.additional_info,
            organization_id: row.organization_id,
        }
    }
}

const VEHICLE_COLUMNS: &str = "v.id, v.updated_at, v.archived_at, v.number_plate, \
    v.description, v.water_capacity, v.type AS vehicle_type, v.availability, \
    v.model, v.driving_license, v.height, v.width, v.length, v.weight, \
    v.provider, v.additional_informations AS additional_info, v.organization_id, \
    EXISTS (SELECT 1 FROM vehicle_watering_plans vwp \
            JOIN watering_plans wp ON wp.id = vwp.watering_plan_id \
            WHERE vwp.vehicle_id = v.id AND wp.status = 'active') AS on_active_plan";

// Availability beats a running plan, the same precedence vehicle::derive_status
// applies. Kept as one expression so WHERE, ORDER BY and the projection cannot
// drift apart.
const VEHICLE_STATUS_SQL: &str = "CASE \
    WHEN v.availability = 'not_available' THEN 'not_available' \
    WHEN EXISTS (SELECT 1 FROM vehicle_watering_plans vwp \
                 JOIN watering_plans wp ON wp.id = vwp.watering_plan_id \
                 WHERE vwp.vehicle_id = v.id AND wp.status = 'active') THEN 'active' \
    ELSE 'available' END";

const VEHICLE_SORT_COLUMNS: SortColumns = &[
    ("number_plate", &["v.number_plate"]),
    ("water_capacity", &["v.water_capacity"]),
    ("model", &["v.model"]),
    ("type", &["v.type"]),
];

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
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info,
                      organization_id
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
        let id_values: Vec<RawId> = ids.to_values();
        let snaps = sqlx::query_as!(
            VehicleSnapshot,
            r#"SELECT id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info,
                      organization_id
            FROM vehicles WHERE id = ANY($1::uuid[])"#,
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
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info,
                      organization_id
            FROM vehicles WHERE number_plate = $1"#,
            plate.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snap.map(Vehicle::reconstitute))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<Vehicle>) -> Result<VehicleView, RepositoryError> {
        let row = sqlx::query_as!(
            VehicleViewRow,
            r#"SELECT id, updated_at,
                      archived_at,
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS "additional_info: Value",
                      organization_id,
                      EXISTS (SELECT 1 FROM vehicle_watering_plans vwp
                              JOIN watering_plans wp ON wp.id = vwp.watering_plan_id
                              WHERE vwp.vehicle_id = vehicles.id
                                AND wp.status = 'active') AS "on_active_plan!: bool"
            FROM vehicles WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(row.into())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<VehicleView>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let rows = sqlx::query_as!(
            VehicleViewRow,
            r#"SELECT id, updated_at,
                      archived_at,
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS "additional_info: Value",
                      organization_id,
                      EXISTS (SELECT 1 FROM vehicle_watering_plans vwp
                              JOIN watering_plans wp ON wp.id = vwp.watering_plan_id
                              WHERE vwp.vehicle_id = vehicles.id
                                AND wp.status = 'active') AS "on_active_plan!: bool"
            FROM vehicles WHERE id = ANY($1::uuid[])"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: VehicleSearchQuery,
        pagination: Pagination,
    ) -> Result<SearchPage<VehicleView>, RepositoryError> {
        let archive = match query.archive {
            ArchiveFilterKind::ActiveOnly => ArchiveFilter::ActiveOnly,
            ArchiveFilterKind::Include => ArchiveFilter::Include,
            ArchiveFilterKind::ArchivedOnly => ArchiveFilter::ArchivedOnly,
        };

        let status_texts: Vec<String> = query
            .statuses
            .iter()
            .map(|status| match status {
                VehicleStatus::Active => "active".to_owned(),
                VehicleStatus::Available => "available".to_owned(),
                VehicleStatus::NotAvailable => "not_available".to_owned(),
            })
            .collect();

        let page = ListSpec::new("vehicles v", "v.id")
            .scope(Predicate::equals(
                "v.provider",
                query.provider.as_ref().map(|p| p.as_str().to_owned()),
            ))
            .scope(Predicate::any_of_opt(
                "v.organization_id",
                query.visible.into_raw_ids(),
            ))
            .filter(Predicate::text_search(
                &["v.number_plate", "v.model", "v.description"],
                query.q,
            ))
            .filter(Predicate::any_of(VEHICLE_STATUS_SQL, status_texts))
            .filter(Predicate::any_of("v.type", query.types))
            .filter(Predicate::any_of(
                "v.driving_license",
                query.driving_licenses,
            ))
            .filter(Predicate::equals("v.type", query.vehicle_type))
            .filter(Predicate::archived("v.archived_at", archive))
            .sort(
                query.sort.field.as_sql_key(),
                query.sort.direction.is_descending(),
                VEHICLE_SORT_COLUMNS,
            )
            .page(pagination)
            .fetch::<VehicleViewRow>(&self.pool, VEHICLE_COLUMNS)
            .await?;

        Ok(SearchPage {
            page: Page {
                items: page.page.items.into_iter().map(Into::into).collect(),
                total: page.page.total,
            },
            total_unfiltered: page.total_unfiltered,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_type(
        &self,
        vehicle_type: VehicleType,
        pagination: Pagination,
        visible: Visibility,
    ) -> Result<Page<VehicleView>, RepositoryError> {
        let query = VehicleSearchQuery {
            vehicle_type: Some(vehicle_type),
            visible,
            ..Default::default()
        };
        Ok(self.view_search(query, pagination).await?.page)
    }
}

#[async_trait]
impl VehicleWriter for PgVehicleRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: VehicleDraft) -> Result<Vehicle, RepositoryError> {
        let id = Id::<Vehicle>::new_v7();
        let snap = sqlx::query_as!(
            VehicleSnapshot,
            r#"INSERT INTO vehicles (id, number_plate, description, water_capacity, type, availability,
                                     model, driving_license, height, length, width, weight,
                                     provider, additional_informations, organization_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id,
                      archived_at AS "archived_at: DateTime<chrono::Utc>",
                      number_plate,
                      description,
                      water_capacity,
                      type AS "vehicle_type: VehicleType",
                      availability AS "availability: VehicleAvailability",
                      model,
                      driving_license AS "driving_license: DrivingLicense",
                      height, width, length, weight,
                      provider,
                      additional_informations AS additional_info,
                      organization_id"#,
            id.value(),
            draft.number_plate.as_str(),
            draft.description,
            draft.water_capacity.liters(),
            draft.vehicle_type as VehicleType,
            draft.availability as VehicleAvailability,
            draft.model.as_str(),
            draft.driving_license as DrivingLicense,
            draft.dimension.height,
            draft.dimension.length,
            draft.dimension.width,
            draft.dimension.weight,
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info(),
            draft.organization_id.value(),
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
                availability = $6,
                model = $7,
                driving_license = $8,
                height = $9,
                length = $10,
                width = $11,
                weight = $12,
                provider = $13,
                additional_informations = $14,
                archived_at = $15,
                organization_id = $16
            WHERE id = $1"#,
            vehicle.id.value(),
            vehicle.number_plate.as_str(),
            vehicle.description.as_deref(),
            vehicle.water_capacity.liters(),
            vehicle.vehicle_type as VehicleType,
            vehicle.availability as VehicleAvailability,
            vehicle.model.as_str(),
            vehicle.driving_license as DrivingLicense,
            vehicle.dimension.height,
            vehicle.dimension.length,
            vehicle.dimension.width,
            vehicle.dimension.weight,
            vehicle.provenance().provider().map(|p| p.as_str()),
            vehicle.provenance().additional_info(),
            vehicle.archived_at().map(|dt| dt.naive_utc()),
            vehicle.organization_id().value(),
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
