//! Derives the watering status from volumetric soil-moisture readings using
//! per-soil-type, per-depth REW thresholds. Young trees (< 3 years) are scored
//! on the 40 cm probe only; older trees use both depths, worst case wins.
//!
//! # KA5 pore classes
//!
//! `depth_params` stores three capacities per soil type and depth, all in
//! Vol.-%. They are sums over KA5 pore classes, which are defined by
//! equivalent pore diameter:
//!
//! | Class                       | Diameter  | Role                                        |
//! | --------------------------- | --------- | ------------------------------------------- |
//! | `wGP` — weite Grobporen     | > 50 µm   | air-filled at field capacity → air capacity |
//! | `eGP` — enge Grobporen      | 10–50 µm  | plant-available, loosely held               |
//! | `MP`  — Mittelporen         | 0.2–10 µm | plant-available, held against gravity       |
//! | `FP`  — Feinporen           | < 0.2 µm  | bound, not extractable by plants            |
//!
//! From which `FK = eGP + MP + FP`, `nFK = eGP + MP`, `PWP = FP`, `LK = wGP`,
//! and total pore volume `GPV = FK + LK`.
//!
//! That resolves both terms in `DepthParams::thresholds`: `FK − nFK` is `FP`,
//! the water still present at the permanent wilting point, and `nFK − eGP` is
//! `MP`. Scaling the thresholds by `MP` alone leaves out the loosely held
//! narrow-coarse-pore water; whether that was a deliberate modelling choice is
//! not recorded anywhere.
//!
//! Note what the table *cannot* give: `GPV`, because the air capacity `wGP` is
//! not stored. A saturation ceiling for plausibility checks therefore cannot be
//! derived here — see the fixed bound in `sensor::plausibility` instead.
//!
//! Pore-class definitions per LfU Bayern and Geologischer Dienst NRW. The
//! per-soil-type numbers themselves arrived with PR #828 (GECO-151) without a
//! cited source; which KA5 table and edition they come from is unverified.

use crate::cluster::SoilCondition;
use crate::sensor::data::VolumetricReading;
use crate::shared::watering_status::WateringStatus;
use crate::tree::TreeError;

/// REW coefficient for the onset of moderate stress. Below REW 0.4 stomata
/// close and transpiration is throttled; the threshold comes from Granier et
/// al. (1999) and is widely adopted in forest hydrology.
pub const REW_MIN: f64 = 0.40;
/// REW coefficient for acute stress. Project-internal: the literature names
/// 0.4 and 0.2, not 0.3, and no source for this value could be traced.
pub const REW_CRIT: f64 = 0.30;
/// Below this lifetime the 80 cm probe is ignored.
const YOUNG_TREE_YEARS: i64 = 3;

