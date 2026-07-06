use domain::cluster::SoilCondition;
use domain::cluster::{ClusterAddress, ClusterName};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::coerce::validate_enum;
use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClusterDraftInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
    pub description: String,
    #[serde(default)]
    pub soil_condition: Option<String>,
}

pub(crate) fn collect_cluster_issues(input: &ClusterDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = ClusterName::new(&input.name) {
        issues.push(ValidationIssue::from_error(&err, "name"));
    }
    if let Err(err) = ClusterAddress::new(&input.address) {
        issues.push(ValidationIssue::from_error(&err, "address"));
    }
    if let Some(soil) = input.soil_condition.as_deref()
        && let Some(issue) =
            validate_enum::<SoilCondition>(soil, "cluster.soil_condition", "soilCondition")
    {
        issues.push(issue);
    }

    issues
}

#[wasm_bindgen(js_name = validateTreeClusterDraft)]
pub fn validate_tree_cluster_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: ClusterDraftInput = from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    let issues = collect_cluster_issues(&draft);
    to_value(&issues).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> ClusterDraftInput {
        ClusterDraftInput {
            name: "Park West".into(),
            address: "Mainstreet 1".into(),
            description: "".into(),
            soil_condition: Some("Uu".into()),
        }
    }

    #[test]
    fn valid_cluster_yields_no_issues() {
        assert!(collect_cluster_issues(&valid()).is_empty());
    }

    #[test]
    fn empty_name_and_address_yield_two_issues() {
        let input = ClusterDraftInput {
            name: "".into(),
            address: "".into(),
            description: "".into(),
            soil_condition: None,
        };
        let issues = collect_cluster_issues(&input);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].path, "name");
        assert_eq!(issues[0].key, "cluster.name.empty");
        assert_eq!(issues[1].path, "address");
        assert_eq!(issues[1].key, "cluster.address.empty");
    }

    #[test]
    fn invalid_soil_condition_is_rejected() {
        let mut input = valid();
        input.soil_condition = Some("kies".into());
        let issues = collect_cluster_issues(&input);
        let issue = issues
            .iter()
            .find(|i| i.path == "soilCondition")
            .expect("soilCondition issue");
        assert_eq!(issue.key, "cluster.soil_condition.invalidFormat");
    }

    #[test]
    fn missing_soil_condition_is_accepted() {
        let mut input = valid();
        input.soil_condition = None;
        assert!(collect_cluster_issues(&input).is_empty());
    }

    #[test]
    fn missing_optional_fields_in_json_use_defaults() {
        // Frontend may omit soilCondition / description; serde must not throw.
        let json = serde_json::json!({ "name": "X", "address": "Y" });
        let input: ClusterDraftInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.name, "X");
        assert_eq!(input.address, "Y");
        assert!(input.soil_condition.is_none());
    }
}
