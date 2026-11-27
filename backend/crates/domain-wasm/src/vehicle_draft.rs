use std::collections::BTreeMap;

use domain::shared::error::ValidationError;
use domain::vehicle::{
    DrivingLicense, NumberPlate, VehicleAvailability, VehicleDimension, VehicleModel, VehicleType,
};
use serde::Deserialize;
use serde_wasm_bindgen::from_value;

use crate::issue::to_js;
use wasm_bindgen::prelude::*;

use crate::coerce::{LooseF64, OptionalF64, invalid_number_issue, validate_enum};
use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VehicleDraftInput {
    #[serde(default)]
    pub number_plate: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub driving_license: String,
    #[serde(default)]
    pub availability: String,
    /// One entry per capacity kind the form offers; the range of an amount
    /// is the module's validator's business.
    #[serde(default)]
    pub capacities: BTreeMap<String, OptionalF64>,
    #[serde(default)]
    pub height: LooseF64,
    #[serde(default)]
    pub width: LooseF64,
    #[serde(default)]
    pub length: LooseF64,
    #[serde(default)]
    pub weight: LooseF64,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
    pub description: Option<String>,
}

pub(crate) fn collect_vehicle_issues(input: &VehicleDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = NumberPlate::new(&input.number_plate) {
        issues.push(ValidationIssue::from_error(&err, "numberPlate"));
    }
    if let Err(err) = VehicleModel::new(&input.model) {
        issues.push(ValidationIssue::from_error(&err, "model"));
    }
    if let Some(issue) = validate_enum::<VehicleType>(&input.r#type, "vehicle.type", "type") {
        issues.push(issue);
    }
    if let Some(issue) = validate_enum::<DrivingLicense>(
        &input.driving_license,
        "vehicle.driving_license",
        "drivingLicense",
    ) {
        issues.push(issue);
    }
    if let Some(issue) = validate_enum::<VehicleAvailability>(
        &input.availability,
        "vehicle.availability",
        "availability",
    ) {
        issues.push(issue);
    }

    for (kind, value) in &input.capacities {
        if *value == OptionalF64::Invalid {
            issues.push(invalid_number_issue(
                "vehicle.capacity",
                format!("capacities.{kind}"),
            ));
        }
    }
    if !input.capacities.is_empty()
        && input
            .capacities
            .values()
            .all(|value| *value == OptionalF64::Absent)
    {
        issues.push(ValidationIssue::from_error(
            &ValidationError::EmptyString {
                field: "vehicle.capacities",
            },
            "capacities",
        ));
    }

    let dims = (
        input.height.0,
        input.width.0,
        input.length.0,
        input.weight.0,
    );
    match dims {
        (Some(h), Some(w), Some(l), Some(wt)) => {
            if let Err(err) = VehicleDimension::new(h, w, l, wt) {
                let path = match &err {
                    ValidationError::OutOfRange { field, .. }
                    | ValidationError::InvalidFormat { field, .. } => {
                        field.rsplit('.').next().unwrap_or("dimension")
                    }
                    _ => "dimension",
                };
                issues.push(ValidationIssue::from_error(&err, path));
            }
        }
        _ => {
            if input.height.0.is_none() {
                issues.push(invalid_number_issue("vehicle.dimension.height", "height"));
            }
            if input.width.0.is_none() {
                issues.push(invalid_number_issue("vehicle.dimension.width", "width"));
            }
            if input.length.0.is_none() {
                issues.push(invalid_number_issue("vehicle.dimension.length", "length"));
            }
            if input.weight.0.is_none() {
                issues.push(invalid_number_issue("vehicle.dimension.weight", "weight"));
            }
        }
    }

    issues
}

