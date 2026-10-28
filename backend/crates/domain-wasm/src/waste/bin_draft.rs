use domain::shared::coordinates::Coordinate;
use domain::shared::error::ValidationError;
use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use waste_domain::values::{BinDepth, BinVolume, WasteBinNumber};

use crate::coerce::{LooseF64, invalid_number_issue};
use crate::issue::{ValidationIssue, to_js};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WasteBinDraftInput {
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub latitude: LooseF64,
    #[serde(default)]
    pub longitude: LooseF64,
    #[serde(default)]
    pub volume_liters: LooseF64,
    #[serde(default)]
    pub depth_cm: LooseF64,
}

pub(crate) fn collect_bin_issues(input: &WasteBinDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = WasteBinNumber::new(&input.number) {
        issues.push(ValidationIssue::from_error(&err, "number"));
    }

    match (input.latitude.0, input.longitude.0) {
        (Some(lat), Some(lng)) => {
            if let Err(err) = Coordinate::new(lat, lng) {
                let path = match &err {
                    ValidationError::OutOfRange { field, .. }
                        if *field == "coordinate.longitude" =>
                    {
                        "longitude"
                    }
                    _ => "latitude",
                };
                issues.push(ValidationIssue::from_error(&err, path));
            }
        }
        _ => {
            if input.latitude.0.is_none() {
                issues.push(invalid_number_issue("coordinate.latitude", "latitude"));
            }
            if input.longitude.0.is_none() {
                issues.push(invalid_number_issue("coordinate.longitude", "longitude"));
            }
        }
    }

    match input.volume_liters.0 {
        None => issues.push(invalid_number_issue("waste_bin.volume", "volumeLiters")),
        Some(liters) => {
            if let Err(err) = BinVolume::new(liters) {
                issues.push(ValidationIssue::from_error(&err, "volumeLiters"));
            }
        }
    }

    match input.depth_cm.0 {
        Some(cm) if cm.is_finite() && cm.fract() == 0.0 => {
            if let Err(err) = BinDepth::new(cm as i64) {
                issues.push(ValidationIssue::from_error(&err, "depthCm"));
            }
        }
        _ => issues.push(invalid_number_issue("waste_bin.depth", "depthCm")),
    }

    issues
}

#[wasm_bindgen(js_name = wasteValidateBinDraft)]
pub fn validate_bin_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: WasteBinDraftInput = from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    to_js(&collect_bin_issues(&draft))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> WasteBinDraftInput {
        WasteBinDraftInput {
            number: "K-12".into(),
            latitude: LooseF64(Some(54.78)),
            longitude: LooseF64(Some(9.43)),
            volume_liters: LooseF64(Some(1100.0)),
            depth_cm: LooseF64(Some(120.0)),
        }
    }

    #[test]
    fn valid_draft_yields_no_issues() {
        assert!(collect_bin_issues(&valid()).is_empty());
    }

    #[test]
    fn empty_number_is_reported_on_number() {
        let mut input = valid();
        input.number = "".into();
        let issues = collect_bin_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "number");
        assert_eq!(issues[0].key, "waste_bin.number.empty");
    }

    #[test]
    fn zero_depth_is_out_of_range() {
        let mut input = valid();
        input.depth_cm = LooseF64(Some(0.0));
        let issues = collect_bin_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "depthCm");
        assert_eq!(issues[0].key, "waste_bin.depth.outOfRange");
    }

    #[test]
    fn fractional_depth_is_invalid_format() {
        let mut input = valid();
        input.depth_cm = LooseF64(Some(12.5));
        let issues = collect_bin_issues(&input);
        assert_eq!(issues[0].key, "waste_bin.depth.invalidFormat");
    }

    #[test]
    fn unparsable_volume_is_invalid_format() {
        let json = serde_json::json!({
            "number": "K-12", "latitude": 54.78, "longitude": 9.43,
            "volumeLiters": "abc", "depthCm": 120
        });
        let input: WasteBinDraftInput = serde_json::from_value(json).unwrap();
        let issues = collect_bin_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "volumeLiters");
        assert_eq!(issues[0].key, "waste_bin.volume.invalidFormat");
    }

    #[test]
    fn invalid_coordinate_is_reported_per_axis() {
        let mut input = valid();
        input.latitude = LooseF64(Some(95.0));
        let issues = collect_bin_issues(&input);
        assert!(issues.iter().any(|i| i.path == "latitude"));
    }
}
