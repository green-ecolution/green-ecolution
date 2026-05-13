//! App-level metadata: version, build, git, server, map.

use std::net::IpAddr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use url::Url;

use super::ServiceStatus;

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
