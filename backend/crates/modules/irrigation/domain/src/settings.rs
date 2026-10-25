use domain::shared::error::ValidationError;

/// Litres of water a single tree is planned with. The upper bound is not
/// cosmetic: a typo here shifts route planning by orders of magnitude,
/// because the demand of a cluster is this value times its tree count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterDemand(f64);

impl WaterDemand {
    pub const MIN: f64 = 1.0;
    pub const MAX: f64 = 1_000.0;

    pub fn new(liters: f64) -> Result<Self, ValidationError> {
        if !liters.is_finite() || !(Self::MIN..=Self::MAX).contains(&liters) {
            return Err(ValidationError::OutOfRange {
                field: "settings.water_demand",
                min: Self::MIN,
                max: Self::MAX,
                got: liters,
            });
        }
        Ok(Self(liters))
    }

    pub fn liters(&self) -> f64 {
        self.0
    }
}

domain::seconds_value!(
    JustWateredTtl,
    "settings.just_watered_ttl",
    3_600,
    1_209_600,
    "How long a finished watering keeps covering the sensor-derived status."
);

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn water_demand_accepts_the_documented_range() {
        assert_ok!(WaterDemand::new(1.0));
        assert_ok!(WaterDemand::new(80.0));
        assert_ok!(WaterDemand::new(1000.0));
    }

    #[test]
    fn water_demand_rejects_outside_the_range_with_a_field_label() {
        let err = WaterDemand::new(0.5).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::OutOfRange {
                field: "settings.water_demand",
                ..
            }
        ));
        assert_err!(WaterDemand::new(1000.1));
        assert_err!(WaterDemand::new(f64::NAN));
    }

    #[test]
    fn just_watered_ttl_spans_an_hour_to_a_fortnight() {
        assert_ok!(JustWateredTtl::new(3_600));
        assert_ok!(JustWateredTtl::new(1_209_600));
        assert_err!(JustWateredTtl::new(3_599));
        assert_err!(JustWateredTtl::new(1_209_601));
    }
}
