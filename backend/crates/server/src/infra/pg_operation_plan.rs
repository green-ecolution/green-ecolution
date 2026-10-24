use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::Value;
use sqlx::PgPool;

use domain::{
    Id, RawId, RepositoryError,
    module::ModuleKey,
    operation_plan::{
        OperationPlan, OperationPlanDraft, OperationPlanReader, OperationPlanSearchQuery,
        OperationPlanSnapshot, OperationPlanStatus, OperationPlanView, OperationPlanWriter,
        StationVisit,
    },
    planning::{StopOutcome, StopRef},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    start_point::StartPointName,
};

use crate::infra::list::{ListSpec, Predicate, SortColumns};

pub struct PgOperationPlanRepository {
    pool: PgPool,
}

impl PgOperationPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Flat row shape shared by `view_by_id` and `view_search` on
/// `operation_plans`. The conversion derives `created_at` from the UUID v7 id;
/// transporter/trailer come from the role column on the vehicle join table.
#[derive(sqlx::FromRow)]
struct OperationPlanViewRow {
    id: RawId,
    module_key: String,
    updated_at: NaiveDateTime,
    date: NaiveDate,
    description: String,
    start_point_name: Option<String>,
    status: OperationPlanStatus,
    distance: Option<f64>,
    total_demand: Option<f64>,
    cancellation_note: String,
    refill_count: i32,
    duration: f64,
    provider: Option<String>,
    additional_informations: Option<Value>,
    transporter_id: Option<RawId>,
    trailer_id: Option<RawId>,
    transporter_capacity_used: bool,
    stops: Vec<RawId>,
    user_ids: Vec<RawId>,
    organization_id: RawId,
}

impl TryFrom<OperationPlanViewRow> for OperationPlanView {
    type Error = RepositoryError;

    fn try_from(row: OperationPlanViewRow) -> Result<Self, Self::Error> {
        let created_at = Id::<OperationPlan>::new(row.id)
            .created_at()
            .expect("operation_plans.id is minted as uuid v7");
        Ok(Self {
            id: row.id,
            module: ModuleKey::new(&row.module_key)?,
            created_at,
            updated_at: row.updated_at.and_utc(),
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description).filter(|s| !s.is_empty()),
            start_point_name: row.start_point_name,
            status: row.status,
            distance: row.distance,
            total_demand: row.total_demand,
            stops: row.stops,
            user_ids: row.user_ids,
            transporter_id: row.transporter_id,
            trailer_id: row.trailer_id,
            transporter_capacity_used: row.transporter_capacity_used,
            cancellation_note: Some(row.cancellation_note).filter(|s| !s.is_empty()),
            refill_count: row.refill_count,
            duration: std::time::Duration::from_secs_f64(row.duration),
            provider: row.provider,
            additional_info: row.additional_informations,
            organization_id: row.organization_id,
        })
    }
}

const OPERATION_PLAN_COLUMNS: &str = "op.id, op.module_key, op.updated_at, op.date, \
    op.description, op.start_point_name, op.status, op.distance, op.total_demand, \
    op.cancellation_note, op.refill_count, op.duration, op.provider, \
    op.additional_informations, op.organization_id, op.transporter_capacity_used, \
    (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'transporter'))[1] AS transporter_id, \
    (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'trailer'))[1] AS trailer_id, \
    COALESCE(ARRAY_AGG(DISTINCT ops.stop_id) FILTER (WHERE ops.stop_id IS NOT NULL), ARRAY[]::uuid[]) AS stops, \
    COALESCE(ARRAY_AGG(DISTINCT opu.user_id) FILTER (WHERE opu.user_id IS NOT NULL), ARRAY[]::uuid[]) AS user_ids";

const OPERATION_PLAN_SORT_COLUMNS: SortColumns = &[("date", &["op.date"])];

#[async_trait]
impl OperationPlanReader for PgOperationPlanRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<OperationPlan>) -> Result<OperationPlan, RepositoryError> {
        struct Row {
            id: RawId,
            module_key: String,
            date: chrono::NaiveDate,
            description: String,
            start_point_name: Option<String>,
            status: OperationPlanStatus,
            distance: Option<f64>,
            total_demand: Option<f64>,
            cancellation_note: String,
            refill_count: i32,
            duration: f64,
            provider: Option<String>,
            additional_informations: Option<Value>,
            transporter_id: Option<RawId>,
            trailer_id: Option<RawId>,
            transporter_capacity_used: bool,
            stops: Vec<RawId>,
            user_ids: Vec<RawId>,
            route_geometry: Option<String>,
            organization_id: RawId,
        }

