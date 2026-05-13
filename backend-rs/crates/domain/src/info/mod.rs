//! System-info and health DTOs returned by the `/info` endpoints.

pub mod app;
pub mod runtime;
pub mod service;
pub mod statistics;

pub use app::{App, Git, Map, Server, VersionInfo};
pub use runtime::{CpuStats, DbPoolStats, MemoryStats, ProcessStats, RuntimeStats, TokioStats};
pub use service::{ServiceMessage, ServiceName, ServiceStatus};
pub use statistics::DataStatistics;

use crate::RepositoryError;

#[async_trait::async_trait]
pub trait SystemInfoProvider: Send + Sync {
    async fn app_info(&self) -> Result<App, RepositoryError>;
    async fn map_info(&self) -> Result<Map, RepositoryError>;
    async fn server_info(&self) -> Result<Server, RepositoryError>;
    async fn services_info(&self) -> Result<ServiceStatus, RepositoryError>;
    async fn statistics_info(&self) -> Result<DataStatistics, RepositoryError>;
}
