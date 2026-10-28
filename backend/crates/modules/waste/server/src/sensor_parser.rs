//! Wire format of the simulated distance sensor.

use domain::{
    events::SensorReadings,
    sensor::{SensorId, data::DistanceReading, repository::NormalizedValue},
    sensor_model::{SensorAbilityName, SensorModel},
};
use rust_decimal::Decimal;
use server::service::{
    sensor_parsers::{PayloadError, SensorPayloadParser},
    sensor_service::ReadingIngest,
};

pub const DISTANCE_MODEL: &str = "GE Distance (simuliert)";

pub struct DistanceSensorParser;

impl SensorPayloadParser for DistanceSensorParser {
    fn model_name(&self) -> &'static str {
        DISTANCE_MODEL
    }

    fn parse(
        &self,
        model: &SensorModel,
        body: &serde_json::Value,
        sensor_id: SensorId,
    ) -> Result<ReadingIngest, PayloadError> {
        let centimeters = body
            .get("distance_cm")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| PayloadError("missing distance_cm".into()))?;
        let battery = body.get("battery").and_then(serde_json::Value::as_f64);

        let mut normalized = Vec::with_capacity(2);
        for (ability, value) in [
            (SensorAbilityName::Distance, Some(centimeters)),
            (SensorAbilityName::Battery, battery),
        ] {
            let (Some(value), Some(model_ability_id)) = (value, model.ability_id_for(ability, 0))
            else {
                continue;
            };
            normalized.push(NormalizedValue {
                model_ability_id,
                value: Decimal::from_f64_retain(value).unwrap_or_default(),
                issue: None,
            });
        }

        Ok(ReadingIngest {
            sensor_id,
            raw_payload: body.clone(),
            normalized,
            typed: SensorReadings::Distances(vec![DistanceReading {
                depth_cm: 0,
                centimeters,
            }]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        Id,
        sensor_model::{SensorAbility, SensorAbilityUnit, SensorModelAbility, SensorModelName},
    };
    use serde_json::json;

    fn distance_model() -> SensorModel {
        let ability = |name, unit| SensorModelAbility {
            id: uuid::Uuid::now_v7(),
            ability: SensorAbility {
                id: uuid::Uuid::now_v7(),
                name,
                unit,
            },
            depth_cm: 0,
        };
        SensorModel {
            id: Id::new_v7(),
            name: SensorModelName::new(DISTANCE_MODEL).unwrap(),
            description: None,
            abilities: vec![
                ability(SensorAbilityName::Distance, SensorAbilityUnit::Centimeter),
                ability(SensorAbilityName::Battery, SensorAbilityUnit::Volt),
            ],
        }
    }

    fn sensor() -> SensorId {
        SensorId::new("eui-1").unwrap()
    }

    #[test]
    fn a_flat_payload_yields_a_distance_and_the_battery() {
        let ingest = DistanceSensorParser
            .parse(
                &distance_model(),
                &json!({"device": "eui-1", "distance_cm": 42.0, "battery": 3.6}),
                sensor(),
            )
            .unwrap();
        assert!(matches!(
            ingest.typed,
            SensorReadings::Distances(ref d)
                if d == &[DistanceReading { depth_cm: 0, centimeters: 42.0 }]
        ));
        assert_eq!(ingest.normalized.len(), 2);
    }

    #[test]
    fn the_battery_is_optional() {
        let ingest = DistanceSensorParser
            .parse(&distance_model(), &json!({"distance_cm": 42.0}), sensor())
            .unwrap();
        assert_eq!(ingest.normalized.len(), 1);
    }

    #[test]
    fn a_payload_without_distance_is_rejected() {
        assert!(
            DistanceSensorParser
                .parse(&distance_model(), &json!({"battery": 3.6}), sensor())
                .is_err()
        );
    }
}
