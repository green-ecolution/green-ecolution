use serde::Serialize;

use domain::info::{DataStatistics, Git, Map, Server, ServiceStatus, VersionInfo};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VersionInfoResponse {
    #[schema(example = "0.1.0")]
    pub current: String,
    #[schema(example = "0.1.0")]
    pub latest: String,
    #[serde(rename = "updateAvailable")]
    #[schema(example = false)]
    pub update_available: bool,
    #[serde(rename = "isDevelopment")]
    #[schema(example = true)]
    pub is_development: bool,
    #[serde(rename = "isStage")]
    #[schema(example = false)]
    pub is_stage: bool,
    #[serde(rename = "releaseUrl")]
    #[schema(example = "https://github.com/green-ecolution/backend-rs/releases/")]
    pub release_url: String,
}

impl From<&VersionInfo> for VersionInfoResponse {
    fn from(value: &VersionInfo) -> Self {
        Self {
            current: value.current.clone(),
            latest: value.latest.clone(),
            update_available: value.update_available,
            is_development: value.is_development,
            is_stage: value.is_stage,
            release_url: value.release_url.to_string(),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GitInfoResponse {
    #[schema(example = "main")]
    pub branch: String,
    #[schema(example = "a1b2c3d")]
    pub commit: String,
    #[schema(example = "https://github.com/green-ecolution/backend-rs/")]
    pub repository: String,
}

impl From<&Git> for GitInfoResponse {
    fn from(value: &Git) -> Self {
        Self {
            branch: value.branch.clone(),
            commit: value.commit.clone(),
            repository: value.repository.to_string(),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MapInfoResponse {
    #[schema(example = json!([54.7937, 9.4469]))]
    pub center: [f64; 2],
    #[schema(example = json!([54.75, 9.40, 54.83, 9.50]))]
    pub bbox: [f64; 4],
}

impl From<&Map> for MapInfoResponse {
    fn from(value: &Map) -> Self {
        Self {
            center: value.center,
            bbox: value.bbox,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServerInfoResponse {
    #[schema(example = "linux")]
    pub os: String,
    #[schema(example = "x86_64")]
    pub arch: String,
    #[schema(example = "green-ecolution-prod")]
    pub hostname: String,
    #[schema(example = "https://api.green-ecolution.de")]
    pub url: String,
    #[schema(example = 3000)]
    pub port: u16,
    #[schema(example = "0.0.0.0")]
    pub interface: String,
    #[serde(rename = "uptimeSeconds")]
    #[schema(example = 86400)]
    pub uptime_seconds: u64,
}

impl From<&Server> for ServerInfoResponse {
    fn from(value: &Server) -> Self {
        Self {
            os: value.os.clone(),
            arch: value.arch.clone(),
            hostname: value.hostname.clone(),
            url: value.url.to_string(),
            port: value.port,
            interface: value.interface.clone(),
            uptime_seconds: value.uptime.as_secs(),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServiceStatusResponse {
    #[schema(example = "database")]
    pub name: String,
    #[schema(example = true)]
    pub enabled: bool,
    #[schema(example = true)]
    pub healthy: bool,
    #[serde(rename = "lastChecked")]
    #[schema(example = "2024-08-01T12:00:00+00:00")]
    pub last_checked: String,
    #[serde(rename = "responseTimeMs")]
    #[schema(example = 2.5)]
    pub response_time_ms: f64,
    #[schema(example = "service.status.connected")]
    pub message: String,
}

impl From<&ServiceStatus> for ServiceStatusResponse {
    fn from(value: &ServiceStatus) -> Self {
        Self {
            name: value.name.as_key().to_string(),
            enabled: value.enabled,
            healthy: value.healthy,
            last_checked: value.last_checked.to_rfc3339(),
            response_time_ms: value.response_time.as_secs_f64() * 1000.0,
            message: value.message.as_key().to_string(),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServicesInfoResponse {
    pub items: Vec<ServiceStatusResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataStatisticsResponse {
    #[serde(rename = "treeCount")]
    #[schema(example = 342)]
    pub tree_count: i64,
    #[serde(rename = "sensorCount")]
    #[schema(example = 85)]
    pub sensor_count: i64,
    #[serde(rename = "vehicleCount")]
    #[schema(example = 12)]
    pub vehicle_count: i64,
    #[serde(rename = "treeClusterCount")]
    #[schema(example = 28)]
    pub cluster_count: i64,
    #[serde(rename = "wateringPlanCount")]
    #[schema(example = 15)]
    pub watering_plan_count: i64,
}

impl From<&DataStatistics> for DataStatisticsResponse {
    fn from(value: &DataStatistics) -> Self {
        Self {
            tree_count: value.tree_count,
            sensor_count: value.sensor_count,
            vehicle_count: value.vehicle_count,
            cluster_count: value.cluster_count,
            watering_plan_count: value.watering_plan_count,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AppInfoResponse {
    #[schema(example = "0.1.0")]
    pub version: String,
    #[serde(rename = "versionInfo")]
    pub version_info: VersionInfoResponse,
    #[serde(rename = "rustVersion")]
    #[schema(example = "1.88.0")]
    pub rust_version: String,
    #[serde(rename = "rustChannel")]
    #[schema(example = "stable")]
    pub rust_channel: String,
    #[serde(rename = "rustEdition")]
    #[schema(example = "2024")]
    pub rust_edition: String,
    #[serde(rename = "buildTime")]
    #[schema(example = "2024-08-01T10:00:00+00:00")]
    pub build_time: String,
    pub git: GitInfoResponse,
    pub map: MapInfoResponse,
}
