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
pub use values::{
    DefectStreak, JustWateredTtl, MapBounds, MapView, SensorOfflineAfter, WaterDemand, ZoomLevel,
};

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

    /// Every field this update actually moves. A field submitted with the
    /// value it already has produces nothing — the history records changes,
    /// not saves.
    pub fn changes(&self, current: &OrganizationSettings) -> Vec<SettingChange> {
        let next = self.apply_to(current);
        let mut changes = Vec::new();

        push_change(
            &mut changes,
            SettingKey::WaterDemand,
            current
                .water_demand
                .map(|v| SettingValue::Liters(v.liters())),
            next.water_demand.map(|v| SettingValue::Liters(v.liters())),
        );
        push_change(
            &mut changes,
            SettingKey::JustWateredTtl,
            current
                .just_watered_ttl
                .map(|v| SettingValue::Seconds(v.seconds())),
            next.just_watered_ttl
                .map(|v| SettingValue::Seconds(v.seconds())),
        );
        push_change(
            &mut changes,
            SettingKey::SensorOfflineAfter,
            current
                .sensor_offline_after
                .map(|v| SettingValue::Seconds(v.seconds())),
            next.sensor_offline_after
                .map(|v| SettingValue::Seconds(v.seconds())),
        );
        push_change(
            &mut changes,
            SettingKey::DefectStreak,
            current
                .defect_streak
                .map(|v| SettingValue::Count(v.count())),
            next.defect_streak.map(|v| SettingValue::Count(v.count())),
        );
        push_change(
            &mut changes,
            SettingKey::MapView,
            current.map_view.map(SettingValue::Viewport),
            next.map_view.map(SettingValue::Viewport),
        );
        push_change(
            &mut changes,
            SettingKey::DescendantsMayOverride,
            Some(SettingValue::Flag(current.descendants_may_override)),
            Some(SettingValue::Flag(next.descendants_may_override)),
        );

        changes
    }
}

/// One recorded value, in the unit it is stored in. A typed enum rather than a
/// free-form value, so the history cannot record something that is not a
/// setting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingValue {
    Liters(f64),
    Seconds(i64),
    Count(i32),
    Viewport(MapView),
    Flag(bool),
}

/// What changed about one setting. `None` on either side means the value was,
/// or becomes, inherited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingChange {
    pub key: SettingKey,
    pub previous: Option<SettingValue>,
    pub next: Option<SettingValue>,
}

fn push_change(
    out: &mut Vec<SettingChange>,
    key: SettingKey,
    previous: Option<SettingValue>,
    next: Option<SettingValue>,
) {
    if previous != next {
        out.push(SettingChange {
            key,
            previous,
            next,
        });
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

    #[test]
    fn only_touched_fields_produce_a_change() {
        let current = stored();
        let update = SettingsUpdate {
            water_demand: Patch::Set(WaterDemand::new(120.0).unwrap()),
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&current);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key, SettingKey::WaterDemand);
        assert_eq!(changes[0].previous, Some(SettingValue::Liters(70.0)));
        assert_eq!(changes[0].next, Some(SettingValue::Liters(120.0)));
    }

    #[test]
    fn returning_to_inheritance_is_a_change_with_no_next_value() {
        let update = SettingsUpdate {
            water_demand: Patch::Clear,
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&stored());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].previous, Some(SettingValue::Liters(70.0)));
        assert_eq!(changes[0].next, None);
    }

    #[test]
    fn setting_a_previously_inherited_field_has_no_previous_value() {
        let update = SettingsUpdate {
            defect_streak: Patch::Set(DefectStreak::new(5).unwrap()),
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&stored());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key, SettingKey::DefectStreak);
        assert_eq!(changes[0].previous, None);
        assert_eq!(changes[0].next, Some(SettingValue::Count(5)));
    }

    #[test]
    fn writing_the_same_value_again_produces_no_change() {
        let update = SettingsUpdate {
            water_demand: Patch::Set(WaterDemand::new(70.0).unwrap()),
            ..SettingsUpdate::default()
        };
        assert!(update.changes(&stored()).is_empty());
    }

    /// `changes` is hand-written and has no exhaustive match, so a seventh
    /// setting could change with no history entry at all. This fails the day
    /// one is added without a matching `push_change`.
    #[test]
    fn every_known_setting_can_produce_a_change() {
        use crate::shared::{coordinates::Coordinate, geo::BoundingBox};
        use std::collections::BTreeSet;

        let update = SettingsUpdate {
            water_demand: Patch::Set(WaterDemand::new(120.0).unwrap()),
            just_watered_ttl: Patch::Set(JustWateredTtl::new(7_200).unwrap()),
            sensor_offline_after: Patch::Set(SensorOfflineAfter::new(7_200).unwrap()),
            defect_streak: Patch::Set(DefectStreak::new(5).unwrap()),
            map_view: Patch::Set(
                MapView::new(
                    Coordinate::new(54.8, 9.4).unwrap(),
                    Some(
                        MapBounds::new(
                            BoundingBox::try_new(54.7, 9.2, 54.9, 9.6).unwrap(),
                            ZoomLevel::new(13).unwrap(),
                            ZoomLevel::new(18).unwrap(),
                        )
                        .unwrap(),
                    ),
                )
                .unwrap(),
            ),
            descendants_may_override: Some(false),
        };

        let moved: BTreeSet<SettingKey> = update
            .changes(&OrganizationSettings::empty(Id::new_v7()))
            .into_iter()
            .map(|change| change.key)
            .collect();

        assert_eq!(moved, BTreeSet::from(SettingKey::ALL));
    }

    #[test]
    fn flipping_the_lock_is_recorded() {
        let update = SettingsUpdate {
            descendants_may_override: Some(false),
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&stored());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key, SettingKey::DescendantsMayOverride);
        assert_eq!(changes[0].previous, Some(SettingValue::Flag(true)));
        assert_eq!(changes[0].next, Some(SettingValue::Flag(false)));
    }
}
