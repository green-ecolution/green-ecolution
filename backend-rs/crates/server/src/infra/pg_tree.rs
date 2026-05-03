use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::tree::snapshot::TreeSnapshot;
use crate::domain::{
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
        PlantingYear, Tree, TreeDraft, TreeReader, TreeSearchQuery, TreeView, TreeViewWithDistance,
        TreeWriter,
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
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
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
            FROM trees WHERE id = ANY($1)"#,
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
        let row = sqlx::query!(
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
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

        Ok(TreeView {
            id: row.id,
            created_at: row.created_at.and_utc(),
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
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_sensor_id(
        &self,
        sensor_id: &SensorId,
    ) -> Result<Option<TreeView>, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
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

        Ok(row.map(|row| TreeView {
            id: row.id,
            created_at: row.created_at.and_utc(),
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
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<TreeView>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        let rows = sqlx::query!(
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info
            FROM trees WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| TreeView {
                id: row.id,
                created_at: row.created_at.and_utc(),
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
            })
            .collect())
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

        let rows = sqlx::query!(
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
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

        let items = rows
            .into_iter()
            .map(|row| TreeView {
                id: row.id,
                created_at: row.created_at.and_utc(),
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
            })
            .collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn view_nearest(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeViewWithDistance>, RepositoryError> {
        let rows = sqlx::query!(
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
            SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
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

        rows.into_iter()
            .map(|row| {
                Ok(TreeViewWithDistance {
                    tree: TreeView {
                        id: row.id,
                        created_at: row.created_at.and_utc(),
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
                    },
                    distance: Distance::new(row.distance)?,
                })
            })
            .collect()
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

        let snap = sqlx::query_as!(
            TreeSnapshot,
            r#"INSERT INTO trees (tree_cluster_id, sensor_id, planting_year, species, number,
                                  description, watering_status, latitude, longitude,
                                  geometry, provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, $6, 'unknown', $7, $8,
                    ST_SetSRID(ST_MakePoint($8, $7), 4326), $9, $10)
            RETURNING id, tree_cluster_id AS cluster_id, sensor_id,
                      planting_year, species, number AS tree_number,
                      latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description,
                      last_watered AS "last_watered: DateTime<Utc>",
                      provider,
                      additional_informations AS additional_info"#,
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
        .fetch_one(&self.pool)
        .await?;

        Ok(Tree::reconstitute(snap))
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
