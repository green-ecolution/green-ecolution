use chrono::{DateTime, Utc};
use domain::shared::error::ValidationError;
use domain::watering_plan::WateringPlanStatus;
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::coerce::validate_enum;
use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WateringPlanDraftInput {
    pub date: DateTime<Utc>,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
    pub description: Option<String>,
    #[serde(default)]
    pub cluster_ids: Vec<String>,
    #[serde(default)]
    pub transporter_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // accepted but not currently validated
    pub trailer_id: Option<String>,
    #[serde(default)]
    pub driver_ids: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Treat `Some("")` as "not provided" — RHF stores unselected select inputs
/// as the empty string rather than `undefined`.
fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|v| (!v.is_empty()).then_some(v))
}

pub(crate) fn collect_plan_issues(input: &WateringPlanDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if input.cluster_ids.iter().all(|s| s.is_empty()) {
        issues.push(ValidationIssue::from_error(
            &ValidationError::EmptyString {
                field: "watering_plan.cluster_ids",
            },
            "clusterIds",
        ));
    }

    if input.driver_ids.is_empty() {
        issues.push(ValidationIssue::from_error(
            &ValidationError::EmptyString {
                field: "watering_plan.driver_ids",
            },
            "driverIds",
        ));
    } else {
        for id in &input.driver_ids {
            if Uuid::parse_str(id).is_err() {
                issues.push(ValidationIssue::from_error(
                    &ValidationError::InvalidFormat {
                        field: "watering_plan.driver_ids",
                        reason: format!("`{}` is not a valid UUID", id),
                    },
                    "driverIds",
                ));
                break;
            }
        }
    }

    if non_empty(input.transporter_id.as_deref()).is_none() {
        issues.push(ValidationIssue::from_error(
            &ValidationError::EmptyString {
                field: "watering_plan.transporter_id",
            },
            "transporterId",
        ));
    }
    // trailer_id is optional; nothing to do when None or empty.

    if let Some(status) = input.status.as_deref()
        && let Some(issue) =
            validate_enum::<WateringPlanStatus>(status, "watering_plan.status", "status")
    {
        issues.push(issue);
    }

    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_utc();
    if input.date < today_start {
        issues.push(ValidationIssue::from_error(
            &ValidationError::OutOfRange {
                field: "watering_plan.date",
                min: today_start.timestamp() as f64,
                max: f64::MAX,
                got: input.date.timestamp() as f64,
            },
            "date",
        ));
    }

    issues
}

#[wasm_bindgen(js_name = validateWateringPlanDraft)]
pub fn validate_watering_plan_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: WateringPlanDraftInput =
        from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    let issues = collect_plan_issues(&draft);
    to_value(&issues).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn future_date() -> DateTime<Utc> {
        Utc::now() + Duration::days(2)
    }

    fn valid() -> WateringPlanDraftInput {
        WateringPlanDraftInput {
            date: future_date(),
            description: None,
            cluster_ids: vec!["cluster-a".into(), "cluster-b".into()],
            transporter_id: Some("vehicle-1".into()),
            trailer_id: None,
            driver_ids: vec!["00000000-0000-0000-0000-000000000001".into()],
            status: Some("planned".into()),
        }
    }

    #[test]
    fn valid_plan_yields_no_issues() {
        let issues = collect_plan_issues(&valid());
        assert!(issues.is_empty(), "expected no issues, got {:?}", issues);
    }

    #[test]
    fn empty_cluster_ids_yields_issue() {
        let mut input = valid();
        input.cluster_ids.clear();
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "clusterIds"));
    }

    #[test]
    fn cluster_ids_of_only_empty_strings_yields_issue() {
        let mut input = valid();
        input.cluster_ids = vec!["".into(), "".into()];
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "clusterIds"));
    }

    #[test]
    fn missing_transporter_yields_issue() {
        let mut input = valid();
        input.transporter_id = None;
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "transporterId"));
    }

    #[test]
    fn empty_transporter_yields_issue() {
        let mut input = valid();
        input.transporter_id = Some("".into());
        let issues = collect_plan_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "transporterId")
            .expect("transporterId issue");
        assert_eq!(issue.key, "watering_plan.transporter_id.empty");
    }

    #[test]
    fn empty_trailer_is_accepted() {
        let mut input = valid();
        input.trailer_id = Some("".into());
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().all(|i| i.path != "trailerId"));
    }

    #[test]
    fn invalid_uuid_in_driver_ids_yields_issue() {
        let mut input = valid();
        input.driver_ids = vec!["not-a-uuid".into()];
        let issues = collect_plan_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "driverIds")
            .expect("driverIds issue");
        assert_eq!(issue.key, "watering_plan.driver_ids.invalidFormat");
    }

    #[test]
    fn invalid_status_is_rejected() {
        let mut input = valid();
        input.status = Some("merged".into());
        let issues = collect_plan_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "status")
            .expect("status issue");
        assert_eq!(issue.key, "watering_plan.status.invalidFormat");
    }

    #[test]
    fn missing_status_is_accepted() {
        let mut input = valid();
        input.status = None;
        assert!(collect_plan_issues(&input).is_empty());
    }

    #[test]
    fn past_date_yields_date_issue() {
        let mut input = valid();
        input.date = Utc::now() - Duration::days(2);
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "date"));
    }
}
