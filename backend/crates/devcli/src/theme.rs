use chrono::{DateTime, Utc};

use crate::supervisor::UnitState;

pub const UNIT_SLOTS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Good,
    Busy,
    Bad,
    Idle,
}

pub fn tone(state: &UnitState) -> Tone {
    match state {
        UnitState::Ready | UnitState::Done => Tone::Good,
        UnitState::Waiting { .. } | UnitState::Starting => Tone::Busy,
        UnitState::Failed { .. } => Tone::Bad,
        UnitState::Stopped => Tone::Idle,
    }
}

pub fn symbol(tone: Tone) -> &'static str {
    match tone {
        Tone::Good => "●",
        Tone::Busy => "◐",
        Tone::Bad => "✖",
        Tone::Idle => "○",
    }
}

/// Derived from the name rather than the position, so a unit keeps its colour across runs.
pub fn unit_slot(name: &str) -> usize {
    name.bytes().fold(0usize, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(usize::from(byte))
    }) % UNIT_SLOTS
}

pub fn ago(since: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (now - since).num_seconds().max(0);
    match seconds {
        0..60 => format!("{seconds}s"),
        60..3600 => format!("{}m", seconds / 60),
        _ => format!("{}h", seconds / 3600),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn maps_states_to_tones() {
        assert_eq!(tone(&UnitState::Ready), Tone::Good);
        assert_eq!(tone(&UnitState::Done), Tone::Good);
        assert_eq!(tone(&UnitState::Waiting { on: vec![] }), Tone::Busy);
        assert_eq!(tone(&UnitState::Starting), Tone::Busy);
        assert_eq!(
            tone(&UnitState::Failed {
                reason: String::new()
            }),
            Tone::Bad
        );
        assert_eq!(tone(&UnitState::Stopped), Tone::Idle);
    }

    #[test]
    fn unit_slots_are_stable_and_in_range() {
        assert_eq!(unit_slot("backend"), unit_slot("backend"));
        assert!(
            ["db", "backend", "frontend", "keycloak"]
                .iter()
                .all(|u| unit_slot(u) < UNIT_SLOTS)
        );
    }

    #[test]
    fn formats_elapsed_time_compactly() {
        let now = Utc::now();
        assert_eq!(ago(now - Duration::seconds(12), now), "12s");
        assert_eq!(ago(now - Duration::seconds(180), now), "3m");
        assert_eq!(ago(now - Duration::hours(2), now), "2h");
        assert_eq!(ago(now + Duration::seconds(5), now), "0s");
    }
}
