use chrono::{DateTime, NaiveDateTime, Utc};
use domain::{IdSliceExt, RawId};
use serde_json::Value;
use sqlx::PgPool;

use crate::infra::sql::like_escape;
use domain::cluster::snapshot::TreeClusterSnapshot;
use domain::{
    Id, RepositoryError,
    cluster::{
        ClusterBoundaryView, ClusterMarker, ClusterStatistics, ClusterWateringEvent, SoilCondition,
        SoilMoistureBucket, SoilMoistureDepthSeries, SoilMoisturePoint, TreeCluster,
        TreeClusterDraft, TreeClusterReader, TreeClusterSearchQuery, TreeClusterView,
        TreeClusterWriter,
    },
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
        watering_status::WateringStatus,
    },
};

/// Boundary buffer applied to a cluster's convex hull, in meters. Keeps the
/// outermost trees inside the drawn area and rounds off the corners. A buffer
/// also turns the degenerate hulls (1 tree → point, 2 trees → line) into a
/// proper polygon, so no special-casing is needed.
const CLUSTER_BOUNDARY_BUFFER_METERS: f64 = 10.0;

pub struct PgTreeClusterRepository {
    pool: PgPool,
}

impl PgTreeClusterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Flat row shape for every `view_*` query on `tree_clusters` (incl. the
/// trees aggregate join). Field names match the SELECT column names so the
/// `.sqlx/` query cache stays valid; `From` derives `created_at` from the
/// UUID v7 id.
#[allow(dead_code)] // fields are read via the `From<TreeClusterViewRow>` impl
struct TreeClusterViewRow {
    id: RawId,
    updated_at: NaiveDateTime,
    name: String,
    address: String,
    description: String,
    archived: bool,
    moisture_level: f64,
    region_id: Option<RawId>,
    watering_status: WateringStatus,
    soil_condition: Option<SoilCondition>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    last_watered: Option<DateTime<Utc>>,
    provider: Option<String>,
    additional_info: Option<Value>,
    tree_ids: Vec<RawId>,
    sensor_count: i64,
    organization_id: RawId,
    shared_with: Vec<RawId>,
}

impl From<TreeClusterViewRow> for TreeClusterView {
    fn from(row: TreeClusterViewRow) -> Self {
        let created_at = Id::<TreeCluster>::new(row.id)
            .created_at()
            .expect("tree_clusters.id is minted as uuid v7");
        Self {
            id: row.id,
            created_at,
            updated_at: row.updated_at.and_utc(),
            name: row.name,
            address: row.address,
            description: row.description,
            watering_status: row.watering_status,
            last_watered: row.last_watered,
            moisture_level: row.moisture_level,
            region_id: row.region_id,
            archived: row.archived,
            latitude: row.latitude,
            longitude: row.longitude,
            soil_condition: row.soil_condition,
            tree_ids: row.tree_ids,
            sensor_count: row.sensor_count,
            provider: row.provider,
            additional_info: row.additional_info,
            organization_id: row.organization_id,
            shared_with: row.shared_with,
        }
    }
}

#[async_trait::async_trait]
impl TreeClusterReader for PgTreeClusterRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError> {
        let snap = sqlx::query_as!(
            TreeClusterSnapshot,
            r#"SELECT tc.id, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: Option<SoilCondition>",
                      tc.latitude, tc.longitude,
                      tc.last_watered AS "last_watered: DateTime<Utc>",
                      tc.provider,
                      tc.additional_informations AS additional_info,
                      tc.organization_id,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>",
                      COALESCE((SELECT array_agg(tcs.organization_id) FROM tree_cluster_shares tcs WHERE tcs.tree_cluster_id = tc.id), '{}') AS "shared_with!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = $1
            GROUP BY tc.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(TreeCluster::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<TreeCluster>]) -> Result<Vec<TreeCluster>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let snaps = sqlx::query_as!(
            TreeClusterSnapshot,
            r#"SELECT tc.id, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: Option<SoilCondition>",
                      tc.latitude, tc.longitude,
                      tc.last_watered AS "last_watered: DateTime<Utc>",
                      tc.provider,
                      tc.additional_informations AS additional_info,
                      tc.organization_id,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>",
                      COALESCE((SELECT array_agg(tcs.organization_id) FROM tree_cluster_shares tcs WHERE tcs.tree_cluster_id = tc.id), '{}') AS "shared_with!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = ANY($1::uuid[])
            GROUP BY tc.id"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(TreeCluster::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<TreeCluster>) -> Result<TreeClusterView, RepositoryError> {
        let row = sqlx::query_as!(
            TreeClusterViewRow,
            r#"SELECT tc.id, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: Option<SoilCondition>",
                      tc.latitude, tc.longitude,
                      tc.last_watered AS "last_watered: DateTime<Utc>",
                      tc.provider,
                      tc.additional_informations AS additional_info,
                      tc.organization_id,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>",
                      COUNT(t.id) FILTER (WHERE t.sensor_id IS NOT NULL AND t.sensor_id <> '') AS "sensor_count!: i64",
                      COALESCE((SELECT array_agg(tcs.organization_id) FROM tree_cluster_shares tcs WHERE tcs.tree_cluster_id = tc.id), '{}') AS "shared_with!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = $1
            GROUP BY tc.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(row.into())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(
        &self,
        ids: &[Id<TreeCluster>],
    ) -> Result<Vec<TreeClusterView>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let rows = sqlx::query_as!(
            TreeClusterViewRow,
            r#"SELECT tc.id, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: Option<SoilCondition>",
                      tc.latitude, tc.longitude,
                      tc.last_watered AS "last_watered: DateTime<Utc>",
                      tc.provider,
                      tc.additional_informations AS additional_info,
                      tc.organization_id,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>",
                      COUNT(t.id) FILTER (WHERE t.sensor_id IS NOT NULL AND t.sensor_id <> '') AS "sensor_count!: i64",
                      COALESCE((SELECT array_agg(tcs.organization_id) FROM tree_cluster_shares tcs WHERE tcs.tree_cluster_id = tc.id), '{}') AS "shared_with!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = ANY($1::uuid[])
            GROUP BY tc.id"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: TreeClusterSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeClusterView>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);
        let provider = query.provider.as_ref().map(|p| p.as_str().to_string());
        let search = query
            .query
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", like_escape(s)));
        let search = search.as_deref();
        let sort = query.sort.as_str();
        let order = query.order.as_str();

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM tree_clusters tc
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::uuid[] = '{}' OR tc.region_id = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)
              AND ($4::text IS NULL OR tc.name ILIKE $4 ESCAPE '\')
              AND ($5::tree_soil_condition[] = '{}' OR tc.soil_condition = ANY($5))"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            provider,
            search,
            &query.soil_conditions as &[SoilCondition],
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            TreeClusterViewRow,
            r#"SELECT tc.id, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: Option<SoilCondition>",
                      tc.latitude, tc.longitude,
                      tc.last_watered AS "last_watered: DateTime<Utc>",
                      tc.provider,
                      tc.additional_informations AS additional_info,
                      tc.organization_id,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>",
                      COUNT(t.id) FILTER (WHERE t.sensor_id IS NOT NULL AND t.sensor_id <> '') AS "sensor_count!: i64",
                      COALESCE((SELECT array_agg(tcs.organization_id) FROM tree_cluster_shares tcs WHERE tcs.tree_cluster_id = tc.id), '{}') AS "shared_with!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::uuid[] = '{}' OR tc.region_id = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)
              AND ($4::text IS NULL OR tc.name ILIKE $4 ESCAPE '\')
              AND ($5::tree_soil_condition[] = '{}' OR tc.soil_condition = ANY($5))
            GROUP BY tc.id
            ORDER BY
              CASE WHEN $6 = 'moisture' AND $7 = 'asc'  THEN tc.moisture_level END ASC NULLS LAST,
              CASE WHEN $6 = 'moisture' AND $7 = 'desc' THEN tc.moisture_level END DESC NULLS LAST,
              CASE WHEN $6 = 'trees'    AND $7 = 'asc'  THEN COUNT(t.id) END ASC,
              CASE WHEN $6 = 'trees'    AND $7 = 'desc' THEN COUNT(t.id) END DESC,
              CASE WHEN $6 = 'name'     AND $7 = 'desc' THEN tc.name END DESC,
              tc.name ASC, tc.id ASC
            LIMIT $8 OFFSET $9"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            provider,
            search,
            &query.soil_conditions as &[SoilCondition],
            sort,
            order,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows.into_iter().map(Into::into).collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_markers(&self) -> Result<Vec<ClusterMarker>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT tc.id, tc.name,
                      tc.latitude AS "latitude!: f64",
                      tc.longitude AS "longitude!: f64",
                      tc.watering_status AS "watering_status: WateringStatus",
                      COUNT(t.id) AS "tree_count!: i64"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.archived = false
              AND tc.latitude IS NOT NULL
              AND tc.longitude IS NOT NULL
            GROUP BY tc.id
            ORDER BY tc.id"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ClusterMarker {
                id: row.id,
                name: row.name,
                latitude: row.latitude,
                longitude: row.longitude,
                watering_status: row.watering_status,
                tree_count: u32::try_from(row.tree_count).unwrap_or(0),
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn center_point(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Option<Coordinate>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT
                ST_X(ST_Centroid(ST_Collect(geometry)))::float8 AS "center_x: f64",
                ST_Y(ST_Centroid(ST_Collect(geometry)))::float8 AS "center_y: f64"
            FROM trees WHERE tree_cluster_id = $1"#,
            id.value()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(match (row.center_x, row.center_y) {
            (Some(x), Some(y)) => Coordinate::new(y, x).ok(),
            _ => None,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn boundaries(&self) -> Result<Vec<ClusterBoundaryView>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT
                tc.id                                            AS "cluster_id!",
                tc.name                                          AS "name!",
                tc.watering_status AS "watering_status: WateringStatus",
                ST_AsGeoJSON(
                    ST_Buffer(
                        ST_ConvexHull(ST_Collect(t.geometry))::geography,
                        $1::float8
                    )::geometry
                )::jsonb                                         AS "boundary!: serde_json::Value"
            FROM trees t
            JOIN tree_clusters tc ON tc.id = t.tree_cluster_id
            WHERE t.geometry IS NOT NULL
              AND tc.archived = false
            GROUP BY tc.id, tc.name, tc.watering_status"#,
            CLUSTER_BOUNDARY_BUFFER_METERS,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ClusterBoundaryView {
                cluster_id: row.cluster_id,
                name: row.name,
                watering_status: row.watering_status,
                boundary: row.boundary,
            })
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn statistics(&self) -> Result<ClusterStatistics, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT
                COUNT(*)                                                  AS "total!: i64",
                COUNT(*) FILTER (WHERE watering_status = 'bad')           AS "bad!: i64",
                COUNT(*) FILTER (WHERE watering_status = 'moderate')      AS "moderate!: i64",
                COUNT(*) FILTER (WHERE watering_status = 'good')          AS "good!: i64",
                COUNT(*) FILTER (WHERE watering_status = 'just watered')  AS "just_watered!: i64",
                COUNT(*) FILTER (WHERE watering_status = 'unknown')       AS "unknown!: i64",
                COALESCE((
                    SELECT COUNT(*) FROM trees t
                    JOIN tree_clusters c ON t.tree_cluster_id = c.id
                    WHERE c.archived = false
                ), 0)                                                     AS "trees!: i64"
            FROM tree_clusters
            WHERE archived = false"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ClusterStatistics {
            total: row.total,
            trees: row.trees,
            bad: row.bad,
            moderate: row.moderate,
            good: row.good,
            just_watered: row.just_watered,
            unknown: row.unknown,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn soil_moisture_series(
        &self,
        id: Id<TreeCluster>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        bucket: SoilMoistureBucket,
    ) -> Result<Vec<SoilMoistureDepthSeries>, RepositoryError> {
        let bucket_kind = match bucket {
            SoilMoistureBucket::Hour => "hour",
            SoilMoistureBucket::Day => "day",
        };
        // Sentinel guard: disconnected Dragino probes report ~6553.5, so
        // anything outside 0–100 Vol.-% is a sensor error, not a reading.
        let rows = sqlx::query!(
            r#"SELECT sma.depth_cm AS "depth_cm!",
                      date_trunc($4, sd.updated_at) AS "bucket_start!",
                      AVG(dav.value)::float8 AS "mean!",
                      MIN(dav.value)::float8 AS "min!",
                      MAX(dav.value)::float8 AS "max!",
                      COUNT(*) AS "sample_count!"
               FROM trees t
               JOIN sensor_data sd ON sd.sensor_id = t.sensor_id
               JOIN sensor_data_ability_values dav ON dav.sensor_data_id = sd.id
               JOIN sensor_model_abilities sma ON sma.id = dav.sensor_model_ability_id
               JOIN sensor_abilities sa ON sa.id = sma.sensor_ability_id
               WHERE t.tree_cluster_id = $1
                 AND sa.ability = 'soil_moisture'
                 AND dav.value >= 0 AND dav.value <= 100
                 AND sd.updated_at >= $2
                 AND sd.updated_at <= $3
               GROUP BY sma.depth_cm, date_trunc($4, sd.updated_at)
               ORDER BY sma.depth_cm, date_trunc($4, sd.updated_at)"#,
            id.value(),
            from.naive_utc(),
            to.naive_utc(),
            bucket_kind,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut series: Vec<SoilMoistureDepthSeries> = Vec::new();
        for r in rows {
            let point = SoilMoisturePoint {
                bucket_start: r.bucket_start.and_utc(),
                mean: r.mean,
                min: r.min,
                max: r.max,
                sample_count: r.sample_count,
            };
            match series.last_mut() {
                Some(s) if s.depth_cm == r.depth_cm => s.points.push(point),
                _ => series.push(SoilMoistureDepthSeries {
                    depth_cm: r.depth_cm,
                    points: vec![point],
                }),
            }
        }
        Ok(series)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn watering_events(
        &self,
        id: Id<TreeCluster>,
    ) -> Result<Vec<ClusterWateringEvent>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT wp.id AS "watering_plan_id!",
                      wp.date AS "date!",
                      tcwp.consumed_water AS "consumed_water!"
               FROM tree_cluster_watering_plans tcwp
               JOIN watering_plans wp ON wp.id = tcwp.watering_plan_id
               WHERE tcwp.tree_cluster_id = $1
                 AND wp.status = 'finished'
               ORDER BY wp.date DESC"#,
            id.value(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ClusterWateringEvent {
                watering_plan_id: r.watering_plan_id,
                date: r.date,
                consumed_water_liters: r.consumed_water,
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl TreeClusterWriter for PgTreeClusterRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: TreeClusterDraft) -> Result<TreeCluster, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let soil = draft.soil_condition.unwrap_or(SoilCondition::Unknown);
        let provider = draft.provenance.provider().map(|p| p.as_str().to_string());
        let additional_info = draft.provenance.additional_info().cloned();
        let id = Id::<TreeCluster>::new_v7();

        sqlx::query!(
            r#"INSERT INTO tree_clusters (id, name, address, description, moisture_level,
                                          watering_status, soil_condition,
                                          provider, additional_informations, organization_id)
            VALUES ($1, $2, $3, $4, $5, 'unknown', $6, $7, $8, $9)"#,
            id.value(),
            draft.name.as_str(),
            draft.address.as_str(),
            draft.description,
            draft.moisture_level,
            soil as SoilCondition,
            provider,
            additional_info,
            draft.organization_id.value(),
        )
        .execute(&mut *tx)
        .await?;

        let tree_id_values: Vec<RawId> = draft.tree_ids.to_values();
        if !tree_id_values.is_empty() {
            sqlx::query!(
                "UPDATE trees SET tree_cluster_id = $1 WHERE id = ANY($2::uuid[])",
                id.value(),
                &tree_id_values
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.by_id(id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, cluster: &TreeCluster) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let soil = cluster.soil_condition.unwrap_or(SoilCondition::Unknown);
        let provider = cluster
            .provenance()
            .provider()
            .map(|p| p.as_str().to_string());
        let additional_info = cluster.provenance().additional_info().cloned();
        let latitude = cluster.coordinates().map(|c| c.latitude());
        let longitude = cluster.coordinates().map(|c| c.longitude());
        let region_id = cluster.region_id().map(|id| id.value());
        let last_watered = cluster.last_watered.map(|dt| dt.naive_utc());

        let result = sqlx::query!(
            r#"UPDATE tree_clusters SET
                name = $2, address = $3, description = $4, soil_condition = $5,
                provider = $6, additional_informations = $7,
                latitude = $8, longitude = $9,
                geometry = ST_SetSRID(ST_MakePoint($9, $8), 4326),
                region_id = $10,
                watering_status = $11, last_watered = $12, archived = $13,
                moisture_level = $14
            WHERE id = $1"#,
            cluster.id.value(),
            cluster.name.as_str(),
            cluster.address.as_str(),
            cluster.description,
            soil as SoilCondition,
            provider,
            additional_info,
            latitude,
            longitude,
            region_id,
            cluster.watering_status() as WateringStatus,
            last_watered,
            cluster.archived(),
            cluster.moisture_level,
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        let tree_id_values: Vec<RawId> = cluster.tree_ids.to_values();
        sqlx::query!(
            r#"UPDATE trees SET tree_cluster_id = CASE
                WHEN id = ANY($2::uuid[]) THEN $1
                ELSE NULL
            END
            WHERE tree_cluster_id = $1 OR id = ANY($2::uuid[])"#,
            cluster.id.value(),
            &tree_id_values,
        )
        .execute(&mut *tx)
        .await?;

        let shared_with: Vec<RawId> = cluster.shared_with().iter().map(|o| o.value()).collect();
        sqlx::query!(
            "DELETE FROM tree_cluster_shares WHERE tree_cluster_id = $1",
            cluster.id.value()
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"INSERT INTO tree_cluster_shares (tree_cluster_id, organization_id)
            SELECT $1, unnest($2::uuid[]) WHERE cardinality($2::uuid[]) > 0"#,
            cluster.id.value(),
            &shared_with,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE trees SET tree_cluster_id = NULL WHERE tree_cluster_id = $1",
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        let result = sqlx::query!("DELETE FROM tree_clusters WHERE id = $1", id.value())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            "UPDATE tree_clusters SET archived = true WHERE id = $1",
            id.value()
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
