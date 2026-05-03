use domain::shared::coordinates::Coordinate;
use domain::tree::{PlantingYear, Species, TreeNumber};
use serde::Deserialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

use crate::coerce::{LooseF64, LooseU32, invalid_number_issue};
use crate::issue::ValidationIssue;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TreeDraftInput {
    pub number: String,
    pub species: String,
    pub planting_year: LooseU32,
    pub latitude: LooseF64,
    pub longitude: LooseF64,
    #[serde(default)]
    #[allow(dead_code)] // free-text DTO field, not validated
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

    match input.planting_year.0 {
        None => issues.push(invalid_number_issue("tree.planting_year", "plantingYear")),
        Some(year) => {
            if let Err(err) = PlantingYear::new(year) {
                issues.push(ValidationIssue::from_error(&err, "plantingYear"));
            }
        }
    }

    match (input.latitude.0, input.longitude.0) {
        (Some(lat), Some(lng)) => {
            if let Err(err) = Coordinate::new(lat, lng) {
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
            planting_year: LooseU32(Some(2020)),
            latitude: LooseF64(Some(52.5)),
            longitude: LooseF64(Some(13.4)),
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
        input.planting_year = LooseU32(Some(9999));
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "plantingYear");
        assert_eq!(issues[0].key, "tree.planting_year.outOfRange");
    }

    #[test]
    fn unparseable_planting_year_yields_issue() {
        let mut input = valid_input();
        input.planting_year = LooseU32(None);
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "plantingYear");
        assert_eq!(issues[0].key, "tree.planting_year.invalidFormat");
    }

    #[test]
    fn out_of_range_longitude_yields_longitude_issue() {
        let mut input = valid_input();
        input.longitude = LooseF64(Some(181.0));
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "longitude");
        assert_eq!(issues[0].key, "coordinate.longitude.outOfRange");
    }

    #[test]
    fn unparseable_latitude_yields_issue() {
        let mut input = valid_input();
        input.latitude = LooseF64(None);
        let issues = collect_tree_issues(&input);
        assert!(
            issues
                .iter()
                .any(|i| i.path == "latitude" && i.key == "coordinate.latitude.invalidFormat")
        );
    }

    #[test]
    fn collects_multiple_issues_not_fail_fast() {
        let input = TreeDraftInput {
            number: "".into(),
            species: "".into(),
            planting_year: LooseU32(Some(9999)),
            latitude: LooseF64(Some(91.0)),
            longitude: LooseF64(Some(0.0)),
            description: None,
        };
        let issues = collect_tree_issues(&input);
        assert_eq!(issues.len(), 4, "got {:?}", issues);
        let paths: Vec<&str> = issues.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["number", "species", "plantingYear", "latitude"]);
    }
}
