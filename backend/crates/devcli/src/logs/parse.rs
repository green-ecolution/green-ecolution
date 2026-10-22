use chrono::{DateTime, Utc};
use serde_json::{Map, Value};

use super::{Level, LogRecord, formats};
use crate::config::LogFormat;

pub fn parse_line(unit: &str, format: LogFormat, line: &str, now: DateTime<Utc>) -> LogRecord {
    classify(unit, format, line, now).0
}

/// Parses the lines of one source in order, so continuation lines inherit their header's level.
pub struct LineParser {
    unit: String,
    format: LogFormat,
    last_level: Level,
}

impl LineParser {
    pub fn new(unit: impl Into<String>, format: LogFormat) -> Self {
        Self {
            unit: unit.into(),
            format,
            last_level: Level::Unknown,
        }
    }

    pub fn parse(&mut self, line: &str, now: DateTime<Utc>) -> LogRecord {
        let (mut record, continuation) = classify(&self.unit, self.format, line, now);
        if !continuation {
            self.last_level = record.level;
        } else if record.level == Level::Unknown {
            record.level = self.last_level;
        }
        record
    }
}

fn classify(unit: &str, format: LogFormat, line: &str, now: DateTime<Utc>) -> (LogRecord, bool) {
    let line = line.trim_end_matches(['\r', '\n']);
    if format == LogFormat::TracingJson
        && let Some(record) = parse_tracing_json(unit, line)
    {
        return (record, false);
    }
    parse_plain(unit, line, now)
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
    let target = take_string(&mut fields, "target");
    let request_id = take_string(&mut fields, "request_id");
    let message = match take_string(&mut fields, "message") {
        Some(message) => {
            span_close_message(&message, target.as_deref(), &fields).unwrap_or(message)
        }
        None => message_from_fields(&fields),
    };
    Some(LogRecord {
        seq: 0,
        unit: unit.to_owned(),
        timestamp,
        level,
        target,
        message,
        request_id,
        fields,
        raw: line.to_owned(),
    })
}

/// `FmtSpan::CLOSE` events say only "close": the closing span is neither in `spans` nor named.
fn span_close_message(
    message: &str,
    target: Option<&str>,
    fields: &Map<String, Value>,
) -> Option<String> {
    let (Some(busy), Some(idle)) = (fields.get("time.busy"), fields.get("time.idle")) else {
        return None;
    };
    if message != "close" {
        return None;
    }
    let target = target
        .map(|target| format!(" {target}"))
        .unwrap_or_default();
    Some(format!(
        "close{target} busy={} idle={}",
        field_value(busy),
        field_value(idle)
    ))
}

/// `summary` leads the message instead; on sqlx query events `db.statement` and `elapsed_secs`
/// only repeat `summary` and `elapsed`.
const REDUNDANT_FIELDS: [&str; 4] = ["spans", "summary", "db.statement", "elapsed_secs"];

/// Some emitters (sqlx) log without a `message`; render their fields like tracing's pretty format.
fn message_from_fields(fields: &Map<String, Value>) -> String {
    let summary = fields.get("summary").map(single_line);
    let pairs = fields
        .iter()
        .filter(|(key, _)| !REDUNDANT_FIELDS.contains(&key.as_str()))
        .map(|(key, value)| format!("{key}={}", field_value(value)));
    summary
        .into_iter()
        .chain(pairs)
        .collect::<Vec<_>>()
        .join(" ")
}

fn field_value(value: &Value) -> String {
    let text = single_line(value);
    if text.contains(' ') {
        format!("{text:?}")
    } else {
        text
    }
}

fn single_line(value: &Value) -> String {
    match value {
        Value::String(text) => text.split_whitespace().collect::<Vec<_>>().join(" "),
        other => other.to_string(),
    }
}

fn take_string(fields: &mut Map<String, Value>, key: &str) -> Option<String> {
    match fields.remove(key)? {
        Value::String(value) => Some(value),
        other => Some(other.to_string()),
    }
}

fn parse_plain(unit: &str, line: &str, now: DateTime<Utc>) -> (LogRecord, bool) {
    let stripped = strip_ansi(line);
    let mut record = LogRecord {
        seq: 0,
        unit: unit.to_owned(),
        timestamp: now,
        level: Level::Unknown,
        target: None,
        message: String::new(),
        request_id: None,
        fields: Map::new(),
        raw: line.to_owned(),
    };
    let Some(known) = formats::recognize(&stripped) else {
        record.level = sniff_level(&stripped);
        record.message = stripped;
        return (record, false);
    };
    record.timestamp = known.timestamp.unwrap_or(now);
    record.level = known.level;
    record.target = known.target;
    record.message = known.message;
    record.fields = known.fields;
    (record, known.continuation)
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
    fn builds_message_from_fields_when_event_has_none() {
        let line = r#"{"timestamp":"2026-10-22T11:52:58.096154Z","level":"DEBUG","summary":"SELECT id, parent_id FROM …","db.statement":"\n\nSELECT id, parent_id FROM organizations\n","rows_affected":4,"rows_returned":4,"elapsed":"202.957µs","elapsed_secs":0.000202957,"target":"sqlx::query","spans":[{"name":"expire_just_watered"}]}"#;

        let record = parse_line("backend", LogFormat::TracingJson, line, now());

        assert_eq!(
            record.message,
            "SELECT id, parent_id FROM … elapsed=202.957µs rows_affected=4 rows_returned=4"
        );
        assert_eq!(record.target.as_deref(), Some("sqlx::query"));
        assert!(record.fields.contains_key("db.statement"));
    }

    #[test]
    fn quotes_field_values_with_whitespace_and_collapses_newlines() {
        let line = r#"{"timestamp":"2026-10-22T11:52:58Z","level":"DEBUG","reason":"pool\n  closed","tries":2}"#;

        let record = parse_line("backend", LogFormat::TracingJson, line, now());

        assert_eq!(record.message, r#"reason="pool closed" tries=2"#);
    }

    #[test]
    fn names_target_and_timings_on_span_close() {
        let line = r#"{"timestamp":"2026-10-22T11:52:58.109617Z","level":"DEBUG","message":"close","time.busy":"1.19ms","time.idle":"12.1ms","target":"server::service::cluster_service","spans":[]}"#;

        let record = parse_line("backend", LogFormat::TracingJson, line, now());

        assert_eq!(
            record.message,
            "close server::service::cluster_service busy=1.19ms idle=12.1ms"
        );
    }

    #[test]
    fn keeps_a_plain_close_message_without_span_timings() {
        let line = r#"{"timestamp":"2026-10-22T11:52:58Z","level":"INFO","message":"close","target":"server::mqtt"}"#;

        let record = parse_line("backend", LogFormat::TracingJson, line, now());

        assert_eq!(record.message, "close");
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
            plain("2026-10-22 12:17:38,888 INFO  [org.infinispan.CONTAINER] (main) ISPN000974: Virtual threads support: enabled").level,
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