        let row = sqlx::query_as!(
            Row,
            r#"SELECT op.id, op.module_key, op.date, op.description, op.start_point_name,
                      op.status AS "status: OperationPlanStatus",
                      op.distance, op.total_demand, op.cancellation_note,
                      op.refill_count, op.duration,
                      op.provider, op.additional_informations,
                      op.organization_id, op.transporter_capacity_used,
                      ST_AsGeoJSON(op.route_geometry) AS route_geometry,
                      (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'transporter'))[1] AS "transporter_id: RawId",
                      (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'trailer'))[1] AS "trailer_id: RawId",
                      COALESCE(ARRAY_AGG(DISTINCT ops.stop_id) FILTER (WHERE ops.stop_id IS NOT NULL), ARRAY[]::uuid[]) AS "stops!: Vec<RawId>",
                      COALESCE(ARRAY_AGG(DISTINCT opu.user_id) FILTER (WHERE opu.user_id IS NOT NULL), ARRAY[]::uuid[]) AS "user_ids!: Vec<RawId>"
            FROM operation_plans op
            LEFT JOIN operation_plan_vehicles opv ON opv.plan_id = op.id
            LEFT JOIN operation_plan_stops ops ON ops.plan_id = op.id
            LEFT JOIN operation_plan_users opu ON opu.plan_id = op.id
            WHERE op.id = $1
            GROUP BY op.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        struct StationRow {
            name: String,
            latitude: f64,
            longitude: f64,
        }
        let station_visits = sqlx::query_as!(
            StationRow,
            r#"SELECT name, latitude, longitude FROM operation_plan_station_visits
               WHERE plan_id = $1 ORDER BY position"#,
            id.value()
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            Ok(StationVisit {
                name: StartPointName::reconstitute(r.name),
                coordinate: Coordinate::new(r.latitude, r.longitude).map_err(|e| {
                    RepositoryError::DataIntegrity(format!("invalid station visit: {e}"))
                })?,
            })
        })
        .collect::<Result<Vec<_>, RepositoryError>>()?;

        OperationPlan::reconstitute(OperationPlanSnapshot {
            id: row.id,
            module: row.module_key,
            date: row.date.and_time(NaiveTime::MIN).and_utc(),
            description: Some(row.description).filter(|s| !s.is_empty()),
            start_point_name: row.start_point_name,
            status: row.status,
            distance: row.distance,
            total_demand: row.total_demand,
            stops: row.stops,
            user_ids: row.user_ids,
            transporter_id: row.transporter_id,
            trailer_id: row.trailer_id,
            transporter_capacity_used: row.transporter_capacity_used,
            cancellation_note: Some(row.cancellation_note).filter(|s| !s.is_empty()),
            refill_count: row.refill_count,
            duration: std::time::Duration::from_secs_f64(row.duration),
            provider: row.provider,
            additional_info: row.additional_informations,
            route_geometry: geojson_to_geometry(row.route_geometry)?,
            station_visits,
            organization_id: row.organization_id,
        })
        .map_err(|e| RepositoryError::DataIntegrity(format!("invalid operation plan: {e}")))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(
        &self,
        id: Id<OperationPlan>,
    ) -> Result<OperationPlanView, RepositoryError> {
        let row = sqlx::query_as!(
            OperationPlanViewRow,
            r#"SELECT op.id, op.module_key, op.updated_at, op.date, op.description, op.start_point_name,
                      op.status AS "status: OperationPlanStatus",
                      op.distance, op.total_demand, op.cancellation_note,
                      op.refill_count, op.duration,
                      op.provider, op.additional_informations,
                      op.organization_id, op.transporter_capacity_used,
                      (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'transporter'))[1] AS "transporter_id: RawId",
                      (ARRAY_AGG(opv.vehicle_id) FILTER (WHERE opv.role = 'trailer'))[1] AS "trailer_id: RawId",
                      COALESCE(ARRAY_AGG(DISTINCT ops.stop_id) FILTER (WHERE ops.stop_id IS NOT NULL), ARRAY[]::uuid[]) AS "stops!: Vec<RawId>",
                      COALESCE(ARRAY_AGG(DISTINCT opu.user_id) FILTER (WHERE opu.user_id IS NOT NULL), ARRAY[]::uuid[]) AS "user_ids!: Vec<RawId>"
            FROM operation_plans op
            LEFT JOIN operation_plan_vehicles opv ON opv.plan_id = op.id
            LEFT JOIN operation_plan_stops ops ON ops.plan_id = op.id
            LEFT JOIN operation_plan_users opu ON opu.plan_id = op.id
            WHERE op.id = $1
            GROUP BY op.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        row.try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: OperationPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<OperationPlanView>, RepositoryError> {
        let page = ListSpec::new("operation_plans op", "op.id")
            .projection_join("LEFT JOIN operation_plan_vehicles opv ON opv.plan_id = op.id")
            .projection_join("LEFT JOIN operation_plan_stops ops ON ops.plan_id = op.id")
            .projection_join("LEFT JOIN operation_plan_users opu ON opu.plan_id = op.id")
            .group_by("op.id")
            .scope(Predicate::equals(
                "op.module_key",
                Some(query.module.as_str().to_owned()),
            ))
            .scope(Predicate::equals(
                "op.provider",
                query.provider.as_ref().map(|p| p.as_str().to_owned()),
            ))
            .scope(Predicate::any_of_opt(
                "op.organization_id",
                query.visible.into_raw_ids(),
            ))
            .filter(Predicate::any_of("op.status", query.statuses))
            .sort("date", true, OPERATION_PLAN_SORT_COLUMNS)
            .page(pagination)
            .fetch::<OperationPlanViewRow>(&self.pool, OPERATION_PLAN_COLUMNS)
            .await?;

        Ok(Page {
            items: page
                .page
                .items
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            total: page.page.total,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn outcomes(&self, id: Id<OperationPlan>) -> Result<Vec<StopOutcome>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT stop_id, completed, amount
            FROM operation_plan_stops
            WHERE plan_id = $1"#,
            id.value()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| StopOutcome {
                stop: StopRef::new(row.stop_id),
                completed: row.completed,
                amount: row.amount,
            })
            .collect())
    }
}

