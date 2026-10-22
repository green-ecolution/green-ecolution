mod buffer;
mod parse;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use buffer::LogStore;
pub use parse::parse_line;

use crate::config::LogFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Unknown,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "",
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRecord {
    pub seq: u64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub target: Option<String>,
    pub message: String,
    pub request_id: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
    pub raw: String,
}

#[derive(Clone)]
pub struct LogSink {
    store: Arc<LogStore>,
    unit: String,
    format: LogFormat,
}

impl LogSink {
    pub fn new(store: Arc<LogStore>, unit: impl Into<String>, format: LogFormat) -> Self {
        Self {
            store,
            unit: unit.into(),
            format,
        }
    }

    pub fn line(&self, line: &str) {
        self.store
            .push(parse_line(&self.unit, self.format, line, Utc::now()));
    }
}
