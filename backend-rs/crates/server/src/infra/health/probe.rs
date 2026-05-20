use async_trait::async_trait;

use domain::info::{ServiceName, ServiceStatus};

#[async_trait]
pub trait HealthProbe: Send + Sync {
    fn name(&self) -> ServiceName;
    async fn check(&self) -> ServiceStatus;
}
