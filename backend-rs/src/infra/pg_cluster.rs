use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    cluster::{
        SoilCondition, TreeCluster, TreeClusterCreate, TreeClusterQuery, TreeClusterRepository,
        TreeClusterUpdate,
    },
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
        provider_info::ProviderInfo,
        watering_status::WateringStatus,
    },
};

struct TreeClusterRow {
    id: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    name: String,
    address: String,
    description: String,
    archived: bool,
    moisture_level: f64,
    region_id: Option<i32>,
    last_watered: Option<NaiveDateTime>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    watering_status: WateringStatus,
    soil_condition: SoilCondition,
    provider: Option<String>,
    additional_informations: Option<Value>,
    tree_ids: Vec<i32>,
}

impl From<TreeClusterRow> for TreeCluster {
    fn from(row: TreeClusterRow) -> Self {
        TreeCluster {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            watering_status: row.watering_status,
            last_watered: row.last_watered.map(|dt| dt.and_utc()),
            moisture_level: row.moisture_level,
            region_id: row.region_id.map(Id::new),
            address: row.address,
            description: row.description,
            archived: row.archived,
            coordinates: row
                .latitude
                .zip(row.longitude)
                .and_then(|(lat, lng)| Coordinate::new(lat, lng).ok()),
            tree_ids: row.tree_ids.into_iter().map(Id::new).collect(),
            soil_condition: Some(row.soil_condition),
            name: row.name,
            provider_info: ProviderInfo {
                provider: row.provider,
                additional_info: row.additional_informations,
            },
        }
    }
}

pub struct PgTreeClusterRepository {
    pool: PgPool,
}

