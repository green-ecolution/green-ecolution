use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    authorization::OrgHierarchy,
    organization::{Organization, OrganizationDraft},
};

#[async_trait]
pub trait OrganizationReader: Send + Sync {
    async fn all(&self) -> Result<Vec<Organization>, RepositoryError>;
    async fn by_id(&self, id: Id<Organization>) -> Result<Organization, RepositoryError>;
    /// Loads the full parent map in one query; the tree is small by design.
    async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError>;
}

#[async_trait]
pub trait OrganizationWriter: Send + Sync {
    async fn save_new(&self, draft: OrganizationDraft) -> Result<Organization, RepositoryError>;
    async fn save(&self, org: &Organization) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Organization>) -> Result<(), RepositoryError>;
}
