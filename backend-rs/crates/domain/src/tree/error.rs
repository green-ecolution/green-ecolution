use thiserror::Error;

use crate::shared::error::ValidationError;

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
}
