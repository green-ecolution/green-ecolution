use green_ecolution::{configuration::get_configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("failed to read configuration");

    telemetry::init(&config.log);

    let app = Application::build(config).await?;
    app.run_until_stopped().await
}
