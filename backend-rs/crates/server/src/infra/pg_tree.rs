use chrono::{DateTime, NaiveDateTime, Utc};
use domain::{IdSliceExt, RawId};
use serde_json::Value;
use sqlx::PgPool;

use domain::tree::snapshot::TreeSnapshot;
use domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    sensor::SensorId,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        watering_status::WateringStatus,
    },
    tree::{
        PlantingYear, Tree, TreeDraft, TreeMarker, TreeReader, TreeSearchQuery, TreeView,
        TreeViewWithDistance, TreeWriter,
    },
};

pub struct PgTreeRepository {
    pool: PgPool,
}

impl PgTreeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Flat row shape shared by every `view_*` query on `trees`. Field names
/// match the column names from `SELECT id, updated_at, tree_cluster_id, …`
/// so the original SQL (and `.sqlx/` query cache) stays unchanged; the
/// rename to `TreeView` (`tree_cluster_id` → `cluster_id`, `number` →
/// `tree_number`) happens in the `From` impl.
struct TreeViewRow {
    id: RawId,
    updated_at: NaiveDateTime,
    tree_cluster_id: Option<RawId>,
    sensor_id: Option<String>,
    planting_year: i32,
    species: String,
    number: String,
    latitude: f64,
    longitude: f64,
    watering_status: WateringStatus,
    description: Option<String>,
    last_watered: Option<DateTime<Utc>>,
    provider: Option<String>,
    additional_info: Option<Value>,
}

impl From<TreeViewRow> for TreeView {
    fn from(row: TreeViewRow) -> Self {
        let created_at = Id::<Tree>::new(row.id)
            .created_at()
            .expect("trees.id is minted as uuid v7");
        Self {
            id: row.id,
            created_at,
            updated_at: row.updated_at.and_utc(),
            cluster_id: row.tree_cluster_id,
            sensor_id: row.sensor_id,
            planting_year: row.planting_year as u32,
            species: row.species,
            tree_number: row.number,
            latitude: row.latitude,
            longitude: row.longitude,
            watering_status: row.watering_status,
            description: row.description,
            last_watered: row.last_watered,
            provider: row.provider,
            additional_info: row.additional_info,
        }
    }
}

/// `view_nearest` extends [`TreeViewRow`] with the haversine distance from
/// the query point.
struct TreeViewWithDistanceRow {
    id: RawId,
    updated_at: NaiveDateTime,
    tree_cluster_id: Option<RawId>,
    sensor_id: Option<String>,
    planting_year: i32,
    species: String,
    number: String,
    latitude: f64,
    longitude: f64,
    watering_status: WateringStatus,
    description: Option<String>,
    last_watered: Option<DateTime<Utc>>,
    provider: Option<String>,
    additional_info: Option<Value>,
    distance: f64,
}

impl TryFrom<TreeViewWithDistanceRow> for TreeViewWithDistance {
    type Error = RepositoryError;

    fn try_from(row: TreeViewWithDistanceRow) -> Result<Self, Self::Error> {
        let distance = Distance::new(row.distance)?;
        let tree = TreeView::from(TreeViewRow {
            id: row.id,
            updated_at: row.updated_at,
            tree_cluster_id: row.tree_cluster_id,
            sensor_id: row.sensor_id,
            planting_year: row.planting_year,
            species: row.species,
            number: row.number,
            latitude: row.latitude,
            longitude: row.longitude,
            watering_status: row.watering_status,
            description: row.description,
            last_watered: row.last_watered,
            provider: row.provider,
            additional_info: row.additional_info,
        });
        Ok(Self { tree, distance })
    }
}

