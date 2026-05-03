use domain::cluster::{ClusterAddress, ClusterName};
use domain::region::RegionName;
use domain::sensor::SensorId;
use domain::shared::coordinates::Coordinate;
use domain::shared::distance::Distance;
use domain::shared::email::Email;
use domain::shared::water_capacity::WaterCapacity;
use domain::tree::{PlantingYear, Species, TreeNumber};
use domain::user::Username;
use domain::vehicle::{NumberPlate, VehicleDimension, VehicleModel};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::issue::ValidationIssue;

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
            Ok(to_value(&issue).map_err(|e| JsError::new(&e.to_string()))?)
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
            Ok(to_value(&issue).map_err(|e| JsError::new(&e.to_string()))?)
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
                | domain::shared::error::ValidationError::InvalidFormat { field, .. } => field
                    .rsplit('.')
                    .next()
                    .unwrap_or("dimension"),
                _ => "dimension",
            };
            let issue = ValidationIssue::from_error(&err, path);
            Ok(to_value(&issue).map_err(|e| JsError::new(&e.to_string()))?)
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

#[cfg(test)]
mod tests {
    // The wasm-bindgen exports cannot run on the host target. The pure-Rust
    // logic is covered by the value-object tests in `domain` and by the
    // issue-mapping tests. Aggregate validators (next module) cover the
    // serialisation round-trip via serde_json, which exercises the same
    // serde::Serialize impl as serde-wasm-bindgen.
}
