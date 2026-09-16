use domain::cluster::{ClusterAddress, ClusterName};
use domain::region::RegionName;
use domain::sensor::SensorId;
use domain::settings::{JustWateredTtl, MapBounds, MapView, WaterDemand, ZoomLevel};
use domain::shared::coordinates::Coordinate;
use domain::shared::distance::Distance;
use domain::shared::email::Email;
use domain::shared::error::ValidationError;
use domain::shared::geo::BoundingBox;
use domain::shared::phone_number::PhoneNumber;
use domain::shared::water_capacity::WaterCapacity;
use domain::tree::{MAX_PLANTING_YEAR, MIN_PLANTING_YEAR, PlantingYear, Species, TreeNumber};
use domain::user::Username;
use domain::vehicle::{NumberPlate, VehicleDimension, VehicleModel};
use wasm_bindgen::prelude::*;

use crate::issue::{ValidationIssue, to_js};

/// Convert a value-object construction result into a `JsValue` containing
/// either `null` (success) or a serialized [`ValidationIssue`] (failure).
fn finish<T>(
    result: Result<T, domain::shared::error::ValidationError>,
    path: &str,
) -> Result<JsValue, JsError> {
    match result {
        Ok(_) => Ok(JsValue::NULL),
        Err(err) => {
            let issue = ValidationIssue::from_error(&err, path);
            to_js(&issue)
        }
    }
}

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

#[wasm_bindgen(js_name = validateCoordinate)]
pub fn validate_coordinate(latitude: f64, longitude: f64) -> Result<JsValue, JsError> {
    // The domain returns the first failing axis; the path maps accordingly.
    match Coordinate::new(latitude, longitude) {
        Ok(_) => Ok(JsValue::NULL),
        Err(err) => {
            let path = match &err {
                domain::shared::error::ValidationError::OutOfRange { field, .. }
                    if *field == "coordinate.longitude" =>
                {
                    "longitude"
                }
                _ => "latitude",
            };
            let issue = ValidationIssue::from_error(&err, path);
            to_js(&issue)
        }
    }
}

#[wasm_bindgen(js_name = validateClusterName)]
pub fn validate_cluster_name(value: &str) -> Result<JsValue, JsError> {
    finish(ClusterName::new(value), "name")
}

#[wasm_bindgen(js_name = validateClusterAddress)]
pub fn validate_cluster_address(value: &str) -> Result<JsValue, JsError> {
    finish(ClusterAddress::new(value), "address")
}

#[wasm_bindgen(js_name = validateRegionName)]
pub fn validate_region_name(value: &str) -> Result<JsValue, JsError> {
    finish(RegionName::new(value), "name")
}

#[wasm_bindgen(js_name = validateNumberPlate)]
pub fn validate_number_plate(value: &str) -> Result<JsValue, JsError> {
    finish(NumberPlate::new(value), "numberPlate")
}

#[wasm_bindgen(js_name = validateVehicleModel)]
pub fn validate_vehicle_model(value: &str) -> Result<JsValue, JsError> {
    finish(VehicleModel::new(value), "model")
}

#[wasm_bindgen(js_name = validateVehicleDimension)]
pub fn validate_vehicle_dimension(
    height: f64,
    width: f64,
    length: f64,
    weight: f64,
) -> Result<JsValue, JsError> {
    match VehicleDimension::new(height, width, length, weight) {
        Ok(_) => Ok(JsValue::NULL),
        Err(err) => {
            // VehicleDimension uses fields like "vehicle.dimension.height"
            // — strip the prefix to derive the form path.
            let path = match &err {
                domain::shared::error::ValidationError::OutOfRange { field, .. }
                | domain::shared::error::ValidationError::InvalidFormat { field, .. } => {
                    field.rsplit('.').next().unwrap_or("dimension")
                }
                _ => "dimension",
            };
            let issue = ValidationIssue::from_error(&err, path);
            to_js(&issue)
        }
    }
}

#[wasm_bindgen(js_name = validateWaterCapacity)]
pub fn validate_water_capacity(liters: f64) -> Result<JsValue, JsError> {
    finish(WaterCapacity::new(liters), "waterCapacity")
}

#[wasm_bindgen(js_name = validateDistance)]
pub fn validate_distance(meters: f64) -> Result<JsValue, JsError> {
    finish(Distance::new(meters), "distance")
}

#[wasm_bindgen(js_name = validateEmail)]
pub fn validate_email(value: &str) -> Result<JsValue, JsError> {
    finish(Email::new(value), "email")
}

#[wasm_bindgen(js_name = validateUsername)]
pub fn validate_username(value: &str) -> Result<JsValue, JsError> {
    finish(Username::new(value), "username")
}

#[wasm_bindgen(js_name = validateSensorId)]
pub fn validate_sensor_id(value: &str) -> Result<JsValue, JsError> {
    finish(SensorId::new(value), "sensorId")
}

#[wasm_bindgen(js_name = validatePhoneNumber)]
pub fn validate_phone_number(value: &str) -> Result<JsValue, JsError> {
    finish(PhoneNumber::new(value), "phoneNumber")
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

/// The whole viewport at once, because its rules are relational: the box must
/// not be inverted and the centre has to lie inside it. Checking centre and
/// corners separately cannot see either condition.
///
/// Leaving the limits out entirely means the map is not penned in, which is a
/// viewport with nothing to contradict; passing only some of them is a caller
/// error and is reported as such rather than silently ignored.
#[wasm_bindgen(js_name = validateMapView)]
#[allow(clippy::too_many_arguments)] // reason: one flat call per wire field, to keep the JS side free of a wrapper type
pub fn validate_map_view(
    center_lat: f64,
    center_lng: f64,
    sw_lat: Option<f64>,
    sw_lng: Option<f64>,
    ne_lat: Option<f64>,
    ne_lng: Option<f64>,
    min_zoom: Option<u8>,
    max_zoom: Option<u8>,
) -> Result<JsValue, JsError> {
    let bounds = match (sw_lat, sw_lng, ne_lat, ne_lng, min_zoom, max_zoom) {
        (None, None, None, None, None, None) => Ok(None),
        (Some(s_lat), Some(s_lng), Some(n_lat), Some(n_lng), Some(min), Some(max)) => {
            BoundingBox::try_new(s_lat, s_lng, n_lat, n_lng).and_then(|bbox| {
                Ok(Some(MapBounds::new(
                    bbox,
                    ZoomLevel::new(min)?,
                    ZoomLevel::new(max)?,
                )?))
            })
        }
        _ => Err(ValidationError::InvalidFormat {
            field: "settings.map_view",
            reason: "the map limits are given in full or not at all".into(),
        }),
    };
    let view = Coordinate::new(center_lat, center_lng)
        .and_then(|center| bounds.and_then(|bounds| MapView::new(center, bounds)));
    finish(view, "mapView")
}

#[cfg(test)]
mod tests {
    // The exports returning `JsValue` need a JS runtime and cannot run on the
    // host target; what is testable here is the pure-Rust logic around them.
    // The ranges themselves are covered by the value-object tests in `domain`
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
