use chrono::{DateTime, NaiveDateTime, Utc};
use domain::{IdSliceExt, RawId};
use serde_json::Value;
use sqlx::PgPool;

use domain::cluster::snapshot::TreeClusterSnapshot;
use domain::{
    Id, RepositoryError,
    cluster::{
        ClusterBoundaryView, ClusterMarker, SoilCondition, TreeCluster, TreeClusterDraft,
        TreeClusterReader, TreeClusterSearchQuery, TreeClusterView, TreeClusterWriter,
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
            provider: row.provider,
            additional_info: row.additional_info,
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
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>"
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
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>"
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
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>"
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
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>"
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

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM tree_clusters tc
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::uuid[] = '{}' OR tc.region_id = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            provider,
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
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::uuid[]) AS "tree_ids!: Vec<RawId>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::uuid[] = '{}' OR tc.region_id = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)
            GROUP BY tc.id
            ORDER BY tc.name ASC, tc.id ASC
            LIMIT $4 OFFSET $5"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
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
                                          provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, 'unknown', $6, $7, $8)"#,
            id.value(),
            draft.name.as_str(),
            draft.address.as_str(),
            draft.description,
            draft.moisture_level,
            soil as SoilCondition,
            provider,
            additional_info,
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
                latitude = $8, longitude = $9, region_id = $10,
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
