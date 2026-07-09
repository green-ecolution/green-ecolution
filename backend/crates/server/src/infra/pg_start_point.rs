use sqlx::PgPool;

use domain::start_point::StartPointSnapshot;
use domain::{
    Id, RepositoryError,
    start_point::{StartPoint, StartPointDraft, StartPointReader, StartPointWriter},
};

pub struct PgStartPointRepository {
    pool: PgPool,
}

impl PgStartPointRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StartPointReader for PgStartPointRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn all(&self) -> Result<Vec<StartPoint>, RepositoryError> {
        let points = sqlx::query_as!(
            StartPointSnapshot,
            r#"SELECT id, name, lat, lon, watering_point, is_default
               FROM depots ORDER BY name ASC, id ASC"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(StartPoint::reconstitute)
        .collect();

        Ok(points)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<StartPoint>) -> Result<StartPoint, RepositoryError> {
        sqlx::query_as!(
            StartPointSnapshot,
            r#"SELECT id, name, lat, lon, watering_point, is_default
               FROM depots WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)
        .map(StartPoint::reconstitute)
    }
}

#[async_trait::async_trait]
impl StartPointWriter for PgStartPointRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: StartPointDraft) -> Result<StartPoint, RepositoryError> {
        let id = Id::<StartPoint>::new_v7();
        sqlx::query!(
            r#"INSERT INTO depots (id, name, lat, lon, watering_point, is_default)
               VALUES ($1, $2, $3, $4, $5, FALSE)"#,
            id.value(),
            draft.name.as_str(),
            draft.coordinate.latitude(),
            draft.coordinate.longitude(),
            draft.watering_point,
        )
        .execute(&self.pool)
        .await?;

        Ok(StartPoint::reconstitute(StartPointSnapshot {
            id: id.value(),
            name: draft.name.as_str().to_string(),
            lat: draft.coordinate.latitude(),
            lon: draft.coordinate.longitude(),
            watering_point: draft.watering_point,
            is_default: false,
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, start_point: &StartPoint) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE depots SET name = $2, lat = $3, lon = $4, watering_point = $5
               WHERE id = $1"#,
            start_point.id.value(),
            start_point.name.as_str(),
            start_point.coordinate.latitude(),
            start_point.coordinate.longitude(),
            start_point.watering_point(),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<StartPoint>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM depots WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn set_default(&self, id: Id<StartPoint>) -> Result<(), RepositoryError> {
        // Single statement: exactly one row ends up TRUE, so the partial unique
        // index is never violated mid-transaction.
        let result = sqlx::query!(r#"UPDATE depots SET is_default = (id = $1)"#, id.value())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
