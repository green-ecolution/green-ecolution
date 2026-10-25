use domain::{
    RepositoryError,
    module::{ModuleCount, ModuleStatistics},
};
use sqlx::PgPool;

pub struct IrrigationStatistics {
    pool: PgPool,
}

impl IrrigationStatistics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ModuleStatistics for IrrigationStatistics {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn counts(&self) -> Result<Vec<ModuleCount>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM trees)                                AS "trees!: i64",
                (SELECT COUNT(*) FROM tree_clusters WHERE archived = false) AS "clusters!: i64"
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(vec![
            ModuleCount {
                kind: "tree",
                count: row.trees,
            },
            ModuleCount {
                kind: "cluster",
                count: row.clusters,
            },
        ])
    }
}
