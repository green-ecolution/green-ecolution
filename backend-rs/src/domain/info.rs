use std::net::IpAddr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use url::Url;

use crate::domain::RepositoryError;

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub enabled: bool,
    pub healthy: bool,
    pub response_time: Duration,
    pub last_checked: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub tree_count: i64,
    pub sensor_count: i64,
    pub vehicle_count: i64,
    pub cluster_count: i64,
    pub watering_plan_count: i64,
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub current: String,
    pub latest: String,
    pub update_available: bool,
    pub is_development: bool,
    pub is_stage: bool,
    pub release_url: Url,
}

#[derive(Debug, Clone)]
pub struct App {
    pub version: String,
    pub version_info: VersionInfo,
    pub rust_version: String,
    pub build_time: DateTime<Utc>,
    pub git: Git,
    pub server: Server,
    pub map: Map,
    pub services: Vec<ServiceStatus>,
}

#[derive(Debug, Clone)]
pub struct Git {
    pub branch: String,
    pub commit: String,
    pub repository: Url,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub url: Url,
    pub ip: IpAddr,
    pub port: u16,
    pub interface: String,
    pub uptime: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Map {
    pub center: [f64; 2],
    pub bbox: [f64; 4],
}

#[async_trait::async_trait]
pub trait SystemInfoProvider: Send + Sync {
    async fn app_info(&self) -> Result<App, RepositoryError>;
    async fn map_info(&self) -> Result<Map, RepositoryError>;
    async fn server_info(&self) -> Result<Server, RepositoryError>;
    async fn services_info(&self) -> Result<ServiceStatus, RepositoryError>;
    async fn statistics_info(&self) -> Result<DataStatistics, RepositoryError>;
}
