//! Irrigation's own settings. They inherit down the organization tree like the
//! core's and obey the same lock, which stays a core setting.

pub mod http;
pub mod repository;
pub mod service;

use domain::{
    Id, RepositoryError,
    organization::Organization,
    settings::{InheritLevel, JustWateredTtl, Patch, SettingOrigin, WaterDemand, inherit},
    shared::error::ValidationError,
};

use crate::configuration::Settings;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrrigationSettings {
    pub organization_id: Id<Organization>,
    pub water_demand: Option<WaterDemand>,
    pub just_watered_ttl: Option<JustWateredTtl>,
}

impl IrrigationSettings {
    pub fn empty(organization_id: Id<Organization>) -> Self {
        Self {
            organization_id,
            water_demand: None,
            just_watered_ttl: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrrigationDefaults {
    pub water_demand: WaterDemand,
    pub just_watered_ttl: JustWateredTtl,
}

impl IrrigationDefaults {
    pub fn from_config(config: &Settings) -> Result<Self, ValidationError> {
        Ok(Self {
            water_demand: WaterDemand::new(config.routing.tree_demand_liters)?,
            just_watered_ttl: JustWateredTtl::new(
                i64::try_from(config.watering.just_watered_ttl_secs).unwrap_or(i64::MAX),
            )?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveIrrigationSettings {
    pub water_demand: WaterDemand,
    pub just_watered_ttl: JustWateredTtl,
    pub enforced_by: Option<Id<Organization>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrrigationOrigins {
    pub water_demand: SettingOrigin,
    pub just_watered_ttl: SettingOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrrigationResolution {
    pub effective: EffectiveIrrigationSettings,
    pub origins: IrrigationOrigins,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IrrigationSettingsUpdate {
    pub water_demand: Patch<WaterDemand>,
    pub just_watered_ttl: Patch<JustWateredTtl>,
}

impl IrrigationSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        !self.water_demand.touches() && !self.just_watered_ttl.touches()
    }
}

/// One organization's level: its own irrigation values plus the core lock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrrigationLevel {
    pub settings: IrrigationSettings,
    pub descendants_may_override: bool,
}

/// `chain` runs root first, target last.
pub fn resolve(
    defaults: &IrrigationDefaults,
    chain: &[IrrigationLevel],
    target: Id<Organization>,
) -> IrrigationResolution {
    fn levels<T: Copy>(
        chain: &[IrrigationLevel],
        pick: impl Fn(&IrrigationSettings) -> Option<T>,
    ) -> Vec<InheritLevel<T>> {
        chain
            .iter()
            .map(|l| InheritLevel {
                organization_id: l.settings.organization_id,
                value: pick(&l.settings),
                descendants_may_override: l.descendants_may_override,
            })
            .collect()
    }
    let water_demand = inherit(
        defaults.water_demand,
        &levels(chain, |s| s.water_demand),
        target,
    );
    let just_watered_ttl = inherit(
        defaults.just_watered_ttl,
        &levels(chain, |s| s.just_watered_ttl),
        target,
    );
    IrrigationResolution {
        effective: EffectiveIrrigationSettings {
            water_demand: water_demand.value,
            just_watered_ttl: just_watered_ttl.value,
            enforced_by: water_demand.enforced_by,
        },
        origins: IrrigationOrigins {
            water_demand: water_demand.origin,
            just_watered_ttl: just_watered_ttl.origin,
        },
    }
}

#[async_trait::async_trait]
pub trait IrrigationSettingsResolver: Send + Sync {
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveIrrigationSettings, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> IrrigationDefaults {
        IrrigationDefaults {
            water_demand: WaterDemand::new(80.0).unwrap(),
            just_watered_ttl: JustWateredTtl::new(86_400).unwrap(),
        }
    }

    fn level(org: Id<Organization>, demand: Option<f64>, may_override: bool) -> IrrigationLevel {
        IrrigationLevel {
            settings: IrrigationSettings {
                organization_id: org,
                water_demand: demand.map(|d| WaterDemand::new(d).unwrap()),
                just_watered_ttl: None,
            },
            descendants_may_override: may_override,
        }
    }

    #[test]
    fn defaults_apply_without_values() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, None, true), level(leaf, None, true)],
            leaf,
        );
        assert_eq!(r.effective.water_demand.liters(), 80.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Default);
    }

    #[test]
    fn inherited_value_names_its_source() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, Some(50.0), true), level(leaf, None, true)],
            leaf,
        );
        assert_eq!(r.effective.water_demand.liters(), 50.0);
        assert_eq!(r.origins.water_demand, SettingOrigin::Inherited(root));
    }

    #[test]
    fn lock_above_freezes_the_value() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[
                level(root, Some(50.0), false),
                level(leaf, Some(90.0), true),
            ],
            leaf,
        );
        assert_eq!(r.effective.water_demand.liters(), 50.0);
        assert_eq!(r.effective.enforced_by, Some(root));
    }
}
