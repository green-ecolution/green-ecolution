//! Derives the watering status from volumetric soil-moisture readings using
//! per-soil-type, per-depth REW thresholds. Young trees (< 3 years) are scored
//! on the 40 cm probe only; older trees use both depths, worst case wins.

use crate::cluster::SoilCondition;
use crate::sensor::data::VolumetricReading;
use crate::shared::watering_status::WateringStatus;
use crate::tree::TreeError;

/// REW coefficient for the onset of moderate stress.
const REW_MIN: f64 = 0.40;
/// REW coefficient for acute stress.
const REW_CRIT: f64 = 0.30;
/// Below this lifetime the 80 cm probe is ignored.
const YOUNG_TREE_YEARS: i64 = 3;

/// Hydrological values for one soil type at a fixed depth, in Vol.-%.
struct DepthParams {
    fk: i32,
    nfk: i32,
    egp: i32,
}

impl DepthParams {
    /// `(VWC_min, VWC_crit)` thresholds in Vol.-%.
    fn thresholds(&self) -> (f64, f64) {
        let nfk_eff = (self.nfk - self.egp) as f64;
        let pwp = (self.fk - self.nfk) as f64;
        (pwp + REW_MIN * nfk_eff, pwp + REW_CRIT * nfk_eff)
    }
}

/// Volumetric soil-moisture thresholds in Vol.-% for one probe depth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumetricThresholds {
    pub depth_cm: i32,
    /// Below this the tree counts as moderately stressed (`REW_MIN`).
    pub moderate: f64,
    /// Below this the tree counts as acutely stressed (`REW_CRIT`).
    pub critical: f64,
}

/// Thresholds for `soil` at `depth_cm`; `None` for `Unknown` soil or a depth
/// without calibration data (only 40 cm and 80 cm are defined).
pub fn volumetric_thresholds(soil: SoilCondition, depth_cm: i32) -> Option<VolumetricThresholds> {
    let params = depth_params(soil, depth_cm)?;
    let (moderate, critical) = params.thresholds();
    Some(VolumetricThresholds {
        depth_cm,
        moderate,
        critical,
    })
}

/// Lookup of `(FK, nFK, eGp)` for `soil` at `depth_cm` (only 40 cm and 80 cm defined).
fn depth_params(soil: SoilCondition, depth_cm: i32) -> Option<DepthParams> {
    // (fk40, nfk40, egp40, fk80, nfk80, egp80)
    let (fk40, nfk40, egp40, fk80, nfk80, egp80) = match soil {
        SoilCondition::Ss => (20, 16, 9, 17, 14, 7),
        SoilCondition::Sl2 => (25, 18, 9, 23, 17, 8),
        SoilCondition::Sl3 => (27, 18, 6, 25, 17, 5),
        SoilCondition::Sl4 => (30, 18, 6, 26, 15, 5),
        SoilCondition::Slu => (33, 21, 6, 30, 19, 5),
        SoilCondition::St2 => (22, 16, 7, 18, 13, 6),
        SoilCondition::St3 => (30, 15, 5, 26, 12, 4),
        SoilCondition::Su2 => (23, 18, 10, 21, 17, 9),
        SoilCondition::Su3 => (29, 21, 11, 26, 20, 8),
        SoilCondition::Su4 => (32, 23, 9, 28, 21, 7),
        SoilCondition::Ls2 => (34, 16, 5, 31, 14, 3),
        SoilCondition::Ls3 => (33, 16, 4, 30, 14, 2),
        SoilCondition::Ls4 => (32, 16, 4, 28, 13, 3),
        SoilCondition::Lt2 => (36, 14, 2, 32, 11, 2),
        SoilCondition::Lt3 => (39, 12, 2, 35, 10, 2),
        SoilCondition::Lts => (37, 14, 2, 31, 11, 2),
        SoilCondition::Lu => (36, 17, 3, 33, 15, 2),
        SoilCondition::Uu => (38, 26, 6, 35, 23, 3),
        SoilCondition::Uls => (35, 22, 6, 33, 21, 5),
        SoilCondition::Us => (35, 25, 8, 32, 22, 4),
        SoilCondition::Ut2 => (37, 26, 6, 35, 23, 3),
        SoilCondition::Ut3 => (37, 25, 4, 35, 23, 2),
        SoilCondition::Ut4 => (37, 21, 3, 35, 19, 2),
        SoilCondition::Tt => (43, 13, 1, 35, 12, 0),
        SoilCondition::Tl => (41, 13, 1, 35, 11, 0),
        SoilCondition::Tu2 => (42, 12, 1, 36, 10, 0),
        SoilCondition::Tu3 => (38, 13, 1, 35, 10, 0),
        SoilCondition::Tu4 => (37, 17, 2, 35, 16, 1),
        SoilCondition::Ts2 => (39, 13, 1, 34, 12, 0),
        SoilCondition::Ts3 => (37, 13, 2, 32, 11, 1),
        SoilCondition::Ts4 => (32, 14, 3, 30, 11, 2),
        SoilCondition::Fs => (23, 18, 9, 19, 15, 7),
        SoilCondition::Ms => (19, 15, 9, 15, 12, 7),
        SoilCondition::Gs => (16, 13, 8, 13, 11, 7),
        SoilCondition::Unknown => return None,
    };
    match depth_cm {
        40 => Some(DepthParams {
            fk: fk40,
            nfk: nfk40,
            egp: egp40,
        }),
        80 => Some(DepthParams {
            fk: fk80,
            nfk: nfk80,
            egp: egp80,
        }),
        _ => None,
    }
}

