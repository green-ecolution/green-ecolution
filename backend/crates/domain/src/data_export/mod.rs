//! Export of an organization's objects, states and readings to an urban data
//! platform (a SensorThings server). This module holds the target
//! configuration; the sync itself lives in the server crate.

pub mod error;
pub mod url;

pub use error::DataExportError;
pub use url::{ExportUrl, ExportUrlPolicy};

pub const BASE_URL_FIELD: &str = "data_export.base_url";
pub const TOKEN_URL_FIELD: &str = "data_export.auth.token_url";
