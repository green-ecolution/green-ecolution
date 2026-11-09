use chrono::{DateTime, Utc};

/// Lets services read the time without tying them to a runtime and lets tests
/// move it.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
