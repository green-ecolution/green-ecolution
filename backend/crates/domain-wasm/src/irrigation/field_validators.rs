use domain::shared::error::ValidationError;
use irrigation_domain::cluster::{ClusterAddress, ClusterName};
use irrigation_domain::settings::{JustWateredTtl, WaterDemand};
use irrigation_domain::tree::{
    MAX_PLANTING_YEAR, MIN_PLANTING_YEAR, PlantingYear, Species, TreeNumber,
};
use wasm_bindgen::prelude::*;

use crate::field_validators::finish;
use crate::issue::{ValidationIssue, to_js};

#[wasm_bindgen(js_name = validateSpecies)]
pub fn validate_species(value: &str) -> Result<JsValue, JsError> {
    finish(Species::new(value), "species")
}

#[wasm_bindgen(js_name = validateTreeNumber)]
pub fn validate_tree_number(value: &str) -> Result<JsValue, JsError> {
    finish(TreeNumber::new(value), "number")
}

#[wasm_bindgen(js_name = validatePlantingYear)]
pub fn validate_planting_year(year: u32) -> Result<JsValue, JsError> {
    finish(PlantingYear::new(year), "plantingYear")
}

/// Exposed so input controls can constrain their steppers to the same range the
/// domain enforces, instead of restating the bounds in TypeScript.
#[wasm_bindgen(js_name = plantingYearMin)]
pub fn planting_year_min() -> u32 {
    MIN_PLANTING_YEAR
}

#[wasm_bindgen(js_name = plantingYearMax)]
pub fn planting_year_max() -> u32 {
    MAX_PLANTING_YEAR
}

/// Whether the planting still lies ahead, by the same rule the backend uses to
/// refuse a sensor and to withhold a watering status.
#[wasm_bindgen(js_name = plantingYearIsFuture)]
pub fn planting_year_is_future(year: u32) -> bool {
    PlantingYear::reconstitute(year).is_future(chrono::Utc::now())
}

#[wasm_bindgen(js_name = validateClusterName)]
pub fn validate_cluster_name(value: &str) -> Result<JsValue, JsError> {
    finish(ClusterName::new(value), "name")
}

#[wasm_bindgen(js_name = validateClusterAddress)]
pub fn validate_cluster_address(value: &str) -> Result<JsValue, JsError> {
    finish(ClusterAddress::new(value), "address")
}

#[wasm_bindgen(js_name = validateWaterDemand)]
pub fn validate_water_demand(liters: f64) -> Result<JsValue, JsError> {
    finish(WaterDemand::new(liters), "waterDemand")
}

/// Exposed so the settings form can constrain its input to the same range the
/// domain enforces, instead of restating the bounds in TypeScript.
#[wasm_bindgen(js_name = waterDemandMin)]
pub fn water_demand_min() -> f64 {
    WaterDemand::MIN
}

#[wasm_bindgen(js_name = waterDemandMax)]
pub fn water_demand_max() -> f64 {
    WaterDemand::MAX
}

const SECONDS_PER_HOUR: f64 = 3600.0;

/// Hours, not seconds: the form asks in hours, and a range check that speaks
/// a different unit than the field is how off-by-3600 bugs get in.
#[wasm_bindgen(js_name = validateJustWateredTtlHours)]
pub fn validate_just_watered_ttl_hours(hours: f64) -> Result<JsValue, JsError> {
    match JustWateredTtl::new((hours * SECONDS_PER_HOUR).round() as i64) {
        Ok(_) => Ok(JsValue::NULL),
        Err(err) => to_js(&ttl_hours_issue(&err)),
    }
}

/// `JustWateredTtl` checks seconds while the field asks for hours, so the raw
/// domain error would tell someone typing hours to stay between 3600 and
/// 1209600.
fn ttl_hours_issue(err: &ValidationError) -> ValidationIssue {
    let ValidationError::OutOfRange {
        field,
        min,
        max,
        got,
    } = *err
    else {
        return ValidationIssue::from_error(err, "justWateredTtlHours");
    };
    let in_hours = ValidationError::OutOfRange {
        field,
        min: min / SECONDS_PER_HOUR,
        max: max / SECONDS_PER_HOUR,
        got: got / SECONDS_PER_HOUR,
    };
    ValidationIssue::from_error(&in_hours, "justWateredTtlHours")
}

#[wasm_bindgen(js_name = justWateredTtlMinHours)]
pub fn just_watered_ttl_min_hours() -> f64 {
    JustWateredTtl::MIN as f64 / SECONDS_PER_HOUR
}

#[wasm_bindgen(js_name = justWateredTtlMaxHours)]
pub fn just_watered_ttl_max_hours() -> f64 {
    JustWateredTtl::MAX as f64 / SECONDS_PER_HOUR
}

#[cfg(test)]
mod tests {
    // The exports returning `JsValue` need a JS runtime and cannot run on the
    // host target; what is testable here is the pure-Rust logic around them.
    // The ranges themselves are covered by the value-object tests in `irrigation-domain`
    // and by the issue-mapping tests. Aggregate validators (next module) cover the
    // serialisation round-trip via serde_json, which exercises the same
    // serde::Serialize impl as serde-wasm-bindgen.
    use super::*;

    #[test]
    fn just_watered_ttl_reports_its_range_in_the_unit_the_field_asks_for() {
        let err = JustWateredTtl::new(1_800).unwrap_err();
        let issue = ttl_hours_issue(&err);

        assert_eq!(issue.path, "justWateredTtlHours");
        assert_eq!(issue.key, "settings.just_watered_ttl.outOfRange");
        assert_eq!(issue.params["min"].as_f64(), Some(1.0));
        assert_eq!(issue.params["max"].as_f64(), Some(336.0));
        assert_eq!(issue.params["got"].as_f64(), Some(0.5));
    }

    #[test]
    fn the_hour_bounds_agree_with_the_reported_range() {
        assert_eq!(just_watered_ttl_min_hours(), 1.0);
        assert_eq!(just_watered_ttl_max_hours(), 336.0);
    }
}
