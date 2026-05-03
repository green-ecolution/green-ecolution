use domain::shared::coordinates::Coordinate;
use domain::tree::{PlantingYear, Species, TreeNumber};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TreeDraftInput {
    pub number: String,
    pub species: String,
    pub planting_year: u32,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    #[allow(dead_code)]
    // reason: part of the expected frontend DTO shape; validation only checks specific fields
    pub description: Option<String>,
}

pub(crate) fn collect_tree_issues(input: &TreeDraftInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Err(err) = TreeNumber::new(&input.number) {
        issues.push(ValidationIssue::from_error(&err, "number"));
    }
    if let Err(err) = Species::new(&input.species) {
        issues.push(ValidationIssue::from_error(&err, "species"));
    }
    if let Err(err) = PlantingYear::new(input.planting_year) {
        issues.push(ValidationIssue::from_error(&err, "plantingYear"));
    }
    if let Err(err) = Coordinate::new(input.latitude, input.longitude) {
        let path = match &err {
            domain::shared::error::ValidationError::OutOfRange { field, .. }
                if *field == "coordinate.longitude" =>
            {
                "longitude"
            }
            _ => "latitude",
        };
        issues.push(ValidationIssue::from_error(&err, path));
    }

    issues
}

#[wasm_bindgen(js_name = validateTreeDraft)]
pub fn validate_tree_draft(input: JsValue) -> Result<JsValue, JsError> {
    let draft: TreeDraftInput = from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
    let issues = collect_tree_issues(&draft);
    to_value(&issues).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> TreeDraftInput {
        TreeDraftInput {
            number: "FL-001".into(),
            species: "Quercus".into(),
            planting_year: 2020,
            latitude: 52.5,
            longitude: 13.4,
            description: None,
        }
    }

    #[test]
    fn valid_draft_yields_no_issues() {
        let issues = collect_tree_issues(&valid_input());
        assert!(issues.is_empty(), "expected no issues, got {:?}", issues);
    }

    #[test]
    fn empty_species_yields_species_issue() {
        let mut input = valid_input();
        input.species = "".into();
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "species");
        assert_eq!(issues[0].key, "tree.species.empty");
    }

    #[test]
    fn future_planting_year_yields_planting_year_issue() {
        let mut input = valid_input();
        input.planting_year = 9999;
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "plantingYear");
        assert_eq!(issues[0].key, "tree.planting_year.outOfRange");
    }

    #[test]
    fn out_of_range_longitude_yields_longitude_issue() {
        let mut input = valid_input();
        input.longitude = 181.0;
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "longitude");
        assert_eq!(issues[0].key, "coordinate.longitude.outOfRange");
    }

    #[test]
    fn collects_multiple_issues_not_fail_fast() {
        let input = TreeDraftInput {
            number: "".into(),
            species: "".into(),
            planting_year: 9999,
            latitude: 91.0,
            longitude: 0.0,
            description: None,
        };
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 4, "got {:?}", issues);
        let paths: Vec<&str> = issues.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["number", "species", "plantingYear", "latitude"]);
    }
}
