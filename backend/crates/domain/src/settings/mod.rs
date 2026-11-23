//! Operational defaults an organization sets for itself.
//!
//! Three levels resolve onto one another: the instance default at the bottom,
//! the organizations along the organization tree above it, each overriding
//! what it sets itself, and (later) the user level for preferences only. A
//! field that is not set means "inherit"; that is why every value field is an
//! `Option`.

pub mod inherit;
pub mod repository;
pub mod resolve;
pub mod snapshot;
pub mod values;

use crate::{Id, organization::Organization};

pub use inherit::{InheritLevel, Inherited, SettingOrigin, inherit};
pub use repository::{SettingsReader, SettingsResolver, SettingsWriter};
pub use resolve::{Resolution, resolve};
#[doc(hidden)]
pub use snapshot::OrganizationSettingsSnapshot;
pub use values::{DefectStreak, MapBounds, MapView, SensorOfflineAfter, ZoomLevel};

/// What one organization set for itself. `None` means inherit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrganizationSettings {
    pub organization_id: Id<Organization>,
    pub sensor_offline_after: Option<SensorOfflineAfter>,
    pub defect_streak: Option<DefectStreak>,
    pub map_view: Option<MapView>,
}

impl OrganizationSettings {
    /// An organization that has never set anything: everything inherits.
    pub fn empty(organization_id: Id<Organization>) -> Self {
        Self {
            organization_id,
            sensor_offline_after: None,
            defect_streak: None,
            map_view: None,
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

impl<T: Copy + PartialEq> Patch<T> {
    /// A value equal to what the organization would inherit is no own value:
    /// it follows its parent again.
    pub fn settled(self, inherited: T) -> Self {
        match self {
            Patch::Set(value) if value == inherited => Patch::Clear,
            other => other,
        }
    }
}

/// One submitted change to an organization's own values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SettingsUpdate {
    pub sensor_offline_after: Patch<SensorOfflineAfter>,
    pub defect_streak: Patch<DefectStreak>,
    pub map_view: Patch<MapView>,
}

impl SettingsUpdate {
    pub fn is_empty(&self) -> bool {
        !self.sensor_offline_after.touches()
            && !self.defect_streak.touches()
            && !self.map_view.touches()
    }

    /// The values as they would stand afterwards.
    pub fn apply_to(&self, current: &OrganizationSettings) -> OrganizationSettings {
        OrganizationSettings {
            organization_id: current.organization_id,
            sensor_offline_after: self
                .sensor_offline_after
                .resolve(current.sensor_offline_after),
            defect_streak: self.defect_streak.resolve(current.defect_streak),
            map_view: self.map_view.resolve(current.map_view),
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

        changes
    }

    /// Applies the "same value follows again" rule to every field.
    pub fn settled(self, inherited: &EffectiveSettings) -> Self {
        Self {
            sensor_offline_after: self
                .sensor_offline_after
                .settled(inherited.sensor_offline_after),
            defect_streak: self.defect_streak.settled(inherited.defect_streak),
            map_view: self.map_view.settled(inherited.map_view),
        }
    }
}

/// One recorded value, in the unit it is stored in. A typed enum rather than a
/// free-form value, so the history cannot record something that is not a
/// setting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingValue {
    Seconds(i64),
    Count(i32),
    Viewport(MapView),
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
    pub sensor_offline_after: SensorOfflineAfter,
    pub defect_streak: DefectStreak,
    pub map_view: MapView,
}

/// The resolved values a request works with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveSettings {
    pub sensor_offline_after: SensorOfflineAfter,
    pub defect_streak: DefectStreak,
    pub map_view: MapView,
}

/// Names one settable thing. History entries and error messages need a word
/// for "which value", and an enum keeps it in one place instead of spreading
/// string literals across three layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingKey {
    SensorOfflineAfter,
    DefectStreak,
    MapView,
}

impl SettingKey {
    pub const ALL: [SettingKey; 3] = [
        SettingKey::SensorOfflineAfter,
        SettingKey::DefectStreak,
        SettingKey::MapView,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SettingKey::SensorOfflineAfter => "sensor_offline_after",
            SettingKey::DefectStreak => "defect_streak",
            SettingKey::MapView => "map_view",
        }
    }
}

