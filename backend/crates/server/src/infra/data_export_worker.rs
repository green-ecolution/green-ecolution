use std::{sync::Arc, time::Duration};

use tokio::{
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

use crate::service::data_export_sync::DataExportSync;

pub fn spawn(sync: Arc<DataExportSync>, every: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        // `interval` panics on zero; a misconfigured 0 becomes one second.
        let mut ticker = interval(every.max(Duration::from_secs(1)));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            sync.run_due().await;
        }
    })
}
