use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    organization::Organization,
    settings::{
        InstanceDefaults, MapView, OrganizationSettings, Resolution, SettingChangeEntry,
        SettingKey, SettingsReader, SettingsResolver, SettingsUpdate, SettingsWriter,
    },
    user::UserProfileReader,
};

use super::ServiceError;

/// Reads and writes an organization's own values. Thin by design: the
/// resolution itself is domain logic, the storage belongs to the adapter.
pub struct SettingsService {
    reader: Arc<dyn SettingsReader>,
    writer: Arc<dyn SettingsWriter>,
    resolver: Arc<dyn SettingsResolver>,
    profiles: Arc<dyn UserProfileReader>,
    defaults: InstanceDefaults,
}

impl SettingsService {
    pub fn new(
        reader: Arc<dyn SettingsReader>,
        writer: Arc<dyn SettingsWriter>,
        resolver: Arc<dyn SettingsResolver>,
        profiles: Arc<dyn UserProfileReader>,
        defaults: InstanceDefaults,
    ) -> Self {
        Self {
            reader,
            writer,
            resolver,
            profiles,
            defaults,
        }
    }

    /// The viewport one user's map opens at. Falls back to the instance
    /// default wherever no organization is on file, which also covers the demo
    /// bypass: there the caller is an anonymous id that resolves to no profile
    /// at all. Deliberately without a permission check — this is the caller's
    /// own opening view, not a reading of someone else's settings.
    #[tracing::instrument(level = "info", skip(self))]
    pub async fn map_view_for_user(&self, user: Uuid) -> Result<MapView, ServiceError> {
        let organization = self
            .profiles
            .organizations_for(&[user])
            .await?
            .into_iter()
            .next()
            .map(|(_, org)| org);

        match organization {
            Some(org) => Ok(self.resolver.effective_for(org).await?.map_view),
            None => Ok(self.defaults.map_view),
        }
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
        // Checked after the lock, so a write into a frozen subtree still fails
        // rather than quietly succeeding because it happened to be empty.
        if update.is_empty() {
            return Ok(self.reader.own(org).await?);
        }
        // A nil actor is the demo bypass from `AuthorizationService`, not a
        // user, so it is stored as NULL — which is also why the `changed_by`
        // foreign key cannot be reached by a real caller.
        let actor = (!actor.is_nil()).then_some(actor);
        Ok(self.writer.apply(org, update, actor).await?)
    }
}
