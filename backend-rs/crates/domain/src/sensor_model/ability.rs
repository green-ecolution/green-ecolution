use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "sensor_ability_unit", rename_all = "snake_case")
)]
pub enum SensorAbilityUnit {
    Percent,
    Centibar,
    Ohm,
    Celsius,
    Volt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorAbilityName {
    SoilTension,
    SoilMoisture,
    Temperature,
    Humidity,
    Battery,
}

impl SensorAbilityName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SoilTension => "soil_tension",
            Self::SoilMoisture => "soil_moisture",
            Self::Temperature => "temperature",
            Self::Humidity => "humidity",
            Self::Battery => "battery",
        }
    }
}

impl FromStr for SensorAbilityName {
    type Err = UnknownAbility;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "soil_tension" => Ok(Self::SoilTension),
            "soil_moisture" => Ok(Self::SoilMoisture),
            "temperature" => Ok(Self::Temperature),
            "humidity" => Ok(Self::Humidity),
            "battery" => Ok(Self::Battery),
            other => Err(UnknownAbility(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("unknown sensor ability name: {0}")]
pub struct UnknownAbility(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorAbility {
    pub id: uuid::Uuid,
    pub name: SensorAbilityName,
    pub unit: SensorAbilityUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorModelAbility {
    pub id: uuid::Uuid,
    pub ability: SensorAbility,
    pub depth_cm: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_roundtrip() {
        for n in [
            SensorAbilityName::SoilTension,
            SensorAbilityName::SoilMoisture,
            SensorAbilityName::Temperature,
            SensorAbilityName::Humidity,
            SensorAbilityName::Battery,
        ] {
            assert_eq!(n.as_str().parse::<SensorAbilityName>().unwrap(), n);
        }
    }

    #[test]
    fn unknown_ability_errors() {
        let err = "salinity".parse::<SensorAbilityName>().unwrap_err();
        assert_eq!(err.0, "salinity");
    }
}
