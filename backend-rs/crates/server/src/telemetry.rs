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
//!   scope (`with_span_list`; surfaced as `spans[0].request_id` in JSON); inline events must
//!   not set it again.
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
//! - `request_id` is currently nested: extract via `| json request_id="spans[0].request_id"`.
//!   Promoting it to a flat top-level field is a separate, forthcoming change.

use tracing_subscriber::{
    EnvFilter, fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::configuration::{LogFormat, LogSettings};

/// Initialize the tracing subscriber.
pub fn init(config: &LogSettings) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.format {
        LogFormat::Json => registry
            .with(
                fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(false)
                    .with_span_list(true)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init(),
        LogFormat::Pretty => registry
            .with(fmt::layer().with_span_events(FmtSpan::CLOSE))
            .init(),
    }
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

    #[test]
    fn json_event_fields_are_flat_and_span_carries_request_id() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(
            fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(false)
                .with_span_list(true)
                .with_writer(BufWriter(buf.clone())),
        );

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("http_request", request_id = "req-1");
            let _guard = span.enter();
            tracing::info!(tree.id = 5, "tree watered");
        });

        let raw = String::from_utf8(buf.lock().expect("log buffer not poisoned").clone())
            .expect("log output is valid utf-8");
        let line = raw.lines().next().expect("one log line");
        let json: serde_json::Value = serde_json::from_str(line).expect("log line is valid json");

        // event fields flattened to top level — no `fields` wrapper, no current-span object
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

        // request_id stays in span context, reachable via spans[0]
        assert_eq!(json["spans"][0]["request_id"], "req-1");
        assert_eq!(json["spans"][0]["name"], "http_request");
    }
}
