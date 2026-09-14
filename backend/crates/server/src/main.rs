use std::env;
use std::error::Error;
use std::process::ExitCode;

use server::{
    configuration::{ConfigError, get_configuration},
    startup::Application,
    telemetry,
};

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let healthcheck_probe = match args.next().as_deref() {
        Some("healthcheck") => Some(args.next().unwrap_or_else(|| "health".into())),
        _ => None,
    };

    let config = match get_configuration() {
        Ok(c) => c,
        Err(err) => {
            print_config_error(&err);
            return ExitCode::FAILURE;
        }
    };

    if let Some(probe) = healthcheck_probe {
        return run_healthcheck(config.application.port, &probe).await;
    }

    telemetry::init(&config.log);

    if let Err(err) = config.ensure_secure() {
        tracing::error!(error = %err, kind = "security", "refusing to start");
        return ExitCode::FAILURE;
    }

    for advisory in config.security_advisories() {
        tracing::warn!(kind = "security", "{advisory}");
    }

    let app = match Application::build(config).await {
        Ok(app) => app,
        Err(err) => {
            tracing::error!(error = %err, "failed to start application");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = app.run_until_stopped().await {
        tracing::error!(error = %err, "server stopped with error");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Container healthcheck: no telemetry, and no `ensure_secure` — a probe must
/// not fail on a startup advisory the running server already accepted.
async fn run_healthcheck(port: u16, probe: &str) -> ExitCode {
    let url = format!("http://127.0.0.1:{port}/api/{probe}");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!("healthcheck: {err}");
            return ExitCode::FAILURE;
        }
    };

    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => ExitCode::SUCCESS,
        Ok(response) => {
            eprintln!("healthcheck: {url} returned {}", response.status());
            ExitCode::FAILURE
        }
        Err(err) => {
            eprintln!("healthcheck: {url} unreachable: {err}");
            ExitCode::FAILURE
        }
    }
}

fn print_config_error(err: &ConfigError) {
    eprintln!("error: failed to load configuration");
    eprintln!("  {err}");

    let mut source = err.source();
    while let Some(s) = source {
        eprintln!("  caused by: {s}");
        source = s.source();
    }
}
