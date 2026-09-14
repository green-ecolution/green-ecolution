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
pub use repository::{SettingsReader, SettingsResolver, SettingsWriter};
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

/// Three states a submitted field can carry. `Option<T>` only has two, and
/// without the third every partial save would reset every field the caller
/// did not send back to inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Patch<T> {
    #[default]
    Unchanged,
    Clear,
    Set(T),
}

impl<T: Copy> Patch<T> {
    /// The value this patch leaves behind, given what is stored today.
    pub fn resolve(self, current: Option<T>) -> Option<T> {
        match self {
            Patch::Unchanged => current,
            Patch::Clear => None,
            Patch::Set(value) => Some(value),
        }
    }

    pub fn touches(self) -> bool {
        !matches!(self, Patch::Unchanged)
    }
}

/// One submitted change to an organization's own values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SettingsUpdate {
    pub water_demand: Patch<WaterDemand>,
    pub just_watered_ttl: Patch<JustWateredTtl>,
    pub sensor_offline_after: Patch<SensorOfflineAfter>,
    pub defect_streak: Patch<DefectStreak>,
    pub map_view: Patch<MapView>,
    pub descendants_may_override: Option<bool>,
}

impl SettingsUpdate {
    pub fn is_empty(&self) -> bool {
        !self.water_demand.touches()
            && !self.just_watered_ttl.touches()
            && !self.sensor_offline_after.touches()
            && !self.defect_streak.touches()
            && !self.map_view.touches()
            && self.descendants_may_override.is_none()
    }

    /// The values as they would stand afterwards.
    pub fn apply_to(&self, current: &OrganizationSettings) -> OrganizationSettings {
        OrganizationSettings {
            organization_id: current.organization_id,
            water_demand: self.water_demand.resolve(current.water_demand),
            just_watered_ttl: self.just_watered_ttl.resolve(current.just_watered_ttl),
            sensor_offline_after: self
                .sensor_offline_after
                .resolve(current.sensor_offline_after),
            defect_streak: self.defect_streak.resolve(current.defect_streak),
            map_view: self.map_view.resolve(current.map_view),
            descendants_may_override: self
                .descendants_may_override
                .unwrap_or(current.descendants_may_override),
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

    fn stored() -> OrganizationSettings {
        OrganizationSettings {
            water_demand: Some(WaterDemand::new(70.0).unwrap()),
            ..OrganizationSettings::empty(Id::new_v7())
        }
    }

    #[test]
    fn an_unchanged_patch_leaves_the_stored_value_alone() {
        let next = SettingsUpdate::default().apply_to(&stored());
        assert_eq!(next.water_demand, Some(WaterDemand::new(70.0).unwrap()));
    }

    #[test]
    fn clearing_a_field_returns_it_to_inheritance() {
        let update = SettingsUpdate {
            water_demand: Patch::Clear,
            ..SettingsUpdate::default()
        };
        assert_eq!(update.apply_to(&stored()).water_demand, None);
    }

    #[test]
    fn setting_a_field_replaces_the_stored_value() {
        let update = SettingsUpdate {
            water_demand: Patch::Set(WaterDemand::new(120.0).unwrap()),
            ..SettingsUpdate::default()
        };
        assert_eq!(
            update.apply_to(&stored()).water_demand,
            Some(WaterDemand::new(120.0).unwrap())
        );
    }

    #[test]
    fn an_update_that_touches_nothing_is_empty() {
        assert!(SettingsUpdate::default().is_empty());
        assert!(
            !SettingsUpdate {
                water_demand: Patch::Clear,
                ..SettingsUpdate::default()
            }
            .is_empty()
        );
    }
}
