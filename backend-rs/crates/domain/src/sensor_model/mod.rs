//! Sensor model stamm-data — `SensorModel` and its allowed abilities.
//!
//! Models are seeded via migration; this module is read-only. Aggregates that
//! reference a model carry an `Id<SensorModel>`; the model itself is rehydrated
//! through `SensorModelReader`.

pub mod ability;
pub mod repository;

pub use ability::{
    SensorAbility, SensorAbilityName, SensorAbilityUnit, SensorModelAbility, UnknownAbility,
};
pub use repository::SensorModelReader;

use crate::Id;

crate::newtype_nonempty! {
    /// Display name of a sensor model; matches `sensor_models.name` in the DB.
    SensorModelName, "sensor_model.name", 1, 64
}

#[derive(Debug, Clone)]
pub struct SensorModel {
    pub id: Id<SensorModel>,
    pub name: SensorModelName,
    pub description: Option<String>,
    pub abilities: Vec<SensorModelAbility>,
}

impl SensorModel {
    pub fn ability_id_for(&self, name: SensorAbilityName, depth_cm: i32) -> Option<i32> {
        self.abilities
            .iter()
            .find(|a| a.ability.name == name && a.depth_cm == depth_cm)
            .map(|a| a.id)
    }

    pub fn depths_for(&self, name: SensorAbilityName) -> Vec<i32> {
        self.abilities
            .iter()
            .filter(|a| a.ability.name == name)
            .map(|a| a.depth_cm)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eco_drizzler() -> SensorModel {
        let st = |id, depth_cm| SensorModelAbility {
            id,
            ability: SensorAbility {
                id: 1,
                name: SensorAbilityName::SoilTension,
                unit: SensorAbilityUnit::Centibar,
            },
            depth_cm,
        };
        let moisture_15 = SensorModelAbility {
            id: 4,
            ability: SensorAbility {
                id: 2,
                name: SensorAbilityName::SoilMoisture,
                unit: SensorAbilityUnit::Percent,
            },
            depth_cm: 15,
        };
        SensorModel {
            id: Id::new(1),
            name: SensorModelName::new("EcoDrizzler").unwrap(),
            description: None,
            abilities: vec![st(1, 30), st(2, 60), st(3, 90), moisture_15],
        }
    }

    #[test]
    fn ability_id_for_known_returns_some() {
        let m = eco_drizzler();
        assert_eq!(
            m.ability_id_for(SensorAbilityName::SoilTension, 60),
            Some(2)
        );
        assert_eq!(
            m.ability_id_for(SensorAbilityName::SoilMoisture, 15),
            Some(4)
        );
    }

    #[test]
    fn ability_id_for_unknown_returns_none() {
        let m = eco_drizzler();
        assert_eq!(m.ability_id_for(SensorAbilityName::SoilTension, 99), None);
        assert_eq!(m.ability_id_for(SensorAbilityName::Humidity, 15), None);
    }

    #[test]
    fn depths_for_returns_all_matching() {
        let m = eco_drizzler();
        let mut depths = m.depths_for(SensorAbilityName::SoilTension);
        depths.sort();
        assert_eq!(depths, vec![30, 60, 90]);
    }
}
