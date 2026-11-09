use super::{ExportTargetConfig, StoredExportTarget};
use crate::{Id, RepositoryError, organization::Organization};

pub const DATA_ARRAY_CONFORMANCE: &str =
    "http://www.opengis.net/spec/iot_sensing/1.1/req/data-array/data-array";

#[async_trait::async_trait]
pub trait ExportTargetReader: Send + Sync {
    async fn find(
        &self,
        org: Id<Organization>,
    ) -> Result<Option<StoredExportTarget>, RepositoryError>;
}

#[async_trait::async_trait]
pub trait ExportTargetWriter: Send + Sync {
    /// A changed base URL counts as a new target: activation and cursor restart,
    /// the refs are dropped. Resuming a paused target restarts activation and
    /// cursor as well; it keeps the refs.
    async fn upsert(
        &self,
        org: Id<Organization>,
        config: &ExportTargetConfig,
    ) -> Result<StoredExportTarget, RepositoryError>;

    /// `NotFound` when the organization has no target.
    async fn delete(&self, org: Id<Organization>) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOutcome {
    Ok,
    UrlNotAllowed,
    Unreachable,
    Unauthorized,
    NotSensorThings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionReport {
    pub outcome: ConnectionOutcome,
    pub http_status: Option<u16>,
    pub data_array_supported: bool,
    pub detail: Option<String>,
}

impl ConnectionReport {
    pub fn failed(
        outcome: ConnectionOutcome,
        http_status: Option<u16>,
        detail: Option<String>,
    ) -> Self {
        Self {
            outcome,
            http_status,
            data_array_supported: false,
            detail,
        }
    }
}

/// A failed test is a normal answer, not an error: the report says why.
#[async_trait::async_trait]
pub trait ConnectionTester: Send + Sync {
    async fn test(&self, config: &ExportTargetConfig) -> ConnectionReport;
}
