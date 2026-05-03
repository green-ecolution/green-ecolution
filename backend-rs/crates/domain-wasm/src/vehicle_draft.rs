use domain::shared::error::ValidationError;
use domain::shared::water_capacity::WaterCapacity;
use domain::vehicle::{NumberPlate, VehicleDimension, VehicleModel};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VehicleDraftInput {
    pub number_plate: String,
    pub model: String,
    pub r#type: String,
    pub driving_license: String,
    pub status: String,
    pub water_capacity: f64,
    pub height: f64,
    pub width: f64,
    pub length: f64,
    pub weight: f64,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
    pub description: Option<String>,
}

const VALID_VEHICLE_TYPES: &[&str] = &["transporter", "trailer"];
const VALID_VEHICLE_STATUSES: &[&str] = &["active", "available", "not available", "unknown"];
const VALID_DRIVING_LICENSES: &[&str] = &["B", "BE", "C", "CE"];

fn invalid_enum_issue(
    field: &'static str,
    path: &'static str,
    got: &str,
    valid: &[&str],
) -> ValidationIssue {
    ValidationIssue::from_error(
        &ValidationError::InvalidFormat {
            field,
            reason: format!("got `{}`, expected one of {:?}", got, valid),
        },
        path,
    )
}

pub(crate) fn collect_vehicle_issues(input: &VehicleDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = NumberPlate::new(&input.number_plate) {
        issues.push(ValidationIssue::from_error(&err, "numberPlate"));
    }
    if let Err(err) = VehicleModel::new(&input.model) {
        issues.push(ValidationIssue::from_error(&err, "model"));
    }
    if !VALID_VEHICLE_TYPES.contains(&input.r#type.as_str()) {
        issues.push(invalid_enum_issue(
            "vehicle.type",
            "type",
            &input.r#type,
            VALID_VEHICLE_TYPES,
        ));
    }
    if !VALID_DRIVING_LICENSES.contains(&input.driving_license.as_str()) {
        issues.push(invalid_enum_issue(
            "vehicle.driving_license",
            "drivingLicense",
            &input.driving_license,
            VALID_DRIVING_LICENSES,
        ));
    }
    if !VALID_VEHICLE_STATUSES.contains(&input.status.as_str()) {
        issues.push(invalid_enum_issue(
            "vehicle.status",
            "status",
            &input.status,
            VALID_VEHICLE_STATUSES,
        ));
    }
    if let Err(err) = WaterCapacity::new(input.water_capacity) {
        issues.push(ValidationIssue::from_error(&err, "waterCapacity"));
    }
    if let Err(err) = VehicleDimension::new(input.height, input.width, input.length, input.weight) {
        let path = match &err {
            ValidationError::OutOfRange { field, .. }
            | ValidationError::InvalidFormat { field, .. } => {
                field.rsplit('.').next().unwrap_or("dimension")
            }
            _ => "dimension",
        };
        issues.push(ValidationIssue::from_error(&err, path));
    }

    issues
}

#[wasm_bindgen(js_name = validateVehicleDraft)]
pub fn validate_vehicle_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: VehicleDraftInput = from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    let issues = collect_vehicle_issues(&draft);
    to_value(&issues).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> VehicleDraftInput {
        VehicleDraftInput {
            number_plate: "FL-EC-100".into(),
            model: "Mercedes Sprinter".into(),
            r#type: "transporter".into(),
            driving_license: "B".into(),
            status: "available".into(),
            water_capacity: 100.0,
            height: 2.5,
            width: 2.0,
            length: 6.0,
            weight: 3500.0,
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
    fn invalid_status_is_rejected() {
        let mut input = valid();
        input.status = "broken".into();
        let issues = collect_vehicle_issues(&input);
        assert!(issues.iter().any(|i| i.path == "status"));
    }

    #[test]
    fn negative_water_capacity_yields_issue() {
        let mut input = valid();
        input.water_capacity = -10.0;
        let issues = collect_vehicle_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "waterCapacity")
            .expect("water_capacity issue");
        assert!(issue.key.contains("water_capacity"));
    }
}
