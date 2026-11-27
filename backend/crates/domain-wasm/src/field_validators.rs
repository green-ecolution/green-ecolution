use domain::capacity::Amount;
use domain::region::RegionName;
use domain::sensor::SensorId;
use domain::settings::{DefectStreak, MapBounds, MapView, SensorOfflineAfter, ZoomLevel};
use domain::shared::coordinates::Coordinate;
use domain::shared::distance::Distance;
use domain::shared::email::Email;
use domain::shared::error::ValidationError;
use domain::shared::geo::BoundingBox;
use domain::shared::phone_number::PhoneNumber;
use domain::user::Username;
use domain::vehicle::{NumberPlate, VehicleDimension, VehicleModel};
use wasm_bindgen::prelude::*;

use crate::coerce::invalid_number_issue;
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

/// Rejects a fractional value instead of rounding it, so the field reports the
/// same issue the draft validation does on save.
pub(crate) fn whole_number_issue<T>(
    value: f64,
    field: &'static str,
    path: &'static str,
    build: impl FnOnce(i64) -> Result<T, ValidationError>,
) -> Option<ValidationIssue> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Some(invalid_number_issue(field, path));
    }
    build(value as i64)
        .err()
        .map(|err| ValidationIssue::from_error(&err, path))
}

pub(crate) fn report(issue: Option<ValidationIssue>) -> Result<JsValue, JsError> {
    issue.map_or(Ok(JsValue::NULL), |issue| to_js(&issue))
}

pub(crate) const SECONDS_PER_HOUR: f64 = 3600.0;

/// A duration value object checks seconds while its field asks for hours, so
/// the raw error would tell someone typing hours to stay between 3600 and
/// 2592000.
pub(crate) fn hours_issue(err: &ValidationError, path: &str) -> ValidationIssue {
    let ValidationError::OutOfRange {
        field,
        min,
        max,
        got,
    } = *err
    else {
        return ValidationIssue::from_error(err, path);
    };
    let in_hours = ValidationError::OutOfRange {
        field,
        min: min / SECONDS_PER_HOUR,
        max: max / SECONDS_PER_HOUR,
        got: got / SECONDS_PER_HOUR,
    };
    ValidationIssue::from_error(&in_hours, path)
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

pub(crate) fn capacity_amount_issue(amount: f64) -> Option<ValidationIssue> {
    Amount::new(amount)
        .err()
        .map(|err| ValidationIssue::from_error(&err, "amount"))
}

/// The range check modules hand the vehicle form for their capacity kinds.
#[wasm_bindgen(js_name = validateCapacityAmount)]
pub fn validate_capacity_amount(amount: f64) -> Result<JsValue, JsError> {
    report(capacity_amount_issue(amount))
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

fn sensor_offline_after_issue(hours: f64) -> Option<ValidationIssue> {
    SensorOfflineAfter::new((hours * SECONDS_PER_HOUR).round() as i64)
        .err()
        .map(|err| hours_issue(&err, "sensorOfflineAfterHours"))
}

/// Hours, not seconds: the form asks in hours.
#[wasm_bindgen(js_name = validateSensorOfflineAfterHours)]
pub fn validate_sensor_offline_after_hours(hours: f64) -> Result<JsValue, JsError> {
    report(sensor_offline_after_issue(hours))
}

#[wasm_bindgen(js_name = sensorOfflineAfterMinHours)]
pub fn sensor_offline_after_min_hours() -> f64 {
    SensorOfflineAfter::MIN as f64 / SECONDS_PER_HOUR
}

#[wasm_bindgen(js_name = sensorOfflineAfterMaxHours)]
pub fn sensor_offline_after_max_hours() -> f64 {
    SensorOfflineAfter::MAX as f64 / SECONDS_PER_HOUR
}

fn defect_streak_issue(count: f64) -> Option<ValidationIssue> {
    whole_number_issue(count, "settings.defect_streak", "defectStreak", |count| {
        DefectStreak::new(i32::try_from(count).unwrap_or(i32::MAX))
    })
}

#[wasm_bindgen(js_name = validateDefectStreak)]
pub fn validate_defect_streak(count: f64) -> Result<JsValue, JsError> {
    report(defect_streak_issue(count))
}

#[wasm_bindgen(js_name = defectStreakMin)]
pub fn defect_streak_min() -> f64 {
    f64::from(DefectStreak::MIN)
}

#[wasm_bindgen(js_name = defectStreakMax)]
pub fn defect_streak_max() -> f64 {
    f64::from(DefectStreak::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_negative_amount_has_a_translated_key() {
        let issue = capacity_amount_issue(-1.0).expect("negative amounts are refused");
        assert_eq!(issue.path, "amount");
        assert_eq!(issue.key, "capacity.amount.outOfRange");
        assert!(capacity_amount_issue(0.0).is_none());
        assert!(capacity_amount_issue(6000.0).is_none());
    }

    #[test]
    fn sensor_offline_after_reports_its_range_in_hours() {
        let issue = sensor_offline_after_issue(0.5).expect("half an hour is below the minimum");
        assert_eq!(issue.path, "sensorOfflineAfterHours");
        assert_eq!(issue.key, "settings.sensor_offline_after.outOfRange");
        assert_eq!(issue.params["min"].as_f64(), Some(1.0));
        assert_eq!(issue.params["max"].as_f64(), Some(720.0));
    }

    #[test]
    fn sensor_offline_after_accepts_both_ends_of_the_range() {
        assert!(sensor_offline_after_issue(1.0).is_none());
        assert!(sensor_offline_after_issue(720.0).is_none());
        assert_eq!(sensor_offline_after_min_hours(), 1.0);
        assert_eq!(sensor_offline_after_max_hours(), 720.0);
    }

    #[test]
    fn defect_streak_refuses_a_fraction_and_an_out_of_range_count() {
        let fraction = defect_streak_issue(2.5).expect("a fraction is not a count");
        assert_eq!(fraction.key, "settings.defect_streak.invalidFormat");
        let zero = defect_streak_issue(0.0).expect("zero is below the minimum");
        assert_eq!(zero.key, "settings.defect_streak.outOfRange");
        assert!(defect_streak_issue(3.0).is_none());
    }
}
