use serde::Serialize;

use domain::info::{DataStatistics, Git, Map, Server, ServiceStatus, VersionInfo};

/// Version information for the running application, including update status.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VersionInfoResponse {
    /// Currently deployed version string.
    #[schema(example = "0.1.0")]
    pub current: String,

    /// Latest available version from the release channel.
    #[schema(example = "0.1.0")]
    pub latest: String,

    /// Whether a newer version is available.
    #[serde(rename = "updateAvailable")]
    #[schema(example = false)]
    pub update_available: bool,

    /// Whether this instance is running in development mode.
    #[serde(rename = "isDevelopment")]
    #[schema(example = true)]
    pub is_development: bool,

    /// Whether this instance is running in a staging environment.
    #[serde(rename = "isStage")]
    #[schema(example = false)]
    pub is_stage: bool,

    /// URL to the GitHub releases page.
    #[serde(rename = "releaseUrl")]
    #[schema(example = "https://github.com/green-ecolution/backend/releases")]
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

/// Git repository metadata for the current build.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GitInfoResponse {
    /// Branch the build was created from.
    #[schema(example = "main")]
    pub branch: String,

    /// Short commit hash of the build.
    #[schema(example = "a1b2c3d")]
    pub commit: String,

    /// URL of the source repository.
    #[schema(example = "https://github.com/green-ecolution/backend")]
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

/// Default map viewport configuration (center point and bounding box).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MapInfoResponse {
    /// Center of the map as [latitude, longitude].
    #[schema(example = json!([54.7937, 9.4469]))]
    pub center: Vec<f64>,

    /// Bounding box as [south, west, north, east].
    #[schema(example = json!([54.75, 9.40, 54.83, 9.50]))]
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

/// Host and runtime details of the server.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServerInfoResponse {
    /// Operating system name.
    #[schema(example = "linux")]
    pub os: String,

    /// CPU architecture.
    #[schema(example = "x86_64")]
    pub arch: String,

    /// Machine hostname.
    #[schema(example = "green-ecolution-prod")]
    pub hostname: String,

    /// Public-facing URL of the server.
    #[schema(example = "https://api.green-ecolution.de")]
    pub url: String,

    /// TCP port the server listens on.
    #[schema(example = 3000)]
    pub port: u16,

    /// Network interface the server is bound to.
    #[schema(example = "0.0.0.0")]
    pub interface: String,

    /// Server uptime in seconds (e.g. "86400s").
    #[schema(example = "86400s")]
    pub uptime: String,
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
            uptime: format!("{}s", value.uptime.as_secs()),
        }
    }
}

/// Health and availability status of an individual backend service.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServiceStatusResponse {
    /// Name of the service (e.g. "database", "s3", "routing").
    #[schema(example = "database")]
    pub name: String,

    /// Whether the service is enabled in the configuration.
    #[schema(example = true)]
    pub enabled: bool,

    /// Whether the last health check succeeded.
    #[schema(example = true)]
    pub healthy: bool,

    /// ISO 8601 timestamp of the last health check.
    #[serde(skip_serializing_if = "Option::is_none", rename = "lastChecked")]
    #[schema(example = "2024-08-01T12:00:00+00:00", nullable)]
    pub last_checked: Option<String>,

    /// Response time of the last health check in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseTimeMs")]
    #[schema(example = 2.5, nullable)]
    pub response_time_ms: Option<f64>,

    /// Human-readable status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "OK", nullable)]
    pub message: Option<String>,
}

impl From<&ServiceStatus> for ServiceStatusResponse {
    fn from(value: &ServiceStatus) -> Self {
        Self {
            name: value.name.as_key().to_string(),
            enabled: value.enabled,
            healthy: value.healthy,
            last_checked: Some(value.last_checked.to_rfc3339()),
            response_time_ms: Some(value.response_time.as_secs_f64() * 1000.0),
            message: Some(value.message.as_key().to_string()),
        }
    }
}

/// Collection of service status entries.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServicesInfoResponse {
    /// List of monitored backend services and their current status.
    pub items: Vec<ServiceStatusResponse>,
}

/// Aggregate counts of core domain entities in the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataStatisticsResponse {
    /// Total number of trees.
    #[serde(rename = "treeCount")]
    #[schema(example = 342)]
    pub tree_count: i64,

    /// Total number of sensors.
    #[serde(rename = "sensorCount")]
    #[schema(example = 85)]
    pub sensor_count: i64,

    /// Total number of vehicles.
    #[serde(rename = "vehicleCount")]
    #[schema(example = 12)]
    pub vehicle_count: i64,

    /// Total number of tree clusters.
    #[serde(rename = "treeClusterCount")]
    #[schema(example = 28)]
    pub cluster_count: i64,

    /// Total number of watering plans.
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

/// Top-level application info combining version, build, git, and map data.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AppInfoResponse {
    /// Application version string.
    #[schema(example = "0.1.0")]
    pub version: String,

    /// Detailed version and update information.
    #[serde(rename = "versionInfo")]
    pub version_info: VersionInfoResponse,

    /// Rust compiler version used for the build (serialized as "goVersion" for backward compatibility).
    #[serde(rename = "goVersion")]
    #[schema(example = "1.82.0")]
    pub rust_version: String,

    /// ISO 8601 timestamp of when the binary was built.
    #[serde(rename = "buildTime")]
    #[schema(example = "2024-08-01T10:00:00+00:00")]
    pub build_time: String,

    /// Git metadata for the current build.
    pub git: GitInfoResponse,

    /// Default map viewport configuration.
    pub map: MapInfoResponse,
}
