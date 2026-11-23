//! Waste's own settings. They inherit down the organization tree like the core's.

pub mod http;
pub mod repository;
pub mod service;

use domain::{
    Id, RepositoryError,
    organization::Organization,
    settings::{InheritLevel, Patch, inherit},
    shared::error::ValidationError,
};
use waste_domain::settings::FullThreshold;

use super::WasteConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasteSettings {
    pub organization_id: Id<Organization>,
    pub full_threshold: Option<FullThreshold>,
}

impl WasteSettings {
    pub fn empty(organization_id: Id<Organization>) -> Self {
        Self {
            organization_id,
            full_threshold: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasteDefaults {
    pub full_threshold: FullThreshold,
}

impl WasteDefaults {
    pub fn from_config(config: &WasteConfig) -> Result<Self, ValidationError> {
        Ok(Self {
            full_threshold: FullThreshold::new(i64::from(config.full_threshold_percent))?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveWasteSettings {
    pub full_threshold: FullThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasteResolution {
    pub effective: EffectiveWasteSettings,
    /// The value in force without the organization's own one.
    pub inherited: EffectiveWasteSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WasteSettingsUpdate {
    pub full_threshold: Patch<FullThreshold>,
}

impl WasteSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        !self.full_threshold.touches()
    }

    /// A value equal to the inherited one is no own value, as in the core.
    pub fn settled(self, inherited: &EffectiveWasteSettings) -> Self {
        Self {
            full_threshold: self.full_threshold.settled(inherited.full_threshold),
        }
    }
}

/// `chain` runs root first, target last.
pub fn resolve(
    defaults: &WasteDefaults,
    chain: &[WasteSettings],
    target: Id<Organization>,
) -> WasteResolution {
    let levels: Vec<InheritLevel<FullThreshold>> = chain
        .iter()
        .map(|s| InheritLevel {
            organization_id: s.organization_id,
            value: s.full_threshold,
        })
        .collect();
    let full_threshold = inherit(defaults.full_threshold, &levels, target);
    WasteResolution {
        effective: EffectiveWasteSettings {
            full_threshold: full_threshold.value,
        },
        inherited: EffectiveWasteSettings {
            full_threshold: full_threshold.inherited,
        },
    }
}

#[async_trait::async_trait]
pub trait WasteSettingsResolver: Send + Sync {
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveWasteSettings, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> WasteDefaults {
        WasteDefaults {
            full_threshold: FullThreshold::new(80).unwrap(),
        }
    }

    fn level(org: Id<Organization>, threshold: Option<i64>) -> WasteSettings {
        WasteSettings {
            organization_id: org,
            full_threshold: threshold.map(|t| FullThreshold::new(t).unwrap()),
        }
    }

    #[test]
    fn defaults_apply_without_values() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(&defaults(), &[level(root, None), level(leaf, None)], leaf);
        assert_eq!(r.effective.full_threshold.percent(), 80);
        assert_eq!(r.inherited.full_threshold.percent(), 80);
    }

    #[test]
    fn inherited_value_comes_from_the_nearest_ancestor() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, Some(50)), level(leaf, None)],
            leaf,
        );
        assert_eq!(r.effective.full_threshold.percent(), 50);
        assert_eq!(r.inherited.full_threshold.percent(), 50);
    }

    #[test]
    fn a_value_equal_to_the_inherited_one_is_dropped() {
        let inherited = EffectiveWasteSettings {
            full_threshold: FullThreshold::new(80).unwrap(),
        };
        let same = WasteSettingsUpdate {
            full_threshold: Patch::Set(FullThreshold::new(80).unwrap()),
        }
        .settled(&inherited);
        assert_eq!(same.full_threshold, Patch::Clear);
        let other = WasteSettingsUpdate {
            full_threshold: Patch::Set(FullThreshold::new(60).unwrap()),
        }
        .settled(&inherited);
        assert_eq!(
            other.full_threshold,
            Patch::Set(FullThreshold::new(60).unwrap())
        );
    }

    #[test]
    fn an_out_of_range_config_value_is_rejected() {
        let err = WasteDefaults::from_config(&WasteConfig {
            full_threshold_percent: 0,
        })
        .unwrap_err();
        assert!(err.to_string().contains("full_threshold"), "{err}");
        assert!(
            WasteDefaults::from_config(&WasteConfig {
                full_threshold_percent: 101
            })
            .is_err()
        );
    }
}
