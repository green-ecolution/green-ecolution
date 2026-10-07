use std::sync::Arc;

use domain::{
    authorization::{Action, Permission, Resource},
    plugin::Plugin,
    sensor::{SensorDraft, SensorView},
    sensor_model::SensorModel,
};

use super::{
    AuthError, ServiceError, authorization::AuthorizationService,
    plugin_ingest_service::plugin_access_context, sensor_service::SensorService,
};

pub struct PluginSensorIngestService {
    sensor: Arc<SensorService>,
    authorization: Arc<AuthorizationService>,
}

impl PluginSensorIngestService {
    pub fn new(sensor: Arc<SensorService>, authorization: Arc<AuthorizationService>) -> Self {
        Self {
            sensor,
            authorization,
        }
    }

    async fn require(&self, plugin: &Plugin, action: Action) -> Result<(), ServiceError> {
        let ctx = plugin_access_context(plugin, &self.authorization).await?;
        if ctx.allows_in(
            Permission::new(Resource::Sensor, action),
            plugin.organization_id(),
        ) {
            Ok(())
        } else {
            Err(AuthError::Forbidden.into())
        }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn list_models(&self, plugin: &Plugin) -> Result<Vec<SensorModel>, ServiceError> {
        self.require(plugin, Action::Read).await?;
        self.sensor.list_models().await
    }

    /// The draft must already carry the plugin's organization and provider;
    /// the HTTP layer builds it that way so a body can never choose either.
    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn create(
        &self,
        plugin: &Plugin,
        draft: SensorDraft,
    ) -> Result<SensorView, ServiceError> {
        self.require(plugin, Action::Create).await?;
        self.sensor.create(draft).await
    }
}
