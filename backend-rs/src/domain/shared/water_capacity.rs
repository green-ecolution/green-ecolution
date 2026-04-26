use std::{fmt::Display, ops::Add};

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WaterCapacity(f64);

impl WaterCapacity {
    pub fn new(liters: f64) -> Result<Self, DomainError> {
        if liters.is_sign_negative() {
            return Err(DomainError::InvalidWaterCapacity(liters));
        }
        Ok(Self(liters))
    }

    pub fn liters(&self) -> f64 {
        self.0
    }
}

impl Add for WaterCapacity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for WaterCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}L", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::WaterCapacity;
    use claims::{assert_err, assert_ok};

    #[derive(Debug, Clone)]
    struct ValidCapacityFixture(f64);

    impl quickcheck::Arbitrary for ValidCapacityFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let liters = (u32::arbitrary(g) as f64) / 100.0;
            Self(liters)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn non_negative_capacities_are_accepted(valid: ValidCapacityFixture) -> bool {
        WaterCapacity::new(valid.0).is_ok()
    }

    #[test]
    fn zero_is_valid() {
        assert_ok!(WaterCapacity::new(0.0));
    }

    #[test]
    fn positive_value_is_valid() {
        let c = WaterCapacity::new(1500.0).unwrap();
        assert_eq!(c.liters(), 1500.0);
    }

    #[test]
    fn negative_value_is_rejected() {
        assert_err!(WaterCapacity::new(-0.1));
    }

    #[test]
    fn add_two_capacities() {
        let a = WaterCapacity::new(1000.0).unwrap();
        let b = WaterCapacity::new(500.0).unwrap();
        assert_eq!((a + b).liters(), 1500.0);
    }

    #[test]
    fn display_format() {
        let c = WaterCapacity::new(2500.0).unwrap();
        assert_eq!(c.to_string(), "2500L");
    }
}
