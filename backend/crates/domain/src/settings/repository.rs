use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    Id, RepositoryError,
    organization::Organization,
    settings::{
        EffectiveSettings, OrganizationSettings, Resolution, SettingChangeEntry, SettingKey,
    },
};

#[async_trait]
pub trait SettingsReader: Send + Sync {
    /// The organization's own values, all-inherit when it never set anything.
    async fn own(&self, org: Id<Organization>) -> Result<OrganizationSettings, RepositoryError>;
    /// Values plus per-field origin for one organization.
    async fn resolution(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError>;
    /// Most recent change per key, for the "last changed" line under a field.
    async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<SettingKey, SettingChangeEntry>, RepositoryError>;
}

/// Read port for the places that consume a value, kept separate from
/// [`SettingsReader`] so a consumer cannot reach the write path or the
/// per-field provenance it has no use for.
#[async_trait]
pub trait SettingsResolver: Send + Sync {
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveSettings, RepositoryError>;
}
