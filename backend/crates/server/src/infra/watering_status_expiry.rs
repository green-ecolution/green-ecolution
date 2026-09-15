use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use domain::organization::OrganizationReader;
use domain::settings::SettingsResolver;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

use crate::service::cluster_service::ClusterService;

/// Periodically releases the `JustWatered` status once its grace period has
/// passed, per organization's own TTL. The first sweep runs immediately so
/// statuses that expired while the server was down are cleaned up on boot.
///
/// The cutoff is computed once per sweep from a single `Utc::now()` and then
/// combined with each organization's TTL, so it does not drift between
/// organizations within the same run.
pub fn spawn(
    cluster_service: Arc<ClusterService>,
    org_reader: Arc<dyn OrganizationReader>,
    settings: Arc<dyn SettingsResolver>,
    sweep_interval: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(sweep_interval.max(Duration::from_secs(1)));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let orgs = match org_reader.all().await {
                Ok(orgs) => orgs,
                Err(error) => {
                    tracing::warn!(%error, "just-watered sweep could not read organizations");
                    continue;
                }
            };
            let now = Utc::now();
            let mut released = 0;
            for org in &orgs {
                let ttl = match settings.effective_for(org.id).await {
                    Ok(settings) => chrono::Duration::seconds(settings.just_watered_ttl.seconds()),
                    Err(error) => {
                        tracing::warn!(%error, organization.id = %org.id, "skipping organization in just-watered sweep");
                        continue;
                    }
                };
                match cluster_service.expire_just_watered(org.id, now - ttl).await {
                    Ok(count) => released += count,
                    Err(error) => {
                        tracing::warn!(%error, organization.id = %org.id, "just-watered expiry failed")
                    }
                }
            }
            if released > 0 {
                tracing::info!(clusters = released, "released just-watered status");
            }
        }
    })
}
