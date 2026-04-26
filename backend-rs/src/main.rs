use green_ecolution::{
    configuration::{Environment, get_configuration},
    startup::Application,
    telemetry,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("failed to read configuration");

    let json_logging = matches!(config.application.environment, Environment::Production);
    telemetry::init(json_logging);

    let app = Application::build(config).await?;
    app.run_until_stopped().await
}
