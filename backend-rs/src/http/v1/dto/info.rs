use serde::Serialize;

use crate::domain::info::{DataStatistics, Git, Map, Server, ServiceStatus, VersionInfo};

#[derive(Debug, Serialize)]
pub struct VersionInfoResponse {
    pub current: String,
    pub latest: String,
    #[serde(rename = "updateAvailable")]
    pub update_available: bool,
    #[serde(rename = "isDevelopment")]
    pub is_development: bool,
    #[serde(rename = "isStage")]
    pub is_stage: bool,
    #[serde(rename = "releaseUrl")]
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

#[derive(Debug, Serialize)]
pub struct GitInfoResponse {
    pub branch: String,
    pub commit: String,
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

#[derive(Debug, Serialize)]
pub struct MapInfoResponse {
    pub center: Vec<f64>,
    pub bbox: Vec<f64>,
}

impl From<&Map> for MapInfoResponse {
    fn from(value: &Map) -> Self {
        Self {
            center: value.center.to_vec(),
            bbox: value.bbox.to_vec(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServerInfoResponse {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub url: String,
    pub ip: String,
    pub port: u16,
    pub interface: String,
    pub uptime: String,
}

impl From<&Server> for ServerInfoResponse {
    fn from(value: &Server) -> Self {
        Self {
            os: value.os.clone(),
            arch: value.arch.clone(),
            hostname: value.hostname.clone(),
            url: value.url.to_string(),
            ip: value.ip.to_string(),
            port: value.port,
            interface: value.interface.clone(),
            uptime: format!("{}s", value.uptime.as_secs()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceStatusResponse {
    pub name: String,
    pub enabled: bool,
    pub healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastChecked")]
    pub last_checked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseTimeMs")]
    pub response_time_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl From<&ServiceStatus> for ServiceStatusResponse {
    fn from(value: &ServiceStatus) -> Self {
        Self {
            name: value.name.clone(),
            enabled: value.enabled,
            healthy: value.healthy,
            last_checked: Some(value.last_checked.to_rfc3339()),
            response_time_ms: Some(value.response_time.as_secs_f64() * 1000.0),
            message: Some(value.message.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServicesInfoResponse {
    pub items: Vec<ServiceStatusResponse>,
}

#[derive(Debug, Serialize)]
pub struct DataStatisticsResponse {
    #[serde(rename = "treeCount")]
    pub tree_count: i64,
    #[serde(rename = "sensorCount")]
    pub sensor_count: i64,
    #[serde(rename = "vehicleCount")]
    pub vehicle_count: i64,
    #[serde(rename = "treeClusterCount")]
    pub cluster_count: i64,
    #[serde(rename = "wateringPlanCount")]
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

#[derive(Debug, Serialize)]
pub struct AppInfoResponse {
    pub version: String,
    #[serde(rename = "versionInfo")]
    pub version_info: VersionInfoResponse,
    #[serde(rename = "goVersion")]
    pub rust_version: String,
    #[serde(rename = "buildTime")]
    pub build_time: String,
    pub git: GitInfoResponse,
    pub map: MapInfoResponse,
}
