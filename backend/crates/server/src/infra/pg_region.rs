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

use crate::infra::list::{ListSpec, SortColumns};

pub struct PgRegionRepository {
    pool: PgPool,
}

impl PgRegionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// `RegionSnapshot` lives in `domain`, which must not depend on sqlx outside
/// its optional feature; `search` runs through the runtime-built list query,
/// which requires `sqlx::FromRow`, so this row type stays local to `infra`
/// and is mapped onto the snapshot.
#[derive(sqlx::FromRow)]
struct RegionRow {
    id: RawId,
    name: String,
}

impl From<RegionRow> for RegionSnapshot {
    fn from(row: RegionRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
        }
    }
}

const REGION_COLUMNS: &str = "r.id, r.name";
const REGION_SORT_COLUMNS: SortColumns = &[("name", &["r.name"])];

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
        let page = ListSpec::new("regions r", "r.id")
            .sort("name", false, REGION_SORT_COLUMNS)
            .page(pagination)
            .fetch::<RegionRow>(&self.pool, REGION_COLUMNS)
            .await?;

        Ok(Page {
            items: page
                .page
                .items
                .into_iter()
                .map(|row| Region::reconstitute(row.into()))
                .collect(),
            total: page.page.total,
        })
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
