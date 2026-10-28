use domain::shared::error::ValidationError;
use wasm_bindgen::prelude::*;
use waste_domain::settings::FullThreshold;
use waste_domain::values::{BinDepth, BinVolume, WasteBinNumber};

use crate::coerce::invalid_number_issue;
use crate::field_validators::finish;
use crate::issue::{ValidationIssue, to_js};

/// Rejects a fractional value instead of rounding it, so the field reports the
/// same issue the draft validation does on save.
fn whole_number_issue<T>(
    value: f64,
    field: &'static str,
    path: &'static str,
    build: impl FnOnce(i64) -> Result<T, ValidationError>,
) -> Option<ValidationIssue> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Some(invalid_number_issue(field, path));
    }
    build(value as i64)
        .err()
        .map(|err| ValidationIssue::from_error(&err, path))
}

fn bin_depth_issue(centimeters: f64) -> Option<ValidationIssue> {
    whole_number_issue(centimeters, "waste_bin.depth", "depthCm", BinDepth::new)
}

fn full_threshold_issue(percent: f64) -> Option<ValidationIssue> {
    whole_number_issue(
        percent,
        "settings.full_threshold",
        "fullThresholdPercent",
        FullThreshold::new,
    )
}

fn report(issue: Option<ValidationIssue>) -> Result<JsValue, JsError> {
    issue.map_or(Ok(JsValue::NULL), |issue| to_js(&issue))
}

#[wasm_bindgen(js_name = wasteValidateBinNumber)]
pub fn validate_bin_number(value: &str) -> Result<JsValue, JsError> {
    finish(WasteBinNumber::new(value), "number")
}

#[wasm_bindgen(js_name = wasteValidateBinVolume)]
pub fn validate_bin_volume(liters: f64) -> Result<JsValue, JsError> {
    finish(BinVolume::new(liters), "volumeLiters")
}

#[wasm_bindgen(js_name = wasteValidateBinDepth)]
pub fn validate_bin_depth(centimeters: f64) -> Result<JsValue, JsError> {
    report(bin_depth_issue(centimeters))
}

#[wasm_bindgen(js_name = wasteValidateFullThreshold)]
pub fn validate_full_threshold(percent: f64) -> Result<JsValue, JsError> {
    report(full_threshold_issue(percent))
}

#[wasm_bindgen(js_name = wasteBinVolumeMin)]
pub fn bin_volume_min() -> f64 {
    BinVolume::MIN
}

#[wasm_bindgen(js_name = wasteBinVolumeMax)]
pub fn bin_volume_max() -> f64 {
    BinVolume::MAX
}

#[wasm_bindgen(js_name = wasteBinDepthMin)]
pub fn bin_depth_min() -> f64 {
    BinDepth::MIN as f64
}

#[wasm_bindgen(js_name = wasteBinDepthMax)]
pub fn bin_depth_max() -> f64 {
    BinDepth::MAX as f64
}

#[wasm_bindgen(js_name = wasteFullThresholdMin)]
pub fn full_threshold_min() -> f64 {
    FullThreshold::MIN as f64
}

#[wasm_bindgen(js_name = wasteFullThresholdMax)]
pub fn full_threshold_max() -> f64 {
    FullThreshold::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fractional_threshold_is_invalid_format() {
        let issue = full_threshold_issue(80.5).expect("80.5 must be rejected");
        assert_eq!(issue.path, "fullThresholdPercent");
        assert_eq!(issue.key, "settings.full_threshold.invalidFormat");
    }

    #[test]
    fn a_threshold_outside_the_range_is_out_of_range() {
        assert_eq!(
            full_threshold_issue(0.0).unwrap().key,
            "settings.full_threshold.outOfRange"
        );
        assert!(full_threshold_issue(80.0).is_none());
    }

    #[test]
    fn a_fractional_depth_is_invalid_format() {
        let issue = bin_depth_issue(12.5).expect("12.5 must be rejected");
        assert_eq!(issue.path, "depthCm");
        assert_eq!(issue.key, "waste_bin.depth.invalidFormat");
    }

    #[test]
    fn a_whole_depth_in_range_passes() {
        assert!(bin_depth_issue(120.0).is_none());
        assert!(bin_depth_issue(f64::NAN).is_some());
    }
}