#[async_trait::async_trait]
impl TreeReader for PgTreeRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError> {
        let snap = sqlx::query_as!(
            TreeSnapshot,
            r#"SELECT id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Tree::reconstitute(snap))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let snaps = sqlx::query_as!(
            TreeSnapshot,
            r#"SELECT id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE id = ANY($1::uuid[])"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(Tree::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_sensor_id(&self, sensor_id: &SensorId) -> Result<Option<Tree>, RepositoryError> {
        let snap = sqlx::query_as!(
            TreeSnapshot,
            r#"SELECT id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE sensor_id = $1 LIMIT 1"#,
            sensor_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snap.map(Tree::reconstitute))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_cluster_id(
        &self,
        cluster_id: Id<TreeCluster>,
    ) -> Result<Vec<Tree>, RepositoryError> {
        let snaps = sqlx::query_as!(
            TreeSnapshot,
            r#"SELECT id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE tree_cluster_id = $1"#,
            cluster_id.value()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(snaps.into_iter().map(Tree::reconstitute).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_id(&self, id: Id<Tree>) -> Result<TreeView, RepositoryError> {
        let row = sqlx::query_as!(
            TreeViewRow,
            r#"SELECT id, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(row.into())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_sensor_id(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Option<TreeView>, RepositoryError> {
        let row = sqlx::query_as!(
            TreeViewRow,
            r#"SELECT id, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE sensor_id = $1"#,
            sensor_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<TreeView>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let rows = sqlx::query_as!(
            TreeViewRow,
            r#"SELECT id, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE id = ANY($1::uuid[])"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_search(
        &self,
        query: TreeSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeView>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;
        let planting_years: Vec<i32> = query
            .planting_years
            .iter()
            .map(|py| py.year() as i32)
            .collect();
        let provider = query.provider.as_ref().map(|p| p.as_str().to_string());
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM trees
            WHERE ($1::watering_status[] = '{}' OR watering_status = ANY($1))
              AND ($2::int[] = '{}' OR planting_year = ANY($2))
              AND ($3::text IS NULL OR provider = $3)
              AND ($4::bool IS NULL OR ($4 = true AND tree_cluster_id IS NOT NULL) OR ($4 = false AND tree_cluster_id IS NULL))"#,
            &watering_statuses as &[WateringStatus],
            &planting_years,
            provider.as_deref(),
            query.has_cluster,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            TreeViewRow,
            r#"SELECT id, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees
            WHERE ($1::watering_status[] = '{}' OR watering_status = ANY($1))
              AND ($2::int[] = '{}' OR planting_year = ANY($2))
              AND ($3::text IS NULL OR provider = $3)
              AND ($4::bool IS NULL OR ($4 = true AND tree_cluster_id IS NOT NULL) OR ($4 = false AND tree_cluster_id IS NULL))
            ORDER BY number ASC
            LIMIT $5 OFFSET $6"#,
            &watering_statuses as &[WateringStatus],
            &planting_years,
            provider.as_deref(),
            query.has_cluster,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows.into_iter().map(Into::into).collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_markers(
        &self,
        query: TreeSearchQuery,
    ) -> Result<Vec<TreeMarker>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;
        let planting_years: Vec<i32> = query
            .planting_years
            .iter()
            .map(|py| py.year() as i32)
            .collect();
        let provider = query.provider.as_ref().map(|p| p.as_str().to_string());
        let bbox = query.bbox;

        let rows = sqlx::query!(
            r#"SELECT id, latitude, longitude, number,
                      watering_status AS "watering_status: WateringStatus",
                      (sensor_id IS NOT NULL) AS "has_sensor!: bool"
            FROM trees
            WHERE ($1::watering_status[] = '{}' OR watering_status = ANY($1))
              AND ($2::int[] = '{}' OR planting_year = ANY($2))
              AND ($3::text IS NULL OR provider = $3)
              AND ($4::bool IS NULL
                   OR ($4 = true AND tree_cluster_id IS NOT NULL)
                   OR ($4 = false AND tree_cluster_id IS NULL))
              AND ($5::bool IS FALSE
                   OR ST_Intersects(
                       geometry,
                       ST_MakeEnvelope($6, $7, $8, $9, 4326)
                   ))
            ORDER BY id"#,
            &watering_statuses as &[WateringStatus],
            &planting_years,
            provider.as_deref(),
            query.has_cluster,
            bbox.is_some(),
            bbox.map(|b| b.sw_lng()),
            bbox.map(|b| b.sw_lat()),
            bbox.map(|b| b.ne_lng()),
            bbox.map(|b| b.ne_lat()),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| TreeMarker {
                id: row.id,
                latitude: row.latitude,
                longitude: row.longitude,
                watering_status: row.watering_status,
                tree_number: row.number,
                has_sensor: row.has_sensor,
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeViewWithDistance>, RepositoryError> {
        let rows = sqlx::query_as!(
            TreeViewWithDistanceRow,
            r#"WITH distances AS (
                SELECT *,
                    ST_Distance(
                        geometry::geography,
                        ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
                    )::float8 AS dist
                FROM trees
                WHERE ST_DWithin(
                    geometry::geography,
                    ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography,
                    $3
                )
            )
            SELECT id, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info,
                      dist AS "distance!: f64"
            FROM distances
            ORDER BY dist ASC
            LIMIT $4"#,
            coord.latitude(),
            coord.longitude(),
            radius.meters(),
            limit as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn find_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
    ) -> Result<Option<Tree>, RepositoryError> {
        let snap = sqlx::query_as!(
            TreeSnapshot,
            r#"SELECT id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees
            WHERE ST_DWithin(
                geometry::geography,
                ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography,
                $3
            )
            ORDER BY ST_Distance(
                geometry::geography,
                ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
            ) ASC
            LIMIT 1"#,
            coord.latitude(),
            coord.longitude(),
            radius.meters(),
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(snap.map(Tree::reconstitute))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, RepositoryError> {
        let rows = sqlx::query_scalar!(
            "SELECT DISTINCT planting_year FROM trees ORDER BY planting_year ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|y| PlantingYear::reconstitute(y as u32))
            .collect())
    }
}

#[async_trait::async_trait]
impl TreeWriter for PgTreeRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: TreeDraft) -> Result<Tree, RepositoryError> {
        let lat = draft.coordinate.latitude();
        let lng = draft.coordinate.longitude();
        let id = Id::<Tree>::new_v7();

        sqlx::query!(
            r#"INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number,
                                  description, watering_status, latitude, longitude,
                                  geometry, provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'unknown', $8, $9,
                    ST_SetSRID(ST_MakePoint($9, $8), 4326), $10, $11)"#,
            id.value(),
            draft.cluster_id.map(|id| id.value()),
            draft.sensor_id.as_ref().map(|s| s.as_str().to_string()),
            draft.planting_year.year() as i32,
            draft.species.as_str(),
            draft.tree_number.as_str(),
            draft.description.as_deref(),
            lat,
            lng,
            draft.provenance.provider().map(|p| p.as_str().to_string()),
            draft.provenance.additional_info().cloned(),
        )
        .execute(&self.pool)
        .await?;

        self.by_id(id).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, tree: &Tree) -> Result<(), RepositoryError> {
        let lat = tree.coordinate.latitude();
        let lng = tree.coordinate.longitude();

        let result = sqlx::query!(
            r#"UPDATE trees SET
                tree_cluster_id = $2,
                sensor_id = $3,
                planting_year = $4,
                species = $5,
                number = $6,
                description = $7,
                watering_status = $8,
                last_watered = $9,
                latitude = $10,
                longitude = $11,
                geometry = ST_SetSRID(ST_MakePoint($11, $10), 4326),
                provider = $12,
                additional_informations = $13
            WHERE id = $1"#,
            tree.id.value(),
            tree.cluster_id().map(|id| id.value()),
            tree.sensor_id().map(|s| s.as_str().to_string()),
            tree.planting_year.year() as i32,
            tree.species.as_str(),
            tree.tree_number.as_str(),
            tree.description.as_deref(),
            tree.watering_status() as WateringStatus,
            tree.last_watered.map(|dt| dt.naive_utc()),
            lat,
            lng,
            tree.provenance().provider().map(|p| p.as_str().to_string()),
            tree.provenance().additional_info().cloned(),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError> {
        let result = sqlx::query!("DELETE FROM trees WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

/// Escapes Postgres `LIKE`/`ILIKE` wildcards in user-provided input so the
/// stored pattern is matched literally. Used together with `ESCAPE '\'` in
/// the SQL clause.
///
/// Backslash MUST be escaped first; otherwise an injected `\%` would survive
/// as a usable wildcard.
#[allow(dead_code)] // Used in tree name search (Task 3)
fn like_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str(r"\\"),
            '%' => out.push_str(r"\%"),
            '_' => out.push_str(r"\_"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod like_escape_tests {
    use super::like_escape;

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(like_escape(""), "");
    }

    #[test]
    fn plain_input_unchanged() {
        assert_eq!(like_escape("Eiche"), "Eiche");
        assert_eq!(like_escape("T-001"), "T-001");
    }

    #[test]
    fn escapes_percent() {
        assert_eq!(like_escape("50%"), r"50\%");
    }

    #[test]
    fn escapes_underscore() {
        assert_eq!(like_escape("a_b"), r"a\_b");
    }

    #[test]
    fn escapes_backslash_first() {
        // Backslash must be escaped before % and _ so we don't double-escape.
        assert_eq!(like_escape(r"a\b"), r"a\\b");
    }

    #[test]
    fn escapes_combined() {
        assert_eq!(like_escape(r"50%\_x"), r"50\%\\\_x");
    }
}
