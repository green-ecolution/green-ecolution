use uuid::Uuid;

use crate::{http::AppState, service::ServiceError};
use domain::{
    Id, RepositoryError,
    authorization::{AccessContext, Permission},
    organization::{Organization, root_organization_id},
};

/// Target org for a create: explicit payload wins, then the acting user's
/// own organization; the demo bypass falls back to the root org.
pub async fn resolve_target_org(
    state: &AppState,
    user_id: Uuid,
    payload_org: Option<Uuid>,
) -> Result<Id<Organization>, ServiceError> {
    if let Some(org) = payload_org {
        return Ok(Id::new(org));
    }
    if let Some(org) = state.user_service.organization_of(user_id).await? {
        return Ok(org);
    }
    if !state.authorization_service.enforced() {
        return Ok(root_organization_id());
    }
    Err(ServiceError::MissingOrganization)
}

/// Detail-endpoint gate: an invisible resource reads as 404, never 403,
/// so callers cannot probe for existence.
pub fn ensure_visible(
    ctx: &AccessContext,
    read: Permission,
    owner: Uuid,
) -> Result<(), ServiceError> {
    if ctx.allows_in(read, Id::new(owner)) {
        Ok(())
    } else {
        Err(RepositoryError::NotFound.into())
    }
}
