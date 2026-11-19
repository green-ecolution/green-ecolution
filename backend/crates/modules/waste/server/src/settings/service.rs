use std::sync::Arc;

use domain::{Id, organization::Organization};
use uuid::Uuid;

use super::{WasteResolution, WasteSettings, WasteSettingsUpdate, repository::PgWasteSettings};
use server::service::ServiceError;

pub struct WasteSettingsService {
    repo: Arc<PgWasteSettings>,
}

impl WasteSettingsService {
    pub fn new(repo: Arc<PgWasteSettings>) -> Self {
        Self { repo }
    }

    pub async fn resolution(&self, org: Id<Organization>) -> Result<WasteResolution, ServiceError> {
        Ok(self.repo.resolution(org).await?)
    }

    pub async fn own(&self, org: Id<Organization>) -> Result<WasteSettings, ServiceError> {
        Ok(self.repo.own(org).await?)
    }

    /// A value equal to the inherited one is dropped, like in the core's settings.
    pub async fn update(
        &self,
        actor: Uuid,
        org: Id<Organization>,
        update: WasteSettingsUpdate,
    ) -> Result<WasteSettings, ServiceError> {
        let inherited = self.repo.resolution(org).await?.inherited;
        let update = update.settled(&inherited);
        if update.is_empty() {
            return Ok(self.repo.own(org).await?);
        }
        let actor = (!actor.is_nil()).then_some(actor);
        Ok(self.repo.apply(org, update, actor).await?)
    }
}
