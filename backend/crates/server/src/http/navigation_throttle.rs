use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use uuid::Uuid;

/// Per-replica guard against a client hammering the routing engine while
/// rerouting. Not a security boundary: replicas do not share it.
pub struct NavigationThrottle {
    min_interval: Duration,
    last: Mutex<HashMap<Uuid, Instant>>,
}

const PRUNE_ABOVE: usize = 10_000;

impl NavigationThrottle {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last: Mutex::new(HashMap::new()),
        }
    }

    pub fn admit(&self, user: Uuid, now: Instant) -> bool {
        let mut last = self
            .last
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if last.len() > PRUNE_ABOVE {
            last.retain(|_, at| now.duration_since(*at) < self.min_interval);
        }
        match last.get(&user) {
            Some(at) if now.duration_since(*at) < self.min_interval => false,
            _ => {
                last.insert(user, now);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_call_inside_the_interval_is_refused() {
        let t = NavigationThrottle::new(Duration::from_secs(2));
        let user = Uuid::now_v7();
        let now = Instant::now();
        assert!(t.admit(user, now));
        assert!(!t.admit(user, now + Duration::from_millis(500)));
        assert!(t.admit(user, now + Duration::from_secs(2)));
    }

    #[test]
    fn users_do_not_share_a_budget() {
        let t = NavigationThrottle::new(Duration::from_secs(2));
        let now = Instant::now();
        assert!(t.admit(Uuid::now_v7(), now));
        assert!(t.admit(Uuid::now_v7(), now));
    }
}
