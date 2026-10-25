use std::sync::Arc;

use domain::{organization::OrganizationReader, region::RegionReader};

use crate::{
    service::{
        cluster_service::ClusterService, evaluation_service::EvaluationService,
        plugin_ingest_service::PluginIngestService, sensor_overview::IrrigationSensorService,
        tree_service::TreeService,
    },
    settings::service::IrrigationSettingsService,
};
use server::{
    http::extractors::PluginAuthenticator,
    service::{
        authorization::AuthorizationService, comment_service::CommentService,
        module_access::ModuleAccess, operation_plan_service::OperationPlanService,
        sensor_service::SensorService, user_service::UserService,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct NearestTreeLimits {
    pub max_radius_meters: f64,
    pub default_limit: u32,
    pub max_limit: u32,
}

pub struct IrrigationHttpState {
    pub trees: Arc<TreeService>,
    pub clusters: Arc<ClusterService>,
    pub evaluation: Arc<EvaluationService>,
    pub plugin_ingest: Arc<PluginIngestService>,
    pub sensor_overview: Arc<IrrigationSensorService>,
    pub settings: Arc<IrrigationSettingsService>,
    pub sensors: Arc<SensorService>,
    pub plans: Arc<OperationPlanService>,
    pub regions: Arc<dyn RegionReader>,
    pub comments: Arc<CommentService>,
    pub access: ModuleAccess,
    pub authorization: Arc<AuthorizationService>,
    pub organizations: Arc<dyn OrganizationReader>,
    pub users: Arc<UserService>,
    pub plugin_auth: PluginAuthenticator,
    pub nearest_tree_limits: NearestTreeLimits,
}

impl AsRef<PluginAuthenticator> for IrrigationHttpState {
    fn as_ref(&self) -> &PluginAuthenticator {
        &self.plugin_auth
    }
}
