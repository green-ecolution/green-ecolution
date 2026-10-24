use std::{collections::HashMap, sync::Arc};

use domain::{Id, organization::Organization};
use uuid::Uuid;

use super::{
    IrrigationResolution, IrrigationSettings, IrrigationSettingsUpdate,
    repository::PgIrrigationSettings,
};
use crate::{http::v1::dto::settings::ChangeStamp, service::ServiceError};

pub struct IrrigationSettingsService {
    repo: Arc<PgIrrigationSettings>,
}

impl IrrigationSettingsService {
    pub fn new(repo: Arc<PgIrrigationSettings>) -> Self {
        Self { repo }
    }

    pub async fn resolution(
        &self,
        org: Id<Organization>,
    ) -> Result<IrrigationResolution, ServiceError> {
        Ok(self.repo.resolution(org).await?)
    }

    pub async fn own(&self, org: Id<Organization>) -> Result<IrrigationSettings, ServiceError> {
        Ok(self.repo.own(org).await?)
    }

    pub async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<&'static str, ChangeStamp>, ServiceError> {
        Ok(self.repo.last_changes(org).await?)
    }

    /// Rejected while an ancestor froze the subtree, like the core's settings.
    pub async fn update(
        &self,
        actor: Uuid,
        org: Id<Organization>,
        update: IrrigationSettingsUpdate,
    ) -> Result<IrrigationSettings, ServiceError> {
        if let Some(enforcer) = self.repo.resolution(org).await?.effective.enforced_by {
            return Err(ServiceError::SettingsEnforcedByAncestor {
                organization_id: enforcer.value(),
            });
        }
        if update.is_empty() {
            return Ok(self.repo.own(org).await?);
        }
        let actor = (!actor.is_nil()).then_some(actor);
        Ok(self.repo.apply(org, update, actor).await?)
    }
}
