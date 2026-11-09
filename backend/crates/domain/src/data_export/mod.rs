//! Export of an organization's objects, states and readings to an urban data
//! platform (a SensorThings server). This module holds the target
//! configuration; the sync itself lives in the server crate.
//! Which platforms accept such writes (CIVITAS/CORE V2 and CIVORA do not) is
//! recorded in `docs/integrations/urban-data-platforms.md`.

pub mod auth;
pub mod catalog;
pub mod error;
pub mod readings;
pub mod repository;
pub mod scope;
pub mod snapshot;
pub mod sync;
pub mod target;
pub mod url;

pub use auth::{
    ExportAuth, ExportAuthDraft, ExportAuthKind, ExportPrincipal, ExportScope, ExportSecret,
};
pub use catalog::{
    ExportCatalog, ExportProperty, ExportState, ExportThing, ExportUnit, ObservationKind, PERCENT,
    StateValue, ge_key,
};
pub use error::DataExportError;
pub use readings::{
    BatchOutcome, ExportReading, ExportReadingSource, ObservationRow, RawStream, ReadingBatch,
    device_sensor_key, raw_property, raw_stream_key, ucum, v7_floor,
};
pub use repository::{
    ConnectionOutcome, ConnectionReport, ConnectionTester, DATA_ARRAY_CONFORMANCE,
    ExportTargetReader, ExportTargetWriter,
};
pub use scope::{export_scope, responsible_target};
#[doc(hidden)]
pub use snapshot::ExportTargetSnapshot;
pub use sync::{
    DeletionMark, ExportEntity, ExportLease, ExportRef, ExportRefReader, ExportRefWriter,
    ExportRunStore, RemoteId, RunResult, SENSOR_KEY, SensorThingsSink, SinkError, SinkSession,
    StateStream, location_key, property_key, state_stream_key,
};
pub use target::{
    ExportAuthSummary, ExportRunStats, ExportTarget, ExportTargetConfig, ExportTargetDraft,
    ExportTargetStatus, ExportTargetView, StoredExportTarget,
};
pub use url::{ExportUrl, ExportUrlPolicy};

pub const BASE_URL_FIELD: &str = "data_export.base_url";
pub const TOKEN_URL_FIELD: &str = "data_export.auth.token_url";
