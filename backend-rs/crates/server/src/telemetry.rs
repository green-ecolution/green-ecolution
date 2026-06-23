//! Tracing subscriber setup and the structured-log field-name convention.
//!
//! # Log field naming convention
//!
//! Field keys are kept stable and identical across the HTTP, service, and infra layers
//! so Loki/LogQL can filter reliably. One concept maps to exactly one key. Keys use
//! dot-notation namespaces in the style of the OpenTelemetry semantic conventions.
//!
//! Rules:
//! - `request_id` is the correlation key. It is set once on the `http_request` span
//!   (see `http/tracing.rs`) and inherited by every event within the request via the span
//!   scope; in JSON it is lifted to a top-level `request_id` field (see the Loki/Promtail
//!   contract below). Inline events must not set it again.
//! - `error` always carries the error's `Display` string; `kind` carries a stable,
//!   low-cardinality category (`"auth"`, `"repository"`, ...).
//! - Domain identifiers are `<entity>.id` (`tree.id`, `cluster.id`, ...); other attributes
//!   are `<entity>.<attr>` (e.g. `vehicle.plate`).
//! - This is a convention, not machine-enforced. New fields follow the taxonomy below.
//!
//! | Namespace | Fields |
//! |---|---|
//! | cross-cutting | `request_id`, `error`, `kind` |
//! | `http.*` / `url.*` / `network.*` | `http.request.method`, `http.response.status_code`, `url.path`, `url.query`, `network.protocol.version`, `latency_ms` |
//! | `<entity>.*` | `tree.id`, `cluster.id`, `sensor.id`, `region.id`, `vehicle.id`, `vehicle.plate`, `plan.id`, `model.id` |
//! | `sensor.*` | `sensor.ability`, `sensor.depth_cm`, `sensor.probe`, `sensor.probe_id` |
//! | `mqtt.*` | `mqtt.topic` |
//! | `keycloak.*` | `keycloak.role`, `keycloak.status`, `keycloak.jwk.kid`, `keycloak.jwks.count` |
//! | `auth.*` | `auth.issuer_url` |
//! | `event.*` | `event.handler`, `event` |
//! | `query.*` | `query.len`, `query.limit` |
//!
//! # Loki/Promtail output contract
//!
//! In `json` format every event is one line-delimited JSON object on stdout, ready for a
//! standard Loki/Promtail `| json` pipeline:
//!
//! - Top level: `timestamp` (RFC3339 UTC), `level` (`INFO`/`WARN`/…), `message`, `target`, and
//!   every event field flattened to the top level (e.g. `tree.id`) — there is no `fields` wrapper.
//! - Span context lives in the `spans` array (root→leaf). `spans[0]` is the request root; for an
//!   HTTP request it carries `request_id`, `http.request.method`, `url.path`,
//!   `http.response.status_code`, `latency_ms`.
//!
//! Pipeline expectations (cluster-side, not the backend's concern):
//! - Labels stay low-cardinality: `app` (from k8s metadata) and `level`. High-cardinality values
//!   (`request_id`, `<entity>.id`) MUST remain query-time fields, never stream labels — one stream
//!   per request would blow up the index.
//! - Parse the `timestamp` field as RFC3339.
//! - `level` is upper-case; Grafana detects it case-insensitively. Lower-casing, if wanted, is a
//!   Promtail `template` stage, not a backend change.
//! - `request_id` is a top-level field on every event within a request — filter with
//!   `| json | request_id="…"`. It is also retained inside `spans[0]` for span context, and is
//!   always a query-time field, never a stream label (cardinality).

