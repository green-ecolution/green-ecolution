use std::{fmt, str::FromStr};

use crate::shared::error::ValidationError;

/// Browser features a plugin view may use inside its iframe. The list is
/// closed on purpose: each entry widens what a third-party document can reach
/// on the operator's device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceCapability {
    Camera,
    Bluetooth,
}

impl DeviceCapability {
    pub const ALL: [DeviceCapability; 2] = [DeviceCapability::Camera, DeviceCapability::Bluetooth];

    pub fn as_str(self) -> &'static str {
        match self {
            DeviceCapability::Camera => "camera",
            DeviceCapability::Bluetooth => "bluetooth",
        }
    }
}

impl fmt::Display for DeviceCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DeviceCapability {
    type Err = ValidationError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        DeviceCapability::ALL
            .into_iter()
            .find(|c| c.as_str() == raw)
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "plugin.device_capabilities",
                reason: format!("unknown device capability '{raw}', expected camera or bluetooth"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_capabilities() {
        assert_eq!(
            "camera".parse::<DeviceCapability>(),
            Ok(DeviceCapability::Camera)
        );
        assert_eq!(
            "bluetooth".parse::<DeviceCapability>(),
            Ok(DeviceCapability::Bluetooth)
        );
    }

    #[test]
    fn rejects_unknown_capability() {
        assert!("geolocation".parse::<DeviceCapability>().is_err());
        assert!("Camera".parse::<DeviceCapability>().is_err());
    }

    #[test]
    fn round_trips_through_as_str() {
        for capability in DeviceCapability::ALL {
            assert_eq!(
                capability.as_str().parse::<DeviceCapability>(),
                Ok(capability)
            );
        }
    }
}