/// GeoJSON positions are (lon, lat); a LineString needs at least two points,
/// so shorter tracks collapse to None ("no route").
fn geometry_to_geojson(geometry: Option<&[Coordinate]>) -> Option<String> {
    let coords = geometry.filter(|c| c.len() >= 2)?;
    let coordinates: Vec<[f64; 2]> = coords
        .iter()
        .map(|c| [c.longitude(), c.latitude()])
        .collect();
    Some(serde_json::json!({"type": "LineString", "coordinates": coordinates}).to_string())
}

fn geojson_to_geometry(value: Option<String>) -> Result<Option<Vec<Coordinate>>, RepositoryError> {
    #[derive(serde::Deserialize)]
    struct LineString {
        coordinates: Vec<[f64; 2]>,
    }

    let Some(value) = value else { return Ok(None) };
    let line: LineString = serde_json::from_str(&value)
        .map_err(|e| RepositoryError::DataIntegrity(format!("invalid route_geometry: {e}")))?;
    let coords = line
        .coordinates
        .into_iter()
        .map(|[lon, lat]| {
            Coordinate::new(lat, lon)
                .map_err(|e| RepositoryError::DataIntegrity(format!("invalid route_geometry: {e}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Empty geometry means "no route" — collapse to None so readers get one representation.
    Ok(if coords.is_empty() {
        None
    } else {
        Some(coords)
    })
}

fn stop_values(stops: &[StopRef]) -> Vec<RawId> {
    stops.iter().map(StopRef::value).collect()
}

/// Persists the plan row and syncs the join tables inside the caller's
/// transaction. Stop rows are diffed rather than rewritten so surviving rows
/// keep their recorded outcome.
async fn persist_plan(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    plan: &OperationPlan,
) -> Result<(), RepositoryError> {
    let result = sqlx::query!(
        r#"UPDATE operation_plans SET
            date = $2,
            description = $3,
            status = $4,
            cancellation_note = $5,
            distance = $6,
            total_demand = $7,
            refill_count = $8,
            duration = $9,
            provider = $10,
            additional_informations = $11,
            route_geometry = ST_GeomFromGeoJSON($12::text),
            start_point_name = $13,
            transporter_capacity_used = $14
        WHERE id = $1"#,
        plan.id.value(),
        plan.date.date_naive(),
        plan.description.as_deref().unwrap_or(""),
        plan.status() as OperationPlanStatus,
        plan.cancellation_note().unwrap_or(""),
        plan.distance.as_ref().map(|d| d.meters()),
        plan.total_demand,
        plan.refill_count as i32,
        plan.duration.as_secs_f64(),
        plan.provenance().provider().map(|p| p.as_str()),
        plan.provenance().additional_info(),
        geometry_to_geojson(plan.route_geometry()) as Option<String>,
        plan.start_point_name.as_deref(),
        plan.transporter_capacity_used(),
    )
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }

    sqlx::query!(
        "DELETE FROM operation_plan_station_visits WHERE plan_id = $1",
        plan.id.value()
    )
    .execute(&mut **tx)
    .await?;

    let station_visits = plan.station_visits();
    if !station_visits.is_empty() {
        let names: Vec<String> = station_visits
            .iter()
            .map(|p| p.name.as_str().to_owned())
            .collect();
        let lats: Vec<f64> = station_visits
            .iter()
            .map(|p| p.coordinate.latitude())
            .collect();
        let lons: Vec<f64> = station_visits
            .iter()
            .map(|p| p.coordinate.longitude())
            .collect();
        sqlx::query!(
            r#"INSERT INTO operation_plan_station_visits
               (plan_id, position, name, latitude, longitude, geometry)
               SELECT $1, (ord - 1)::int, name, lat, lon,
                      ST_SetSRID(ST_MakePoint(lon, lat), 4326)
               FROM UNNEST($2::text[], $3::float8[], $4::float8[])
                    WITH ORDINALITY AS t(name, lat, lon, ord)"#,
            plan.id.value(),
            &names,
            &lats,
            &lons,
        )
        .execute(&mut **tx)
        .await?;
    }

    sqlx::query!(
        "DELETE FROM operation_plan_vehicles WHERE plan_id = $1",
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

    sqlx::query!(
        "DELETE FROM operation_plan_users WHERE plan_id = $1",
        plan.id.value()
    )
    .execute(&mut **tx)
    .await?;

    let user_id_values: Vec<RawId> = plan.user_ids().to_vec();
    if !user_id_values.is_empty() {
        sqlx::query!(
            "INSERT INTO operation_plan_users (user_id, plan_id) SELECT UNNEST($1::uuid[]), $2",
            &user_id_values,
            plan.id.value(),
        )
        .execute(&mut **tx)
        .await?;
    }

    let stop_ids = stop_values(plan.stops());
    sqlx::query!(
        "DELETE FROM operation_plan_stops WHERE plan_id = $1 AND stop_id <> ALL($2::uuid[])",
        plan.id.value(),
        &stop_ids,
    )
    .execute(&mut **tx)
    .await?;

    if !stop_ids.is_empty() {
        sqlx::query!(
            r#"INSERT INTO operation_plan_stops (stop_id, plan_id)
               SELECT UNNEST($1::uuid[]), $2
               ON CONFLICT (stop_id, plan_id) DO NOTHING"#,
            &stop_ids,
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
            "INSERT INTO operation_plan_vehicles (vehicle_id, plan_id, role) VALUES ($1, $2, 'transporter')",
            id,
            plan_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    if let Some(id) = trailer_id {
        sqlx::query!(
            "INSERT INTO operation_plan_vehicles (vehicle_id, plan_id, role) VALUES ($1, $2, 'trailer')",
            id,
            plan_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

#[async_trait]
impl OperationPlanWriter for PgOperationPlanRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: OperationPlanDraft) -> Result<OperationPlan, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let plan_id = Id::<OperationPlan>::new_v7();
        sqlx::query!(
            r#"INSERT INTO operation_plans (id, module_key, date, description, status, provider, additional_informations, start_point_name, organization_id, transporter_capacity_used)
            VALUES ($1, $2, $3, $4, 'planned', $5, $6, $7, $8, $9)"#,
            plan_id.value(),
            draft.module.as_str(),
            draft.date.date_naive(),
            draft.description.as_deref().unwrap_or(""),
            draft.provenance.provider().map(|p| p.as_str()),
            draft.provenance.additional_info(),
            draft.start_point_name.as_deref(),
            draft.organization_id.value(),
            draft.transporter_capacity_used,
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

        let stop_ids = stop_values(&draft.stops);
        if !stop_ids.is_empty() {
            sqlx::query!(
                "INSERT INTO operation_plan_stops (stop_id, plan_id) SELECT UNNEST($1::uuid[]), $2",
                &stop_ids,
                plan_id.value()
            )
            .execute(&mut *tx)
            .await?;
        }

        if !draft.user_ids.is_empty() {
            sqlx::query!(
                "INSERT INTO operation_plan_users (user_id, plan_id) SELECT UNNEST($1::uuid[]), $2",
                &draft.user_ids,
                plan_id.value(),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.by_id(plan_id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, plan: &OperationPlan) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        persist_plan(&mut tx, plan).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_finished(
        &self,
        plan: &OperationPlan,
        outcomes: &[StopOutcome],
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        persist_plan(&mut tx, plan).await?;

        let stops: Vec<RawId> = outcomes.iter().map(|o| o.stop.value()).collect();
        let completed: Vec<bool> = outcomes.iter().map(|o| o.completed).collect();
        let amounts: Vec<Option<f64>> = outcomes.iter().map(|o| o.amount).collect();
        sqlx::query!(
            r#"UPDATE operation_plan_stops
               SET completed = u.completed, amount = u.amount
               FROM UNNEST($2::uuid[], $3::bool[], $4::float8[]) AS u(stop_id, completed, amount)
               WHERE plan_id = $1 AND operation_plan_stops.stop_id = u.stop_id"#,
            plan.id.value(),
            &stops,
            &completed,
            &amounts as &[Option<f64>],
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<OperationPlan>) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "DELETE FROM operation_plan_vehicles WHERE plan_id = $1",
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "DELETE FROM operation_plan_stops WHERE plan_id = $1",
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        let result = sqlx::query!("DELETE FROM operation_plans WHERE id = $1", id.value())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn remove_stop(&self, module: &ModuleKey, stop: StopRef) -> Result<u64, RepositoryError> {
        let result = sqlx::query!(
            r#"DELETE FROM operation_plan_stops s
               USING operation_plans p
               WHERE s.plan_id = p.id AND p.module_key = $1 AND s.stop_id = $2"#,
            module.as_str(),
            stop.value(),
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geojson_to_geometry_none_and_empty_collapse_to_none() {
        assert_eq!(geojson_to_geometry(None).unwrap(), None);
        assert_eq!(
            geojson_to_geometry(Some(
                r#"{"type":"LineString","coordinates":[]}"#.to_string()
            ))
            .unwrap(),
            None
        );
    }

    #[test]
    fn geojson_round_trips_coordinates() {
        let coords = vec![
            Coordinate::new(54.76, 9.43).unwrap(),
            Coordinate::new(54.80, 9.44).unwrap(),
        ];
        let geojson = geometry_to_geojson(Some(&coords)).unwrap();
        assert_eq!(geojson_to_geometry(Some(geojson)).unwrap(), Some(coords));
    }

    #[test]
    fn geometry_to_geojson_collapses_degenerate_lines_to_none() {
        assert_eq!(geometry_to_geojson(None), None);
        assert_eq!(geometry_to_geojson(Some(&[])), None);
        let single = [Coordinate::new(54.76, 9.43).unwrap()];
        assert_eq!(geometry_to_geojson(Some(&single)), None);
    }

    #[test]
    fn geojson_to_geometry_rejects_malformed_json() {
        let err = geojson_to_geometry(Some(r#"{"lat": 1.0}"#.to_string())).unwrap_err();
        assert!(matches!(err, RepositoryError::DataIntegrity(_)));
    }

    #[test]
    fn geojson_to_geometry_rejects_out_of_range_coordinates() {
        // GeoJSON position order is (lon, lat); lat 999 is out of range.
        let err = geojson_to_geometry(Some(
            r#"{"type":"LineString","coordinates":[[9.43,999.0],[9.44,54.8]]}"#.to_string(),
        ))
        .unwrap_err();
        assert!(matches!(err, RepositoryError::DataIntegrity(_)));
    }
}
