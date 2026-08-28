//! Plausibility rules for incoming sensor readings.
//!
//! Rules are checked in order and the first violation wins. An ability with no
//! entry in a rule table is deliberately treated as plausible: a newly added
//! ability must not be rejected before its bounds are known.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};

use crate::sensor_model::{SensorAbilityName, SensorAbilityUnit};

/// Stable discriminator persisted alongside a rejected value and localized by
/// the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlausibilityReason {
    OutOfRange,
    ImplausibleJump,
}

impl PlausibilityReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OutOfRange => "out_of_range",
            Self::ImplausibleJump => "implausible_jump",
        }
    }
}

impl fmt::Display for PlausibilityReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("unknown plausibility reason: {0}")]
pub struct UnknownPlausibilityReason(pub String);

impl FromStr for PlausibilityReason {
    type Err = UnknownPlausibilityReason;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "out_of_range" => Ok(Self::OutOfRange),
            "implausible_jump" => Ok(Self::ImplausibleJump),
            other => Err(UnknownPlausibilityReason(other.to_owned())),
        }
    }
}

/// A violated rule, carrying the numbers that produced the verdict so a log
/// line or an API response can explain it without recomputing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlausibilityIssue {
    OutOfRange {
        min: f64,
        max: f64,
    },
    ImplausibleJump {
        previous: f64,
        per_hour: f64,
        limit: f64,
    },
}

impl PlausibilityIssue {
    pub fn reason(self) -> PlausibilityReason {
        match self {
            Self::OutOfRange { .. } => PlausibilityReason::OutOfRange,
            Self::ImplausibleJump { .. } => PlausibilityReason::ImplausibleJump,
        }
    }
}

/// Inclusive bounds for one ability in one unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityRange {
    pub min: f64,
    pub max: f64,
}

/// Physically (or device-technically) possible bounds. `None` for a pair with
/// no entry — see the module docs for why that counts as plausible.
pub fn physical_range(ability: SensorAbilityName, unit: SensorAbilityUnit) -> Option<AbilityRange> {
    use SensorAbilityName as N;
    use SensorAbilityUnit as U;

    let (min, max) = match (ability, unit) {
        (N::SoilMoisture, U::Percent) => (0.0, 100.0),
        // Watermark probes saturate at 239 cb.
        (N::SoilTension, U::Centibar) => (0.0, 239.0),
        (N::SoilTension, U::Ohm) => (0.0, 40_000.0),
        (N::Temperature, U::Celsius) => (-40.0, 80.0),
        (N::Humidity, U::Percent) => (0.0, 100.0),
        (N::Battery, U::Volt) => (0.0, 20.0),
        _ => return None,
    };
    Some(AbilityRange { min, max })
}

/// Largest change per hour still considered real. Generous on purpose:
/// irrigation raises soil moisture fast and must not be flagged. `None`
/// disables the rule for that ability.
pub fn jump_limit_per_hour(ability: SensorAbilityName) -> Option<f64> {
    match ability {
        SensorAbilityName::SoilMoisture => Some(40.0),
        SensorAbilityName::SoilTension => Some(200.0),
        SensorAbilityName::Temperature => Some(15.0),
        SensorAbilityName::Humidity => Some(60.0),
        // A battery swap resets the voltage; any limit would be noise.
        SensorAbilityName::Battery => None,
    }
}

/// What the jump rule needs beyond the value itself.
#[derive(Debug, Clone, Copy)]
pub struct ReadingContext {
    /// Last plausible value of the same ability at the same depth.
    pub previous: Option<(f64, DateTime<Utc>)>,
    pub recorded_at: DateTime<Utc>,
}

