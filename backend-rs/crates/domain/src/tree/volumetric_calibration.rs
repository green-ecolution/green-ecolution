//! Calibration for capacitive soil-moisture sensors (GES-1000 family).
//!
//! Volumetric sensors report water content as a percentage — higher means
//! wetter. The worst-case (driest) probe across all configured depths drives
//! the status. Thresholds are intentionally simple for the first release; they
//! can later be refined per tree age, analogous to
//! [`crate::tree::watermark_calibration::PhaseTuning`].

use crate::sensor::data::VolumetricReading;
use crate::shared::watering_status::WateringStatus;
use crate::tree::TreeError;

pub(crate) fn classify(readings: &[VolumetricReading]) -> Result<WateringStatus, TreeError> {
    if readings.is_empty() {
        return Err(TreeError::MalformedVolumetric);
    }
    let min = readings
        .iter()
        .map(|r| r.moisture_percent)
        .fold(f64::INFINITY, f64::min);
    if !min.is_finite() {
        return Err(TreeError::MalformedVolumetric);
    }
    Ok(match min {
        m if m < 20.0 => WateringStatus::Bad,
        m if m < 35.0 => WateringStatus::Moderate,
        _ => WateringStatus::Good,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(depth_cm: i32, moisture_percent: f64) -> VolumetricReading {
        VolumetricReading {
            depth_cm,
            moisture_percent,
        }
    }

    #[test]
    fn empty_input_is_malformed() {
        assert!(matches!(classify(&[]), Err(TreeError::MalformedVolumetric)));
    }

    #[test]
    fn worst_case_drives_status_bad() {
        assert_eq!(
            classify(&[r(30, 50.0), r(90, 5.0)]).unwrap(),
            WateringStatus::Bad
        );
    }

    #[test]
    fn worst_case_drives_status_moderate() {
        assert_eq!(
            classify(&[r(30, 50.0), r(90, 25.0)]).unwrap(),
            WateringStatus::Moderate
        );
    }

    #[test]
    fn all_wet_is_good() {
        assert_eq!(
            classify(&[r(30, 40.0), r(90, 36.0)]).unwrap(),
            WateringStatus::Good
        );
    }

    #[test]
    fn boundary_at_20_is_moderate() {
        assert_eq!(classify(&[r(30, 20.0)]).unwrap(), WateringStatus::Moderate);
    }

    #[test]
    fn boundary_at_35_is_good() {
        assert_eq!(classify(&[r(30, 35.0)]).unwrap(), WateringStatus::Good);
    }

    #[test]
    fn nan_input_is_malformed() {
        assert!(matches!(
            classify(&[r(30, f64::NAN)]),
            Err(TreeError::MalformedVolumetric)
        ));
    }
}
