use crate::shared::{coordinates::Coordinate, error::ValidationError, geo::BoundingBox};

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

macro_rules! seconds_value {
    ($name:ident, $field:literal, $min:literal, $max:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(i64);

        impl $name {
            pub const MIN: i64 = $min;
            pub const MAX: i64 = $max;

            pub fn new(seconds: i64) -> Result<Self, ValidationError> {
                if !(Self::MIN..=Self::MAX).contains(&seconds) {
                    return Err(ValidationError::OutOfRange {
                        field: $field,
                        min: Self::MIN as f64,
                        max: Self::MAX as f64,
                        got: seconds as f64,
                    });
                }
                Ok(Self(seconds))
            }

            pub fn seconds(&self) -> i64 {
                self.0
            }
        }
    };
}

seconds_value!(
    JustWateredTtl,
    "settings.just_watered_ttl",
    3_600,
    1_209_600,
    "How long a finished watering keeps covering the sensor-derived status."
);

seconds_value!(
    SensorOfflineAfter,
    "settings.sensor_offline_after",
    3_600,
    2_592_000,
    "Silence after which an activated sensor counts as offline. A value of a
few seconds would show every device as offline, hence the lower bound."
);

/// Consecutive unusable uplinks before a sensor is flagged as suspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefectStreak(i32);

impl DefectStreak {
    pub const MIN: i32 = 1;
    pub const MAX: i32 = 100;

    pub fn new(count: i32) -> Result<Self, ValidationError> {
        if !(Self::MIN..=Self::MAX).contains(&count) {
            return Err(ValidationError::OutOfRange {
                field: "settings.defect_streak",
                min: Self::MIN as f64,
                max: Self::MAX as f64,
                got: count as f64,
            });
        }
        Ok(Self(count))
    }

    pub fn count(&self) -> i32 {
        self.0
    }
}

/// Where the map opens: a centre plus the box it may pan in. `BoundingBox`
/// already rejects an inverted or antimeridian-crossing box; the extra rule
/// here is that the centre must lie inside it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapView {
    center: Coordinate,
    bbox: BoundingBox,
}

impl MapView {
    pub fn new(center: Coordinate, bbox: BoundingBox) -> Result<Self, ValidationError> {
        let lat_inside = (bbox.sw_lat()..=bbox.ne_lat()).contains(&center.latitude());
        let lng_inside = (bbox.sw_lng()..=bbox.ne_lng()).contains(&center.longitude());
        if !lat_inside || !lng_inside {
            return Err(ValidationError::InvalidFormat {
                field: "settings.map_view",
                reason: format!("center {center} lies outside the bounding box"),
            });
        }
        Ok(Self { center, bbox })
    }

    pub fn center(&self) -> Coordinate {
        self.center
    }

    pub fn bbox(&self) -> BoundingBox {
        self.bbox
    }
}

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

    #[test]
    fn sensor_offline_after_spans_an_hour_to_thirty_days() {
        assert_ok!(SensorOfflineAfter::new(3_600));
        assert_ok!(SensorOfflineAfter::new(2_592_000));
        assert_err!(SensorOfflineAfter::new(59));
        assert_err!(SensorOfflineAfter::new(2_592_001));
    }

    #[test]
    fn defect_streak_counts_from_one_to_a_hundred() {
        assert_ok!(DefectStreak::new(1));
        assert_ok!(DefectStreak::new(100));
        assert_err!(DefectStreak::new(0));
        assert_err!(DefectStreak::new(101));
    }

    #[test]
    fn map_view_requires_the_centre_inside_the_box() {
        let bbox = BoundingBox::try_new(54.7, 9.2, 54.9, 9.6).unwrap();
        assert_ok!(MapView::new(Coordinate::new(54.8, 9.4).unwrap(), bbox));

        let outside = Coordinate::new(54.6, 9.4).unwrap();
        let err = MapView::new(outside, bbox).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::InvalidFormat {
                field: "settings.map_view",
                ..
            }
        ));
    }
}
