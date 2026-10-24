use crate::{
    Id,
    organization::Organization,
    settings::{
        EffectiveSettings, InheritLevel, Inherited, InstanceDefaults, OrganizationSettings, inherit,
    },
};

/// Where a resolved value came from. `Inherited` names the organization that
/// actually set it, which is what the interface needs to say "from X".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingOrigin {
    Default,
    Inherited(Id<Organization>),
    Own,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingOrigins {
    pub sensor_offline_after: SettingOrigin,
    pub defect_streak: SettingOrigin,
    pub map_view: SettingOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub effective: EffectiveSettings,
    pub origins: SettingOrigins,
}

/// Folds the chain from the root down onto the instance defaults. `chain` must
/// be ordered root first, target last.
pub fn resolve(
    defaults: &InstanceDefaults,
    chain: &[OrganizationSettings],
    target: Id<Organization>,
) -> Resolution {
    let sensor_offline_after = field(defaults.sensor_offline_after, chain, target, |l| {
        l.sensor_offline_after
    });
    let defect_streak = field(defaults.defect_streak, chain, target, |l| l.defect_streak);
    let map_view = field(defaults.map_view, chain, target, |l| l.map_view);
    Resolution {
        effective: EffectiveSettings {
            sensor_offline_after: sensor_offline_after.value,
            defect_streak: defect_streak.value,
            map_view: map_view.value,
            // The lock does not depend on the field, so any of them carries it.
            enforced_by: map_view.enforced_by,
        },
        origins: SettingOrigins {
            sensor_offline_after: sensor_offline_after.origin,
            defect_streak: defect_streak.origin,
            map_view: map_view.origin,
        },
    }
}

fn field<T: Copy>(
    default: T,
    chain: &[OrganizationSettings],
    target: Id<Organization>,
    pick: impl Fn(&OrganizationSettings) -> Option<T>,
) -> Inherited<T> {
    let levels: Vec<InheritLevel<T>> = chain
        .iter()
        .map(|l| InheritLevel {
            organization_id: l.organization_id,
            value: pick(l),
            descendants_may_override: l.descendants_may_override,
        })
        .collect();
    inherit(default, &levels, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::values::{
        DefectStreak, MapBounds, MapView, SensorOfflineAfter, ZoomLevel,
    };
    use crate::settings::{InstanceDefaults, OrganizationSettings};
    use crate::shared::{coordinates::Coordinate, geo::BoundingBox};

    fn defaults() -> InstanceDefaults {
        InstanceDefaults {
            sensor_offline_after: SensorOfflineAfter::new(86_400).unwrap(),
            defect_streak: DefectStreak::new(3).unwrap(),
            map_view: MapView::new(
                Coordinate::new(54.79, 9.43).unwrap(),
                Some(
                    MapBounds::new(
                        BoundingBox::try_new(54.71, 9.28, 54.86, 9.58).unwrap(),
                        ZoomLevel::new(13).unwrap(),
                        ZoomLevel::new(18).unwrap(),
                    )
                    .unwrap(),
                ),
            )
            .unwrap(),
        }
    }

    fn own(org: Id<Organization>, secs: Option<i64>) -> OrganizationSettings {
        OrganizationSettings {
            organization_id: org,
            sensor_offline_after: secs.map(|s| SensorOfflineAfter::new(s).unwrap()),
            defect_streak: None,
            map_view: None,
            descendants_may_override: true,
        }
    }

    #[test]
    fn case_a_nothing_set_anywhere_yields_the_instance_default() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [own(root, None), own(mid, None), own(leaf, None)];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 86_400);
        assert_eq!(r.origins.sensor_offline_after, SettingOrigin::Default);
    }

    #[test]
    fn case_b_a_value_at_the_root_is_inherited_all_the_way_down() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [own(root, Some(100_000)), own(mid, None), own(leaf, None)];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 100_000);
        assert_eq!(
            r.origins.sensor_offline_after,
            SettingOrigin::Inherited(root)
        );
    }

    #[test]
    fn case_c_the_nearest_ancestor_with_a_value_wins() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            own(root, Some(100_000)),
            own(mid, Some(90_000)),
            own(leaf, None),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 90_000);
        assert_eq!(
            r.origins.sensor_offline_after,
            SettingOrigin::Inherited(mid)
        );
    }

    #[test]
    fn case_d_an_own_value_beats_every_inherited_one() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            own(root, Some(100_000)),
            own(mid, Some(90_000)),
            own(leaf, Some(70_000)),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 70_000);
        assert_eq!(r.origins.sensor_offline_after, SettingOrigin::Own);
        assert_eq!(r.effective.enforced_by, None);
    }

    #[test]
    fn fields_resolve_independently_of_one_another() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let mut root_settings = own(root, Some(100_000));
        root_settings.defect_streak = Some(DefectStreak::new(7).unwrap());
        let chain = [root_settings, own(leaf, Some(70_000))];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 70_000);
        assert_eq!(r.origins.sensor_offline_after, SettingOrigin::Own);
        assert_eq!(r.effective.defect_streak.count(), 7);
        assert_eq!(r.origins.defect_streak, SettingOrigin::Inherited(root));
        assert_eq!(r.origins.map_view, SettingOrigin::Default);
    }

    fn locked(org: Id<Organization>, secs: i64) -> OrganizationSettings {
        OrganizationSettings {
            descendants_may_override: false,
            ..own(org, Some(secs))
        }
    }

    #[test]
    fn case_e_a_lock_at_the_root_freezes_the_whole_subtree() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            locked(root, 100_000),
            own(mid, Some(90_000)),
            own(leaf, Some(70_000)),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 100_000);
        assert_eq!(r.effective.enforced_by, Some(root));
        assert_eq!(
            r.origins.sensor_offline_after,
            SettingOrigin::Inherited(root)
        );
    }

    #[test]
    fn case_f_a_lock_in_the_middle_freezes_what_is_below_it() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            own(root, Some(100_000)),
            locked(mid, 90_000),
            own(leaf, Some(70_000)),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 90_000);
        assert_eq!(r.effective.enforced_by, Some(mid));
    }

    #[test]
    fn the_topmost_lock_wins_over_a_lower_one() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            locked(root, 100_000),
            locked(mid, 90_000),
            own(leaf, Some(70_000)),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 100_000);
        assert_eq!(r.effective.enforced_by, Some(root));
    }

    #[test]
    fn an_organizations_own_lock_does_not_freeze_itself() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let chain = [own(root, Some(100_000)), locked(leaf, 70_000)];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.sensor_offline_after.seconds(), 70_000);
        assert_eq!(r.effective.enforced_by, None);
        assert_eq!(r.origins.sensor_offline_after, SettingOrigin::Own);
    }
}
