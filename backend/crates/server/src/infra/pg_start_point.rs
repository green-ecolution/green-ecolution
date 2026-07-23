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
            r#"SELECT id, name, latitude, longitude, watering_point, is_default, organization_id
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
            r#"SELECT id, name, latitude, longitude, watering_point, is_default, organization_id
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
            r#"INSERT INTO depots (id, name, latitude, longitude, geometry, watering_point, is_default, organization_id)
               VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($4, $3), 4326), $5, FALSE, $6)"#,
            id.value(),
            draft.name.as_str(),
            draft.coordinate.latitude(),
            draft.coordinate.longitude(),
            draft.watering_point,
            draft.organization_id.value(),
        )
        .execute(&self.pool)
        .await?;

        Ok(StartPoint::reconstitute(StartPointSnapshot {
            id: id.value(),
            name: draft.name.as_str().to_string(),
            latitude: draft.coordinate.latitude(),
            longitude: draft.coordinate.longitude(),
            watering_point: draft.watering_point,
            is_default: false,
            organization_id: draft.organization_id.value(),
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, start_point: &StartPoint) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE depots
               SET name = $2, latitude = $3, longitude = $4,
                   geometry = ST_SetSRID(ST_MakePoint($4, $3), 4326), watering_point = $5,
                   organization_id = $6, is_default = $7
               WHERE id = $1"#,
            start_point.id.value(),
            start_point.name.as_str(),
            start_point.coordinate.latitude(),
            start_point.coordinate.longitude(),
            start_point.watering_point(),
            start_point.organization_id().value(),
            start_point.is_default(),
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
        let mut tx = self.pool.begin().await?;

        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM depots WHERE id = $1) AS "exists!""#,
            id.value()
        )
        .fetch_one(&mut *tx)
        .await?;
        if !exists {
            return Err(RepositoryError::NotFound);
        }

        // Two statements in one transaction: clearing the target's org's
        // defaults completes (emptying the partial unique index for that org)
        // before the target is set, so no transient two-defaults state can
        // violate depots_single_default_per_org — unlike a single UPDATE,
        // which the index checks per row mid-statement. Scoped to the
        // target's own organization_id, so other orgs' defaults are untouched.
        sqlx::query!(
            r#"UPDATE depots SET is_default = FALSE
               WHERE is_default AND organization_id = (SELECT organization_id FROM depots WHERE id = $1)"#,
            id.value()
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            r#"UPDATE depots SET is_default = TRUE WHERE id = $1"#,
            id.value()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
