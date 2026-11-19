//! Irrigation's own settings. They inherit down the organization tree like the core's.

pub mod http;
pub mod repository;
pub mod service;

use domain::{
    Id, RepositoryError,
    organization::Organization,
    settings::{InheritLevel, Patch, inherit},
    shared::error::ValidationError,
};
use irrigation_domain::settings::{JustWateredTtl, WaterDemand};

use super::IrrigationConfig;

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
    pub fn from_config(config: &IrrigationConfig) -> Result<Self, ValidationError> {
        Ok(Self {
            water_demand: WaterDemand::new(config.tree_demand_liters)?,
            just_watered_ttl: JustWateredTtl::new(
                i64::try_from(config.just_watered_ttl_secs).unwrap_or(i64::MAX),
            )?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveIrrigationSettings {
    pub water_demand: WaterDemand,
    pub just_watered_ttl: JustWateredTtl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrrigationResolution {
    pub effective: EffectiveIrrigationSettings,
    /// The values in force without the organization's own ones.
    pub inherited: EffectiveIrrigationSettings,
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

    /// A value equal to the inherited one is no own value, as in the core.
    pub fn settled(self, inherited: &EffectiveIrrigationSettings) -> Self {
        Self {
            water_demand: self.water_demand.settled(inherited.water_demand),
            just_watered_ttl: self.just_watered_ttl.settled(inherited.just_watered_ttl),
        }
    }
}

/// `chain` runs root first, target last.
pub fn resolve(
    defaults: &IrrigationDefaults,
    chain: &[IrrigationSettings],
    target: Id<Organization>,
) -> IrrigationResolution {
    fn levels<T: Copy>(
        chain: &[IrrigationSettings],
        pick: impl Fn(&IrrigationSettings) -> Option<T>,
    ) -> Vec<InheritLevel<T>> {
        chain
            .iter()
            .map(|s| InheritLevel {
                organization_id: s.organization_id,
                value: pick(s),
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
        },
        inherited: EffectiveIrrigationSettings {
            water_demand: water_demand.inherited,
            just_watered_ttl: just_watered_ttl.inherited,
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

    fn level(org: Id<Organization>, demand: Option<f64>) -> IrrigationSettings {
        IrrigationSettings {
            organization_id: org,
            water_demand: demand.map(|d| WaterDemand::new(d).unwrap()),
            just_watered_ttl: None,
        }
    }

    #[test]
    fn defaults_apply_without_values() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(&defaults(), &[level(root, None), level(leaf, None)], leaf);
        assert_eq!(r.effective.water_demand.liters(), 80.0);
        assert_eq!(r.inherited.water_demand.liters(), 80.0);
    }

    #[test]
    fn inherited_value_comes_from_the_nearest_ancestor() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, Some(50.0)), level(leaf, None)],
            leaf,
        );
        assert_eq!(r.effective.water_demand.liters(), 50.0);
        assert_eq!(r.inherited.water_demand.liters(), 50.0);
    }

    #[test]
    fn a_value_equal_to_the_inherited_one_is_dropped() {
        let inherited = EffectiveIrrigationSettings {
            water_demand: WaterDemand::new(80.0).unwrap(),
            just_watered_ttl: JustWateredTtl::new(86_400).unwrap(),
        };
        let update = IrrigationSettingsUpdate {
            water_demand: Patch::Set(WaterDemand::new(80.0).unwrap()),
            just_watered_ttl: Patch::Set(JustWateredTtl::new(43_200).unwrap()),
        }
        .settled(&inherited);
        assert_eq!(update.water_demand, Patch::Clear);
        assert_eq!(
            update.just_watered_ttl,
            Patch::Set(JustWateredTtl::new(43_200).unwrap())
        );
    }
}
