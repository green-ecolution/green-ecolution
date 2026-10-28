use wasm_bindgen::prelude::*;
use waste_domain::settings::FullThreshold;
use waste_domain::values::{BinDepth, BinVolume, WasteBinNumber};

use crate::field_validators::finish;

#[wasm_bindgen(js_name = wasteValidateBinNumber)]
pub fn validate_bin_number(value: &str) -> Result<JsValue, JsError> {
    finish(WasteBinNumber::new(value), "number")
}

#[wasm_bindgen(js_name = wasteValidateBinVolume)]
pub fn validate_bin_volume(liters: f64) -> Result<JsValue, JsError> {
    finish(BinVolume::new(liters), "volumeLiters")
}

#[wasm_bindgen(js_name = wasteValidateBinDepth)]
pub fn validate_bin_depth(centimeters: f64) -> Result<JsValue, JsError> {
    finish(BinDepth::new(centimeters.round() as i64), "depthCm")
}

#[wasm_bindgen(js_name = wasteValidateFullThreshold)]
pub fn validate_full_threshold(percent: f64) -> Result<JsValue, JsError> {
    finish(
        FullThreshold::new(percent.round() as i64),
        "fullThresholdPercent",
    )
}

#[wasm_bindgen(js_name = wasteBinVolumeMin)]
pub fn bin_volume_min() -> f64 {
    BinVolume::MIN
}

#[wasm_bindgen(js_name = wasteBinVolumeMax)]
pub fn bin_volume_max() -> f64 {
    BinVolume::MAX
}

#[wasm_bindgen(js_name = wasteBinDepthMin)]
pub fn bin_depth_min() -> f64 {
    BinDepth::MIN as f64
}

#[wasm_bindgen(js_name = wasteBinDepthMax)]
pub fn bin_depth_max() -> f64 {
    BinDepth::MAX as f64
}

#[wasm_bindgen(js_name = wasteFullThresholdMin)]
pub fn full_threshold_min() -> f64 {
    FullThreshold::MIN as f64
}

#[wasm_bindgen(js_name = wasteFullThresholdMax)]
pub fn full_threshold_max() -> f64 {
    FullThreshold::MAX as f64
}
