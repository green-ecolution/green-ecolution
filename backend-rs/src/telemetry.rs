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
