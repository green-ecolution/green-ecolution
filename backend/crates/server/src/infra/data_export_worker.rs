use std::{sync::Arc, time::Duration};

use tokio::{
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

use crate::service::data_export_sync::DataExportSync;

/// The database decides when a target is due (`last_run_at` plus the interval),
/// so the worker only polls. Ticking once per interval would skip every second
/// run, because `last_run_at` is the end of the previous pass.
const MAX_POLL: Duration = Duration::from_secs(30);

pub fn spawn(sync: Arc<DataExportSync>, every: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(poll_period(every));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            sync.run_due().await;
        }
    })
}

/// `interval` panics on zero; a misconfigured 0 becomes one second.
fn poll_period(every: Duration) -> Duration {
    every.min(MAX_POLL).max(Duration::from_secs(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_poll_is_capped_and_never_zero() {
        assert_eq!(poll_period(Duration::from_secs(300)), MAX_POLL);
        assert_eq!(poll_period(Duration::from_secs(5)), Duration::from_secs(5));
        assert_eq!(poll_period(Duration::ZERO), Duration::from_secs(1));
    }
}
