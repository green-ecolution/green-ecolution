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
//!   (see `http/tracing.rs`) and inherited by every event within the request via
//!   `with_current_span` + `with_span_list`; inline events must not set it again.
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
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init(),
        LogFormat::Pretty => registry
            .with(fmt::layer().with_span_events(FmtSpan::CLOSE))
            .init(),
    }
}
