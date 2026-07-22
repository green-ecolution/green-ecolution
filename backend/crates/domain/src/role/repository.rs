use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    Id, RepositoryError,
    organization::Organization,
    role::{Role, RoleDraft},
};

#[async_trait]
pub trait RoleReader: Send + Sync {
    async fn by_id(&self, id: Id<Role>) -> Result<Role, RepositoryError>;
    async fn by_organization(&self, org: Id<Organization>) -> Result<Vec<Role>, RepositoryError>;
    async fn templates(&self) -> Result<Vec<Role>, RepositoryError>;
    async fn roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, RepositoryError>;
    async fn roles_for_users(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, Role)>, RepositoryError>;
    async fn user_ids_with_role(&self, role_id: Id<Role>) -> Result<Vec<Uuid>, RepositoryError>;
}

#[async_trait]
pub trait RoleWriter: Send + Sync {
    async fn save_new(&self, draft: RoleDraft) -> Result<Role, RepositoryError>;
    async fn save(&self, role: &Role) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Role>) -> Result<(), RepositoryError>;
    async fn assign_to_user(&self, user_id: Uuid, role_id: Id<Role>)
    -> Result<(), RepositoryError>;
    async fn revoke_from_user(
        &self,
        user_id: Uuid,
        role_id: Id<Role>,
    ) -> Result<(), RepositoryError>;
}
