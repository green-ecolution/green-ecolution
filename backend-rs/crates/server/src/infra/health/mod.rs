pub mod feature_probe;
pub mod keycloak_probe;
pub mod mqtt_probe;
pub mod pg_probe;
pub mod probe;
pub mod readiness;

pub use probe::HealthProbe;
pub use readiness::DefaultReadiness;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

use domain::info::{HealthSnapshotReader, ServiceStatus};

pub struct HealthCoordinator {
    snapshot: Arc<RwLock<Vec<ServiceStatus>>>,
}

#[async_trait]
impl HealthSnapshotReader for HealthCoordinator {
    async fn snapshot(&self) -> Vec<ServiceStatus> {
        self.snapshot.read().await.clone()
    }
}

pub async fn spawn(
    probes: Vec<Arc<dyn HealthProbe>>,
    interval_duration: Duration,
) -> (Arc<HealthCoordinator>, JoinHandle<()>) {
    let initial = futures::future::join_all(probes.iter().map(|p| p.check())).await;
    let snapshot = Arc::new(RwLock::new(initial));

    let coordinator = Arc::new(HealthCoordinator {
        snapshot: snapshot.clone(),
    });

    let handle = tokio::spawn(async move {
        let mut ticker = interval(interval_duration);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let results = futures::future::join_all(probes.iter().map(|p| p.check())).await;
            *snapshot.write().await = results;
        }
    });

    (coordinator, handle)
}
