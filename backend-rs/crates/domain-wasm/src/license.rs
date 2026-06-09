use std::str::FromStr;

use domain::vehicle::DrivingLicense;
use wasm_bindgen::prelude::*;

/// Returns `true` if the `held` license category covers `required`.
///
/// Delegates to the domain's [`DrivingLicense::satisfies`] so the frontend
/// and backend share one license hierarchy. Accepts the wire-format strings
/// (`"B" | "BE" | "C" | "CE"`); an unknown value is a `JsError`.
#[wasm_bindgen(js_name = licenseSatisfies)]
pub fn license_satisfies(held: &str, required: &str) -> Result<bool, JsError> {
    let held = DrivingLicense::from_str(held).map_err(|e| JsError::new(&e.to_string()))?;
    let required = DrivingLicense::from_str(required).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(held.satisfies(required))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_agrees_with_domain_for_all_pairs() {
        let all = ["B", "BE", "C", "CE"];
        for h in all {
            for r in all {
                let held = DrivingLicense::from_str(h).unwrap();
                let required = DrivingLicense::from_str(r).unwrap();
                assert_eq!(
                    license_satisfies(h, r).unwrap(),
                    held.satisfies(required),
                    "wrapper disagrees with domain for held={h} required={r}"
                );
            }
        }
    }

    #[test]
    fn unknown_license_is_an_error() {
        // JsError::new panics on non-wasm targets, so test the parse layer
        // directly — that's where the error originates in license_satisfies.
        assert!(DrivingLicense::from_str("X").is_err());
        assert!(DrivingLicense::from_str("Z").is_err());
    }
}