#[wasm_bindgen(js_name = validateVehicleDraft)]
pub fn validate_vehicle_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: VehicleDraftInput = from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    let issues = collect_vehicle_issues(&draft);
    to_js(&issues)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn capacities(entries: &[(&str, OptionalF64)]) -> BTreeMap<String, OptionalF64> {
        entries
            .iter()
            .map(|(kind, value)| (kind.to_string(), *value))
            .collect()
    }

    fn valid() -> VehicleDraftInput {
        VehicleDraftInput {
            number_plate: "FL-EC-100".into(),
            model: "Mercedes Sprinter".into(),
            r#type: "transporter".into(),
            driving_license: "B".into(),
            availability: "available".into(),
            capacities: capacities(&[
                ("water", OptionalF64::Value(100.0)),
                ("cargo_volume", OptionalF64::Absent),
            ]),
            height: LooseF64(Some(2.5)),
            width: LooseF64(Some(2.0)),
            length: LooseF64(Some(6.0)),
            weight: LooseF64(Some(3500.0)),
            description: None,
        }
    }

    #[test]
    fn valid_vehicle_yields_no_issues() {
        assert!(collect_vehicle_issues(&valid()).is_empty());
    }

    #[test]
    fn empty_number_plate_yields_issue() {
        let mut input = valid();
        input.number_plate = "".into();
        let issues = collect_vehicle_issues(&input);
        assert_eq!(issues[0].path, "numberPlate");
        assert!(issues[0].key.starts_with("vehicle.number_plate"));
    }

    #[test]
    fn unknown_vehicle_type_is_rejected() {
        let mut input = valid();
        input.r#type = "unknown".into();
        let issues = collect_vehicle_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "type")
            .expect("type issue");
        assert_eq!(issue.key, "vehicle.type.invalidFormat");
    }

    #[test]
    fn invalid_driving_license_is_rejected() {
        let mut input = valid();
        input.driving_license = "AM".into();
        let issues = collect_vehicle_issues(&input);
        assert!(issues.iter().any(|i| i.path == "drivingLicense"));
    }

    #[test]
    fn invalid_availability_is_rejected() {
        let mut input = valid();
        input.availability = "broken".into();
        let issues = collect_vehicle_issues(&input);
        assert!(issues.iter().any(|i| i.path == "availability"));
    }

    #[test]
    fn unparseable_height_yields_issue() {
        let mut input = valid();
        input.height = LooseF64(None);
        let issues = collect_vehicle_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "height")
            .expect("height issue");
        assert_eq!(issue.key, "vehicle.dimension.height.invalidFormat");
    }

    #[test]
    fn empty_json_yields_per_field_issues_without_throwing() {
        let input: VehicleDraftInput = serde_json::from_value(serde_json::json!({})).unwrap();
        let issues = collect_vehicle_issues(&input);
        let paths: Vec<&str> = issues.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"numberPlate"));
        assert!(paths.contains(&"model"));
        assert!(paths.contains(&"type"));
        assert!(paths.contains(&"drivingLicense"));
        assert!(paths.contains(&"availability"));
        assert!(paths.contains(&"height"));
        assert!(!paths.iter().any(|p| p.starts_with("capacities")));
    }

    #[test]
    fn an_offered_kind_left_empty_counts_as_absent() {
        let input: VehicleDraftInput = serde_json::from_value(serde_json::json!({
            "capacities": { "water": "", "cargo_volume": 6000 }
        }))
        .unwrap();
        let issues = collect_vehicle_issues(&input);
        assert!(!issues.iter().any(|i| i.path.starts_with("capacities")));
    }

    #[test]
    fn no_offered_kind_needs_no_capacity() {
        let mut input = valid();
        input.capacities = BTreeMap::new();
        assert!(collect_vehicle_issues(&input).is_empty());
    }

    #[test]
    fn all_offered_kinds_empty_yields_the_group_issue() {
        let mut input = valid();
        input.capacities = capacities(&[
            ("water", OptionalF64::Absent),
            ("cargo_volume", OptionalF64::Absent),
        ]);
        let issues = collect_vehicle_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "capacities")
            .expect("group issue");
        assert_eq!(issue.key, "vehicle.capacities.empty");
    }

    #[test]
    fn an_unparseable_amount_is_reported_at_its_kind() {
        let mut input = valid();
        input.capacities = capacities(&[
            ("water", OptionalF64::Value(100.0)),
            ("cargo_volume", OptionalF64::Invalid),
        ]);
        let issue = collect_vehicle_issues(&input)
            .into_iter()
            .find(|i| i.path == "capacities.cargo_volume")
            .expect("cargo issue");
        assert_eq!(issue.key, "vehicle.capacity.invalidFormat");
    }

    #[test]
    fn the_draft_leaves_the_range_to_the_module_validator() {
        let mut input = valid();
        input.capacities = capacities(&[("water", OptionalF64::Value(-10.0))]);
        assert!(collect_vehicle_issues(&input).is_empty());
    }
}
