use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::settings::FullThreshold;
use crate::values::BinDepth;

/// A fill level between 0.0 (empty) and 1.0 (full) taken at a point in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillMeasurement {
    pub fraction: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillState {
    Empty,
    Normal,
    Full,
    Unknown,
}

impl FillState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Normal => "normal",
            Self::Full => "full",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for FillState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "empty" => Ok(Self::Empty),
            "normal" => Ok(Self::Normal),
            "full" => Ok(Self::Full),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!("unknown fill state: {other}")),
        }
    }
}

/// `None` for a reading no container can produce; the caller drops it.
pub fn fill_from_distance(distance_cm: f64, depth: BinDepth) -> Option<f64> {
    if !distance_cm.is_finite() || distance_cm < 0.0 {
        return None;
    }
    Some((1.0 - distance_cm / depth.centimeters()).clamp(0.0, 1.0))
}

/// Emptying after the last measurement wins, because the reading predates it.
pub fn derive_fill_state(
    fill: Option<FillMeasurement>,
    last_emptied_at: Option<DateTime<Utc>>,
    threshold: FullThreshold,
    now: DateTime<Utc>,
    stale_after: chrono::Duration,
) -> FillState {
    let Some(fill) = fill else {
        return FillState::Unknown;
    };
    if now - fill.measured_at > stale_after {
        return FillState::Unknown;
    }
    if last_emptied_at.is_some_and(|emptied| emptied > fill.measured_at) {
        return FillState::Empty;
    }
    if fill.fraction >= threshold.fraction() {
        FillState::Full
    } else {
        FillState::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn depth() -> BinDepth {
        BinDepth::new(100).unwrap()
    }
    fn t(h: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap() + Duration::hours(h)
    }
    fn threshold() -> FullThreshold {
        FullThreshold::new(80).unwrap()
    }
    fn measured(fraction: f64, at: i64) -> Option<FillMeasurement> {
        Some(FillMeasurement {
            fraction,
            measured_at: t(at),
        })
    }

    #[test]
    fn distance_maps_linearly_onto_the_depth() {
        assert_eq!(fill_from_distance(25.0, depth()), Some(0.75));
        assert_eq!(fill_from_distance(0.0, depth()), Some(1.0));
        assert_eq!(fill_from_distance(100.0, depth()), Some(0.0));
    }

    #[test]
    fn a_distance_beyond_the_depth_reads_as_empty() {
        assert_eq!(fill_from_distance(140.0, depth()), Some(0.0));
    }

    #[test]
    fn a_negative_distance_is_discarded() {
        assert_eq!(fill_from_distance(-1.0, depth()), None);
        assert_eq!(fill_from_distance(f64::NAN, depth()), None);
    }

    #[test]
    fn no_measurement_is_unknown() {
        assert_eq!(
            derive_fill_state(None, None, threshold(), t(0), Duration::hours(6)),
            FillState::Unknown
        );
    }

    #[test]
    fn a_stale_measurement_is_unknown() {
        assert_eq!(
            derive_fill_state(
                measured(0.9, 0),
                None,
                threshold(),
                t(7),
                Duration::hours(6)
            ),
            FillState::Unknown
        );
    }

    #[test]
    fn a_measurement_exactly_at_the_limit_still_counts() {
        assert_eq!(
            derive_fill_state(
                measured(0.9, 0),
                None,
                threshold(),
                t(6),
                Duration::hours(6)
            ),
            FillState::Full
        );
    }

    #[test]
    fn emptying_after_the_measurement_wins() {
        assert_eq!(
            derive_fill_state(
                measured(0.9, 0),
                Some(t(1)),
                threshold(),
                t(2),
                Duration::hours(6)
            ),
            FillState::Empty
        );
    }

    #[test]
    fn emptying_before_the_measurement_is_superseded() {
        assert_eq!(
            derive_fill_state(
                measured(0.9, 1),
                Some(t(0)),
                threshold(),
                t(2),
                Duration::hours(6)
            ),
            FillState::Full
        );
    }

    #[test]
    fn the_threshold_itself_counts_as_full() {
        assert_eq!(
            derive_fill_state(
                measured(0.8, 0),
                None,
                threshold(),
                t(1),
                Duration::hours(6)
            ),
            FillState::Full
        );
        assert_eq!(
            derive_fill_state(
                measured(0.79, 0),
                None,
                threshold(),
                t(1),
                Duration::hours(6)
            ),
            FillState::Normal
        );
    }

    #[test]
    fn fill_state_round_trips_through_its_string_form() {
        for s in [
            FillState::Empty,
            FillState::Normal,
            FillState::Full,
            FillState::Unknown,
        ] {
            assert_eq!(s.as_str().parse::<FillState>(), Ok(s));
        }
        assert!("bogus".parse::<FillState>().is_err());
    }
}
