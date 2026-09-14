use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    organization::Organization,
    settings::{
        OrganizationSettings, Resolution, SettingChangeEntry, SettingKey, SettingsReader,
        SettingsUpdate, SettingsWriter,
    },
};

use super::ServiceError;

/// Reads and writes an organization's own values. Thin by design: the
/// resolution itself is domain logic, the cache belongs to the adapter.
pub struct SettingsService {
    reader: Arc<dyn SettingsReader>,
    writer: Arc<dyn SettingsWriter>,
}

impl SettingsService {
    pub fn new(reader: Arc<dyn SettingsReader>, writer: Arc<dyn SettingsWriter>) -> Self {
        Self { reader, writer }
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn resolution(&self, org: Id<Organization>) -> Result<Resolution, ServiceError> {
        Ok(self.reader.resolution(org).await?)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn own(&self, org: Id<Organization>) -> Result<OrganizationSettings, ServiceError> {
        Ok(self.reader.own(org).await?)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<SettingKey, SettingChangeEntry>, ServiceError> {
        Ok(self.reader.last_changes(org).await?)
    }

    /// Rejects the whole write when an ancestor froze the subtree — including
    /// the lock switch itself, so a sub-unit cannot unlock itself. That is
    /// harmless: the topmost lock wins regardless, and it spares the question
    /// of what a switch with no effect would mean.
    #[tracing::instrument(level = "info", skip(self, update))]
    pub async fn update(
        &self,
        actor: Uuid,
        org: Id<Organization>,
        update: SettingsUpdate,
    ) -> Result<OrganizationSettings, ServiceError> {
        if let Some(enforcer) = self.reader.resolution(org).await?.effective.enforced_by {
            return Err(ServiceError::SettingsEnforcedByAncestor {
                organization_id: enforcer.value(),
            });
        }
        let actor = (!actor.is_nil()).then_some(actor);
        Ok(self.writer.apply(org, update, actor).await?)
    }
}
