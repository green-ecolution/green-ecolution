use domain::{
    RepositoryError,
    module::{ModuleCount, ModuleStatistics},
};
use sqlx::PgPool;

pub struct WasteStatistics {
    pool: PgPool,
}

impl WasteStatistics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ModuleStatistics for WasteStatistics {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn counts(&self) -> Result<Vec<ModuleCount>, RepositoryError> {
        let bins = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM waste_bins"#)
            .fetch_one(&self.pool)
            .await?;
        Ok(vec![ModuleCount {
            kind: "bin",
            count: bins,
        }])
    }
}
