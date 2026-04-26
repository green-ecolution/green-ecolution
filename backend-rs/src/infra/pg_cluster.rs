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
    async fn all(
        &self,
        query: TreeClusterQuery,
        pagination: Pagination,
    ) -> Result<Page<TreeCluster>, RepositoryError> {
        let watering_statuses: Vec<WateringStatus> = query.watering_statuses;

        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tree_clusters tc
            LEFT JOIN regions r ON r.id = tc.region_id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::text[] = '{}' OR r.name = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            query.provider,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"SELECT tc.id, tc.created_at, tc.updated_at, tc.name, tc.address,
                      tc.description, tc.archived, tc.moisture_level, tc.region_id,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: SoilCondition",
                      tc.latitude, tc.longitude, tc.last_watered,
                      tc.provider, tc.additional_informations,
                      COALESCE(ARRAY_AGG(t.id) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::int[]) AS "tree_ids!: Vec<i32>"
            FROM tree_clusters tc
            LEFT JOIN regions r ON r.id = tc.region_id
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE ($1::watering_status[] = '{}' OR tc.watering_status = ANY($1))
              AND ($2::text[] = '{}' OR r.name = ANY($2))
              AND ($3::text IS NULL OR tc.provider = $3)
            GROUP BY tc.id
            ORDER BY tc.name ASC
            LIMIT $4 OFFSET $5"#,
            &watering_statuses as &[WateringStatus],
            &query.regions,
            query.provider,
            pagination.limit as i64,
            pagination.offset as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|row| TreeCluster {
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
                    provider: row.provider.unwrap_or_default(),
                    additional_info: row.additional_informations.unwrap_or_default(),
                },
            })
            .collect();

        Ok(Page { items, total })
    }

    async fn by_id(&self, id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError> {
        let row = sqlx::query!(
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
        .ok_or(RepositoryError::NotFound)?;

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
            coordinates: row
                .latitude
                .zip(row.longitude)
                .and_then(|(lat, lng)| Coordinate::new(lat, lng).ok()),
            tree_ids: row.tree_ids.into_iter().map(Id::new).collect(),
            soil_condition: Some(row.soil_condition),
            name: row.name,
            provider_info: ProviderInfo {
                provider: row.provider.unwrap_or_default(),
                additional_info: row.additional_informations.unwrap_or_default(),
            },
        })
    }

    async fn by_ids(
        &self,
        ids: &[Id<TreeCluster>],
    ) -> Result<Vec<TreeCluster>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        let rows = sqlx::query!(
            r#"SELECT tc.id, tc.created_at, tc.updated_at, tc.name, tc.address, tc.description,
                      tc.archived, tc.moisture_level, tc.region_id, tc.last_watered,
                      tc.latitude, tc.longitude,
                      tc.watering_status AS "watering_status: WateringStatus",
                      tc.soil_condition AS "soil_condition: SoilCondition",
                      tc.provider, tc.additional_informations,
                      COALESCE(ARRAY_AGG(t.id) FILTER (WHERE t.id IS NOT NULL), ARRAY[]::int[]) AS "tree_ids!: Vec<i32>"
            FROM tree_clusters tc
            LEFT JOIN trees t ON t.tree_cluster_id = tc.id
            WHERE tc.id = ANY($1)
            GROUP BY tc.id"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| TreeCluster {
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
                    provider: row.provider.unwrap_or_default(),
                    additional_info: row.additional_informations.unwrap_or_default(),
                },
            })
            .collect())
    }

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

    async fn update(
        &self,
        id: Id<TreeCluster>,
        entity: TreeClusterUpdate,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"UPDATE tree_clusters SET
                name = COALESCE($2, name),
                address = COALESCE($3, address),
                description = COALESCE($4, description),
                soil_condition = COALESCE($5, soil_condition),
                provider = COALESCE($6, provider),
                additional_informations = COALESCE($7, additional_informations),
                latitude = COALESCE($8, latitude),
                longitude = COALESCE($9, longitude),
                region_id = CASE WHEN $10::bool THEN $11 ELSE region_id END
            WHERE id = $1"#,
            id.value(),
            entity.name,
            entity.address,
            entity.description,
            entity.soil_condition as Option<SoilCondition>,
            entity.provider_info.as_ref().map(|p| p.provider.as_str()),
            entity
                .provider_info
                .as_ref()
                .map(|p| p.additional_info.clone()),
            entity.coordinates.map(|c| c.latitude()),
            entity.coordinates.map(|c| c.longitude()),
            entity.region_id.is_some(),
            entity.region_id.flatten().map(|id| id.value()),
        )
        .execute(&mut *tx)
        .await?;

        if let Some(tree_ids) = entity.tree_ids {
            sqlx::query!(
                "UPDATE trees SET tree_cluster_id = NULL WHERE tree_cluster_id = $1",
                id.value()
            )
            .execute(&mut *tx)
            .await?;

            let tree_id_values: Vec<i32> = tree_ids.iter().map(|id| id.value()).collect();
            if !tree_id_values.is_empty() {
                sqlx::query!(
                    "UPDATE trees SET tree_cluster_id = $1 WHERE id = ANY($2)",
                    id.value(),
                    &tree_id_values
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE trees SET tree_cluster_id = NULL WHERE tree_cluster_id = $1",
            id.value()
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!("DELETE FROM tree_clusters WHERE id = $1", id.value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn archive(&self, id: Id<TreeCluster>) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE tree_clusters SET archived = true WHERE id = $1",
            id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

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
