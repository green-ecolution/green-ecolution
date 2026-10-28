use std::sync::Arc;

use chrono::Duration;
use domain::{Id, RepositoryError, organization::Organization, settings::SettingsResolver};
use waste_domain::settings::FullThreshold;

use crate::settings::WasteSettingsResolver;

/// What an organization's settings say about when a bin counts as full or its reading as stale.
#[derive(Debug, Clone, Copy)]
pub struct FillRules {
    pub threshold: FullThreshold,
    pub stale_after: Duration,
}

pub struct FillRulesResolver {
    settings: Arc<dyn SettingsResolver>,
    waste_settings: Arc<dyn WasteSettingsResolver>,
}

impl FillRulesResolver {
    pub fn new(
        settings: Arc<dyn SettingsResolver>,
        waste_settings: Arc<dyn WasteSettingsResolver>,
    ) -> Self {
        Self {
            settings,
            waste_settings,
        }
    }

    pub async fn rules_for(&self, org: Id<Organization>) -> Result<FillRules, RepositoryError> {
        let effective = self.settings.effective_for(org).await?;
        let waste = self.waste_settings.effective_for(org).await?;
        Ok(FillRules {
            threshold: waste.full_threshold,
            stale_after: Duration::seconds(effective.sensor_offline_after.seconds()),
        })
    }
}
