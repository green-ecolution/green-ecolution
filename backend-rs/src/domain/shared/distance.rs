use std::{fmt::Display, ops::Add};

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance(f64);

impl Distance {
    pub fn new(meters: f64) -> Result<Self, DomainError> {
        if meters.is_sign_negative() {
            return Err(DomainError::InvalidDistance(meters));
        }

        Ok(Distance(meters))
    }

    pub fn meters(&self) -> f64 {
        self.0
    }
}

impl Add for Distance {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.meters() + rhs.meters())
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}m", self.meters())
    }
}

#[cfg(test)]
mod tests {
    use super::Distance;
    use claims::{assert_err, assert_ok};

    #[derive(Debug, Clone)]
    struct ValidDistanceFixture(f64);

    impl quickcheck::Arbitrary for ValidDistanceFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let meters = (u32::arbitrary(g) as f64) / 100.0;
            Self(meters)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn non_negative_distances_are_accepted(valid: ValidDistanceFixture) -> bool {
        Distance::new(valid.0).is_ok()
    }

    #[test]
    fn zero_is_valid() {
        assert_ok!(Distance::new(0.0));
    }

    #[test]
    fn positive_value_is_valid() {
        let d = Distance::new(42.5).unwrap();
        assert_eq!(d.meters(), 42.5);
    }

    #[test]
    fn negative_value_is_rejected() {
        assert_err!(Distance::new(-0.1));
    }

    #[test]
    fn add_two_distances() {
        let a = Distance::new(10.0).unwrap();
        let b = Distance::new(5.5).unwrap();
        assert_eq!((a + b).meters(), 15.5);
    }

    #[test]
    fn display_format() {
        let d = Distance::new(123.45).unwrap();
        assert_eq!(d.to_string(), "123.45m");
    }
}
