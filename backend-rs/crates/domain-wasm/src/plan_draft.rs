use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WateringPlanDraftInput {
    pub date: DateTime<Utc>,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
    pub description: Option<String>,
    pub cluster_ids: Vec<i32>,
    #[serde(default)]
    pub transporter_id: Option<i32>,
    #[serde(default)]
    #[allow(dead_code)] // accepted but not currently validated
    pub trailer_id: Option<i32>,
    #[serde(default)]
    pub driver_ids: Vec<String>,
}

pub(crate) fn collect_plan_issues(input: &WateringPlanDraftInput) -> Vec<ValidationIssue> {
    use domain::shared::error::ValidationError;
    let mut issues = Vec::new();

    if input.cluster_ids.is_empty() {
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
    }

    if input.transporter_id.is_none() {
        issues.push(ValidationIssue::from_error(
            &ValidationError::EmptyString {
                field: "watering_plan.transporter_id",
            },
            "transporterId",
        ));
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
            cluster_ids: vec![1, 2],
            transporter_id: Some(10),
            trailer_id: None,
            driver_ids: vec!["00000000-0000-0000-0000-000000000001".into()],
        }
    }

    #[test]
    fn valid_plan_yields_no_issues() {
        assert!(collect_plan_issues(&valid()).is_empty());
    }

    #[test]
    fn empty_cluster_ids_yields_issue() {
        let mut input = valid();
        input.cluster_ids.clear();
        let issues = collect_plan_issues(&input);
        assert_eq!(issues[0].path, "clusterIds");
    }

    #[test]
    fn missing_transporter_yields_issue() {
        let mut input = valid();
        input.transporter_id = None;
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "transporterId"));
    }

    #[test]
    fn past_date_yields_date_issue() {
        let mut input = valid();
        input.date = Utc::now() - Duration::days(2);
        let issues = collect_plan_issues(&input);
        assert!(issues.iter().any(|i| i.path == "date"));
    }
}
