use crate::{
    Id,
    organization::Organization,
    settings::{EffectiveSettings, InstanceDefaults, OrganizationSettings},
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
    pub water_demand: SettingOrigin,
    pub just_watered_ttl: SettingOrigin,
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
    let mut effective = EffectiveSettings {
        water_demand: defaults.water_demand,
        just_watered_ttl: defaults.just_watered_ttl,
        sensor_offline_after: defaults.sensor_offline_after,
        defect_streak: defaults.defect_streak,
        map_view: defaults.map_view,
        enforced_by: None,
    };
    let mut origins = SettingOrigins {
        water_demand: SettingOrigin::Default,
        just_watered_ttl: SettingOrigin::Default,
        sensor_offline_after: SettingOrigin::Default,
        defect_streak: SettingOrigin::Default,
        map_view: SettingOrigin::Default,
    };

    for level in chain {
        let origin = if level.organization_id == target {
            SettingOrigin::Own
        } else {
            SettingOrigin::Inherited(level.organization_id)
        };
        apply(
            &mut effective.water_demand,
            &mut origins.water_demand,
            level.water_demand,
            origin,
        );
        apply(
            &mut effective.just_watered_ttl,
            &mut origins.just_watered_ttl,
            level.just_watered_ttl,
            origin,
        );
        apply(
            &mut effective.sensor_offline_after,
            &mut origins.sensor_offline_after,
            level.sensor_offline_after,
            origin,
        );
        apply(
            &mut effective.defect_streak,
            &mut origins.defect_streak,
            level.defect_streak,
            origin,
        );
        apply(
            &mut effective.map_view,
            &mut origins.map_view,
            level.map_view,
            origin,
        );
    }

    Resolution { effective, origins }
}

fn apply<T: Copy>(
    slot: &mut T,
    origin_slot: &mut SettingOrigin,
    value: Option<T>,
    origin: SettingOrigin,
) {
    if let Some(value) = value {
        *slot = value;
        *origin_slot = origin;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::values::{
        DefectStreak, JustWateredTtl, MapView, SensorOfflineAfter, WaterDemand,
    };
    use crate::settings::{InstanceDefaults, OrganizationSettings};
    use crate::shared::{coordinates::Coordinate, geo::BoundingBox};

    fn defaults() -> InstanceDefaults {
        InstanceDefaults {
            water_demand: WaterDemand::new(80.0).unwrap(),
            just_watered_ttl: JustWateredTtl::new(86_400).unwrap(),
            sensor_offline_after: SensorOfflineAfter::new(86_400).unwrap(),
            defect_streak: DefectStreak::new(3).unwrap(),
            map_view: MapView::new(
                Coordinate::new(54.79, 9.43).unwrap(),
                BoundingBox::try_new(54.71, 9.28, 54.86, 9.58).unwrap(),
            )
            .unwrap(),
        }
    }

    fn own(org: Id<Organization>, liters: Option<f64>) -> OrganizationSettings {
        OrganizationSettings {
            organization_id: org,
            water_demand: liters.map(|l| WaterDemand::new(l).unwrap()),
            just_watered_ttl: None,
            sensor_offline_after: None,
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
        assert_eq!(r.effective.water_demand.liters(), 80.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Default);
    }

    #[test]
    fn case_b_a_value_at_the_root_is_inherited_all_the_way_down() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [own(root, Some(100.0)), own(mid, None), own(leaf, None)];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.water_demand.liters(), 100.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Inherited(root));
    }

    #[test]
    fn case_c_the_nearest_ancestor_with_a_value_wins() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            own(root, Some(100.0)),
            own(mid, Some(90.0)),
            own(leaf, None),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.water_demand.liters(), 90.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Inherited(mid));
    }

    #[test]
    fn case_d_an_own_value_beats_every_inherited_one() {
        let (root, mid, leaf) = (Id::new_v7(), Id::new_v7(), Id::new_v7());
        let chain = [
            own(root, Some(100.0)),
            own(mid, Some(90.0)),
            own(leaf, Some(70.0)),
        ];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.water_demand.liters(), 70.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Own);
        assert_eq!(r.effective.enforced_by, None);
    }

    #[test]
    fn fields_resolve_independently_of_one_another() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let mut root_settings = own(root, Some(100.0));
        root_settings.defect_streak = Some(DefectStreak::new(7).unwrap());
        let chain = [root_settings, own(leaf, Some(70.0))];
        let r = resolve(&defaults(), &chain, leaf);
        assert_eq!(r.effective.water_demand.liters(), 70.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Own);
        assert_eq!(r.effective.defect_streak.count(), 7);
        assert_eq!(r.origins.defect_streak, SettingOrigin::Inherited(root));
        assert_eq!(r.origins.map_view, SettingOrigin::Default);
    }
}
