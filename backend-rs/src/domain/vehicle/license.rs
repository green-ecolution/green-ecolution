use std::str::FromStr;

use crate::domain::shared::error::ValidationError;

/// EU driving license categories relevant to the fleet.
///
/// The hierarchy is B < BE < C < CE for the purposes of [`DrivingLicense::satisfies`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "driving_license")]
pub enum DrivingLicense {
    B,
    BE,
    C,
    CE,
}

impl DrivingLicense {
    /// Returns `true` if this license covers `required`.
    ///
    /// A higher category satisfies lower ones (CE covers B, BE, and C).
    pub fn satisfies(&self, required: DrivingLicense) -> bool {
        *self == required
            || matches!(
                (self, required),
                (DrivingLicense::BE, DrivingLicense::B)
                    | (DrivingLicense::C, DrivingLicense::B)
                    | (
                        DrivingLicense::CE,
                        DrivingLicense::B | DrivingLicense::BE | DrivingLicense::C
                    )
            )
    }
}

impl FromStr for DrivingLicense {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(Self::B),
            "BE" => Ok(Self::BE),
            "C" => Ok(Self::C),
            "CE" => Ok(Self::CE),
            other => Err(ValidationError::InvalidFormat {
                field: "vehicle.driving_license",
                reason: format!("unknown license '{other}'"),
            }),
        }
    }
}
