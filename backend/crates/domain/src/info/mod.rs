//! System-info and health DTOs returned by the `/info` endpoints.

pub mod app;
pub mod readiness;
pub mod service;
pub mod statistics;

pub use app::{App, Git, Map, Server, VersionInfo};
pub use readiness::{Readiness, ReadinessReader};
pub use service::{ServiceMessage, ServiceName, ServiceStatus};
pub use statistics::DataStatistics;

use crate::RepositoryError;

#[async_trait::async_trait]
pub trait SystemInfoProvider: Send + Sync {
    async fn app_info(&self) -> Result<App, RepositoryError>;
    async fn map_info(&self) -> Result<Map, RepositoryError>;
    async fn server_info(&self) -> Result<Server, RepositoryError>;
}

#[async_trait::async_trait]
pub trait HealthSnapshotReader: Send + Sync {
    async fn snapshot(&self) -> Vec<ServiceStatus>;
}

#[async_trait::async_trait]
pub trait StatisticsReader: Send + Sync {
    async fn statistics(&self) -> Result<DataStatistics, RepositoryError>;
}
