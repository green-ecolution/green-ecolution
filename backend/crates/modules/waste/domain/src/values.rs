use domain::capacity::Liters;
use domain::shared::error::ValidationError;

domain::newtype_nonempty! {
    /// Identifier printed on the container, 1-50 characters after trimming.
    WasteBinNumber, "waste_bin.number", 1, 50
}

/// Inner height of a container, the distance a sensor reads when it is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinDepth(u16);

impl BinDepth {
    pub const MIN: i64 = 1;
    pub const MAX: i64 = 1_000;

    pub fn new(cm: i64) -> Result<Self, ValidationError> {
        if !(Self::MIN..=Self::MAX).contains(&cm) {
            return Err(ValidationError::OutOfRange {
                field: "waste_bin.depth",
                min: Self::MIN as f64,
                max: Self::MAX as f64,
                got: cm as f64,
            });
        }
        Ok(Self(cm as u16))
    }

    pub fn centimeters(&self) -> f64 {
        f64::from(self.0)
    }
}

/// Capacity of a container in liters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinVolume(Liters);

impl BinVolume {
    pub const MIN: f64 = 1.0;
    pub const MAX: f64 = 100_000.0;

    pub fn new(liters: f64) -> Result<Self, ValidationError> {
        if !liters.is_finite() || !(Self::MIN..=Self::MAX).contains(&liters) {
            return Err(ValidationError::OutOfRange {
                field: "waste_bin.volume",
                min: Self::MIN,
                max: Self::MAX,
                got: liters,
            });
        }
        Ok(Self(Liters::new(liters)?))
    }

    pub fn liters(&self) -> f64 {
        self.0.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn depth_accepts_one_to_a_thousand_centimeters() {
        assert_ok!(BinDepth::new(1));
        assert_ok!(BinDepth::new(1000));
        assert_err!(BinDepth::new(0));
        assert_err!(BinDepth::new(1001));
        assert_eq!(BinDepth::new(120).unwrap().centimeters(), 120.0);
    }

    #[test]
    fn depth_error_carries_the_field_label() {
        assert!(matches!(
            BinDepth::new(0).unwrap_err(),
            ValidationError::OutOfRange {
                field: "waste_bin.depth",
                ..
            }
        ));
    }

    #[test]
    fn volume_accepts_one_to_a_hundred_thousand_liters() {
        assert_ok!(BinVolume::new(1.0));
        assert_ok!(BinVolume::new(100_000.0));
        assert_err!(BinVolume::new(0.5));
        assert_err!(BinVolume::new(100_000.1));
        assert_err!(BinVolume::new(f64::NAN));
        assert_eq!(BinVolume::new(1100.0).unwrap().liters(), 1100.0);
    }

    #[test]
    fn number_rejects_empty_and_too_long() {
        assert_err!(WasteBinNumber::new(""));
        assert_err!(WasteBinNumber::new("   "));
        assert_err!(WasteBinNumber::new("x".repeat(51)));
        assert_ok!(WasteBinNumber::new("x".repeat(50)));
        assert_eq!(WasteBinNumber::new("  K-12 ").unwrap().as_str(), "K-12");
    }
}
