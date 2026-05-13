use domain::info::RuntimeStatsProvider;
use server::infra::runtime_stats::DefaultRuntimeStatsProvider;

use crate::helpers::spawn_app;

#[tokio::test]
async fn runtime_stats_snapshot_returns_nonzero_memory_and_cpu_cores() {
    let app = spawn_app().await;
    let provider = DefaultRuntimeStatsProvider::new(app.db_pool.clone());

    // sysinfo CPU usage is 0.0 on the very first sample.
    // Sleep + take a second snapshot so we exercise the refresh path.
    let _first = provider.snapshot().await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let stats = provider.snapshot().await;

    assert!(stats.memory.resident_bytes > 0, "RSS should be > 0");
    assert!(stats.cpu.cores >= 1, "at least 1 core");
    assert!(stats.tokio.worker_threads >= 1, "tokio workers >= 1");
    assert!(stats.db_pool.max >= 1, "pool max >= 1");
}