use tracing::{Event, Subscriber};
use tracing_subscriber::{
    EnvFilter, fmt,
    fmt::format::{FmtSpan, Format, Json, Writer},
    fmt::{FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

use crate::configuration::{LogFormat, LogSettings};

/// Initialize the tracing subscriber.
pub fn init(config: &LogSettings) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.format {
        LogFormat::Json => {
            let mut layer = fmt::layer().json().event_format(request_id_json_format());
            layer.set_span_events(FmtSpan::CLOSE);
            registry.with(layer).init()
        }
        LogFormat::Pretty => registry
            .with(fmt::layer().with_span_events(FmtSpan::CLOSE))
            .init(),
    }
}

/// Builds the JSON event formatter used in production: the standard flat-JSON layout plus
/// `request_id` lifted to the top level.
fn request_id_json_format() -> RequestIdJson {
    RequestIdJson {
        inner: fmt::format()
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(true),
    }
}

/// JSON event formatter that lifts `request_id` from the span scope to the top level.
///
/// `tracing-subscriber` cannot promote a span field to the top level, so the inner formatter
/// renders the standard line and we move the span's `request_id` up afterwards. The value is
/// still retained inside `spans[0]`.
struct RequestIdJson {
    inner: Format<Json>,
}

impl<S, N> FormatEvent<S, N> for RequestIdJson
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let mut buf = String::new();
        self.inner.format_event(ctx, Writer::new(&mut buf), event)?;

        let mut line = match serde_json::from_str::<serde_json::Value>(buf.trim_end()) {
            Ok(value) => value,
            Err(_) => return write!(writer, "{buf}"),
        };

        if lift_request_id(&mut line) {
            let rendered = serde_json::to_string(&line).map_err(|_| std::fmt::Error)?;
            writeln!(writer, "{rendered}")
        } else {
            write!(writer, "{buf}")
        }
    }
}

/// Moves a span's `request_id` to the top level of `value`. Returns `false` when nothing
/// changed — no `request_id` in scope, or the event already carries one — so the caller keeps
/// the already-rendered line instead of re-serializing.
fn lift_request_id(value: &mut serde_json::Value) -> bool {
    let Some(object) = value.as_object_mut() else {
        return false;
    };
    if object.contains_key("request_id") {
        return false;
    }
    let Some(request_id) = object
        .get("spans")
        .and_then(serde_json::Value::as_array)
        .and_then(|spans| {
            spans
                .iter()
                .find_map(|span| span.get("request_id").cloned())
        })
    else {
        return false;
    };
    object.insert("request_id".to_owned(), request_id);
    true
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::{
        fmt::{self, MakeWriter},
        layer::SubscriberExt,
    };

    #[derive(Clone, Default)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("log buffer not poisoned")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn first_line_json(buf: &Arc<Mutex<Vec<u8>>>) -> serde_json::Value {
        let raw = String::from_utf8(buf.lock().expect("log buffer not poisoned").clone())
            .expect("log output is valid utf-8");
        let line = raw.lines().next().expect("one log line").to_owned();
        serde_json::from_str(&line).expect("log line is valid json")
    }

    #[test]
    fn json_lifts_request_id_to_top_level_and_keeps_fields_flat() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(
            fmt::layer()
                .json()
                .event_format(super::request_id_json_format())
                .with_writer(BufWriter(buf.clone())),
        );

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("http_request", request_id = "req-1");
            let _guard = span.enter();
            tracing::info!(tree.id = 5, "tree watered");
        });

        let json = first_line_json(&buf);

        // event fields flat — no `fields` wrapper, no current-span object
        assert!(
            json.get("fields").is_none(),
            "event fields must be flat: {json}"
        );
        assert!(
            json.get("span").is_none(),
            "current-span object must be disabled: {json}"
        );
        assert_eq!(json["message"], "tree watered");
        assert_eq!(json["tree.id"], 5);
        assert_eq!(json["level"], "INFO");
        assert!(json["timestamp"].is_string());

        // request_id lifted to the top level, and still retained in span context
        assert_eq!(json["request_id"], "req-1");
        assert_eq!(json["spans"][0]["request_id"], "req-1");
    }

    #[test]
    fn json_without_request_id_passes_through_unchanged() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(
            fmt::layer()
                .json()
                .event_format(super::request_id_json_format())
                .with_writer(BufWriter(buf.clone())),
        );

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(mqtt.topic = "v3/devices/up", "reading ingested");
        });

        let json = first_line_json(&buf);

        assert!(
            json.get("request_id").is_none(),
            "no request_id without a span carrying one: {json}"
        );
        assert_eq!(json["message"], "reading ingested");
        assert_eq!(json["mqtt.topic"], "v3/devices/up");
    }

    #[test]
    fn pretty_output_exposes_request_id_inline() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(BufWriter(buf.clone())),
        );

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("http_request", request_id = "req-1");
            let _guard = span.enter();
            tracing::info!("tree watered");
        });

        let raw = String::from_utf8(buf.lock().expect("log buffer not poisoned").clone())
            .expect("log output is valid utf-8");

        assert!(
            raw.contains("request_id") && raw.contains("req-1"),
            "pretty line must expose request_id inline: {raw}"
        );
    }
}
