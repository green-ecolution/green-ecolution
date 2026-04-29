use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    cluster::TreeCluster,
    shared::{
        coordinates::Coordinate,
        distance::Distance,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        watering_status::WateringStatus,
    },
    tree::{
        PlantingYear, Tree, TreeCreate, TreeQuery, TreeRepository, TreeUpdate, TreeWithDistance,
    },
};

struct TreeRow {
    id: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    tree_cluster_id: Option<i32>,
    sensor_id: Option<String>,
    planting_year: i32,
    species: String,
    number: String,
    latitude: f64,
    longitude: f64,
    watering_status: WateringStatus,
    description: Option<String>,
    last_watered: Option<NaiveDateTime>,
    provider: Option<String>,
    additional_informations: Option<Value>,
}

impl TryFrom<TreeRow> for Tree {
    type Error = RepositoryError;

    fn try_from(row: TreeRow) -> Result<Self, Self::Error> {
        Ok(Tree {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            cluster_id: row.tree_cluster_id.map(Id::new),
            sensor_id: row.sensor_id,
            planting_year: PlantingYear::new(row.planting_year as u32)?,
            species: row.species,
            tree_number: row.number,
            coordinate: Coordinate::new(row.latitude, row.longitude)?,
            watering_status: row.watering_status,
            description: row.description,
            last_watered: row.last_watered.map(|dt| dt.and_utc()),
            provider_info: ProviderInfo {
                provider: row.provider,
                additional_info: row.additional_informations,
            },
        })
    }
}

pub struct PgTreeRepository {
    pool: PgPool,
}

impl PgTreeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TreeRepository for PgTreeRepository {
    async fn all(
        &self,
        query: TreeQuery,
        pagination: Pagination,
    ) -> Result<Page<Tree>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;
        let planting_years: Vec<i32> = query.planting_years.iter().map(|&y| y as i32).collect();
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
            query.provider,
            query.has_cluster,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let rows = sqlx::query_as!(
            TreeRow,
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description, last_watered,
                      provider, additional_informations
            FROM trees
            WHERE ($1::watering_status[] = '{}' OR watering_status = ANY($1))
              AND ($2::int[] = '{}' OR planting_year = ANY($2))
              AND ($3::text IS NULL OR provider = $3)
              AND ($4::bool IS NULL OR ($4 = true AND tree_cluster_id IS NOT NULL) OR ($4 = false AND tree_cluster_id IS NULL))
            ORDER BY number ASC
            LIMIT $5 OFFSET $6"#,
            &watering_statuses as &[WateringStatus],
            &planting_years,
            query.provider,
            query.has_cluster,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(Tree::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page { items, total })
    }

    async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError> {
        sqlx::query_as!(
            TreeRow,
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description, last_watered,
                      provider, additional_informations
            FROM trees WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    async fn by_ids(&self, ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        sqlx::query_as!(
            TreeRow,
            r#"SELECT id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description, last_watered,
                      provider, additional_informations
            FROM trees WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Tree::try_from)
        .collect()
    }

    async fn create(&self, entity: TreeCreate) -> Result<Tree, RepositoryError> {
        let lat = entity.coordinate.latitude();
        let lng = entity.coordinate.longitude();

        sqlx::query_as!(
            TreeRow,
            r#"INSERT INTO trees (tree_cluster_id, sensor_id, planting_year, species, number,
                                  description, watering_status, latitude, longitude,
                                  geometry, provider, additional_informations)
            VALUES ($1, $2, $3, $4, $5, $6, 'unknown', $7, $8,
                    ST_SetSRID(ST_MakePoint($8, $7), 4326), $9, $10)
            RETURNING id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description, last_watered,
                      provider, additional_informations"#,
            entity.cluster_id.map(|id| id.value()),
            entity.sensor_id,
            entity.planting_year.year() as i32,
            entity.species,
            entity.tree_number,
            entity.description,
            lat,
            lng,
            entity.provider_info.provider,
            entity.provider_info.additional_info,
        )
        .fetch_one(&self.pool)
        .await?
        .try_into()
    }

    async fn update(&self, id: Id<Tree>, entity: TreeUpdate) -> Result<Tree, RepositoryError> {
        let lat = entity.coordinate.map(|c| c.latitude());
        let lng = entity.coordinate.map(|c| c.longitude());

        sqlx::query_as!(
            TreeRow,
            r#"UPDATE trees SET
                tree_cluster_id = COALESCE($2, tree_cluster_id),
                sensor_id = COALESCE($3, sensor_id),
                planting_year = COALESCE($4, planting_year),
                species = COALESCE($5, species),
                number = COALESCE($6, number),
                description = COALESCE($7, description),
                provider = COALESCE($8, provider),
                additional_informations = COALESCE($9, additional_informations),
                latitude = COALESCE($10, latitude),
                longitude = COALESCE($11, longitude),
                geometry = COALESCE(
                    CASE WHEN $10 IS NOT NULL THEN ST_SetSRID(ST_MakePoint($11, $10), 4326) END,
                    geometry
                )
            WHERE id = $1
            RETURNING id, created_at, updated_at, tree_cluster_id, sensor_id,
                      planting_year, species, number, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      description, last_watered,
                      provider, additional_informations"#,
            id.value(),
            entity.cluster_id.map(|id| id.value()),
            entity.sensor_id,
            entity.planting_year.map(|py| py.year() as i32),
            entity.species,
            entity.tree_number,
            entity.description,
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.provider.as_deref()),
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.additional_info.clone()),
            lat,
            lng,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .try_into()
    }

    async fn archive(&self, _id: Id<Tree>) -> Result<(), RepositoryError> {
        todo!("implement archive for trees")
    }

    async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM trees WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn nearest_trees(
        &self,
        coord: Coordinate,
        radius: Distance,
        limit: u32,
    ) -> Result<Vec<TreeWithDistance>, RepositoryError> {
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
                      description, last_watered,
                      provider, additional_informations,
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
                Ok(TreeWithDistance {
                    tree: Tree {
                        id: Id::new(row.id),
                        created_at: row.created_at.and_utc(),
                        updated_at: row.updated_at.and_utc(),
                        cluster_id: row.tree_cluster_id.map(Id::new),
                        sensor_id: row.sensor_id,
                        planting_year: PlantingYear::new(row.planting_year as u32)?,
                        species: row.species,
                        tree_number: row.number,
                        coordinate: Coordinate::new(row.latitude, row.longitude)?,
                        watering_status: row.watering_status,
                        description: row.description,
                        last_watered: row.last_watered.map(|dt| dt.and_utc()),
                        provider_info: ProviderInfo {
                            provider: row.provider,
                            additional_info: row.additional_informations,
                        },
                    },
                    distance: Distance::new(row.distance)?,
                })
            })
            .collect()
    }

    async fn distinct_planting_years(&self) -> Result<Vec<PlantingYear>, RepositoryError> {
        let rows = sqlx::query_scalar!(
            "SELECT DISTINCT planting_year FROM trees ORDER BY planting_year ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|y| PlantingYear::new(y as u32).ok())
            .collect())
    }

    async fn unlink_cluster_id(
        &self,
        cluster_id: Id<TreeCluster>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE trees SET tree_cluster_id = NULL WHERE tree_cluster_id = $1",
            cluster_id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn unlink_sensor_id(&self, sensor_id: &str) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE trees SET sensor_id = NULL, watering_status = 'unknown' WHERE sensor_id = $1",
            sensor_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
