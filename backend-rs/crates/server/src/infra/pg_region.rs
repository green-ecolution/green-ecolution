use sqlx::PgPool;

use domain::region::RegionSnapshot;
use domain::{
    Id, IdSliceExt, RawId, RepositoryError,
    region::{Region, RegionDraft, RegionName, RegionReader, RegionSearchQuery, RegionWriter},
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
impl RegionReader for PgRegionRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Region>) -> Result<Region, RepositoryError> {
        sqlx::query_as!(
            RegionSnapshot,
            r#"SELECT id, name FROM regions WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)
        .map(Region::reconstitute)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Id<Region>]) -> Result<Vec<Region>, RepositoryError> {
        let id_values: Vec<RawId> = ids.to_values();
        let regions = sqlx::query_as!(
            RegionSnapshot,
            r#"SELECT id, name FROM regions WHERE id = ANY($1::uuid[])"#,
            &id_values
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Region::reconstitute)
        .collect();

        Ok(regions)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_name(&self, name: &RegionName) -> Result<Option<Region>, RepositoryError> {
        let region = sqlx::query_as!(
            RegionSnapshot,
            r#"SELECT id, name FROM regions WHERE name = $1"#,
            name.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Region::reconstitute);

        Ok(region)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_point(&self, coord: Coordinate) -> Result<Option<Region>, RepositoryError> {
        let region = sqlx::query_as!(
            RegionSnapshot,
            r#"SELECT id, name FROM regions
            WHERE ST_Contains(geometry, ST_SetSRID(ST_MakePoint($1, $2), 4326))
            LIMIT 1"#,
            coord.longitude(),
            coord.latitude()
        )
        .fetch_optional(&self.pool)
        .await?
        .map(Region::reconstitute);

        Ok(region)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn search(
        &self,
        _query: RegionSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<Region>, RepositoryError> {
        let limit = i64::try_from(pagination.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(pagination.offset()).unwrap_or(i64::MAX);

        let total = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM regions"#)
            .fetch_one(&self.pool)
            .await? as u64;

        let items = sqlx::query_as!(
            RegionSnapshot,
            r#"SELECT id, name FROM regions ORDER BY name ASC, id ASC LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Region::reconstitute)
        .collect();

        Ok(Page { items, total })
    }
}

#[async_trait::async_trait]
impl RegionWriter for PgRegionRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: RegionDraft) -> Result<Region, RepositoryError> {
        let id = Id::<Region>::new_v7();
        sqlx::query!(
            r#"INSERT INTO regions (id, name) VALUES ($1, $2)"#,
            id.value(),
            draft.name.as_str()
        )
        .execute(&self.pool)
        .await?;

        Ok(Region {
            id,
            name: draft.name,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, region: &Region) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE regions SET name = $2 WHERE id = $1"#,
            region.id.value(),
            region.name.as_str()
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Region>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM regions WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
