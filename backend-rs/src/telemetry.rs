use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the tracing subscriber.
///
/// - `RUST_LOG` controls log levels (default: `info`).
/// - `json = true` switches to structured JSON output (for production).
pub fn init(json: bool) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if json {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer()).init();
    };
}
