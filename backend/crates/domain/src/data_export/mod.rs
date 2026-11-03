//! Export of an organization's objects, states and readings to an urban data
//! platform (a SensorThings server). This module holds the target
//! configuration; the sync itself lives in the server crate.

pub mod auth;
pub mod error;
pub mod repository;
pub mod snapshot;
pub mod target;
pub mod url;

pub use auth::{
    ExportAuth, ExportAuthDraft, ExportAuthKind, ExportPrincipal, ExportScope, ExportSecret,
};
pub use error::DataExportError;
pub use repository::{
    ConnectionOutcome, ConnectionReport, ConnectionTester, DATA_ARRAY_CONFORMANCE,
    ExportTargetReader, ExportTargetWriter,
};
#[doc(hidden)]
pub use snapshot::ExportTargetSnapshot;
pub use target::{
    ExportAuthSummary, ExportRunStats, ExportTarget, ExportTargetConfig, ExportTargetDraft,
    ExportTargetStatus, ExportTargetView, StoredExportTarget,
};
pub use url::{ExportUrl, ExportUrlPolicy};

pub const BASE_URL_FIELD: &str = "data_export.base_url";
pub const TOKEN_URL_FIELD: &str = "data_export.auth.token_url";