/// Hydrological values for one soil type at a fixed depth, in Vol.-%. See the
/// module docs for how these map onto the KA5 pore classes.
struct DepthParams {
    /// Field capacity: `eGP + MP + FP`.
    fk: i32,
    /// Plant-available field capacity: `eGP + MP`.
    nfk: i32,
    /// Narrow coarse pores, 10–50 µm. Part of `nfk`, not on top of `fk`.
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

/// Fraction of plant-available water (REW) for a volumetric reading:
/// `(vwc − PWP) / nFK_eff`. 0.0 = permanent wilting point, 1.0 = full
/// effective capacity; values above 1.0 (wetter than field capacity) are
/// not clamped. `None` for unknown soil, an uncalibrated depth, or a
/// non-finite reading.
pub fn rew_fraction(soil: SoilCondition, depth_cm: i32, vwc: f64) -> Option<f64> {
    if !vwc.is_finite() {
        return None;
    }
    let params = depth_params(soil, depth_cm)?;
    let nfk_eff = (params.nfk - params.egp) as f64;
    let pwp = (params.fk - params.nfk) as f64;
    Some((vwc - pwp) / nfk_eff)
}

/// Depths the KA5 table calibrates; [`depth_params`] gates on this.
fn is_calibrated_depth(depth_cm: i32) -> bool {
    matches!(depth_cm, 40 | 80)
}

/// Lookup of `(FK, nFK, eGp)` for `soil` at `depth_cm` (only 40 cm and 80 cm defined).
fn depth_params(soil: SoilCondition, depth_cm: i32) -> Option<DepthParams> {
    if !is_calibrated_depth(depth_cm) {
        return None;
    }
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
    let prefer_40_only = lifetime_years < YOUNG_TREE_YEARS;
    let has_40 = readings.iter().any(|r| r.depth_cm == 40);
    let considered: Vec<&VolumetricReading> = readings
        .iter()
        .filter(|r| {
            if prefer_40_only && has_40 {
                r.depth_cm == 40
            } else {
                true
            }
        })
        .filter(|r| r.moisture_percent.is_finite() && is_calibrated_depth(r.depth_cm))
        .collect();

    // Order matters: callers invalidate a stored status on `UncalibratedSoil`,
    // but keep it on a payload they could not have scored anyway.
    if considered.is_empty() {
        return Err(TreeError::MalformedVolumetric);
    }
    if matches!(soil, SoilCondition::Unknown) {
        return Err(TreeError::UncalibratedSoil);
    }

    let mut worst: Option<i32> = None;
    for r in considered {
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

    const ALL_SOILS: [SoilCondition; 34] = [
        SoilCondition::Ss,
        SoilCondition::Sl2,
        SoilCondition::Sl3,
        SoilCondition::Sl4,
        SoilCondition::Slu,
        SoilCondition::St2,
        SoilCondition::St3,
        SoilCondition::Su2,
        SoilCondition::Su3,
        SoilCondition::Su4,
        SoilCondition::Ls2,
        SoilCondition::Ls3,
        SoilCondition::Ls4,
        SoilCondition::Lt2,
        SoilCondition::Lt3,
        SoilCondition::Lts,
        SoilCondition::Lu,
        SoilCondition::Uu,
        SoilCondition::Uls,
        SoilCondition::Us,
        SoilCondition::Ut2,
        SoilCondition::Ut3,
        SoilCondition::Ut4,
        SoilCondition::Tt,
        SoilCondition::Tl,
        SoilCondition::Tu2,
        SoilCondition::Tu3,
        SoilCondition::Tu4,
        SoilCondition::Ts2,
        SoilCondition::Ts3,
        SoilCondition::Ts4,
        SoilCondition::Fs,
        SoilCondition::Ms,
        SoilCondition::Gs,
    ];

    /// The pore-class composition documented at module level implies
    /// `eGP <= nFK <= FK` for every row. A typo breaking it would silently
    /// produce a negative medium-pore reservoir and nonsensical thresholds.
    #[test]
    fn every_soil_row_satisfies_the_pore_class_composition() {
        for soil in ALL_SOILS {
            for depth in [40, 80] {
                let p = depth_params(soil, depth).expect("calibrated depth");
                assert!(
                    p.egp <= p.nfk,
                    "{soil:?} @ {depth} cm: eGP {} exceeds nFK {}",
                    p.egp,
                    p.nfk
                );
                assert!(
                    p.nfk <= p.fk,
                    "{soil:?} @ {depth} cm: nFK {} exceeds FK {}",
                    p.nfk,
                    p.fk
                );
                assert!(
                    p.nfk - p.egp > 0,
                    "{soil:?} @ {depth} cm: medium-pore reservoir is empty"
                );
            }
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

    #[test]
    fn unknown_soil_without_calibratable_depth_is_malformed() {
        // The EcoDrizzler's 15 cm probe has no KA5 calibration at any soil type.
        assert!(matches!(
            classify(&[r(15, 30.0)], SoilCondition::Unknown, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    #[test]
    fn unknown_soil_without_readings_is_malformed() {
        assert!(matches!(
            classify(&[], SoilCondition::Unknown, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    #[test]
    fn unknown_soil_with_nan_reading_is_malformed() {
        assert!(matches!(
            classify(&[r(40, f64::NAN)], SoilCondition::Unknown, 5),
            Err(TreeError::MalformedVolumetric)
        ));
    }

    #[test]
    fn unknown_soil_reports_uncalibrated_for_young_tree_on_40cm() {
        assert!(matches!(
            classify(&[r(40, 20.0), r(80, 20.0)], SoilCondition::Unknown, 0),
            Err(TreeError::UncalibratedSoil)
        ));
    }

    // Uu @ 40 cm: PWP = 12, nFK_eff = 20.
    #[test]
    fn rew_fraction_maps_vwc_linearly() {
        let rew = |vwc| rew_fraction(SoilCondition::Uu, 40, vwc).unwrap();
        assert!((rew(12.0) - 0.0).abs() < 1e-9);
        assert!((rew(20.0) - 0.4).abs() < 1e-9);
        assert!((rew(32.0) - 1.0).abs() < 1e-9);
        // Wetter than field capacity is not clamped.
        assert!((rew(42.0) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn rew_fraction_none_for_unknown_soil_depth_or_nan() {
        assert!(rew_fraction(SoilCondition::Unknown, 40, 20.0).is_none());
        assert!(rew_fraction(SoilCondition::Uu, 50, 20.0).is_none());
        assert!(rew_fraction(SoilCondition::Uu, 40, f64::NAN).is_none());
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
