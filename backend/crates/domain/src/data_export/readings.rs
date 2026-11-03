use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::{ExportProperty, ExportUnit, ObservationKind, RemoteId, ge_key};
use crate::{
    Id, RepositoryError,
    module::ModuleKey,
    organization::Organization,
    sensor_model::{SensorAbilityName, SensorAbilityUnit},
    target::TargetRef,
};

/// One ability value of one stored reading, at the sensor's current target.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportReading {
    pub reading_id: Uuid,
    pub sensor_id: String,
    pub sensor_model: String,
    pub target: TargetRef,
    pub ability: SensorAbilityName,
    pub unit: SensorAbilityUnit,
    pub depth_cm: i32,
    pub value: f64,
    pub quality_issue: Option<String>,
    pub measured_at: DateTime<Utc>,
}

const UCUM: &str = "http://unitsofmeasure.org/ucum.html";

pub fn ucum(unit: SensorAbilityUnit) -> ExportUnit {
    let (name, symbol) = match unit {
        SensorAbilityUnit::Percent => ("percent", "%"),
        SensorAbilityUnit::Centibar => ("centibar", "cbar"),
        SensorAbilityUnit::Ohm => ("ohm", "Ohm"),
        SensorAbilityUnit::Celsius => ("degree Celsius", "Cel"),
        SensorAbilityUnit::Volt => ("volt", "V"),
        SensorAbilityUnit::Centimeter => ("centimeter", "cm"),
    };
    ExportUnit {
        name,
        symbol,
        definition: UCUM,
    }
}

pub fn raw_property(ability: SensorAbilityName, unit: SensorAbilityUnit) -> ExportProperty {
    let (label, description) = match ability {
        SensorAbilityName::SoilMoisture => ("Bodenfeuchte", "Volumetric soil moisture"),
        SensorAbilityName::SoilTension => ("Saugspannung", "Soil water tension"),
        SensorAbilityName::Temperature => ("Temperatur", "Temperature at the probe"),
        SensorAbilityName::Humidity => ("Feuchte", "Humidity at the probe"),
        SensorAbilityName::Battery => ("Batteriespannung", "Battery voltage of the sensor"),
        SensorAbilityName::Distance => ("Abstand", "Measured distance to the surface"),
    };
    ExportProperty {
        key: ability.as_str(),
        label,
        description,
        kind: ObservationKind::Measurement,
        unit: Some(ucum(unit)),
    }
}

/// `device:` rather than `sensor:` so no sensor id can collide with the virtual
/// sensor key `sensor:green-ecolution`.
pub fn device_sensor_key(sensor_id: &str) -> String {
    format!("device:{sensor_id}")
}

pub fn raw_stream_key(reading: &ExportReading) -> String {
    format!(
        "raw:{}:{}:{}:{}",
        reading.sensor_id,
        reading.ability.as_str(),
        reading.depth_cm,
        ge_key(&reading.target)
    )
}

/// The smallest UUID v7 of `at`'s millisecond: a cursor that starts exactly at
/// the activation without skipping a reading of the same millisecond.
pub fn v7_floor(at: DateTime<Utc>) -> Uuid {
    let ms = u64::try_from(at.timestamp_millis()).unwrap_or(0);
    let mut bytes = [0u8; 16];
    bytes[..6].copy_from_slice(&ms.to_be_bytes()[2..]);
    bytes[6] = 0x70;
    bytes[8] = 0x80;
    Uuid::from_bytes(bytes)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservationRow {
    pub phenomenon_time: DateTime<Utc>,
    pub result: f64,
    pub parameters: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatchOutcome {
    pub accepted: u32,
    pub rejected: u32,
}

pub struct RawStream<'a> {
    pub ge_key: &'a str,
    pub name: &'a str,
    pub thing: &'a RemoteId,
    pub property: &'a ExportProperty,
    pub sensor: &'a RemoteId,
    pub observed_property: &'a RemoteId,
}

#[async_trait::async_trait]
pub trait ExportReadingSource: Send + Sync {
    /// Every ability value of the next `limit` readings after `after` whose
    /// sensor's current target is in one of the (module, organization) pairs.
    async fn readings_after(
        &self,
        active: &[(ModuleKey, Id<Organization>)],
        after: Uuid,
        limit: i64,
    ) -> Result<Vec<ExportReading>, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn reading() -> ExportReading {
        ExportReading {
            reading_id: Uuid::nil(),
            sensor_id: "eui-1".into(),
            sensor_model: "GES-1000".into(),
            target: TargetRef::new("irrigation", "tree", Uuid::from_u128(1)).unwrap(),
            ability: SensorAbilityName::SoilMoisture,
            unit: SensorAbilityUnit::Percent,
            depth_cm: 40,
            value: 42.0,
            quality_issue: None,
            measured_at: Utc::now(),
        }
    }

    #[test]
    fn every_unit_has_a_ucum_symbol() {
        for (unit, symbol) in [
            (SensorAbilityUnit::Percent, "%"),
            (SensorAbilityUnit::Centibar, "cbar"),
            (SensorAbilityUnit::Ohm, "Ohm"),
            (SensorAbilityUnit::Celsius, "Cel"),
            (SensorAbilityUnit::Volt, "V"),
            (SensorAbilityUnit::Centimeter, "cm"),
        ] {
            assert_eq!(ucum(unit).symbol, symbol);
        }
    }

    #[test]
    fn raw_properties_are_keyed_by_ability() {
        let p = raw_property(SensorAbilityName::SoilMoisture, SensorAbilityUnit::Percent);
        assert_eq!(p.key, "soil_moisture");
        assert_eq!(p.label, "Bodenfeuchte");
        assert_eq!(p.kind, ObservationKind::Measurement);
        assert_eq!(p.unit.map(|u| u.symbol), Some("%"));
    }

    #[test]
    fn keys_name_sensor_ability_depth_and_thing() {
        assert_eq!(device_sensor_key("eui-1"), "device:eui-1");
        assert_eq!(
            raw_stream_key(&reading()),
            "raw:eui-1:soil_moisture:40:irrigation:tree:00000000-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn the_v7_floor_sorts_before_every_id_of_its_millisecond() {
        let at = Utc.with_ymd_and_hms(2026, 11, 3, 12, 0, 0).unwrap();
        let floor = v7_floor(at);
        assert_eq!(floor.get_version_num(), 7);
        assert_eq!(crate::uuid_v7_timestamp(&floor), Some(at));
        let same_ms = Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            at.timestamp() as u64,
            0,
        ));
        assert!(floor <= same_ms);
        let earlier = v7_floor(at - chrono::Duration::milliseconds(1));
        assert!(earlier < floor);
    }
}
