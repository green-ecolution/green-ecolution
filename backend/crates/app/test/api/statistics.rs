use domain::info::StatisticsReader;
use server::infra::statistics_repo::PgStatisticsRepo;

use crate::helpers::spawn_app;

#[tokio::test]
async fn statistics_returns_zero_for_empty_database() {
    let app = spawn_app().await;
    let repo = PgStatisticsRepo::new(app.db_pool.clone());

    let stats = repo.statistics().await.unwrap();
    assert_eq!(stats.tree_count, 0);
    assert_eq!(stats.sensor_count, 0);
    assert_eq!(stats.vehicle_count, 0);
    assert_eq!(stats.cluster_count, 0);
    assert_eq!(stats.watering_plan_count, 0);
}
