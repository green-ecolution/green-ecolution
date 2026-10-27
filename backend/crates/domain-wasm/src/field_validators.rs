use domain::capacity::Liters;
use domain::region::RegionName;
use domain::sensor::SensorId;
use domain::settings::{MapBounds, MapView, ZoomLevel};
use domain::shared::coordinates::Coordinate;
use domain::shared::distance::Distance;
use domain::shared::email::Email;
use domain::shared::error::ValidationError;
use domain::shared::geo::BoundingBox;
use domain::shared::phone_number::PhoneNumber;
use domain::user::Username;
use domain::vehicle::{NumberPlate, VehicleDimension, VehicleModel};
use wasm_bindgen::prelude::*;

use crate::issue::{ValidationIssue, to_js};

/// Convert a value-object construction result into a `JsValue` containing
/// either `null` (success) or a serialized [`ValidationIssue`] (failure).
pub(crate) fn finish<T>(
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
    finish(Liters::new(liters), "waterCapacity")
}

#[wasm_bindgen(js_name = validateCargoVolumeCapacity)]
pub fn validate_cargo_volume_capacity(liters: f64) -> Result<JsValue, JsError> {
    finish(Liters::new(liters), "cargoVolumeCapacity")
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
