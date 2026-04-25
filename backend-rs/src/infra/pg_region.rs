use sqlx::PgPool;

use crate::domain::{
    Id, RepositoryError,
    region::{Region, RegionCreate, RegionRepository, RegionUpdate},
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
    async fn all(&self, pagination: Pagination) -> Result<Page<Region>, RepositoryError> {
        let total = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM regions"#)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"SELECT id, name, created_at, updated_at FROM regions LIMIT $1 OFFSET $2"#,
            pagination.limit as i64,
            pagination.offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<Region> = rows
            .into_iter()
            .map(|row| {
                Region::new(
                    Id::new(row.id),
                    row.created_at.and_utc(),
                    row.updated_at.and_utc(),
                    row.name,
                )
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

        Ok(Region::new(
            Id::new(row.id),
            row.created_at.and_utc(),
            row.updated_at.and_utc(),
            row.name,
        ))
    }

    async fn by_point(&self, coord: Coordinate) -> Result<Region, RepositoryError> {
        let row = sqlx::query!(
        r#"SELECT id, name, created_at, updated_at FROM regions WHERE ST_Contains(geometry, ST_GeomFromText($1, 4326))"#,
        format!("POINT({} {})", coord.longitude(), coord.latitude())
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Region::new(
            Id::new(row.id),
            row.created_at.and_utc(),
            row.updated_at.and_utc(),
            row.name,
        ))
    }

    async fn create(&self, entity: RegionCreate) -> Result<Region, RepositoryError> {
        todo!()
    }

    async fn update(
        &self,
        id: Id<Region>,
        entity: RegionUpdate,
    ) -> Result<Region, RepositoryError> {
        todo!()
    }

    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError> {
        todo!()
    }
}
