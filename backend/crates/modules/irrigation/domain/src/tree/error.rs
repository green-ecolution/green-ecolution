use thiserror::Error;

use domain::shared::error::ValidationError;

#[derive(Debug, Error, PartialEq)]
pub enum TreeError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// Watermarks did not contain exactly the expected depths (30, 60, 90).
    #[error("sensor watermarks must contain exactly depths 30, 60, 90")]
    MalformedWatermarks,
    /// Tree is older than the monitoring window the calibration table covers.
    #[error("tree age exceeds monitored growth period (year > 3)")]
    BeyondMonitoring,
    /// Volumetric soil-moisture readings were empty or contained non-finite values.
    #[error("volumetric readings empty or invalid")]
    MalformedVolumetric,
    /// Cluster soil type is `Unknown` (or unset), so no KA5 calibration applies.
    #[error("soil type is uncalibrated (unknown)")]
    UncalibratedSoil,
    /// The planting year has not arrived yet, so the tree is not in the ground:
    /// no reading can describe it and no sensor can be attached to it.
    #[error("tree is not planted yet")]
    NotYetPlanted,
}
