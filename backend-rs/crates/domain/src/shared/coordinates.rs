use std::fmt::Display;

use crate::shared::error::ValidationError;

/// WGS-84 latitude/longitude pair, validated to `[-90, 90] × [-180, 180]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate(f64, f64);

impl Coordinate {
    pub fn new(lat: f64, lng: f64) -> Result<Coordinate, ValidationError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(ValidationError::OutOfRange {
                field: "coordinate.latitude",
                min: -90.0,
                max: 90.0,
                got: lat,
            });
        }

        if !(-180.0..=180.0).contains(&lng) {
            return Err(ValidationError::OutOfRange {
                field: "coordinate.longitude",
                min: -180.0,
                max: 180.0,
                got: lng,
            });
        }

        Ok(Self(lat, lng))
    }

    pub fn latitude(&self) -> f64 {
        self.0
    }

    pub fn longitude(&self) -> f64 {
        self.1
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::Coordinate;
    use claims::{assert_err, assert_ok};

    #[derive(Debug, Clone)]
    struct ValidCoordinateFixture(f64, f64);

    impl quickcheck::Arbitrary for ValidCoordinateFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let lat = (i32::arbitrary(g) % 9000) as f64 / 100.0; // -90.0..90.0
            let lng = (i32::arbitrary(g) % 18000) as f64 / 100.0; // -180.0..180.0
            Self(lat, lng)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_coordinates_are_accepted(valid: ValidCoordinateFixture) -> bool {
        Coordinate::new(valid.0, valid.1).is_ok()
    }

    #[test]
    fn origin_is_valid() {
        assert_ok!(Coordinate::new(0.0, 0.0));
    }

    #[test]
    fn boundary_values_are_valid() {
        assert_ok!(Coordinate::new(90.0, 180.0));
        assert_ok!(Coordinate::new(-90.0, -180.0));
        assert_ok!(Coordinate::new(90.0, -180.0));
        assert_ok!(Coordinate::new(-90.0, 180.0));
    }

    #[test]
    fn latitude_above_90_is_rejected() {
        assert_err!(Coordinate::new(90.1, 0.0));
    }

    #[test]
    fn latitude_below_negative_90_is_rejected() {
        assert_err!(Coordinate::new(-90.1, 0.0));
    }

    #[test]
    fn longitude_above_180_is_rejected() {
        assert_err!(Coordinate::new(0.0, 180.1));
    }

    #[test]
    fn longitude_below_negative_180_is_rejected() {
        assert_err!(Coordinate::new(0.0, -180.1));
    }

    #[test]
    fn latitude_and_longitude_both_invalid_rejects_with_latitude_error() {
        let err = Coordinate::new(91.0, 181.0).unwrap_err();
        assert!(err.to_string().contains("latitude"));
    }

    #[test]
    fn accessors_return_correct_values() {
        let coord = Coordinate::new(52.52, 13.405).unwrap();
        assert_eq!(coord.latitude(), 52.52);
        assert_eq!(coord.longitude(), 13.405);
    }

    #[test]
    fn display_format() {
        let coord = Coordinate::new(52.52, 13.405).unwrap();
        assert_eq!(coord.to_string(), "(52.52, 13.405)");
    }
}
