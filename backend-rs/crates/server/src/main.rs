use std::error::Error;
use std::process::ExitCode;

use server::{
    configuration::{ConfigError, get_configuration},
    startup::Application,
    telemetry,
};

#[tokio::main]
async fn main() -> ExitCode {
    let config = match get_configuration() {
        Ok(c) => c,
        Err(err) => {
            print_config_error(&err);
            return ExitCode::FAILURE;
        }
    };

    telemetry::init(&config.log);

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

fn print_config_error(err: &ConfigError) {
    eprintln!("error: failed to load configuration");
    eprintln!("  {err}");

    let mut source = err.source();
    while let Some(s) = source {
        eprintln!("  caused by: {s}");
        source = s.source();
    }
}