/// First violated rule, or `None` when the value is plausible.
pub fn evaluate(
    ability: SensorAbilityName,
    unit: SensorAbilityUnit,
    value: f64,
    ctx: &ReadingContext,
) -> Option<PlausibilityIssue> {
    if let Some(range) = physical_range(ability, unit)
        && (value < range.min || value > range.max)
    {
        return Some(PlausibilityIssue::OutOfRange {
            min: range.min,
            max: range.max,
        });
    }

    let (previous, previous_at) = ctx.previous?;
    let limit = jump_limit_per_hour(ability)?;
    let hours = (ctx.recorded_at - previous_at).num_milliseconds() as f64 / 3_600_000.0;
    if hours <= 0.0 {
        return None;
    }
    let per_hour = (value - previous).abs() / hours;
    (per_hour > limit).then_some(PlausibilityIssue::ImplausibleJump {
        previous,
        per_hour,
        limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensor_model::{SensorAbilityName as N, SensorAbilityUnit as U};
    use chrono::{Duration, TimeZone, Utc};

    fn now() -> chrono::DateTime<chrono::Utc> {
        Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap()
    }

    fn ctx_without_previous() -> ReadingContext {
        ReadingContext {
            previous: None,
            recorded_at: now(),
        }
    }

    fn ctx_with_previous(value: f64, ago: Duration) -> ReadingContext {
        ReadingContext {
            previous: Some((value, now() - ago)),
            recorded_at: now(),
        }
    }

    #[test]
    fn value_on_the_upper_bound_is_plausible() {
        assert_eq!(
            evaluate(N::SoilMoisture, U::Percent, 100.0, &ctx_without_previous()),
            None
        );
    }

    #[test]
    fn value_on_the_lower_bound_is_plausible() {
        assert_eq!(
            evaluate(N::SoilMoisture, U::Percent, 0.0, &ctx_without_previous()),
            None
        );
    }

    #[test]
    fn dragino_sentinel_is_out_of_range() {
        let issue = evaluate(N::SoilMoisture, U::Percent, 6553.5, &ctx_without_previous())
            .expect("sentinel must be flagged");
        assert_eq!(issue.reason(), PlausibilityReason::OutOfRange);
        assert!(matches!(
            issue,
            PlausibilityIssue::OutOfRange { min, max } if min == 0.0 && max == 100.0
        ));
    }

    #[test]
    fn watermark_saturation_value_is_plausible() {
        assert_eq!(
            evaluate(N::SoilTension, U::Centibar, 239.0, &ctx_without_previous()),
            None
        );
        assert!(evaluate(N::SoilTension, U::Centibar, 240.0, &ctx_without_previous()).is_some());
    }

    #[test]
    fn negative_temperature_within_range_is_plausible() {
        assert_eq!(
            evaluate(N::Temperature, U::Celsius, -40.0, &ctx_without_previous()),
            None
        );
        assert!(evaluate(N::Temperature, U::Celsius, -41.0, &ctx_without_previous()).is_some());
    }

    #[test]
    fn unlisted_ability_unit_pair_has_no_range_rule() {
        assert_eq!(
            evaluate(N::Battery, U::Percent, 9_999.0, &ctx_without_previous()),
            None
        );
    }

    #[test]
    fn missing_previous_value_skips_the_jump_rule() {
        assert_eq!(
            evaluate(N::SoilMoisture, U::Percent, 90.0, &ctx_without_previous()),
            None
        );
    }

    #[test]
    fn watering_sized_moisture_rise_stays_plausible() {
        // 20 Vol.-% in one hour: a real irrigation event, not a fault.
        let ctx = ctx_with_previous(25.0, Duration::hours(1));
        assert_eq!(evaluate(N::SoilMoisture, U::Percent, 45.0, &ctx), None);
    }

    #[test]
    fn moisture_jump_beyond_the_limit_is_flagged() {
        let ctx = ctx_with_previous(10.0, Duration::hours(1));
        let issue =
            evaluate(N::SoilMoisture, U::Percent, 90.0, &ctx).expect("80 Vol.-%/h must be flagged");
        assert_eq!(issue.reason(), PlausibilityReason::ImplausibleJump);
        assert!(matches!(
            issue,
            PlausibilityIssue::ImplausibleJump { previous, limit, .. }
                if previous == 10.0 && limit == 40.0
        ));
    }

    #[test]
    fn jump_rate_is_normalized_per_hour() {
        // 60 Vol.-% over three hours is 20/h and therefore plausible.
        let ctx = ctx_with_previous(10.0, Duration::hours(3));
        assert_eq!(evaluate(N::SoilMoisture, U::Percent, 70.0, &ctx), None);
    }

    #[test]
    fn zero_elapsed_time_skips_the_jump_rule() {
        let ctx = ctx_with_previous(10.0, Duration::zero());
        assert_eq!(evaluate(N::SoilMoisture, U::Percent, 90.0, &ctx), None);
    }

    #[test]
    fn negative_elapsed_time_skips_the_jump_rule() {
        let ctx = ctx_with_previous(10.0, Duration::hours(-2));
        assert_eq!(evaluate(N::SoilMoisture, U::Percent, 90.0, &ctx), None);
    }

    #[test]
    fn battery_has_no_jump_rule() {
        let ctx = ctx_with_previous(2.9, Duration::minutes(10));
        assert_eq!(evaluate(N::Battery, U::Volt, 3.7, &ctx), None);
    }

    #[test]
    fn range_is_checked_before_jump() {
        let ctx = ctx_with_previous(10.0, Duration::hours(1));
        let issue = evaluate(N::SoilMoisture, U::Percent, 6553.5, &ctx).expect("flagged");
        assert_eq!(issue.reason(), PlausibilityReason::OutOfRange);
    }

    #[test]
    fn reason_round_trips_through_display_and_from_str() {
        for reason in [
            PlausibilityReason::OutOfRange,
            PlausibilityReason::ImplausibleJump,
        ] {
            let text = reason.to_string();
            assert_eq!(text.parse::<PlausibilityReason>().unwrap(), reason);
        }
        assert_eq!(PlausibilityReason::OutOfRange.to_string(), "out_of_range");
        assert_eq!(
            PlausibilityReason::ImplausibleJump.to_string(),
            "implausible_jump"
        );
    }

    #[test]
    fn unknown_reason_string_is_rejected() {
        assert!("above_soil_capacity".parse::<PlausibilityReason>().is_err());
    }
}
