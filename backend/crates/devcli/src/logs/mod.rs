mod buffer;
pub mod builds;
mod filter;
mod formats;
mod parse;
pub mod trace;

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use buffer::LogStore;
pub use filter::{Filter, FilterError};
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
    unit: Arc<str>,
    parser: Arc<Mutex<LineParser>>,
}

impl LogSink {
    pub fn new(store: Arc<LogStore>, unit: impl Into<String>, format: LogFormat) -> Self {
        let unit: String = unit.into();
        Self {
            store,
            unit: unit.as_str().into(),
            parser: Arc::new(Mutex::new(LineParser::new(unit, format))),
        }
    }

    /// Takes one line or one `\r`-terminated segment of a redrawn terminal line.
    pub fn line(&self, line: &str) {
        let now = Utc::now();
        let plain = parse::strip_ansi(line);
        match builds::classify(&plain) {
            Some(signal @ builds::Signal::Progress { .. }) => {
                self.store.builds().observe(&self.unit, signal, now);
                return;
            }
            Some(signal) => self.store.builds().observe(&self.unit, signal, now),
            // What is left of a line cargo cleared by overwriting it.
            None if line.ends_with('\r') && plain.trim().is_empty() => return,
            None => {}
        }
        let record = lock(&self.parser).parse(line, now);
        self.store.push(record);
    }
}
