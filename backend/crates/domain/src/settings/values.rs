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

/// One step of the map's zoom scale. The upper end is the deepest level web
/// map tiles are cut to; below 0 there is no map left to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZoomLevel(u8);

impl ZoomLevel {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 24;

    pub fn new(level: u8) -> Result<Self, ValidationError> {
        if level > Self::MAX {
            return Err(ValidationError::OutOfRange {
                field: "settings.map_view.zoom",
                min: Self::MIN as f64,
                max: Self::MAX as f64,
                got: level as f64,
            });
        }
        Ok(Self(level))
    }

    pub fn level(&self) -> u8 {
        self.0
    }
}

/// How far a map may be panned and how far it may be zoomed. The three travel
/// together on purpose: an organization either holds its map to a working area
/// or it does not, and a zoom range without a pan limit would restrict half a
/// map for no reason anyone could explain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapBounds {
    bbox: BoundingBox,
    min_zoom: ZoomLevel,
    max_zoom: ZoomLevel,
}

impl MapBounds {
    pub fn new(
        bbox: BoundingBox,
        min_zoom: ZoomLevel,
        max_zoom: ZoomLevel,
    ) -> Result<Self, ValidationError> {
        if min_zoom > max_zoom {
            return Err(ValidationError::InvalidFormat {
                field: "settings.map_view.zoom",
                reason: format!(
                    "minimum zoom ({}) must not exceed maximum zoom ({})",
                    min_zoom.level(),
                    max_zoom.level()
                ),
            });
        }
        Ok(Self {
            bbox,
            min_zoom,
            max_zoom,
        })
    }

    pub fn bbox(&self) -> BoundingBox {
        self.bbox
    }

    pub fn min_zoom(&self) -> ZoomLevel {
        self.min_zoom
    }

    pub fn max_zoom(&self) -> ZoomLevel {
        self.max_zoom
    }
}

/// Where the map opens: a centre, plus optionally the limits it is held to. No
/// limits means the map is not penned in at all, which is the right answer for
/// an organization whose work is not confined to one municipality. A centre
/// there always is, because the map has to open somewhere.
///
/// `BoundingBox` already rejects an inverted or antimeridian-crossing box; the
/// extra rule here is that the centre must lie inside it, since a map that
/// opens outside its own limit would show a place it may not pan to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapView {
    center: Coordinate,
    bounds: Option<MapBounds>,
}

impl MapView {
    pub fn new(center: Coordinate, bounds: Option<MapBounds>) -> Result<Self, ValidationError> {
        if let Some(bounds) = bounds {
            let bbox = bounds.bbox();
            let lat_inside = (bbox.sw_lat()..=bbox.ne_lat()).contains(&center.latitude());
            let lng_inside = (bbox.sw_lng()..=bbox.ne_lng()).contains(&center.longitude());
            if !lat_inside || !lng_inside {
                return Err(ValidationError::InvalidFormat {
                    field: "settings.map_view",
                    reason: format!("center {center} lies outside the bounding box"),
                });
            }
        }
        Ok(Self { center, bounds })
    }

    pub fn center(&self) -> Coordinate {
        self.center
    }

    /// `None` means the map may be panned and zoomed freely.
    pub fn bounds(&self) -> Option<MapBounds> {
        self.bounds
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
        assert_err!(SensorOfflineAfter::new(3_599));
        assert_err!(SensorOfflineAfter::new(2_592_001));
    }

    #[test]
    fn defect_streak_counts_from_one_to_a_hundred() {
        assert_ok!(DefectStreak::new(1));
        assert_ok!(DefectStreak::new(100));
        assert_err!(DefectStreak::new(0));
        assert_err!(DefectStreak::new(101));
    }

    fn zoom(level: u8) -> ZoomLevel {
        ZoomLevel::new(level).unwrap()
    }

    fn bounds() -> MapBounds {
        MapBounds::new(
            BoundingBox::try_new(54.7, 9.2, 54.9, 9.6).unwrap(),
            zoom(13),
            zoom(18),
        )
        .unwrap()
    }

    #[test]
    fn zoom_level_stops_at_the_deepest_tiled_level() {
        assert_ok!(ZoomLevel::new(0));
        assert_ok!(ZoomLevel::new(24));
        assert_err!(ZoomLevel::new(25));
    }

    #[test]
    fn a_zoom_range_may_not_be_inverted() {
        assert_ok!(MapBounds::new(
            BoundingBox::try_new(54.7, 9.2, 54.9, 9.6).unwrap(),
            zoom(13),
            zoom(13)
        ));
        let err = MapBounds::new(
            BoundingBox::try_new(54.7, 9.2, 54.9, 9.6).unwrap(),
            zoom(18),
            zoom(13),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ValidationError::InvalidFormat {
                field: "settings.map_view.zoom",
                ..
            }
        ));
    }

    #[test]
    fn map_view_requires_the_centre_inside_the_box() {
        assert_ok!(MapView::new(
            Coordinate::new(54.8, 9.4).unwrap(),
            Some(bounds())
        ));

        let outside = Coordinate::new(54.6, 9.4).unwrap();
        let err = MapView::new(outside, Some(bounds())).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::InvalidFormat {
                field: "settings.map_view",
                ..
            }
        ));
    }

    /// Without limits there is nothing for the centre to fall outside of, so a
    /// centre that a bounded view would reject is fine here.
    #[test]
    fn an_unbounded_view_accepts_any_centre() {
        let view = MapView::new(Coordinate::new(54.6, 9.4).unwrap(), None).unwrap();
        assert_eq!(view.bounds(), None);
        assert_eq!(view.center().latitude(), 54.6);
    }
}
