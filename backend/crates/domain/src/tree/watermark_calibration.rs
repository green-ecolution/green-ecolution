//! Watermark-sensor calibration tables for the tree watering status.
//!
//! This module is **specific to the Watermark sensor family** (tensiometric
//! sensors at fixed depths 30/60/90 cm, reading in centibar). Other sensor
//! families with different physical principles (e.g. volumetric water
//! content) get their own sibling modules when introduced; the tree
//! aggregate then exposes one `calculate_watering_status_from_<family>`
//! method per family.
//!
//! Thresholds replicate the calibration the Go backend used. A reading is
//! scored 0 (Good) / 1 (Moderate) / 2 (Bad) per depth; the worst score
//! across the three depths wins.

use crate::sensor::data::Watermark;
use crate::tree::TreeError;

/// Per-depth tension thresholds for one tree-lifecycle phase. The tuple is
/// `(lower, higher)` in centibar: a reading below `lower` is `Good`, between
/// `lower` and `higher` is `Moderate`, at or above `higher` is `Bad`. The
/// special value `NO_MODERATE` collapses the band so only Good/Bad remain.
pub(crate) struct PhaseTuning {
    pub d30: (i32, i32),
    pub d60: (i32, i32),
    pub d90: (i32, i32),
}

/// Defaults for the early lifecycle (years 0/1) and for d60/d90 in year 2.
const DEFAULT: (i32, i32) = (25, 33);

/// Sentinel that disables the Moderate band (year 3): readings are scored
/// only as Good or Bad against the lower threshold.
const NO_MODERATE: i32 = -1;

impl PhaseTuning {
    /// Returns the calibration table for the given lifecycle year.
    /// Anything outside 0..=3 is currently `BeyondMonitoring` — beyond year 3
    /// the calibration has not been validated, and negative values mean the
    /// tree was planted in the future (clock skew or bad data).
    pub(crate) fn for_year(years: i64) -> Result<Self, TreeError> {
        match years {
            0 | 1 => Ok(Self {
                d30: DEFAULT,
                d60: DEFAULT,
                d90: DEFAULT,
            }),
            2 => Ok(Self {
                d30: (62, 81),
                d60: DEFAULT,
                d90: DEFAULT,
            }),
            3 => Ok(Self {
                d30: (1585, NO_MODERATE),
                d60: (80, NO_MODERATE),
                d90: (80, NO_MODERATE),
            }),
            _ => Err(TreeError::BeyondMonitoring),
        }
    }

    pub(crate) fn score(&self, w30: i32, w60: i32, w90: i32) -> [i32; 3] {
        [
            map_kpa(w30, self.d30.0, self.d30.1),
            map_kpa(w60, self.d60.0, self.d60.1),
            map_kpa(w90, self.d90.0, self.d90.1),
        ]
    }
}

fn map_kpa(centibar: i32, lower: i32, higher: i32) -> i32 {
    if centibar < lower {
        0
    } else if centibar < higher {
        1
    } else {
        2
    }
}

/// Validates that `w` carries exactly three readings at the expected depths
/// (30, 60, 90 cm) and returns them sorted shallow-to-deep.
pub(crate) fn sort_watermarks(
    w: &[Watermark],
) -> Result<(Watermark, Watermark, Watermark), TreeError> {
    if w.len() != 3 {
        return Err(TreeError::MalformedWatermarks);
    }
    let mut s = [w[0], w[1], w[2]];
    s.sort_by_key(|m| m.depth);
    if s[0].depth != 30 || s[1].depth != 60 || s[2].depth != 90 {
        return Err(TreeError::MalformedWatermarks);
    }
    Ok((s[0], s[1], s[2]))
}
