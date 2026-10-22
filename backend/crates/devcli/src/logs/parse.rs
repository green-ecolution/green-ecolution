use chrono::{DateTime, Utc};
use serde_json::{Map, Value};

use super::{Level, LogRecord};
use crate::config::LogFormat;

pub fn parse_line(unit: &str, format: LogFormat, line: &str, now: DateTime<Utc>) -> LogRecord {
    let line = line.trim_end_matches(['\r', '\n']);
    match format {
        LogFormat::TracingJson => {
            parse_tracing_json(unit, line).unwrap_or_else(|| parse_plain(unit, line, now))
        }
        LogFormat::Plain => parse_plain(unit, line, now),
    }
}

fn parse_tracing_json(unit: &str, line: &str) -> Option<LogRecord> {
    let Value::Object(mut fields) = serde_json::from_str::<Value>(line).ok()? else {
        return None;
    };
    let timestamp = take_string(&mut fields, "timestamp")
        .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())?
        .with_timezone(&Utc);
    let level =
        take_string(&mut fields, "level").map_or(Level::Unknown, |value| level_from_str(&value));
    Some(LogRecord {
        seq: 0,
        unit: unit.to_owned(),
        timestamp,
        level,
        target: take_string(&mut fields, "target"),
        message: take_string(&mut fields, "message").unwrap_or_default(),
        request_id: take_string(&mut fields, "request_id"),
        fields,
        raw: line.to_owned(),
    })
}

fn take_string(fields: &mut Map<String, Value>, key: &str) -> Option<String> {
    match fields.remove(key)? {
        Value::String(value) => Some(value),
        other => Some(other.to_string()),
    }
}

fn parse_plain(unit: &str, line: &str, now: DateTime<Utc>) -> LogRecord {
    let message = strip_ansi(line);
    LogRecord {
        seq: 0,
        unit: unit.to_owned(),
        timestamp: now,
        level: sniff_level(&message),
        target: None,
        message,
        request_id: None,
        fields: Map::new(),
        raw: line.to_owned(),
    }
}

fn level_from_str(value: &str) -> Level {
    match value.to_ascii_uppercase().as_str() {
        "ERROR" | "FATAL" | "PANIC" => Level::Error,
        "WARN" | "WARNING" => Level::Warn,
        "INFO" | "LOG" => Level::Info,
        "DEBUG" => Level::Debug,
        "TRACE" => Level::Trace,
        _ => Level::Unknown,
    }
}

/// Only upper-case tokens count, so prose like "no error" in a message stays `Unknown`.
fn sniff_level(message: &str) -> Level {
    if message.starts_with("error") {
        return Level::Error;
    }
    if message.starts_with("warning") {
        return Level::Warn;
    }
    if let Some(index) = message.find("level=") {
        let token: String = message[index + "level=".len()..]
            .trim_start_matches('"')
            .chars()
            .take_while(char::is_ascii_alphabetic)
            .collect();
        let level = level_from_str(&token);
        if level != Level::Unknown {
            return level;
        }
    }
    message
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|token| token.chars().all(|c| c.is_ascii_uppercase()))
        .map(level_from_str)
        .find(|level| *level != Level::Unknown)
        .unwrap_or(Level::Unknown)
}

pub(crate) fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        if chars.next() == Some('[') {
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-22T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn plain(line: &str) -> LogRecord {
        parse_line("db", LogFormat::Plain, line, now())
    }

    #[test]
    fn reads_backend_tracing_json() {
        let line = r#"{"timestamp":"2026-10-22T09:00:00.123Z","level":"INFO","message":"finished","target":"server::http","request_id":"req-1","http.response.status_code":200,"spans":[{"name":"http_request","request_id":"req-1"}]}"#;

        let record = parse_line("backend", LogFormat::TracingJson, line, now());

        assert_eq!(record.unit, "backend");
        assert_eq!(record.level, Level::Info);
        assert_eq!(record.message, "finished");
        assert_eq!(record.target.as_deref(), Some("server::http"));
        assert_eq!(record.request_id.as_deref(), Some("req-1"));
        assert_eq!(
            record.timestamp.to_rfc3339(),
            "2026-10-22T09:00:00.123+00:00"
        );
        assert_eq!(record.fields["http.response.status_code"], 200);
        assert!(record.fields.contains_key("spans"));
        assert!(!record.fields.contains_key("message"));
        assert_eq!(record.raw, line);
    }

    #[test]
    fn falls_back_to_plain_for_compiler_output() {
        let record = parse_line(
            "backend",
            LogFormat::TracingJson,
            "error[E0425]: cannot find value `x` in this scope",
            now(),
        );
        assert_eq!(record.level, Level::Error);
        assert_eq!(record.timestamp, now());
    }

    #[test]
    fn detects_common_plain_levels() {
        assert_eq!(plain("warning: unused variable").level, Level::Warn);
        assert_eq!(
            plain("2026-10-22 09:00:00 UTC [1] LOG:  database system is ready").level,
            Level::Info
        );
        assert_eq!(
            plain("2026-10-22 09:00:00,1 INFO  [io.quarkus] started").level,
            Level::Info
        );
        assert_eq!(
            plain(r#"time="2026" level=warn msg="slow""#).level,
            Level::Warn
        );
        assert_eq!(plain("nothing to see").level, Level::Unknown);
    }

    #[test]
    fn strips_ansi_sequences_and_line_endings() {
        let record = plain("\u{1b}[31mERROR\u{1b}[0m boom\r\n");
        assert_eq!(record.message, "ERROR boom");
        assert_eq!(record.level, Level::Error);
    }
}
