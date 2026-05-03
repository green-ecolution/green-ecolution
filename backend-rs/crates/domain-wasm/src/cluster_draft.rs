use domain::cluster::{ClusterAddress, ClusterName};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClusterDraftInput {
    pub name: String,
    pub address: String,
    #[serde(default)]
    #[allow(dead_code)]
    // reason: part of the expected frontend DTO shape; validation only checks specific fields
    pub description: String,
}

pub(crate) fn collect_cluster_issues(input: &ClusterDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = ClusterName::new(&input.name) {
        issues.push(ValidationIssue::from_error(&err, "name"));
    }
    if let Err(err) = ClusterAddress::new(&input.address) {
        issues.push(ValidationIssue::from_error(&err, "address"));
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

    #[test]
    fn valid_cluster_yields_no_issues() {
        let input = ClusterDraftInput {
            name: "Park West".into(),
            address: "Mainstreet 1".into(),
            description: "".into(),
        };
        assert!(collect_cluster_issues(&input).is_empty());
    }

    #[test]
    fn empty_name_and_address_yield_two_issues() {
        let input = ClusterDraftInput {
            name: "".into(),
            address: "".into(),
            description: "".into(),
        };
        let issues = collect_cluster_issues(&input);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].path, "name");
        assert_eq!(issues[0].key, "cluster.name.empty");
        assert_eq!(issues[1].path, "address");
        assert_eq!(issues[1].key, "cluster.address.empty");
    }
}
