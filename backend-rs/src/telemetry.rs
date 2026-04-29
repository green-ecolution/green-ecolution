use tracing_subscriber::{
    EnvFilter, fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Initialize the tracing subscriber.
///
/// - `RUST_LOG` controls log levels (default: `info`).
/// - `json = true` switches to structured JSON output (production).
/// - `FmtSpan::CLOSE` emits one event per span close including elapsed time,
///   so per-handler/per-service timings appear without manual logging.
pub fn init(json: bool) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if json {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        registry
            .with(fmt::layer().with_span_events(FmtSpan::CLOSE))
            .init();
    };
}
