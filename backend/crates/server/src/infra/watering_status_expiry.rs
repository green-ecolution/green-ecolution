use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

use crate::service::cluster_service::ClusterService;

/// Periodically releases the `JustWatered` status once its grace period has
/// passed. The first sweep runs immediately so statuses that expired while the
/// server was down are cleaned up on boot.
pub fn spawn(
    cluster_service: Arc<ClusterService>,
    sweep_interval: Duration,
    ttl: chrono::Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(sweep_interval.max(Duration::from_secs(1)));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            match cluster_service.expire_just_watered(Utc::now() - ttl).await {
                Ok(0) => {}
                Ok(count) => tracing::info!(clusters = count, "released just-watered status"),
                Err(error) => tracing::warn!(%error, "just-watered expiry sweep failed"),
            }
        }
    })
}
