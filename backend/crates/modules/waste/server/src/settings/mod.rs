//! Waste's own settings. They inherit down the organization tree like the
//! core's and obey the same lock, which stays a core setting.

pub mod http;
pub mod repository;
pub mod service;

use domain::{
    Id, RepositoryError,
    organization::Organization,
    settings::{InheritLevel, Patch, SettingOrigin, inherit},
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
    pub enforced_by: Option<Id<Organization>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasteOrigins {
    pub full_threshold: SettingOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasteResolution {
    pub effective: EffectiveWasteSettings,
    pub origins: WasteOrigins,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WasteSettingsUpdate {
    pub full_threshold: Patch<FullThreshold>,
}

impl WasteSettingsUpdate {
    pub fn is_empty(&self) -> bool {
        !self.full_threshold.touches()
    }
}

/// One organization's level: its own waste values plus the core lock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasteLevel {
    pub settings: WasteSettings,
    pub descendants_may_override: bool,
}

/// `chain` runs root first, target last.
pub fn resolve(
    defaults: &WasteDefaults,
    chain: &[WasteLevel],
    target: Id<Organization>,
) -> WasteResolution {
    let levels: Vec<InheritLevel<FullThreshold>> = chain
        .iter()
        .map(|l| InheritLevel {
            organization_id: l.settings.organization_id,
            value: l.settings.full_threshold,
            descendants_may_override: l.descendants_may_override,
        })
        .collect();
    let full_threshold = inherit(defaults.full_threshold, &levels, target);
    WasteResolution {
        effective: EffectiveWasteSettings {
            full_threshold: full_threshold.value,
            enforced_by: full_threshold.enforced_by,
        },
        origins: WasteOrigins {
            full_threshold: full_threshold.origin,
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

    fn level(org: Id<Organization>, threshold: Option<i64>, may_override: bool) -> WasteLevel {
        WasteLevel {
            settings: WasteSettings {
                organization_id: org,
                full_threshold: threshold.map(|t| FullThreshold::new(t).unwrap()),
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
        assert_eq!(r.effective.full_threshold.percent(), 80);
        assert_eq!(r.origins.full_threshold, SettingOrigin::Default);
    }

    #[test]
    fn inherited_value_names_its_source() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, Some(50), true), level(leaf, None, true)],
            leaf,
        );
        assert_eq!(r.effective.full_threshold.percent(), 50);
        assert_eq!(r.origins.full_threshold, SettingOrigin::Inherited(root));
    }

    #[test]
    fn lock_above_freezes_the_value() {
        let (root, leaf) = (Id::new_v7(), Id::new_v7());
        let r = resolve(
            &defaults(),
            &[level(root, Some(50), false), level(leaf, Some(90), true)],
            leaf,
        );
        assert_eq!(r.effective.full_threshold.percent(), 50);
        assert_eq!(r.effective.enforced_by, Some(root));
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
