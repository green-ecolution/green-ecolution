use std::net::IpAddr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use url::Url;

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    name: String,
    enabled: bool,
    healthy: bool,
    response_time: Duration,
    last_checked: DateTime<Utc>,
    message: String,
}

impl ServiceStatus {
    pub fn new(
        name: String,
        enabled: bool,
        healthy: bool,
        response_time: Duration,
        last_checked: DateTime<Utc>,
        message: String,
    ) -> Self {
        Self {
            name,
            enabled,
            healthy,
            response_time,
            last_checked,
            message,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn healthy(&self) -> bool {
        self.healthy
    }
    pub fn response_time(&self) -> Duration {
        self.response_time
    }
    pub fn last_checked(&self) -> DateTime<Utc> {
        self.last_checked
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    tree_count: i64,
    sensor_count: i64,
    vehicle_count: i64,
    cluster_count: i64,
    watering_plan_count: i64,
}

impl DataStatistics {
    pub fn new(
        tree_count: i64,
        sensor_count: i64,
        vehicle_count: i64,
        cluster_count: i64,
        watering_plan_count: i64,
    ) -> Self {
        Self {
            tree_count,
            sensor_count,
            vehicle_count,
            cluster_count,
            watering_plan_count,
        }
    }

    pub fn tree_count(&self) -> i64 {
        self.tree_count
    }
    pub fn sensor_count(&self) -> i64 {
        self.sensor_count
    }
    pub fn vehicle_count(&self) -> i64 {
        self.vehicle_count
    }
    pub fn cluster_count(&self) -> i64 {
        self.cluster_count
    }
    pub fn watering_plan_count(&self) -> i64 {
        self.watering_plan_count
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    current: String,
    latest: String,
    update_available: bool,
    is_development: bool,
    is_stage: bool,
    release_url: Url,
}

impl VersionInfo {
    pub fn new(
        current: String,
        latest: String,
        update_available: bool,
        is_development: bool,
        is_stage: bool,
        release_url: Url,
    ) -> Self {
        Self {
            current,
            latest,
            update_available,
            is_development,
            is_stage,
            release_url,
        }
    }

    pub fn current(&self) -> &str {
        &self.current
    }
    pub fn latest(&self) -> &str {
        &self.latest
    }
    pub fn update_available(&self) -> bool {
        self.update_available
    }
    pub fn is_development(&self) -> bool {
        self.is_development
    }
    pub fn is_stage(&self) -> bool {
        self.is_stage
    }
    pub fn release_url(&self) -> &Url {
        &self.release_url
    }
}

#[derive(Debug, Clone)]
pub struct App {
    version: String,
    version_info: VersionInfo,
    rust_version: String,
    build_time: DateTime<Utc>,
    git: Git,
    server: Server,
    map: Map,
    services: Vec<ServiceStatus>,
}

impl App {
    pub fn new(
        version: String,
        version_info: VersionInfo,
        rust_version: String,
        build_time: DateTime<Utc>,
        git: Git,
        server: Server,
        map: Map,
        services: Vec<ServiceStatus>,
    ) -> Self {
        Self {
            version,
            version_info,
            rust_version,
            build_time,
            git,
            server,
            map,
            services,
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn version_info(&self) -> &VersionInfo {
        &self.version_info
    }
    pub fn rust_version(&self) -> &str {
        &self.rust_version
    }
    pub fn build_time(&self) -> DateTime<Utc> {
        self.build_time
    }
    pub fn git(&self) -> &Git {
        &self.git
    }
    pub fn server(&self) -> &Server {
        &self.server
    }
    pub fn map(&self) -> &Map {
        &self.map
    }
    pub fn services(&self) -> &[ServiceStatus] {
        &self.services
    }
}

#[derive(Debug, Clone)]
pub struct Git {
    branch: String,
    commit: String,
    repository: Url,
}

impl Git {
    pub fn new(branch: String, commit: String, repository: Url) -> Self {
        Self {
            branch,
            commit,
            repository,
        }
    }

    pub fn branch(&self) -> &str {
        &self.branch
    }
    pub fn commit(&self) -> &str {
        &self.commit
    }
    pub fn repository(&self) -> &Url {
        &self.repository
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    os: String,
    arch: String,
    hostname: String,
    url: Url,
    ip: IpAddr,
    port: u16,
    interface: String,
    uptime: Duration,
}

impl Server {
    pub fn new(
        os: String,
        arch: String,
        hostname: String,
        url: Url,
        ip: IpAddr,
        port: u16,
        interface: String,
        uptime: Duration,
    ) -> Self {
        Self {
            os,
            arch,
            hostname,
            url,
            ip,
            port,
            interface,
            uptime,
        }
    }

    pub fn os(&self) -> &str {
        &self.os
    }
    pub fn arch(&self) -> &str {
        &self.arch
    }
    pub fn hostname(&self) -> &str {
        &self.hostname
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn ip(&self) -> IpAddr {
        self.ip
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn interface(&self) -> &str {
        &self.interface
    }
    pub fn uptime(&self) -> Duration {
        self.uptime
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map {
    center: [f64; 2],
    bbox: [f64; 4],
}

impl Map {
    pub fn new(center: [f64; 2], bbox: [f64; 4]) -> Self {
        Self { center, bbox }
    }

    pub fn center(&self) -> [f64; 2] {
        self.center
    }
    pub fn bbox(&self) -> [f64; 4] {
        self.bbox
    }
}
