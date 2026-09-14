//! Operational defaults an organization sets for itself.
//!
//! Three levels resolve onto one another: the instance default at the bottom,
//! the organizations along the tree above it, each overriding what it sets
//! itself, and (later) the user level for preferences only. A field that is
//! not set means "inherit"; that is why every value field is an `Option`.
//!
//! `descendants_may_override` is deliberately not an `Option`: it does not
//! inherit. It is a statement about one's own descendants, not a value taken
//! from above. Where no settings exist at all, it reads as "allowed".

pub mod history;
pub mod repository;
pub mod resolve;
pub mod snapshot;
pub mod values;

use std::str::FromStr;

use crate::{Id, organization::Organization, shared::error::ValidationError};

pub use history::SettingChangeEntry;
pub use repository::{SettingsReader, SettingsResolver};
pub use resolve::{Resolution, SettingOrigin, SettingOrigins, resolve};
#[doc(hidden)]
pub use snapshot::OrganizationSettingsSnapshot;
pub use values::{DefectStreak, JustWateredTtl, MapView, SensorOfflineAfter, WaterDemand};

/// What one organization set for itself. `None` means inherit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrganizationSettings {
    pub organization_id: Id<Organization>,
    pub water_demand: Option<WaterDemand>,
    pub just_watered_ttl: Option<JustWateredTtl>,
    pub sensor_offline_after: Option<SensorOfflineAfter>,
    pub defect_streak: Option<DefectStreak>,
    pub map_view: Option<MapView>,
    pub descendants_may_override: bool,
}

impl OrganizationSettings {
    /// An organization that has never set anything: everything inherits and
    /// descendants may override.
    pub fn empty(organization_id: Id<Organization>) -> Self {
        Self {
            organization_id,
            water_demand: None,
            just_watered_ttl: None,
            sensor_offline_after: None,
            defect_streak: None,
            map_view: None,
            descendants_may_override: true,
        }
    }
}

/// The floor every resolution starts from, filled from the instance's own
/// configuration at startup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceDefaults {
    pub water_demand: WaterDemand,
    pub just_watered_ttl: JustWateredTtl,
    pub sensor_offline_after: SensorOfflineAfter,
    pub defect_streak: DefectStreak,
    pub map_view: MapView,
}

/// The resolved values a request works with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveSettings {
    pub water_demand: WaterDemand,
    pub just_watered_ttl: JustWateredTtl,
    pub sensor_offline_after: SensorOfflineAfter,
    pub defect_streak: DefectStreak,
    pub map_view: MapView,
    pub enforced_by: Option<Id<Organization>>,
}

/// Names one settable thing. History entries and error messages need a word
/// for "which value", and an enum keeps it in one place instead of spreading
/// string literals across three layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingKey {
    WaterDemand,
    JustWateredTtl,
    SensorOfflineAfter,
    DefectStreak,
    MapView,
    DescendantsMayOverride,
}

impl SettingKey {
    pub const ALL: [SettingKey; 6] = [
        SettingKey::WaterDemand,
        SettingKey::JustWateredTtl,
        SettingKey::SensorOfflineAfter,
        SettingKey::DefectStreak,
        SettingKey::MapView,
        SettingKey::DescendantsMayOverride,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SettingKey::WaterDemand => "water_demand",
            SettingKey::JustWateredTtl => "just_watered_ttl",
            SettingKey::SensorOfflineAfter => "sensor_offline_after",
            SettingKey::DefectStreak => "defect_streak",
            SettingKey::MapView => "map_view",
            SettingKey::DescendantsMayOverride => "descendants_may_override",
        }
    }
}

impl std::fmt::Display for SettingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SettingKey {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SettingKey::ALL
            .into_iter()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "settings.key",
                reason: format!("unknown setting '{s}'"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips() {
        for key in SettingKey::ALL {
            assert_eq!(SettingKey::from_str(key.as_str()).unwrap(), key);
        }
    }

    #[test]
    fn an_unknown_key_is_rejected_with_a_field_label() {
        let err = SettingKey::from_str("moisture").unwrap_err();
        assert!(matches!(
            err,
            ValidationError::InvalidFormat {
                field: "settings.key",
                ..
            }
        ));
    }
}
