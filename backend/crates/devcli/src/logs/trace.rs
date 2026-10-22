use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Map, Value};

use super::{Level, LogRecord};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestSummary {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub latency_ms: u64,
    pub finished_at: DateTime<Utc>,
}

impl RequestSummary {
    /// Status and latency are recorded on the `http_request` span when the response leaves, so
    /// only events logged after that carry them in `spans[0]`.
    pub fn from_record(record: &LogRecord) -> Option<Self> {
        let request_id = record.request_id.clone()?;
        let root = spans(record).first()?.as_object()?;
        if root.get("name")?.as_str()? != "http_request" {
            return None;
        }
        let text = |key: &str| root.get(key)?.as_str().map(str::to_owned);
        Some(Self {
            request_id,
            method: text("http.request.method")?,
            path: text("url.path")?,
            status: u16::try_from(root.get("http.response.status_code")?.as_u64()?).ok()?,
            latency_ms: root.get("latency_ms")?.as_u64()?,
            finished_at: record.timestamp,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Trace {
    pub request_id: String,
    pub summary: Option<RequestSummary>,
    pub lines: Vec<TraceLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraceLine {
    pub offset_us: i64,
    pub depth: usize,
    #[serde(flatten)]
    pub kind: TraceLineKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceLineKind {
    Enter {
        span: String,
    },
    Event {
        level: Level,
        target: Option<String>,
        message: String,
    },
    /// The closing span's name is not logged, only its target.
    Close {
        target: Option<String>,
        busy: String,
        idle: String,
    },
}

impl Trace {
    /// `records` belong to `request_id` and are in sequence order.
    pub fn build(request_id: &str, records: &[LogRecord]) -> Self {
        let start = records.first().map(|record| record.timestamp);
        let mut open: Vec<String> = Vec::new();
        let mut lines = Vec::new();
        for record in records {
            let offset_us = start
                .and_then(|start| (record.timestamp - start).num_microseconds())
                .unwrap_or(0);
            let chain = span_names(record);
            let shared = open.iter().zip(&chain).take_while(|(a, b)| a == b).count();
            for (depth, span) in chain.iter().enumerate().skip(shared) {
                lines.push(TraceLine {
                    offset_us,
                    depth,
                    kind: TraceLineKind::Enter { span: span.clone() },
                });
            }
            let kind = match close_timings(&record.fields) {
                Some((busy, idle)) => TraceLineKind::Close {
                    target: record.target.clone(),
                    busy,
                    idle,
                },
                None => TraceLineKind::Event {
                    level: record.level,
                    target: record.target.clone(),
                    message: record.message.clone(),
                },
            };
            lines.push(TraceLine {
                offset_us,
                depth: chain.len(),
                kind,
            });
            open = chain;
        }
        Self {
            request_id: request_id.to_owned(),
            summary: records.iter().find_map(RequestSummary::from_record),
            lines,
        }
    }
}

pub fn offset_label(offset_us: i64) -> String {
    let offset_us = offset_us.max(0);
    format!("+{}.{:03}ms", offset_us / 1_000, offset_us % 1_000)
}

fn spans(record: &LogRecord) -> &[Value] {
    record
        .fields
        .get("spans")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn span_names(record: &LogRecord) -> Vec<String> {
    spans(record)
        .iter()
        .filter_map(|span| span.get("name")?.as_str().map(str::to_owned))
        .collect()
}

fn close_timings(fields: &Map<String, Value>) -> Option<(String, String)> {
    let text = |key: &str| fields.get(key)?.as_str().map(str::to_owned);
    Some((text("time.busy")?, text("time.idle")?))
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{config::LogFormat, logs::parse_line};

    const ID: &str = "1c40b9b5-8486-4ded-975c-d956a3a5cad2";

    const INFO_REQUEST: [&str; 3] = [
        r#"{"level":"DEBUG","message":"started processing request","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","spans":[{"http.request.method":"GET","name":"http_request","network.protocol.version":"HTTP/1.1","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","url.path":"/api/v1/info"}],"target":"tower_http::trace::on_request","timestamp":"2026-10-22T12:26:16.372430Z"}"#,
        r#"{"level":"INFO","message":"close","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","spans":[{"http.request.method":"GET","name":"http_request","network.protocol.version":"HTTP/1.1","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","url.path":"/api/v1/info"}],"target":"server::http::v1::info","time.busy":"33.5µs","time.idle":"8.07µs","timestamp":"2026-10-22T12:26:16.372675Z"}"#,
        r#"{"level":"DEBUG","message":"end of stream","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","spans":[{"http.request.method":"GET","http.response.status_code":200,"latency_ms":0,"name":"http_request","network.protocol.version":"HTTP/1.1","request_id":"1c40b9b5-8486-4ded-975c-d956a3a5cad2","url.path":"/api/v1/info"}],"stream_duration":"0 ms","target":"tower_http::trace::on_eos","timestamp":"2026-10-22T12:26:16.372877Z"}"#,
    ];

    fn backend(line: &str) -> LogRecord {
        parse_line("backend", LogFormat::TracingJson, line, Utc::now())
    }

    /// An event logged after the response, which is what the request list is built from.
    pub(crate) fn finished_request(
        id: &str,
        path: &str,
        status: u16,
        latency_ms: u64,
    ) -> LogRecord {
        backend(&format!(
            r#"{{"level":"DEBUG","message":"end of stream","request_id":"{id}","spans":[{{"http.request.method":"GET","http.response.status_code":{status},"latency_ms":{latency_ms},"name":"http_request","request_id":"{id}","url.path":"{path}"}}],"target":"tower_http::trace::on_eos","timestamp":"2026-10-22T12:26:16Z"}}"#
        ))
    }

    #[test]
    fn summaries_come_from_events_after_the_response() {
        assert_eq!(RequestSummary::from_record(&backend(INFO_REQUEST[0])), None);

        let summary = RequestSummary::from_record(&backend(INFO_REQUEST[2])).unwrap();
        assert_eq!(summary.request_id, ID);
        assert_eq!(summary.method, "GET");
        assert_eq!(summary.path, "/api/v1/info");
        assert_eq!(summary.status, 200);
        assert_eq!(summary.latency_ms, 0);
        assert_eq!(
            summary.finished_at.to_rfc3339(),
            "2026-10-22T12:26:16.372877+00:00"
        );
    }

    #[test]
    fn records_outside_a_request_have_no_summary() {
        let root_close = backend(
            r#"{"timestamp":"2026-10-22T12:26:16.372937Z","level":"INFO","message":"close","time.busy":"473µs","time.idle":"48.0µs","target":"server::http::tracing","spans":[]}"#,
        );
        assert_eq!(RequestSummary::from_record(&root_close), None);
    }

    #[test]
    fn builds_the_trace_of_a_real_request() {
        let records: Vec<LogRecord> = INFO_REQUEST.iter().map(|line| backend(line)).collect();

        let trace = Trace::build(ID, &records);

        assert_eq!(trace.summary.as_ref().map(|s| s.status), Some(200));
        assert_eq!(
            trace.lines,
            [
                TraceLine {
                    offset_us: 0,
                    depth: 0,
                    kind: TraceLineKind::Enter {
                        span: "http_request".into()
                    },
                },
                TraceLine {
                    offset_us: 0,
                    depth: 1,
                    kind: TraceLineKind::Event {
                        level: Level::Debug,
                        target: Some("tower_http::trace::on_request".into()),
                        message: "started processing request".into(),
                    },
                },
                TraceLine {
                    offset_us: 245,
                    depth: 1,
                    kind: TraceLineKind::Close {
                        target: Some("server::http::v1::info".into()),
                        busy: "33.5µs".into(),
                        idle: "8.07µs".into(),
                    },
                },
                TraceLine {
                    offset_us: 447,
                    depth: 1,
                    kind: TraceLineKind::Event {
                        level: Level::Debug,
                        target: Some("tower_http::trace::on_eos".into()),
                        message: "end of stream".into(),
                    },
                },
            ]
        );
    }

    #[test]
    fn nested_spans_are_entered_once_and_closed_at_their_depth() {
        let span = |chain: &str, extra: &str, message: &str, micros: u32| {
            backend(&format!(
                r#"{{"level":"DEBUG","message":"{message}","request_id":"r","spans":[{chain}],"target":"server::service::tree"{extra},"timestamp":"2026-10-22T12:00:00.{micros:06}Z"}}"#
            ))
        };
        let root = r#"{"name":"http_request","request_id":"r"}"#;
        let list = r#"{"name":"list"}"#;
        let records = [
            span(&format!("{root},{list}"), "", "loading trees", 0),
            span(&format!("{root},{list}"), "", "loaded", 10),
            span(root, r#","time.busy":"9µs","time.idle":"1µs""#, "close", 12),
            span(&format!("{root},{list}"), "", "loading again", 20),
        ];

        let trace = Trace::build("r", &records);

        let shape: Vec<(usize, &str)> = trace
            .lines
            .iter()
            .map(|line| {
                let label = match &line.kind {
                    TraceLineKind::Enter { span } => span.as_str(),
                    TraceLineKind::Event { message, .. } => message.as_str(),
                    TraceLineKind::Close { busy, .. } => busy.as_str(),
                };
                (line.depth, label)
            })
            .collect();
        assert_eq!(
            shape,
            [
                (0, "http_request"),
                (1, "list"),
                (2, "loading trees"),
                (2, "loaded"),
                (1, "9µs"),
                (1, "list"),
                (2, "loading again"),
            ]
        );
    }

    #[test]
    fn offsets_render_in_milliseconds() {
        assert_eq!(offset_label(0), "+0.000ms");
        assert_eq!(offset_label(245), "+0.245ms");
        assert_eq!(offset_label(12_345_678), "+12345.678ms");
        assert_eq!(offset_label(-3), "+0.000ms");
    }

    #[test]
    fn traces_serialize_with_flat_line_kinds() {
        let records: Vec<LogRecord> = INFO_REQUEST.iter().map(|line| backend(line)).collect();
        let json = serde_json::to_value(Trace::build(ID, &records)).unwrap();
        assert_eq!(json["lines"][2]["kind"], "close");
        assert_eq!(json["lines"][2]["busy"], "33.5µs");
        assert_eq!(json["summary"]["path"], "/api/v1/info");
    }
}
