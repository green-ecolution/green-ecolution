use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    region::{Region, RegionCreate, RegionQuery, RegionRepository, RegionUpdate},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
};

pub struct PgRegionRepository {
    pool: PgPool,
}

impl PgRegionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RegionRepository for PgRegionRepository {
    async fn all(
        &self,
        _query: RegionQuery,
        pagination: Pagination,
    ) -> Result<Page<Region>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM regions"#)
            .fetch_one(&self.pool)
            .await? as u64;

        let rows = sqlx::query!(
            r#"SELECT id, name, created_at, updated_at FROM regions
            ORDER BY name ASC, id ASC LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<Region> = rows
            .into_iter()
            .map(|row| Region {
                id: Id::new(row.id),
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                name: row.name,
            })
            .collect();

        Ok(Page { items, total })
    }

    async fn by_id(&self, id: Id<Region>) -> Result<Region, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, name, created_at, updated_at FROM regions WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Region {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            name: row.name,
        })
    }

    async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, RepositoryError> {
        let id_values: Vec<i32> = ids.iter().map(|id| id.value()).collect();
        let rows = sqlx::query!(
            r#"SELECT id, name, created_at, updated_at FROM regions WHERE id = ANY($1)"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Region {
                id: Id::new(row.id),
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                name: row.name,
            })
            .collect())
    }

    async fn by_point(&self, coord: Coordinate) -> Result<Region, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, name, created_at, updated_at FROM regions
            WHERE ST_Contains(geometry, ST_SetSRID(ST_MakePoint($1, $2), 4326))"#,
            coord.longitude(),
            coord.latitude()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Region {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            name: row.name,
        })
    }

    // TODO: Handle Geometry
    async fn create(&self, entity: RegionCreate) -> Result<Region, RepositoryError> {
        let row = sqlx::query!(
            r#"INSERT INTO regions (name) VALUES ($1) RETURNING id, created_at, updated_at, name"#,
            entity.name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Region {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            name: row.name,
        })
    }

    async fn update(
        &self,
        id: Id<Region>,
        entity: RegionUpdate,
    ) -> Result<Region, RepositoryError> {
        let row = sqlx::query!(
            r#"UPDATE regions SET name = $2 WHERE id = $1 RETURNING id, name, created_at, updated_at"#,
            id.value(),
            entity.name
        )
          .fetch_one(&self.pool)
          .await?;

        Ok(Region {
            id: Id::new(row.id),
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
            name: row.name,
        })
    }

    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError> {
        sqlx::query!(r#"DELETE FROM regions WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