impl std::fmt::Display for SettingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored() -> OrganizationSettings {
        OrganizationSettings {
            sensor_offline_after: Some(SensorOfflineAfter::new(7_200).unwrap()),
            ..OrganizationSettings::empty(Id::new_v7())
        }
    }

    #[test]
    fn an_unchanged_patch_leaves_the_stored_value_alone() {
        let next = SettingsUpdate::default().apply_to(&stored());
        assert_eq!(
            next.sensor_offline_after,
            Some(SensorOfflineAfter::new(7_200).unwrap())
        );
    }

    #[test]
    fn clearing_a_field_returns_it_to_inheritance() {
        let update = SettingsUpdate {
            sensor_offline_after: Patch::Clear,
            ..SettingsUpdate::default()
        };
        assert_eq!(update.apply_to(&stored()).sensor_offline_after, None);
    }

    #[test]
    fn setting_a_field_replaces_the_stored_value() {
        let update = SettingsUpdate {
            sensor_offline_after: Patch::Set(SensorOfflineAfter::new(14_400).unwrap()),
            ..SettingsUpdate::default()
        };
        assert_eq!(
            update.apply_to(&stored()).sensor_offline_after,
            Some(SensorOfflineAfter::new(14_400).unwrap())
        );
    }

    #[test]
    fn an_update_that_touches_nothing_is_empty() {
        assert!(SettingsUpdate::default().is_empty());
        assert!(
            !SettingsUpdate {
                sensor_offline_after: Patch::Clear,
                ..SettingsUpdate::default()
            }
            .is_empty()
        );
    }

    #[test]
    fn only_touched_fields_produce_a_change() {
        let current = stored();
        let update = SettingsUpdate {
            sensor_offline_after: Patch::Set(SensorOfflineAfter::new(14_400).unwrap()),
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&current);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key, SettingKey::SensorOfflineAfter);
        assert_eq!(changes[0].previous, Some(SettingValue::Seconds(7_200)));
        assert_eq!(changes[0].next, Some(SettingValue::Seconds(14_400)));
    }

    #[test]
    fn returning_to_inheritance_is_a_change_with_no_next_value() {
        let update = SettingsUpdate {
            sensor_offline_after: Patch::Clear,
            ..SettingsUpdate::default()
        };
        let changes = update.changes(&stored());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].previous, Some(SettingValue::Seconds(7_200)));
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
            sensor_offline_after: Patch::Set(SensorOfflineAfter::new(7_200).unwrap()),
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
        };

        let moved: BTreeSet<SettingKey> = update
            .changes(&OrganizationSettings::empty(Id::new_v7()))
            .into_iter()
            .map(|change| change.key)
            .collect();

        assert_eq!(moved, BTreeSet::from(SettingKey::ALL));
    }

    #[test]
    fn a_value_equal_to_the_inherited_one_gives_up_the_own_value() {
        let inherited = SensorOfflineAfter::new(86_400).unwrap();
        assert_eq!(Patch::Set(inherited).settled(inherited), Patch::Clear);
    }

    #[test]
    fn a_different_value_stays_set_and_the_other_states_pass_through() {
        let inherited = SensorOfflineAfter::new(86_400).unwrap();
        let other = SensorOfflineAfter::new(90_000).unwrap();
        assert_eq!(Patch::Set(other).settled(inherited), Patch::Set(other));
        assert_eq!(
            Patch::<SensorOfflineAfter>::Unchanged.settled(inherited),
            Patch::Unchanged
        );
        assert_eq!(
            Patch::<SensorOfflineAfter>::Clear.settled(inherited),
            Patch::Clear
        );
    }

    #[test]
    fn an_update_settles_each_field_against_its_own_inherited_value() {
        use crate::shared::coordinates::Coordinate;

        let inherited = EffectiveSettings {
            sensor_offline_after: SensorOfflineAfter::new(86_400).unwrap(),
            defect_streak: DefectStreak::new(3).unwrap(),
            map_view: MapView::new(Coordinate::new(54.79, 9.43).unwrap(), None).unwrap(),
        };
        let update = SettingsUpdate {
            sensor_offline_after: Patch::Set(SensorOfflineAfter::new(86_400).unwrap()),
            defect_streak: Patch::Set(DefectStreak::new(5).unwrap()),
            map_view: Patch::Unchanged,
        }
        .settled(&inherited);
        assert_eq!(update.sensor_offline_after, Patch::Clear);
        assert_eq!(
            update.defect_streak,
            Patch::Set(DefectStreak::new(5).unwrap())
        );
        assert_eq!(update.map_view, Patch::Unchanged);
    }
}