fn score(vwc: f64, min: f64, crit: f64) -> i32 {
    if vwc >= min {
        0
    } else if vwc >= crit {
        1
    } else {
        2
    }
}

/// Derives a [`WateringStatus`] from volumetric soil-moisture readings, the
/// cluster's KA5 soil type, and the tree's lifetime in years.
pub(crate) fn classify(
    readings: &[VolumetricReading],
    soil: SoilCondition,
    lifetime_years: i64,
) -> Result<WateringStatus, TreeError> {
    if matches!(soil, SoilCondition::Unknown) {
        return Err(TreeError::UncalibratedSoil);
    }

    let prefer_40_only = lifetime_years < YOUNG_TREE_YEARS;
    let has_40 = readings.iter().any(|r| r.depth_cm == 40);
    let considered = readings.iter().filter(|r| {
        if prefer_40_only && has_40 {
            r.depth_cm == 40
        } else {
            true
        }
    });

    let mut worst: Option<i32> = None;
    for r in considered {
        if !r.moisture_percent.is_finite() {
            continue;
        }
        let Some(params) = depth_params(soil, r.depth_cm) else {
            continue;
        };
        let (min, crit) = params.thresholds();
        let s = score(r.moisture_percent, min, crit);
        worst = Some(worst.map_or(s, |w| w.max(s)));
    }

    match worst {
        Some(0) => Ok(WateringStatus::Good),
        Some(1) => Ok(WateringStatus::Moderate),
        Some(_) => Ok(WateringStatus::Bad),
        None => Err(TreeError::MalformedVolumetric),
    }
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

    // Uu @ both depths: nFK_eff=20, PWP=12 → min=20.0, crit=18.0 (symmetric).
    #[test]
    fn uu_good_moderate_bad_boundaries_40cm() {
        let est = 5; // established → 40 cm considered
        assert_eq!(
            classify(&[r(40, 20.0)], SoilCondition::Uu, est).unwrap(),
            WateringStatus::Good
        );
        assert_eq!(
            classify(&[r(40, 19.0)], SoilCondition::Uu, est).unwrap(),
            WateringStatus::Moderate
        );
        assert_eq!(
            classify(&[r(40, 18.0)], SoilCondition::Uu, est).unwrap(),
            WateringStatus::Moderate
        );
        assert_eq!(
            classify(&[r(40, 17.9)], SoilCondition::Uu, est).unwrap(),
            WateringStatus::Bad
        );
    }

    #[test]
    fn established_uses_worst_case_across_depths() {
        // 40 cm Good (25≥20), 80 cm Bad (15<18=crit) → worst = Bad.
        let s = classify(&[r(40, 25.0), r(80, 15.0)], SoilCondition::Uu, 5).unwrap();
        assert_eq!(s, WateringStatus::Bad);
    }

    #[test]
    fn young_ignores_80cm_probe() {
        // Same readings, young tree → only 40 cm (Good) counts.
        let s = classify(&[r(40, 25.0), r(80, 15.0)], SoilCondition::Uu, 0).unwrap();
        assert_eq!(s, WateringStatus::Good);
    }

    #[test]
    fn young_falls_back_to_80cm_when_40_missing() {
        let s = classify(&[r(80, 15.0)], SoilCondition::Uu, 0).unwrap();
        assert_eq!(s, WateringStatus::Bad);
    }

    #[test]
    fn sand_has_lower_thresholds_than_silt() {
        // Su3 @ 40 cm: nFK_eff=10, PWP=8 → min=12, crit=11. 13% is Good on sand…
        assert_eq!(
            classify(&[r(40, 13.0)], SoilCondition::Su3, 5).unwrap(),
            WateringStatus::Good
        );
        // …but Bad on silt Uu (min 20).
        assert_eq!(
            classify(&[r(40, 13.0)], SoilCondition::Uu, 5).unwrap(),
            WateringStatus::Bad
        );
    }

    #[test]
    fn unknown_soil_is_uncalibrated() {
        assert!(matches!(
            classify(&[r(40, 20.0)], SoilCondition::Unknown, 5),
            Err(TreeError::UncalibratedSoil)
        ));
    }

    #[test]
    fn empty_readings_is_malformed() {
        assert!(matches!(
            classify(&[], SoilCondition::Uu, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    #[test]
    fn unknown_depth_is_ignored_and_yields_malformed_if_alone() {
        assert!(matches!(
            classify(&[r(55, 20.0)], SoilCondition::Uu, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    #[test]
    fn nan_reading_is_skipped() {
        assert!(matches!(
            classify(&[r(40, f64::NAN)], SoilCondition::Uu, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    // Uu @ 40 cm: nFK_eff = 26-6 = 20, PWP = 38-26 = 12 → min 20.0, crit 18.0.
    #[test]
    fn thresholds_for_known_soil_and_depth() {
        let t = volumetric_thresholds(SoilCondition::Uu, 40).unwrap();
        assert_eq!(t.depth_cm, 40);
        assert!((t.moderate - 20.0).abs() < 1e-9);
        assert!((t.critical - 18.0).abs() < 1e-9);
    }

    #[test]
    fn thresholds_none_for_unknown_soil() {
        assert!(volumetric_thresholds(SoilCondition::Unknown, 40).is_none());
    }

    #[test]
    fn thresholds_none_for_unsupported_depth() {
        assert!(volumetric_thresholds(SoilCondition::Uu, 50).is_none());
    }
}