impl PgTreeClusterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TreeClusterRepository for PgTreeClusterRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn all(
        &self,
        query: TreeClusterQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeCluster>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM tree_clusters tc
            LEFT JOIN regions r ON r.id = tc.region_id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::text[] = '{}' OR r.name = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            query.provider,
        )
        .fetch_one(&self.pool)
        .await? as u64;

        let items = sqlx::query_as!(
            TreeClusterRow,
            r#"SELECT tc.id, tc.created_at, tc.updated_at, tc.name, tc.address,
                      tc.description, tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: SoilCondition",
                      tc.latitude, tc.longitude, tc.last_watered,
                      tc.provider, tc.additional_informations,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::int[]) AS "tree_ids!: Vec<i32>"
            FROM tree_clusters tc
            LEFT JOIN regions r ON r.id = tc.region_id
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::text[] = '{}' OR r.name = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)
            GROUP BY tc.id
            ORDER BY tc.name ASC, tc.id ASC
            LIMIT $4 OFFSET $5"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            query.provider,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(TreeCluster::from)
        .collect();

        Ok(Page { items, total })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError> {
        Ok(sqlx::query_as!(
            TreeClusterRow,
            r#"SELECT tc.id, tc.created_at, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id, tc.last_watered,
                      tc.latitude, tc.longitude,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: SoilCondition",
                      tc.provider, tc.additional_informations,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::int[]) AS "tree_ids!: Vec<i32>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = $1
            GROUP BY tc.id"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?
        .into())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<TreeCluster>]) -> Result<Vec<TreeCluster>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        Ok(sqlx::query_as!(
            TreeClusterRow,
            r#"SELECT tc.id, tc.created_at, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id, tc.last_watered,
                      tc.latitude, tc.longitude,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: SoilCondition",
                      tc.provider, tc.additional_informations,
                      COALESCE(ARRAY_AGG(t.id ORDER BY t.number) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::int[]) AS "tree_ids!: Vec<i32>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = ANY($1)
            GROUP BY tc.id"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(TreeCluster::from)
        .collect())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn create(&self, entity: TreeClusterCreate) -> Result<TreeCluster, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"INSERT INTO tree_clusters (name, address, description, moisture_level,
                                          watering_status, soil_condition,
                                          provider, additional_informations)
            VALUES ($1, $2, $3, 0, 'unknown', $4, $5, $6)
            RETURNING id, created_at, updated_at, name, address, description, archived,
                      moisture_level, region_id, last_watered, latitude, longitude,
                      watering_status AS "watering_status: WateringStatus",
                      soil_condition AS "soil_condition: SoilCondition",
                      provider, additional_informations"#,
            entity.name,
            entity.address,
            entity.description,
            entity.soil_condition as SoilCondition,
            entity.provider_info.provider,
            entity.provider_info.additional_info,
        )
        .fetch_one(&mut *tx)
        .await?;

        let tree_id_values: Vec<i32> = entity.tree_ids.iter().map(|id| id.value()).collect();
        if !tree_id_values.is_empty() {
            sqlx::query!(
                "UPDATE trees SET tree_cluster_id = $1 WHERE id = ANY($2)",
                row.id,
                &tree_id_values
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(TreeCluster {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            watering_status: row.watering_status,
            last_watered: row.last_watered.map(|dt| dt.and_utc()),
            moisture_level: row.moisture_level,
            region_id: row.region_id.map(Id::new),
            address: row.address,
            description: row.description,
            archived: row.archived,
            coordinates: None,
            tree_ids: entity.tree_ids,
            soil_condition: Some(row.soil_condition),
            name: row.name,
            provider_info: entity.provider_info,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn update(
        &self,
        id: Id<TreeCluster>,
        entity: TreeClusterUpdate,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let coord_change = entity.coordinates.is_change();
        let coord_set = entity.coordinates.as_set().copied();
        let region_change = entity.region_id.is_change();
        let region_set = entity.region_id.as_set().map(|id| id.value());

        sqlx::query!(
            r#"UPDATE tree_clusters SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                description = COALESCE($4, description),
                soil_condition = COALESCE($5, soil_condition),
                provider = COALESCE($6, provider),
                additional_informations = COALESCE($7, additional_informations),
                latitude = CASE WHEN $8::bool THEN $9 ELSE latitude END,
                longitude = CASE WHEN $8::bool THEN $10 ELSE longitude END,
                region_id = CASE WHEN $11::bool THEN $12 ELSE region_id END
            WHERE id = $1"#,
            id.value(),
            entity.name,
            entity.address,
            entity.description,
            entity.soil_condition as Option<SoilCondition>,
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.provider.as_deref()),
            entity
                .provider_info
                .as_ref()
                .and_then(|p| p.additional_info.clone()),
            coord_change,
            coord_set.map(|c| c.latitude()),
            coord_set.map(|c| c.longitude()),
            region_change,
            region_set,
        )
        .execute(&mut *tx)
        .await?;

        if let Some(tree_ids) = entity.tree_ids {
            // Single UPDATE: detach trees no longer in the set, attach new ones,
            // re-set existing members. Saves a roundtrip vs. the previous two-step.
            let tree_id_values: Vec<i32> = tree_ids.iter().map(|id| id.value()).collect();
            sqlx::query!(
                r#"UPDATE trees SET tree_cluster_id = CASE
                    WHEN id = ANY($2) THEN $1
                    ELSE NULL
                END
                WHERE tree_cluster_id = $1 OR id = ANY($2)"#,
                id.value(),
                &tree_id_values,
            )
            .execute(&mut *tx)
            .await?;
        }

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

        sqlx::query!("DELETE FROM tree_clusters WHERE id = $1", id.value())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE tree_clusters SET archived = true WHERE id = $1",
            id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn center_point(&self, id: Id<TreeCluster>) -> Result<Coordinate, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT
                ST_X(ST_Centroid(ST_Collect(geometry)))::float8 AS "center_x!: f64",
                ST_Y(ST_Centroid(ST_Collect(geometry)))::float8 AS "center_y!: f64"
            FROM trees WHERE tree_cluster_id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Coordinate::new(row.center_y, row.center_x)?)
    }
}
