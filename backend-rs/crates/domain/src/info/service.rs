//! Service availability status.

use std::time::Duration;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub enabled: bool,
    pub healthy: bool,
    pub response_time: Duration,
    pub last_checked: DateTime<Utc>,
    pub message: String,
}
