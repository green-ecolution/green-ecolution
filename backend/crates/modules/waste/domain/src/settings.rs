use domain::shared::error::ValidationError;

/// Fill level, in percent, from which a container counts as full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullThreshold(u8);

impl FullThreshold {
    pub const DEFAULT: u8 = 80;
    pub const MIN: i64 = 1;
    pub const MAX: i64 = 100;

    pub fn new(percent: i64) -> Result<Self, ValidationError> {
        if !(Self::MIN..=Self::MAX).contains(&percent) {
            return Err(ValidationError::OutOfRange {
                field: "settings.full_threshold",
                min: Self::MIN as f64,
                max: Self::MAX as f64,
                got: percent as f64,
            });
        }
        Ok(Self(percent as u8))
    }

    pub fn fraction(&self) -> f64 {
        f64::from(self.0) / 100.0
    }

    pub fn percent(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn threshold_spans_one_to_a_hundred_percent() {
        assert_err!(FullThreshold::new(0));
        assert_err!(FullThreshold::new(101));
        assert_ok!(FullThreshold::new(1));
        assert_ok!(FullThreshold::new(100));
    }

    #[test]
    fn threshold_converts_to_a_fraction() {
        let t = FullThreshold::new(FullThreshold::DEFAULT.into()).unwrap();
        assert_eq!(t.fraction(), 0.8);
        assert_eq!(t.percent(), 80);
    }
}
