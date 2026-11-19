use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    organization::Organization,
    settings::{EffectiveSettings, OrganizationSettings, Resolution, SettingsUpdate},
};

#[async_trait]
pub trait SettingsReader: Send + Sync {
    /// The organization's own values, all-inherit when it never set anything.
    async fn own(&self, org: Id<Organization>) -> Result<OrganizationSettings, RepositoryError>;
    /// Values in force plus what would be inherited, for one organization.
    async fn resolution(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError>;
}

#[async_trait]
pub trait SettingsWriter: Send + Sync {
    /// Persists the change and returns the organization's own values as they
    /// stand afterwards. `actor` is the person to credit; `None` where no user
    /// is attributable.
    async fn apply(
        &self,
        org: Id<Organization>,
        update: SettingsUpdate,
        actor: Option<uuid::Uuid>,
    ) -> Result<OrganizationSettings, RepositoryError>;
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
