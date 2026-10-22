mod buffer;
mod formats;
mod parse;

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use buffer::LogStore;
#[cfg(test)]
pub(crate) use parse::strip_ansi;
pub use parse::{LineParser, parse_line};

use crate::{config::LogFormat, sync::lock};

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
    parser: Arc<Mutex<LineParser>>,
}

impl LogSink {
    pub fn new(store: Arc<LogStore>, unit: impl Into<String>, format: LogFormat) -> Self {
        Self {
            store,
            parser: Arc::new(Mutex::new(LineParser::new(unit, format))),
        }
    }

    pub fn line(&self, line: &str) {
        let record = lock(&self.parser).parse(line, Utc::now());
        self.store.push(record);
    }
}
